use std::{
    cell::UnsafeCell,
    sync::{Mutex, Once},
    thread::ThreadId,
};

#[cfg(not(Py_3_12))]
use crate::sync::MutexExt;
#[cfg(Py_3_12)]
use crate::types::{PyString, PyTuple};
use crate::{
    exceptions::{PyBaseException, PyTypeError},
    ffi,
    ffi_ptr_ext::FfiPtrExt,
    types::{PyAnyMethods, PyTraceback, PyType},
    Bound, Py, PyAny, PyErrArguments, PyTypeInfo, Python,
};

pub(crate) struct PyErrState {
    // Safety: can only hand out references when in the "normalized" state. Will never change
    // after normalization.
    normalized: Once,
    // Guard against re-entrancy when normalizing the exception state.
    normalizing_thread: Mutex<Option<ThreadId>>,
    inner: UnsafeCell<Option<PyErrStateInner>>,
}

// Safety: The inner value is protected by locking to ensure that only the normalized state is
// handed out as a reference.
unsafe impl Send for PyErrState {}
unsafe impl Sync for PyErrState {}
#[cfg(feature = "nightly")]
unsafe impl crate::marker::Ungil for PyErrState {}

impl PyErrState {
    pub(crate) fn lazy(f: Box<PyErrStateLazyFn>) -> Self {
        Self::from_inner(PyErrStateInner::Lazy(f))
    }

    pub(crate) fn lazy_arguments(ptype: Py<PyAny>, args: impl PyErrArguments + 'static) -> Self {
        Self::from_inner(PyErrStateInner::Lazy(Box::new(move |py| {
            PyErrStateLazyFnOutput {
                ptype,
                pvalue: args.arguments(py),
            }
        })))
    }

    pub(crate) fn normalized(normalized: PyErrStateNormalized) -> Self {
        let state = Self::from_inner(PyErrStateInner::Normalized(normalized));
        // This state is already normalized, by completing the Once immediately we avoid
        // reaching the `py.detach` in `make_normalized` which is less efficient
        // and introduces a GIL switch which could deadlock.
        // See https://github.com/PyO3/pyo3/issues/4764
        state.normalized.call_once(|| {});
        state
    }

    pub(crate) fn restore(self, py: Python<'_>) {
        self.inner
            .into_inner()
            .expect("PyErr state should never be invalid outside of normalization")
            .restore(py)
    }

    fn from_inner(inner: PyErrStateInner) -> Self {
        Self {
            normalized: Once::new(),
            normalizing_thread: Mutex::new(None),
            inner: UnsafeCell::new(Some(inner)),
        }
    }

    #[inline]
    pub(crate) fn as_normalized(&self, py: Python<'_>) -> &PyErrStateNormalized {
        if self.normalized.is_completed() {
            match unsafe {
                // Safety: self.inner will never be written again once normalized.
                &*self.inner.get()
            } {
                Some(PyErrStateInner::Normalized(n)) => return n,
                _ => unreachable!(),
            }
        }

        self.make_normalized(py)
    }

    #[cold]
    fn make_normalized(&self, py: Python<'_>) -> &PyErrStateNormalized {
        // This process is safe because:
        // - Write happens only once, and then never will change again.
        // - The `Once` ensure that only one thread will do the write.

        // Guard against re-entrant normalization, because `Once` does not provide
        // re-entrancy guarantees.
        if let Some(thread) = self.normalizing_thread.lock().unwrap().as_ref() {
            assert!(
                !(*thread == std::thread::current().id()),
                "Re-entrant normalization of PyErrState detected"
            );
        }

        // avoid deadlock of `.call_once` with the GIL
        py.detach(|| {
            self.normalized.call_once(|| {
                self.normalizing_thread
                    .lock()
                    .unwrap()
                    .replace(std::thread::current().id());

                // Safety: no other thread can access the inner value while we are normalizing it.
                let state = unsafe {
                    (*self.inner.get())
                        .take()
                        .expect("Cannot normalize a PyErr while already normalizing it.")
                };

                let normalized_state =
                    Python::attach(|py| PyErrStateInner::Normalized(state.normalize(py)));

                // Safety: no other thread can access the inner value while we are normalizing it.
                unsafe {
                    *self.inner.get() = Some(normalized_state);
                }
            })
        });

        match unsafe {
            // Safety: self.inner will never be written again once normalized.
            &*self.inner.get()
        } {
            Some(PyErrStateInner::Normalized(n)) => n,
            _ => unreachable!(),
        }
    }
}

