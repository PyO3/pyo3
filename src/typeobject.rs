use object::{PythonObject, PyObject};
use python::Python;
use ffi;
use libc::c_char;
use std;

pub struct PyType<'p> {
    cell : std::cell::UnsafeCell<ffi::PyTypeObject>,
    py : Python<'p>
}

#[test]
fn test_sizeof() {
    // should be a static_assert, but size_of is not a compile-time const
    assert_eq!(std::mem::size_of::<PyType>(), std::mem::size_of::<ffi::PyTypeObject>());
}

impl <'p> PythonObject<'p> for PyType<'p> {
    #[inline]
    fn from_object<'a>(obj : &'a PyObject<'p>) -> Option<&'a PyType<'p>> {
        unsafe {
            if ffi::PyType_Check(obj.as_ptr()) {
                Some(std::mem::transmute(obj))
            } else {
                None
            }
        }
    }
    
    #[inline]
    fn as_object<'a>(&'a self) -> &'a PyObject<'p> {
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    fn python(&self) -> Python<'p> {
        self.py
    }
    
    #[inline]
    fn type_object(py: Python<'p>, _ : Option<&Self>) -> &'p PyType<'p> {
        unsafe { PyType::from_type_ptr(py, &mut ffi::PyType_Type) }
    }
}

impl <'p> PyType<'p> {
    /// Retrieves the underlying FFI pointer associated with this python object.
    #[inline]
    pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        // safe because the PyObject is only accessed while holding the GIL
        self.cell.get()
    }

    /// Retrieves the PyType instance for the given FFI pointer.
    /// Undefined behavior if the pointer is NULL or invalid.
    /// Also, the output lifetime 'a is unconstrained, make sure to use a lifetime
    /// appropriate for the underlying FFI pointer.
    #[inline]
    pub unsafe fn from_type_ptr<'a>(_: Python<'p>, p: *mut ffi::PyTypeObject) -> &'a PyType<'p> {
        debug_assert!(!p.is_null());
        &*(p as *mut PyType)
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

