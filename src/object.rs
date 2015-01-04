use std;
use std::cmp::Ordering;
use libc;
use ffi;
use {Python, Py_ssize_t, PyResult, PyErr, ToPyObject};
use typeobject::PyType;
use pyptr::{PyPtr, PythonPointer, as_ptr};
use err;

/// Trait implemented by all python object types.
pub trait PythonObject<'p> {
    /// Upcast from PyObject to a concrete python object type.
    /// Returns None if the python object is not of the specified type.
    fn from_object<'a>(&'a PyObject<'p>) -> Option<&'a Self>;

    /// Casts the python object to PyObject.
    fn as_object<'a>(&'a self) -> &'a PyObject<'p>;

    /// Retrieves the underlying FFI pointer associated with this python object.
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.as_object().as_ptr()
    }

    /// Retrieves the type object for this python object type.
    /// unused_self is necessary until UFCS is implemented.
    fn type_object(unused_self : Option<&Self>) -> &'p PyType<'p>;

    /// Retrieve python instance from an existing python object.
    fn python(&self) -> Python<'p> {
        self.as_object().python()
    }
}

pub struct PyObject<'p> {
    cell : std::cell::UnsafeCell<ffi::PyObject>,
    py : Python<'p>
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
    
    fn type_object(_ : Option<&Self>) -> &'p PyType<'p> {
        panic!()
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
    pub fn get_refcnt(&self) -> Py_ssize_t {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) }
    }

    /*pub fn get_type(&self) -> &PyType {
        unsafe { PyType::from_type_ptr(self.python(), ffi::Py_TYPE(self.as_ptr())) }
    }*/
    
    /// Casts the PyObject to a concrete python object type.
    /// Returns a python TypeError if the object is not of the expected type.
    pub fn downcast<T : PythonObject<'p>>(&self) -> PyResult<'p, &T> {
        let obj_opt : Option<&T> = PythonObject::from_object(self);
        match obj_opt {
            Some(obj) => Ok(obj),
            None => Err(PyErr::type_error(self, PythonObject::type_object(obj_opt)))
        }
    }
}

impl <'p> std::fmt::Show for PyObject<'p> {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
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

pub trait ObjectProtocol<'p> : PythonObject<'p> {
    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    #[inline]
    fn hasattr<Sized? N: ToPyObject<'p>>(&self, attr_name: &N) -> PyResult<'p, bool> {
        let py = self.python();
        let attr_name = try!(attr_name.to_py_object(py));
        unsafe {
            Ok(ffi::PyObject_HasAttr(self.as_ptr(), as_ptr(&attr_name)) != 0)
        }
    }
    
    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    #[inline]
    fn getattr<Sized? N: ToPyObject<'p>>(&self, attr_name: &N) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        let py = self.python();
        let attr_name = try!(attr_name.to_py_object(py));
        unsafe {
            err::result_from_owned_ptr(py,
                ffi::PyObject_GetAttr(self.as_ptr(), as_ptr(&attr_name)))
        }
    }

    /// Sets an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name = value'.
    #[inline]
    fn setattr<Sized? N: ToPyObject<'p>, Sized? V: ToPyObject<'p>>
            (&self, attr_name: &N, value: &V) -> PyResult<'p, ()> {
        let py = self.python();
        let attr_name = try!(attr_name.to_py_object(py));
        let value = try!(value.to_py_object(py));
        unsafe {
            err::result_from_error_code(py,
                ffi::PyObject_SetAttr(self.as_ptr(), as_ptr(&attr_name), as_ptr(&value)))
        }
    }

    /// Deletes an attribute.
    /// This is equivalent to the Python expression 'del self.attr_name'.
    #[inline]
    fn delattr<Sized? N: ToPyObject<'p>>(&self, attr_name: &N) -> PyResult<'p, ()> {
        let py = self.python();
        let attr_name = try!(attr_name.to_py_object(py));
        unsafe {
            err::result_from_error_code(py,
                ffi::PyObject_DelAttr(self.as_ptr(), as_ptr(&attr_name)))
        }
    }

    /// Compares two python objects.
    /// This is equivalent to the python expression 'cmp(self, other)'.
    fn compare(&self, other: &PyObject<'p>) -> PyResult<'p, Ordering> {
        unsafe {
            let mut result : libc::c_int = std::mem::uninitialized();
            try!(err::result_from_error_code(self.python(),
                ffi::PyObject_Cmp(self.as_ptr(), other.as_ptr(), &mut result)));
            Ok(if result < 0 {
                Ordering::Less
            } else if result > 0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            })
        }
    }

    /// Compute the string representation of self.
    /// This is equivalent to the python expression 'repr(self)'.
    fn repr(&self) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        unsafe {
            err::result_from_owned_ptr(self.python(), ffi::PyObject_Repr(self.as_ptr()))
        }
    }
    
    /// Compute the string representation of self.
    /// This is equivalent to the python expression 'str(self)'.
    fn str(&self) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        unsafe {
            err::result_from_owned_ptr(self.python(), ffi::PyObject_Str(self.as_ptr()))
        }
    }
    
    /// Compute the unicode string representation of self.
    /// This is equivalent to the python expression 'unistr(self)'.
    fn unistr(&self) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        unsafe {
            err::result_from_owned_ptr(self.python(), ffi::PyObject_Unicode(self.as_ptr()))
        }
    }
    
    /// Determines whether this object is callable.
    fn is_callable(&self) -> bool {
        unsafe {
            ffi::PyCallable_Check(self.as_ptr()) != 0
        }
    }
    
    fn call(&self, args: &PyObject<'p>, kw: Option<&PyObject<'p>>) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        unimplemented!()
    }
    
    fn call_method<Sized? N: ToPyObject<'p>>(&self, name: &N, args: &PyObject<'p>, kw: Option<&PyObject<'p>>) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        try!(self.getattr(name)).call(args, kw)
    }
}

impl <'p> ObjectProtocol<'p> for PyObject<'p> {}

