use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectWithTypeObject, ToPythonPointer};
use objects::PyObject;
use ffi;
use libc::c_char;
use std;

pyobject_newtype!(PyType, PyType_Check, PyType_Type);

impl <'p> PyType<'p> {
    /// Retrieves the underlying FFI pointer associated with this python object.
    #[inline]
    pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Retrieves the PyType instance for the given FFI pointer.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_type_ptr<'a>(py: Python<'p>, p: *mut ffi::PyTypeObject) -> PyType<'p> {
        PyObject::from_borrowed_ptr(py, p as *mut ffi::PyObject).unchecked_cast_into::<PyType>()
    }

    /// Return true if self is a subtype of b.
    #[inline]
    pub fn is_subtype_of(&self, b : &PyType<'p>) -> bool {
        unsafe { ffi::PyType_IsSubtype(self.as_type_ptr(), b.as_type_ptr()) != 0 }
    }

    /// Return true if obj is an instance of self.
    #[inline]
    pub fn is_instance(&self, obj : &PyObject<'p>) -> bool {
        unsafe { ffi::PyObject_TypeCheck(obj.as_ptr(), self.as_type_ptr()) }
    }
}

impl <'p> PartialEq for PyType<'p> {
    #[inline]
    fn eq(&self, o : &PyType<'p>) -> bool {
        self.as_type_ptr() == o.as_type_ptr()
    }
}
impl <'p> Eq for PyType<'p> { }

