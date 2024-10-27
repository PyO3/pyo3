use std::cell::UnsafeCell;

use crate::{
    exceptions::{PyBaseException, PyTypeError},
    ffi,
    ffi_ptr_ext::FfiPtrExt,
    types::{PyAnyMethods, PyTraceback, PyType},
    Bound, Py, PyAny, PyErrArguments, PyObject, PyTypeInfo, Python,
};

pub(crate) struct PyErrState {
    // Safety: can only hand out references when in the "normalized" state. Will never change
    // after normalization.
    //
    // The state is temporarily removed from the PyErr during normalization, to avoid
    // concurrent modifications.
    inner: UnsafeCell<Option<PyErrStateInner>>,
}

// The inner value is only accessed through ways that require the gil is held.
unsafe impl Send for PyErrState {}
unsafe impl Sync for PyErrState {}

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
        Self::from_inner(PyErrStateInner::Normalized(normalized))
    }

    pub(crate) fn restore(self, py: Python<'_>) {
        self.inner
            .into_inner()
            .expect("PyErr state should never be invalid outside of normalization")
            .restore(py)
    }

    fn from_inner(inner: PyErrStateInner) -> Self {
        Self {
            inner: UnsafeCell::new(Some(inner)),
        }
    }

    #[inline]
    pub(crate) fn as_normalized(&self, py: Python<'_>) -> &PyErrStateNormalized {
        if let Some(PyErrStateInner::Normalized(n)) = unsafe {
            // Safety: self.inner will never be written again once normalized.
            &*self.inner.get()
        } {
            return n;
        }

        self.make_normalized(py)
    }

    #[cold]
    fn make_normalized(&self, py: Python<'_>) -> &PyErrStateNormalized {
        // This process is safe because:
        // - Access is guaranteed not to be concurrent thanks to `Python` GIL token
        // - Write happens only once, and then never will change again.
        // - State is set to None during the normalization process, so that a second
        //   concurrent normalization attempt will panic before changing anything.

        // FIXME: this needs to be rewritten to deal with free-threaded Python
        // see https://github.com/PyO3/pyo3/issues/4584

        let state = unsafe {
            (*self.inner.get())
                .take()
                .expect("Cannot normalize a PyErr while already normalizing it.")
        };

        unsafe {
            let self_state = &mut *self.inner.get();
            *self_state = Some(PyErrStateInner::Normalized(state.normalize(py)));
            match self_state {
                Some(PyErrStateInner::Normalized(n)) => n,
                _ => unreachable!(),
            }
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
                .map(|b| b.downcast_into_unchecked())
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
                    pvalue: unsafe { pvalue.downcast_into_unchecked() }.unbind(),
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
                        .map(|b| b.downcast_into_unchecked()),
                    pvalue
                        .assume_owned_or_opt(py)
                        .map(|b| b.downcast_into_unchecked()),
                    ptraceback
                        .assume_owned_or_opt(py)
                        .map(|b| b.downcast_into_unchecked()),
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
            ptype: Py::from_owned_ptr_or_opt(py, ptype).expect("Exception type missing"),
            pvalue: Py::from_owned_ptr_or_opt(py, pvalue).expect("Exception value missing"),
            ptraceback: Py::from_owned_ptr_or_opt(py, ptraceback),
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
    pub(crate) ptype: PyObject,
    pub(crate) pvalue: PyObject,
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
