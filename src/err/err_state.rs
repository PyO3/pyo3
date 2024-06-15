use crate::{
    exceptions::{PyBaseException, PyTypeError},
    ffi,
    types::{PyTraceback, PyType},
    Bound, IntoPy, Py, PyAny, PyObject, PyTypeInfo, Python,
};

pub(crate) struct PyErrStateNormalized {
    #[cfg(not(Py_3_12))]
    ptype: Py<PyType>,
    pub pvalue: Py<PyBaseException>,
    #[cfg(not(Py_3_12))]
    ptraceback: Option<Py<PyTraceback>>,
}

impl PyErrStateNormalized {
    #[cfg(not(Py_3_12))]
    pub(crate) fn ptype<'py>(&self, py: Python<'py>) -> Bound<'py, PyType> {
        self.ptype.bind(py).clone()
    }

    #[cfg(Py_3_12)]
    pub(crate) fn ptype<'py>(&self, py: Python<'py>) -> Bound<'py, PyType> {
        use crate::types::any::PyAnyMethods;
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
        use crate::ffi_ptr_ext::FfiPtrExt;
        use crate::types::any::PyAnyMethods;
        unsafe {
            ffi::PyException_GetTraceback(self.pvalue.as_ptr())
                .assume_owned_or_opt(py)
                .map(|b| b.downcast_into_unchecked())
        }
    }

    #[cfg(Py_3_12)]
    pub(crate) fn take(py: Python<'_>) -> Option<PyErrStateNormalized> {
        unsafe { Py::from_owned_ptr_or_opt(py, ffi::PyErr_GetRaisedException()) }
            .map(|pvalue| PyErrStateNormalized { pvalue })
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

pub(crate) enum PyErrState {
    Lazy(Box<PyErrStateLazyFn>),
    #[cfg(not(Py_3_12))]
    FfiTuple {
        ptype: PyObject,
        pvalue: Option<PyObject>,
        ptraceback: Option<PyObject>,
    },
    Normalized(PyErrStateNormalized),
}

/// Helper conversion trait that allows to use custom arguments for lazy exception construction.
pub trait PyErrArguments: Send + Sync {
    /// Arguments for exception
    fn arguments(self, py: Python<'_>) -> PyObject;
}

impl<T> PyErrArguments for T
where
    T: IntoPy<PyObject> + Send + Sync,
{
    fn arguments(self, py: Python<'_>) -> PyObject {
        self.into_py(py)
    }
}

impl PyErrState {
    pub(crate) fn lazy(ptype: Py<PyAny>, args: impl PyErrArguments + 'static) -> Self {
        PyErrState::Lazy(Box::new(move |py| PyErrStateLazyFnOutput {
            ptype,
            pvalue: args.arguments(py),
        }))
    }

    pub(crate) fn normalized(pvalue: Bound<'_, PyBaseException>) -> Self {
        #[cfg(not(Py_3_12))]
        use crate::types::any::PyAnyMethods;

        Self::Normalized(PyErrStateNormalized {
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
        })
    }

    pub(crate) fn normalize(self, py: Python<'_>) -> PyErrStateNormalized {
        match self {
            #[cfg(not(Py_3_12))]
            PyErrState::Lazy(lazy) => {
                let (ptype, pvalue, ptraceback) = lazy_into_normalized_ffi_tuple(py, lazy);
                unsafe {
                    PyErrStateNormalized::from_normalized_ffi_tuple(py, ptype, pvalue, ptraceback)
                }
            }
            #[cfg(Py_3_12)]
            PyErrState::Lazy(lazy) => {
                // To keep the implementation simple, just write the exception into the interpreter,
                // which will cause it to be normalized
                raise_lazy(py, lazy);
                PyErrStateNormalized::take(py)
                    .expect("exception missing after writing to the interpreter")
            }
            #[cfg(not(Py_3_12))]
            PyErrState::FfiTuple {
                ptype,
                pvalue,
                ptraceback,
            } => {
                let mut ptype = ptype.into_ptr();
                let mut pvalue = pvalue.map_or(std::ptr::null_mut(), Py::into_ptr);
                let mut ptraceback = ptraceback.map_or(std::ptr::null_mut(), Py::into_ptr);
                unsafe {
                    ffi::PyErr_NormalizeException(&mut ptype, &mut pvalue, &mut ptraceback);
                    PyErrStateNormalized::from_normalized_ffi_tuple(py, ptype, pvalue, ptraceback)
                }
            }
            PyErrState::Normalized(normalized) => normalized,
        }
    }

    #[cfg(not(Py_3_12))]
    pub(crate) fn restore(self, py: Python<'_>) {
        let (ptype, pvalue, ptraceback) = match self {
            PyErrState::Lazy(lazy) => lazy_into_normalized_ffi_tuple(py, lazy),
            PyErrState::FfiTuple {
                ptype,
                pvalue,
                ptraceback,
            } => (
                ptype.into_ptr(),
                pvalue.map_or(std::ptr::null_mut(), Py::into_ptr),
                ptraceback.map_or(std::ptr::null_mut(), Py::into_ptr),
            ),
            PyErrState::Normalized(PyErrStateNormalized {
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
    pub(crate) fn restore(self, py: Python<'_>) {
        match self {
            PyErrState::Lazy(lazy) => raise_lazy(py, lazy),
            PyErrState::Normalized(PyErrStateNormalized { pvalue }) => unsafe {
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
