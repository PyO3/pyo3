use std;
use libc;
use ffi;
use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectWithTypeObject};
use objects::PyType;
use err::{PyErr, PyResult};

pub struct PyObject<'p> {
    cell : std::cell::UnsafeCell<ffi::PyObject>,
    py : Python<'p>
}

#[test]
fn test_sizeof() {
    // should be a static_assert, but size_of is not a compile-time const
    assert_eq!(std::mem::size_of::<PyObject>(), std::mem::size_of::<ffi::PyObject>());
}

impl <'p> PythonObject<'p> for PyObject<'p> {
    #[inline]
    fn as_object<'a>(&'a self) -> &'a PyObject<'p> {
        self
    }
    
    #[inline]
    fn unchecked_downcast_from<'a>(o: &'a PyObject<'p>) -> &'a PyObject<'p> {
        o
    }

    /// Retrieves the underlying FFI pointer associated with this python object.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.cell.get()
    }

    #[inline]
    fn python(&self) -> Python<'p> {
        self.py
    }
}

impl <'p> PythonObjectWithCheckedDowncast<'p> for PyObject<'p> {
    #[inline]
    fn downcast_from<'a>(obj : &'a PyObject<'p>) -> Option<&'a PyObject<'p>> {
        Some(obj)
    }
}

impl <'p> PythonObjectWithTypeObject<'p> for PyObject<'p> {
    #[inline]
    fn type_object(py: Python<'p>, _ : Option<&Self>) -> &'p PyType<'p> {
        unsafe { PyType::from_type_ptr(py, &mut ffi::PyBaseObject_Type) }
    }
}

impl <'p> PyObject<'p> {


    /// Retrieves the PyObject instance for the given FFI pointer.
    /// Undefined behavior if the pointer is NULL or invalid.
    /// Also, the output lifetime 'a is unconstrained, make sure to use a lifetime
    /// appropriate for the underlying FFI pointer.
    #[inline]
    pub unsafe fn from_ptr<'a>(_ : Python<'p>, p : *mut ffi::PyObject) -> &'a PyObject<'p> {
        debug_assert!(!p.is_null());
        &*(p as *mut PyObject)
    }
    
    /// Retrieves the reference count of this python object.
    #[inline]
    pub fn get_refcnt(&self) -> ffi::Py_ssize_t {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) }
    }

    #[inline]
    pub fn get_type(&self) -> &PyType<'p> {
        unsafe { PyType::from_type_ptr(self.python(), ffi::Py_TYPE(self.as_ptr())) }
    }
    
    /// Casts the PyObject to a concrete python object type.
    /// Returns a python TypeError if the object is not of the expected type.
    #[inline]
    pub fn downcast<T : PythonObjectWithCheckedDowncast<'p>>(&self) -> PyResult<'p, &T> {
        let obj_opt : Option<&T> = PythonObjectWithCheckedDowncast::downcast_from(self);
        match obj_opt {
            Some(obj) => Ok(obj),
            None => Err(unimplemented!())
        }
    }
}

impl <'p> std::fmt::Show for PyObject<'p> {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use objectprotocol::ObjectProtocol;
        let rep = try!(self.repr().map_err(|_| std::fmt::Error));
        let slice = try!(::conversion::string_as_slice(&*rep).map_err(|_| std::fmt::Error));
        f.write_str(try!(std::str::from_utf8(slice).map_err(|_| std::fmt::Error)))
    }
}

impl <'p> PartialEq for PyObject<'p> {
    #[inline]
    fn eq(&self, o : &PyObject<'p>) -> bool {
        self.as_ptr() == o.as_ptr()
    }
}
impl <'p> Eq for PyObject<'p> { }

