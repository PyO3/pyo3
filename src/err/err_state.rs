use crate::{
    exceptions::{PyBaseException, PyTypeError},
    ffi,
    types::{PyTraceback, PyType},
    IntoPy, Py, PyAny, PyObject, PyTypeInfo, Python,
};

#[derive(Clone)]
pub(crate) struct PyErrStateNormalized {
    #[cfg(not(Py_3_12))]
    pub ptype: Py<PyType>,
    pub pvalue: Py<PyBaseException>,
    #[cfg(not(Py_3_12))]
    pub ptraceback: Option<Py<PyTraceback>>,
}

impl PyErrStateNormalized {
    #[cfg(not(Py_3_12))]
    pub(crate) fn ptype<'py>(&'py self, py: Python<'py>) -> &'py PyType {
        self.ptype.as_ref(py)
    }

    #[cfg(Py_3_12)]
    pub(crate) fn ptype<'py>(&'py self, py: Python<'py>) -> &'py PyType {
        self.pvalue.as_ref(py).get_type()
    }

    #[cfg(not(Py_3_12))]
    pub(crate) fn ptraceback<'py>(&'py self, py: Python<'py>) -> Option<&'py PyTraceback> {
        self.ptraceback
            .as_ref()
            .map(|traceback| traceback.as_ref(py))
    }

    #[cfg(Py_3_12)]
    pub(crate) fn ptraceback<'py>(&'py self, py: Python<'py>) -> Option<&'py PyTraceback> {
        unsafe { py.from_owned_ptr_or_opt(ffi::PyException_GetTraceback(self.pvalue.as_ptr())) }
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
    pub(crate) fn lazy(ptype: &PyAny, args: impl PyErrArguments + 'static) -> Self {
        let ptype = ptype.into();
        PyErrState::Lazy(Box::new(move |py| PyErrStateLazyFnOutput {
            ptype,
            pvalue: args.arguments(py),
        }))
    }

    pub(crate) fn normalized(pvalue: &PyBaseException) -> Self {
        Self::Normalized(PyErrStateNormalized {
            #[cfg(not(Py_3_12))]
            ptype: pvalue.get_type().into(),
            pvalue: pvalue.into(),
            #[cfg(not(Py_3_12))]
            ptraceback: unsafe {
                Py::from_owned_ptr_or_opt(
                    pvalue.py(),
                    ffi::PyException_GetTraceback(pvalue.as_ptr()),
                )
            },
        })
    }

    #[cfg(not(Py_3_12))]
    pub(crate) fn into_ffi_tuple(
        self,
        py: Python<'_>,
    ) -> (*mut ffi::PyObject, *mut ffi::PyObject, *mut ffi::PyObject) {
        match self {
            PyErrState::Lazy(lazy) => {
                let PyErrStateLazyFnOutput { ptype, pvalue } = lazy(py);
                if unsafe { ffi::PyExceptionClass_Check(ptype.as_ptr()) } == 0 {
                    PyErrState::lazy(
                        PyTypeError::type_object(py),
                        "exceptions must derive from BaseException",
                    )
                    .into_ffi_tuple(py)
                } else {
                    (ptype.into_ptr(), pvalue.into_ptr(), std::ptr::null_mut())
                }
            }
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
        }
    }

    #[cfg(not(Py_3_12))]
    pub(crate) fn normalize(self, py: Python<'_>) -> PyErrStateNormalized {
        let (mut ptype, mut pvalue, mut ptraceback) = self.into_ffi_tuple(py);

        unsafe {
            ffi::PyErr_NormalizeException(&mut ptype, &mut pvalue, &mut ptraceback);
            PyErrStateNormalized {
                ptype: Py::from_owned_ptr_or_opt(py, ptype).expect("Exception type missing"),
                pvalue: Py::from_owned_ptr_or_opt(py, pvalue).expect("Exception value missing"),
                ptraceback: Py::from_owned_ptr_or_opt(py, ptraceback),
            }
        }
    }

    #[cfg(Py_3_12)]
    pub(crate) fn normalize(self, py: Python<'_>) -> PyErrStateNormalized {
        // To keep the implementation simple, just write the exception into the interpreter,
        // which will cause it to be normalized
        self.restore(py);
        // Safety: self.restore(py) will set the raised exception
        let pvalue = unsafe { Py::from_owned_ptr(py, ffi::PyErr_GetRaisedException()) };
        PyErrStateNormalized { pvalue }
    }

    #[cfg(not(Py_3_12))]
    pub(crate) fn restore(self, py: Python<'_>) {
        let (ptype, pvalue, ptraceback) = self.into_ffi_tuple(py);
        unsafe { ffi::PyErr_Restore(ptype, pvalue, ptraceback) }
    }

    #[cfg(Py_3_12)]
    pub(crate) fn restore(self, py: Python<'_>) {
        match self {
            PyErrState::Lazy(lazy) => {
                let PyErrStateLazyFnOutput { ptype, pvalue } = lazy(py);
                unsafe {
                    if ffi::PyExceptionClass_Check(ptype.as_ptr()) == 0 {
                        ffi::PyErr_SetString(
                            PyTypeError::type_object_raw(py).cast(),
                            "exceptions must derive from BaseException\0"
                                .as_ptr()
                                .cast(),
                        )
                    } else {
                        ffi::PyErr_SetObject(ptype.as_ptr(), pvalue.as_ptr())
                    }
                }
            }
            PyErrState::Normalized(PyErrStateNormalized { pvalue }) => unsafe {
                ffi::PyErr_SetRaisedException(pvalue.into_ptr())
            },
        }
    }
}
