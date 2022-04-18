use crate::{
    exceptions::{PyBaseException, PyTypeError},
    ffi,
    type_object::PyTypeObject,
    types::{PyTraceback, PyType},
    AsPyPointer, IntoPyObject, IntoPyPointer, Py, PyObject, Python,
};

#[derive(Clone)]
pub(crate) struct PyErrStateNormalized {
    pub ptype: Py<PyType>,
    pub pvalue: Py<PyBaseException>,
    pub ptraceback: Option<Py<PyTraceback>>,
}

pub(crate) enum PyErrState {
    LazyTypeAndValue {
        ptype: for<'py> fn(Python<'py>) -> &PyType,
        pvalue: Box<dyn for<'py> FnOnce(Python<'py>) -> PyObject + Send + Sync>,
    },
    LazyValue {
        ptype: Py<PyType>,
        pvalue: Box<dyn for<'py> FnOnce(Python<'py>) -> PyObject + Send + Sync>,
    },
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
    T: IntoPyObject + Send + Sync,
{
    fn arguments(self, py: Python<'_>) -> PyObject {
        self.into_object(py)
    }
}

pub(crate) fn boxed_args(
    args: impl PyErrArguments + 'static,
) -> Box<dyn for<'py> FnOnce(Python<'py>) -> PyObject + Send + Sync> {
    Box::new(|py| args.arguments(py))
}

impl PyErrState {
    pub(crate) fn into_ffi_tuple(
        self,
        py: Python<'_>,
    ) -> (*mut ffi::PyObject, *mut ffi::PyObject, *mut ffi::PyObject) {
        match self {
            PyErrState::LazyTypeAndValue { ptype, pvalue } => {
                let ty = ptype(py);
                if unsafe { ffi::PyExceptionClass_Check(ty.as_ptr()) } == 0 {
                    Self::exceptions_must_derive_from_base_exception(py).into_ffi_tuple(py)
                } else {
                    (
                        ptype(py).into_ptr(),
                        pvalue(py).into_ptr(),
                        std::ptr::null_mut(),
                    )
                }
            }
            PyErrState::LazyValue { ptype, pvalue } => (
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

    #[inline]
    pub(crate) fn exceptions_must_derive_from_base_exception(py: Python<'_>) -> Self {
        PyErrState::LazyValue {
            ptype: PyTypeError::type_object(py).into(),
            pvalue: boxed_args("exceptions must derive from BaseException"),
        }
    }
}
