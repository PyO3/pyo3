use std;
use libc;
use ffi;
use typeobject::PyType;
use python::Python;
use err::{PyErr, PyResult};

/// Trait implemented by all python object types.
pub trait PythonObject<'p> : 'p {
    // TODO: split this trait; not every PythonObject impl has a statically known type,
    // or the ability to perform a typecheck

    /// Upcast from PyObject to a concrete python object type.
    /// Returns None if the python object is not of the specified type.
    fn from_object<'a>(&'a PyObject<'p>) -> Option<&'a Self>;

    /// Casts the python object to PyObject.
    fn as_object<'a>(&'a self) -> &'a PyObject<'p>;

    /// Retrieves the underlying FFI pointer associated with this python object.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.as_object().as_ptr()
    }

    /// Retrieves the type object for this python object type.
    /// unused_self is necessary until UFCS is implemented.
    fn type_object(py: Python<'p>, unused_self : Option<&Self>) -> &'p PyType<'p>;

    /// Retrieve python instance from an existing python object.
    #[inline]
    fn python(&self) -> Python<'p> {
        self.as_object().python()
    }
}

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
    fn from_object<'a>(obj : &'a PyObject<'p>) -> Option<&'a PyObject<'p>> {
        Some(obj)
    }
    
    #[inline]
    fn as_object<'a>(&'a self) -> &'a PyObject<'p> {
        self
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
    pub fn downcast<T : PythonObject<'p>>(&self) -> PyResult<'p, &T> {
        let obj_opt : Option<&T> = PythonObject::from_object(self);
        match obj_opt {
            Some(obj) => Ok(obj),
            None => Err(PyErr::type_error(self, PythonObject::type_object(self.python(), obj_opt)))
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

