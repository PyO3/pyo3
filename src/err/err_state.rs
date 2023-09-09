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
    fn from_value(pvalue: &PyBaseException) -> Self {
        Self {
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
        }
    }

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
        Self::Normalized(PyErrStateNormalized::from_value(pvalue))
    }

    pub(crate) fn normalize(self, py: Python<'_>) -> PyErrStateNormalized {
        use crate::{types::PyTuple, PyResult};

        match self {
            PyErrState::Lazy(lazy) => {
                fn exceptions_must_derive_from_base_exception(
                    py: Python<'_>,
                ) -> PyResult<&PyBaseException> {
                    PyTypeError::type_object(py)
                        .call1(("exceptions must derive from BaseException",))
                        .map(|any| unsafe { any.downcast_unchecked::<PyBaseException>() })
                }

                let PyErrStateLazyFnOutput { ptype, pvalue } = lazy(py);
                let result = if unsafe { ffi::PyExceptionClass_Check(ptype.as_ptr()) } == 0 {
                    exceptions_must_derive_from_base_exception(py)
                } else {
                    // already an exception instance
                    let result = if let Ok(base_exc) = pvalue.downcast::<PyBaseException>(py) {
                        return PyErrStateNormalized::from_value(base_exc);
                    } else if pvalue.is_none(py) {
                        ptype.as_ref(py).call0()
                    } else if let Ok(tup) = pvalue.as_ref(py).downcast::<PyTuple>() {
                        ptype.as_ref(py).call1(tup)
                    } else {
                        ptype.as_ref(py).call1((pvalue,))
                    };
                    result.and_then(|any| match any.downcast::<PyBaseException>() {
                        Ok(base_exc) => Ok(base_exc),
                        Err(_) => exceptions_must_derive_from_base_exception(py),
                    })
                };

                match result {
                    Ok(base_exc) => PyErrStateNormalized::from_value(base_exc),
                    Err(e) => e
                        .state
                        .into_inner()
                        .expect("exception is not being normalized")
                        .normalize(py),
                }
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
                    PyErrStateNormalized {
                        ptype: Py::from_owned_ptr_or_opt(py, ptype)
                            .expect("Exception type missing"),
                        pvalue: Py::from_owned_ptr_or_opt(py, pvalue)
                            .expect("Exception value missing"),
                        ptraceback: Py::from_owned_ptr_or_opt(py, ptraceback),
                    }
                }
            }
            PyErrState::Normalized(normalized) => normalized,
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
            #[cfg(not(Py_3_12))]
            PyErrState::FfiTuple {
                ptype,
                pvalue,
                ptraceback,
            } => unsafe {
                ffi::PyErr_Restore(
                    ptype.into_ptr(),
                    pvalue.map_or(std::ptr::null_mut(), Py::into_ptr),
                    ptraceback.map_or(std::ptr::null_mut(), Py::into_ptr),
                )
            },
            PyErrState::Normalized(PyErrStateNormalized {
                #[cfg(not(Py_3_12))]
                ptype,
                pvalue,
                #[cfg(not(Py_3_12))]
                ptraceback,
            }) => unsafe {
                #[cfg(not(Py_3_12))]
                {
                    ffi::PyErr_Restore(
                        ptype.into_ptr(),
                        pvalue.into_ptr(),
                        ptraceback.map_or(std::ptr::null_mut(), Py::into_ptr),
                    )
                }

                // FIXME if the exception has no traceback, we should probably add one
                // FIXME if sys.exc_info is set (i.e. an exception is being handled),
                //   we should chain it.

                #[cfg(Py_3_12)]
                {
                    ffi::PyErr_SetRaisedException(pvalue.into_ptr())
                }
            },
        }
    }
}