pub(crate) struct PyErrStateNormalized {
    #[cfg(not(Py_3_12))]
    ptype: Py<PyType>,
    pub pvalue: Py<PyBaseException>,
    #[cfg(not(Py_3_12))]
    ptraceback: std::sync::Mutex<Option<Py<PyTraceback>>>,
}

impl PyErrStateNormalized {
    pub(crate) fn new(pvalue: Bound<'_, PyBaseException>) -> Self {
        Self {
            #[cfg(not(Py_3_12))]
            ptype: pvalue.get_type().into(),
            #[cfg(not(Py_3_12))]
            ptraceback: unsafe {
                Mutex::new(
                    ffi::PyException_GetTraceback(pvalue.as_ptr())
                        .assume_owned_or_opt(pvalue.py())
                        .map(|b| b.cast_into_unchecked().unbind()),
                )
            },
            pvalue: pvalue.into(),
        }
    }

    #[cfg(not(Py_3_12))]
    pub(crate) fn ptype<'py>(&self, py: Python<'py>) -> Bound<'py, PyType> {
        self.ptype.bind(py).clone()
    }

    #[cfg(Py_3_12)]
    pub(crate) fn ptype<'py>(&self, py: Python<'py>) -> Bound<'py, PyType> {
        self.pvalue.bind(py).get_type()
    }

    #[cfg(not(Py_3_12))]
    pub(crate) fn ptraceback<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyTraceback>> {
        self.ptraceback
            .lock_py_attached(py)
            .unwrap()
            .as_ref()
            .map(|traceback| traceback.bind(py).clone())
    }

    #[cfg(Py_3_12)]
    pub(crate) fn ptraceback<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyTraceback>> {
        unsafe {
            ffi::PyException_GetTraceback(self.pvalue.as_ptr())
                .assume_owned_or_opt(py)
                .map(|b| b.cast_into_unchecked())
        }
    }

    #[cfg(not(Py_3_12))]
    pub(crate) fn set_ptraceback<'py>(&self, py: Python<'py>, tb: Option<Bound<'py, PyTraceback>>) {
        *self.ptraceback.lock_py_attached(py).unwrap() = tb.map(Bound::unbind);
    }

    #[cfg(Py_3_12)]
    pub(crate) fn set_ptraceback<'py>(&self, py: Python<'py>, tb: Option<Bound<'py, PyTraceback>>) {
        let tb = tb
            .as_ref()
            .map(Bound::as_ptr)
            .unwrap_or_else(|| crate::types::PyNone::get(py).as_ptr());

        unsafe { ffi::PyException_SetTraceback(self.pvalue.as_ptr(), tb) };
    }

    pub(crate) fn take(py: Python<'_>) -> Option<PyErrStateNormalized> {
        #[cfg(Py_3_12)]
        {
            // Safety: PyErr_GetRaisedException can be called when attached to Python and
            // returns either NULL or an owned reference.
            unsafe { ffi::PyErr_GetRaisedException().assume_owned_or_opt(py) }.map(|pvalue| {
                PyErrStateNormalized {
                    // Safety: PyErr_GetRaisedException returns a valid exception type.
                    pvalue: unsafe { pvalue.cast_into_unchecked() }.unbind(),
                }
            })
        }

        #[cfg(not(Py_3_12))]
        {
            let (ptype, pvalue, ptraceback) = unsafe {
                let mut ptype: *mut ffi::PyObject = std::ptr::null_mut();
                let mut pvalue: *mut ffi::PyObject = std::ptr::null_mut();
                let mut ptraceback: *mut ffi::PyObject = std::ptr::null_mut();

                ffi::PyErr_Fetch(&mut ptype, &mut pvalue, &mut ptraceback);

                // Ensure that the exception coming from the interpreter is normalized.
                if !ptype.is_null() {
                    ffi::PyErr_NormalizeException(&mut ptype, &mut pvalue, &mut ptraceback);
                }

                // Safety: PyErr_NormalizeException will have produced up to three owned
                // references of the correct types.
                (
                    ptype
                        .assume_owned_or_opt(py)
                        .map(|b| b.cast_into_unchecked()),
                    pvalue
                        .assume_owned_or_opt(py)
                        .map(|b| b.cast_into_unchecked()),
                    ptraceback
                        .assume_owned_or_opt(py)
                        .map(|b| b.cast_into_unchecked()),
                )
            };

            ptype.map(|ptype| PyErrStateNormalized {
                ptype: ptype.unbind(),
                pvalue: pvalue.expect("normalized exception value missing").unbind(),
                ptraceback: std::sync::Mutex::new(ptraceback.map(Bound::unbind)),
            })
        }
    }

    #[cfg(not(Py_3_12))]
    unsafe fn from_normalized_ffi_tuple(
        py: Python<'_>,
        ptype: *mut ffi::PyObject,
        pvalue: *mut ffi::PyObject,
        ptraceback: *mut ffi::PyObject,
    ) -> Self {
        PyErrStateNormalized {
            ptype: unsafe {
                ptype
                    .assume_owned_or_opt(py)
                    .expect("Exception type missing")
                    .cast_into_unchecked()
            }
            .unbind(),
            pvalue: unsafe {
                pvalue
                    .assume_owned_or_opt(py)
                    .expect("Exception value missing")
                    .cast_into_unchecked()
            }
            .unbind(),
            ptraceback: Mutex::new(
                unsafe { ptraceback.assume_owned_or_opt(py) }
                    .map(|b| unsafe { b.cast_into_unchecked() }.unbind()),
            ),
        }
    }

    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        Self {
            #[cfg(not(Py_3_12))]
            ptype: self.ptype.clone_ref(py),
            pvalue: self.pvalue.clone_ref(py),
            #[cfg(not(Py_3_12))]
            ptraceback: std::sync::Mutex::new(
                self.ptraceback
                    .lock_py_attached(py)
                    .unwrap()
                    .as_ref()
                    .map(|ptraceback| ptraceback.clone_ref(py)),
            ),
        }
    }
}

