use crate::object::PyObject;

#[repr(transparent)]
pub struct PyFileObject(PyObject);
