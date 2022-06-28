use crate::ffi;
use crate::type_object::PyTypeInfo;
use crate::types::{PyTuple, PyType};
use crate::{AsPyPointer, Py, PyAny, PyErr, PyResult, Python};

/// Represents a Python `super` object.
///
/// This type is immutable.
#[repr(transparent)]
pub struct PySuper(PyAny);

pyobject_native_type_core!(PySuper, ffi::PySuper_Type, #checkfunction=ffi::PyType_Check);

impl PySuper {
    pub fn new<'py>(py: Python<'py>, ty: &'py PyType, obj: &'py PyAny) -> PyResult<&'py PySuper> {
        let args = PyTuple::new(py, &[ty, obj]);
        let type_ = PySuper::type_object_raw(py);
        let super_ = unsafe { ffi::PyObject_CallObject(type_ as *mut _, args.as_ptr()) };
        if let Some(exc) = PyErr::take(py) {
            return Err(exc);
        }

        let super_: PyResult<Py<PySuper>> = unsafe { Py::from_borrowed_ptr_or_err(py, super_) };
        super_.map(|o| o.into_ref(py))
    }
}