pub(crate) struct PyErrStateLazyFnOutput {
    pub(crate) ptype: Py<PyAny>,
    pub(crate) pvalue: Py<PyAny>,
}

pub(crate) type PyErrStateLazyFn =
    dyn for<'py> FnOnce(Python<'py>) -> PyErrStateLazyFnOutput + Send + Sync;

enum PyErrStateInner {
    Lazy(Box<PyErrStateLazyFn>),
    Normalized(PyErrStateNormalized),
}

impl PyErrStateInner {
    fn normalize(self, py: Python<'_>) -> PyErrStateNormalized {
        match self {
            #[cfg(not(Py_3_12))]
            PyErrStateInner::Lazy(lazy) => {
                let (ptype, pvalue, ptraceback) = lazy_into_normalized_ffi_tuple(py, lazy);
                unsafe {
                    PyErrStateNormalized::from_normalized_ffi_tuple(py, ptype, pvalue, ptraceback)
                }
            }
            #[cfg(Py_3_12)]
            PyErrStateInner::Lazy(lazy) => {
                // To keep the implementation simple, just write the exception into the interpreter,
                // which will cause it to be normalized
                raise_lazy(py, lazy);
                PyErrStateNormalized::take(py)
                    .expect("exception missing after writing to the interpreter")
            }
            PyErrStateInner::Normalized(normalized) => normalized,
        }
    }

