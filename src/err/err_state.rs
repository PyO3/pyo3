use crate::{
    exceptions::{PyBaseException, PyTypeError},
    ffi,
    types::{PyTraceback, PyType},
    AsPyPointer, IntoPy, IntoPyPointer, Py, PyAny, PyObject, PyTypeInfo, Python,
};

#[derive(Clone)]
pub(crate) struct PyErrStateNormalized {
    pub ptype: Py<PyType>,
    pub pvalue: Py<PyBaseException>,
    pub ptraceback: Option<Py<PyTraceback>>,
}

pub(crate) struct PyErrStateLazyFnOutput {
    pub(crate) ptype: PyObject,
    pub(crate) pvalue: PyObject,
}

pub(crate) type PyErrStateLazyFn =
    dyn for<'py> FnOnce(Python<'py>) -> PyErrStateLazyFnOutput + Send + Sync;

pub(crate) enum PyErrState {
    Lazy(Box<PyErrStateLazyFn>),
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
}

impl PyErrState {
    pub(crate) fn into_ffi_tuple(
        self,
        py: Python<'_>,
    ) -> (*mut ffi::PyObject, *mut ffi::PyObject, *mut ffi::PyObject) {
        match self {
            PyErrState::Lazy(lazy) => {
                let PyErrStateLazyFnOutput { ptype, pvalue } = lazy(py);
                if unsafe { ffi::PyExceptionClass_Check(ptype.as_ptr()) } == 0 {
                    Self::exceptions_must_derive_from_base_exception(py).into_ffi_tuple(py)
                } else {
                    (ptype.into_ptr(), pvalue.into_ptr(), std::ptr::null_mut())
                }
            }
            PyErrState::FfiTuple {
                ptype,
                pvalue,
                ptraceback,
            } => (ptype.into_ptr(), pvalue.into_ptr(), ptraceback.into_ptr()),
            PyErrState::Normalized(PyErrStateNormalized {
                ptype,
                pvalue,
                ptraceback,
            }) => (ptype.into_ptr(), pvalue.into_ptr(), ptraceback.into_ptr()),
        }
    }

    fn exceptions_must_derive_from_base_exception(py: Python<'_>) -> Self {
        PyErrState::lazy(
            PyTypeError::type_object(py),
            "exceptions must derive from BaseException",
        )
    }
}
