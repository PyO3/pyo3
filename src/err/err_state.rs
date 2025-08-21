use std::{
    cell::UnsafeCell,
    sync::{Mutex, Once},
    thread::ThreadId,
};

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
        // - Access is guaranteed not to be concurrent thanks to `Python` GIL token
        // - Write happens only once, and then never will change again.

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
    ptraceback: Option<Py<PyTraceback>>,
}

impl PyErrStateNormalized {
    pub(crate) fn new(pvalue: Bound<'_, PyBaseException>) -> Self {
        Self {
            #[cfg(not(Py_3_12))]
            ptype: pvalue.get_type().into(),
            #[cfg(not(Py_3_12))]
            ptraceback: unsafe {
                Py::from_owned_ptr_or_opt(
                    pvalue.py(),
                    ffi::PyException_GetTraceback(pvalue.as_ptr()),
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
                ptraceback: ptraceback.map(Bound::unbind),
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
            ptype: unsafe { Py::from_owned_ptr_or_opt(py, ptype).expect("Exception type missing") },
            pvalue: unsafe {
                Py::from_owned_ptr_or_opt(py, pvalue).expect("Exception value missing")
            },
            ptraceback: unsafe { Py::from_owned_ptr_or_opt(py, ptraceback) },
        }
    }

    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        Self {
            #[cfg(not(Py_3_12))]
            ptype: self.ptype.clone_ref(py),
            pvalue: self.pvalue.clone_ref(py),
            #[cfg(not(Py_3_12))]
            ptraceback: self
                .ptraceback
                .as_ref()
                .map(|ptraceback| ptraceback.clone_ref(py)),
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
                ptraceback.map_or(std::ptr::null_mut(), Py::into_ptr),
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
///
/// In principle this could be split in two; first a function to create an exception
/// in a normalized state, and then a call to `PyErr_SetRaisedException` to raise it.
///
/// This would require either moving some logic from C to Rust, or requesting a new
/// API in CPython.
fn raise_lazy(py: Python<'_>, lazy: Box<PyErrStateLazyFn>) {
    let PyErrStateLazyFnOutput { ptype, pvalue } = lazy(py);
    unsafe {
        if ffi::PyExceptionClass_Check(ptype.as_ptr()) == 0 {
            ffi::PyErr_SetString(
                PyTypeError::type_object_raw(py).cast(),
                ffi::c_str!("exceptions must derive from BaseException").as_ptr(),
            )
        } else {
            ffi::PyErr_SetObject(ptype.as_ptr(), pvalue.as_ptr())
        }
    }
}

#[cfg(test)]
mod tests {

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
}