    #[cfg(not(Py_3_12))]
    fn restore(self, py: Python<'_>) {
        let (ptype, pvalue, ptraceback) = match self {
            PyErrStateInner::Lazy(lazy) => lazy_into_normalized_ffi_tuple(py, lazy),
            PyErrStateInner::Normalized(PyErrStateNormalized {
                ptype,
                pvalue,
                ptraceback,
            }) => (
                ptype.into_ptr(),
                pvalue.into_ptr(),
                ptraceback
                    .into_inner()
                    .unwrap()
                    .map_or(std::ptr::null_mut(), Py::into_ptr),
            ),
        };
        unsafe { ffi::PyErr_Restore(ptype, pvalue, ptraceback) }
    }

    #[cfg(Py_3_12)]
    fn restore(self, py: Python<'_>) {
        match self {
            PyErrStateInner::Lazy(lazy) => raise_lazy(py, lazy),
            PyErrStateInner::Normalized(PyErrStateNormalized { pvalue }) => unsafe {
                ffi::PyErr_SetRaisedException(pvalue.into_ptr())
            },
        }
    }
}

#[cfg(not(Py_3_12))]
fn lazy_into_normalized_ffi_tuple(
    py: Python<'_>,
    lazy: Box<PyErrStateLazyFn>,
) -> (*mut ffi::PyObject, *mut ffi::PyObject, *mut ffi::PyObject) {
    // To be consistent with 3.12 logic, go via raise_lazy, but also then normalize
    // the resulting exception
    raise_lazy(py, lazy);
    let mut ptype = std::ptr::null_mut();
    let mut pvalue = std::ptr::null_mut();
    let mut ptraceback = std::ptr::null_mut();
    unsafe {
        ffi::PyErr_Fetch(&mut ptype, &mut pvalue, &mut ptraceback);
        ffi::PyErr_NormalizeException(&mut ptype, &mut pvalue, &mut ptraceback);
    }
    (ptype, pvalue, ptraceback)
}

/// Raises a "lazy" exception state into the Python interpreter.
fn raise_lazy(py: Python<'_>, lazy: Box<PyErrStateLazyFn>) {
    let PyErrStateLazyFnOutput { ptype, pvalue } = lazy(py);

    unsafe {
        #[cfg(not(Py_3_12))]
        if ffi::PyExceptionClass_Check(ptype.as_ptr()) == 0 {
            ffi::PyErr_SetString(
                PyTypeError::type_object_raw(py).cast(),
                c"exceptions must derive from BaseException".as_ptr(),
            );
        } else {
            ffi::PyErr_SetObject(ptype.as_ptr(), pvalue.as_ptr());
        }

        #[cfg(Py_3_12)]
        {
            let exc = create_normalized_exception(ptype.bind(py), pvalue.into_bound(py));

            ffi::PyErr_SetRaisedException(exc.into_ptr());
        }
    }
}

