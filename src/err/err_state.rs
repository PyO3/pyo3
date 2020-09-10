use crate::{
    exceptions::PyBaseException, ffi, types::PyType, IntoPy, IntoPyPointer, Py, PyObject, Python,
};

#[derive(Clone)]
pub(crate) struct PyErrStateNormalized {
    pub ptype: Py<PyType>,
    pub pvalue: Py<PyBaseException>,
    pub ptraceback: Option<PyObject>,
}

pub(crate) enum PyErrState {
    Lazy {
        ptype: Py<PyType>,
        pvalue: Box<dyn FnOnce(Python) -> PyObject + Send + Sync>,
    },
    FfiTuple {
        ptype: Option<PyObject>,
        pvalue: Option<PyObject>,
        ptraceback: Option<PyObject>,
    },
    Normalized(PyErrStateNormalized),
}

/// Helper conversion trait that allows to use custom arguments for lazy exception construction.
pub trait PyErrArguments: Send + Sync {
    /// Arguments for exception
    fn arguments(self, py: Python) -> PyObject;
}

impl<T> PyErrArguments for T
where
    T: IntoPy<PyObject> + Send + Sync,
{
    fn arguments(self, py: Python) -> PyObject {
        self.into_py(py)
    }
}

pub(crate) fn boxed_args(
    args: impl PyErrArguments + 'static,
) -> Box<dyn FnOnce(Python) -> PyObject + Send + Sync> {
    Box::new(|py| args.arguments(py))
}

impl PyErrState {
    pub(crate) fn into_ffi_tuple(
        self,
        py: Python,
    ) -> (*mut ffi::PyObject, *mut ffi::PyObject, *mut ffi::PyObject) {
        match self {
            PyErrState::Lazy { ptype, pvalue } => (
                ptype.into_ptr(),
                pvalue(py).into_ptr(),
                std::ptr::null_mut(),
            ),
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
}