#[cfg(Py_3_12)]
fn create_normalized_exception<'py>(
    ptype: &Bound<'py, PyAny>,
    mut pvalue: Bound<'py, PyAny>,
) -> Bound<'py, PyBaseException> {
    let py = ptype.py();

    // 1: check type is a subclass of BaseException
    let ptype: Bound<'py, PyType> = if unsafe { ffi::PyExceptionClass_Check(ptype.as_ptr()) } == 0 {
        pvalue = PyString::new(py, "exceptions must derive from BaseException").into_any();
        PyTypeError::type_object(py)
    } else {
        // Safety: PyExceptionClass_Check guarantees that ptype is a subclass of BaseException
        unsafe { ptype.cast_unchecked() }.clone()
    };

    let mut current_handled_exception: Option<Bound<'_, PyBaseException>> = unsafe {
        ffi::PyErr_GetHandledException()
            .assume_owned_or_opt(py)
            .map(|obj| obj.cast_into_unchecked())
    };

    let pvalue = if pvalue.is_exact_instance(&ptype) {
        // Safety: already an exception value of the correct type
        let exc = unsafe { pvalue.cast_into_unchecked::<PyBaseException>() };

        if current_handled_exception
            .as_ref()
            .map(|current| current.is(&exc))
            .unwrap_or_default()
        {
            // Current exception is the same as it's context so do not set the context to avoid a loop
            current_handled_exception = None;
        } else if let Some(current_context) = current_handled_exception.as_ref() {
            // Check if this exception is already in the context chain, so we do not create reference cycles in the context chain.
            let mut iter = context_chain_iter(current_context.clone()).peekable();
            while let Some((current, next)) = iter.next().zip(iter.peek()) {
                if next.is(&exc) {
                    // Loop in context chain, breaking the loop by not pointing to exc
                    unsafe { ffi::PyException_SetContext(current.as_ptr(), std::ptr::null_mut()) };
                    break;
                }
            }
        }

        Ok(exc)
    } else if pvalue.is_none() {
        // None -> no arguments
        ptype.call0().and_then(|pvalue| Ok(pvalue.cast_into()?))
    } else if let Ok(tup) = pvalue.cast::<PyTuple>() {
        // Tuple -> use as tuple of arguments
        ptype.call1(tup).and_then(|pvalue| Ok(pvalue.cast_into()?))
    } else {
        // Anything else -> use as single argument
        ptype
            .call1((pvalue,))
            .and_then(|pvalue| Ok(pvalue.cast_into()?))
    };

    match pvalue {
        Ok(pvalue) => {
            // Implicitly set the context of the new exception to the currently handled exception, if any.
            if let Some(context) = current_handled_exception {
                unsafe { ffi::PyException_SetContext(pvalue.as_ptr(), context.into_ptr()) };
            }
            pvalue
        }
        Err(e) => e.value(py).clone(),
    }
}

/// Iterates through the context chain of exceptions, starting from `start`, and yields each exception in the chain.
/// When there is a loop in the chain it may yield some elements multiple times, but it will always terminate.
#[inline]
#[cfg(Py_3_12)]
fn context_chain_iter(
    start: Bound<'_, PyBaseException>,
) -> impl Iterator<Item = Bound<'_, PyBaseException>> {
    #[inline]
    fn get_next<'py>(current: &Bound<'py, PyBaseException>) -> Option<Bound<'py, PyBaseException>> {
        unsafe {
            ffi::PyException_GetContext(current.as_ptr())
                .assume_owned_or_opt(current.py())
                .map(|obj| obj.cast_into_unchecked())
        }
    }

    let mut slow = None;
    let mut current = Some(start);
    let mut slow_update_toggle = false;

    std::iter::from_fn(move || {
        let next = get_next(current.as_ref()?);

        // Detect loops in the context chain using Floyd's Tortoise and Hare algorithm.
        if let Some((current_slow, current_fast)) = slow.as_ref().zip(next.as_ref()) {
            if current_fast.is(current_slow) {
                // Loop detected
                return current.take();
            }

            // Every second iteration, advance the slow pointer by one step
            if slow_update_toggle {
                slow = get_next(current_slow);
            }

            slow_update_toggle = !slow_update_toggle;
        }

        // Set the slow pointer after the first iteration
        if slow.is_none() {
            slow = current.clone()
        }

        std::mem::replace(&mut current, next)
    })
}

#[cfg(test)]
mod tests {
    #[cfg(Py_3_12)]
    use crate::{exceptions::PyBaseException, ffi, Bound};
    use crate::{
        exceptions::PyValueError, sync::PyOnceLock, Py, PyAny, PyErr, PyErrArguments, Python,
    };

    #[test]
    #[should_panic(expected = "Re-entrant normalization of PyErrState detected")]
    fn test_reentrant_normalization() {
        static ERR: PyOnceLock<PyErr> = PyOnceLock::new();

        struct RecursiveArgs;

        impl PyErrArguments for RecursiveArgs {
            fn arguments(self, py: Python<'_>) -> Py<PyAny> {
                // .value(py) triggers normalization
                ERR.get(py)
                    .expect("is set just below")
                    .value(py)
                    .clone()
                    .into()
            }
        }

        Python::attach(|py| {
            ERR.set(py, PyValueError::new_err(RecursiveArgs)).unwrap();
            ERR.get(py).expect("is set just above").value(py);
        })
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    fn test_no_deadlock_thread_switch() {
        static ERR: PyOnceLock<PyErr> = PyOnceLock::new();

        struct GILSwitchArgs;

        impl PyErrArguments for GILSwitchArgs {
            fn arguments(self, py: Python<'_>) -> Py<PyAny> {
                // releasing the GIL potentially allows for other threads to deadlock
                // with the normalization going on here
                py.detach(|| {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                });
                py.None()
            }
        }

        Python::attach(|py| ERR.set(py, PyValueError::new_err(GILSwitchArgs)).unwrap());

        // Let many threads attempt to read the normalized value at the same time
        let handles = (0..10)
            .map(|_| {
                std::thread::spawn(|| {
                    Python::attach(|py| {
                        ERR.get(py).expect("is set just above").value(py);
                    });
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }

        // We should never have deadlocked, and should be able to run
        // this assertion
        Python::attach(|py| {
            assert!(ERR
                .get(py)
                .expect("is set above")
                .is_instance_of::<PyValueError>(py))
        });
    }

    #[test]
    #[cfg(feature = "macros")]
    fn test_new_exception_context() {
        use crate::{
            exceptions::{PyRuntimeError, PyValueError},
            pyfunction,
            types::{PyDict, PyDictMethods},
            wrap_pyfunction, PyResult,
        };
        #[pyfunction(crate = "crate")]
        fn throw_exception() -> PyResult<()> {
            Err(PyValueError::new_err("error happened"))
        }

        Python::attach(|py| {
            let globals = PyDict::new(py);
            let f = wrap_pyfunction!(throw_exception, py).unwrap();
            globals.set_item("throw_exception", f).unwrap();
            let err = py
                .run(
                    c"try:\n  raise RuntimeError()\nexcept:\n  throw_exception()\n",
                    Some(&globals),
                    None,
                )
                .unwrap_err();

            let context = err.context(py).unwrap();
            assert!(context.is_instance_of::<PyRuntimeError>(py))
        })
    }

    #[test]
    #[cfg(Py_3_12)]
    fn compare_create_normalized_exception_with_pyerr_setobject() {
        use crate::{
            conversion::IntoPyObjectExt, err::err_state::PyErrStateNormalized,
            exceptions::PyRuntimeError, ffi, type_object::PyTypeInfo, types::any::PyAnyMethods,
            Bound,
        };

        fn test_exception<'py>(
            ptype: &Bound<'py, PyAny>,
            pvalue: Bound<'py, PyAny>,
        ) -> (PyErr, PyErr) {
            let py = ptype.py();

            let exc1 = super::create_normalized_exception(ptype, pvalue.clone());

            unsafe {
                ffi::PyErr_SetObject(ptype.as_ptr(), pvalue.as_ptr());
            }
            let exc2 = PyErrStateNormalized::take(py)
                .unwrap()
                .pvalue
                .into_bound(py);

            let err1 = PyErr::from_value(exc1.into_any());
            let err2 = PyErr::from_value(exc2.into_any());

            assert!(err1.get_type(py).is(err2.get_type(py)));
            assert!(err1.context(py).xor(err2.context(py)).is_none());
            assert!(err1.traceback(py).xor(err2.traceback(py)).is_none());
            assert!(err1.cause(py).xor(err2.cause(py)).is_none());
            assert_eq!(err1.to_string(), err2.to_string());

            super::context_chain_iter(err1.value(py).clone())
                .zip(super::context_chain_iter(err2.value(py).clone()))
                .for_each(|(context1, context2)| {
                    assert!(context1.get_type().is(context2.get_type()));
                    assert_eq!(context1.to_string(), context2.to_string());
                });

            (err1, err2)
        }

        Python::attach(|py| {
            test_exception(&PyRuntimeError::type_object(py), py.None().into_bound(py));

            test_exception(
                &PyRuntimeError::type_object(py),
                "Boom".into_bound_py_any(py).unwrap(),
            );

            test_exception(
                &PyRuntimeError::type_object(py),
                (3, 2, 1, "Boom").into_bound_py_any(py).unwrap(),
            );

            test_exception(
                &PyRuntimeError::type_object(py),
                PyRuntimeError::new_err("Boom")
                    .into_value(py)
                    .into_any()
                    .into_bound(py),
            );

            // Loop where err is not part of the loop
            let looped_context = create_loop(py, 3);
            let err = PyRuntimeError::new_err("Boom");
            with_handled_exception(looped_context.value(py), || {
                let (normalized, _) = test_exception(
                    &PyRuntimeError::type_object(py),
                    err.value(py).clone().into_any(),
                );

                assert!(normalized
                    .context(py)
                    .unwrap()
                    .value(py)
                    .is(looped_context.value(py)));
            });

            // loop where err is part of the loop
            let err_a = PyRuntimeError::new_err("A");
            let err_b = PyRuntimeError::new_err("B");
            // a -> b -> a
            err_a.set_context(py, Some(err_b.clone_ref(py)));
            err_b.set_context(py, Some(err_a.clone_ref(py)));
            // handled = raised = a
            with_handled_exception(err_a.value(py), || {
                let (rust_normal, py_normal) = test_exception(
                    &PyRuntimeError::type_object(py),
                    err_a.value(py).clone().into_any(),
                );

                // a.context -> b
                assert!(rust_normal
                    .context(py)
                    .unwrap()
                    .value(py)
                    .is(err_b.value(py)));
                assert!(py_normal.context(py).unwrap().value(py).is(err_b.value(py)));
            });

            // no loop yet, but implicit context will loop if we set a.context = b
            let err_a = PyRuntimeError::new_err("A");
            let err_b = PyRuntimeError::new_err("B");
            err_b.set_context(py, Some(err_a.clone_ref(py)));
            // raised = a, handled = b
            with_handled_exception(err_b.value(py), || {
                test_exception(
                    &PyRuntimeError::type_object(py),
                    err_b.value(py).clone().into_any(),
                );
            });
        })
    }

    #[cfg(Py_3_12)]
    fn with_handled_exception(exc: &Bound<'_, PyBaseException>, f: impl FnOnce()) {
        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) {
                unsafe { ffi::PyErr_SetHandledException(std::ptr::null_mut()) };
            }
        }

        let guard = Guard;
        unsafe { ffi::PyErr_SetHandledException(exc.as_ptr()) };
        f();
        drop(guard);
    }

    #[cfg(Py_3_12)]
    fn create_loop(py: Python<'_>, size: usize) -> PyErr {
        let first = PyValueError::new_err("exc0");
        let last = (1..size).fold(first.clone_ref(py), |prev, i| {
            let exc = PyValueError::new_err(format!("exc{i}"));
            prev.set_context(py, Some(exc.clone_ref(py)));
            exc
        });
        last.set_context(py, Some(first.clone_ref(py)));

        first
    }

    #[test]
    #[cfg(Py_3_12)]
    fn test_context_chain_iter_terminates() {
        Python::attach(|py| {
            for size in 1..=8 {
                let chain = create_loop(py, size);
                let count = super::context_chain_iter(chain.into_value(py).into_bound(py)).count();
                assert!(
                    count >= size,
                    "We should have seen each element at least once"
                );
            }
        })
    }
}
