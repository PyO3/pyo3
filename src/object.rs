use std;
use std::cmp::Ordering;
use libc;
use ffi;
use {Python, Py_ssize_t, PyResult, PyErr, ToPyObject};
use typeobject::PyType;
use pyptr::{PyPtr, PythonPointer, as_ptr};
use err;

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
    pub fn get_refcnt(&self) -> Py_ssize_t {
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

pub trait ObjectProtocol<'p> : PythonObject<'p> {
    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    #[inline]
    fn hasattr<'n, Sized? N: ToPyObject<'p, 'n>>(&self, attr_name: &'n N) -> PyResult<'p, bool> {
        let py = self.python();
        let attr_name = try!(attr_name.to_py_object(py));
        unsafe {
            Ok(ffi::PyObject_HasAttr(self.as_ptr(), as_ptr(&attr_name)) != 0)
        }
    }
    
    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    #[inline]
    fn getattr<'n, Sized? N: ToPyObject<'p, 'n>>(&self, attr_name: &'n N)
      -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
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
    fn setattr<'n, 'v, Sized? N: ToPyObject<'p, 'n>, Sized? V: ToPyObject<'p, 'v>>
            (&self, attr_name: &'n N, value: &'v V) -> PyResult<'p, ()> {
        let py = self.python();
        let attr_name = try!(attr_name.to_py_object(py));
        let value = try!(value.to_py_object(py));
        unsafe {
            err::error_on_minusone(py,
                ffi::PyObject_SetAttr(self.as_ptr(), as_ptr(&attr_name), as_ptr(&value)))
        }
    }

    /// Deletes an attribute.
    /// This is equivalent to the Python expression 'del self.attr_name'.
    #[inline]
    fn delattr<'n, Sized? N: ToPyObject<'p, 'n>>(&self, attr_name: &'n N) -> PyResult<'p, ()> {
        let py = self.python();
        let attr_name = try!(attr_name.to_py_object(py));
        unsafe {
            err::error_on_minusone(py,
                ffi::PyObject_DelAttr(self.as_ptr(), as_ptr(&attr_name)))
        }
    }

    /// Compares two python objects.
    /// This is equivalent to the python expression 'cmp(self, other)'.
    #[inline]
    fn compare(&self, other: &PyObject<'p>) -> PyResult<'p, Ordering> {
        unsafe {
            let mut result : libc::c_int = std::mem::uninitialized();
            try!(err::error_on_minusone(self.python(),
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
    #[inline]
    fn repr(&self) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        unsafe {
            err::result_from_owned_ptr(self.python(), ffi::PyObject_Repr(self.as_ptr()))
        }
    }
    
    /// Compute the string representation of self.
    /// This is equivalent to the python expression 'str(self)'.
    #[inline]
    fn str(&self) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        unsafe {
            err::result_from_owned_ptr(self.python(), ffi::PyObject_Str(self.as_ptr()))
        }
    }
    
    /// Compute the unicode string representation of self.
    /// This is equivalent to the python expression 'unistr(self)'.
    #[inline]
    fn unistr(&self) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        unsafe {
            err::result_from_owned_ptr(self.python(), ffi::PyObject_Unicode(self.as_ptr()))
        }
    }
    
    /// Determines whether this object is callable.
    #[inline]
    fn is_callable(&self) -> bool {
        unsafe {
            ffi::PyCallable_Check(self.as_ptr()) != 0
        }
    }
    
    /// Calls the object.
    /// This is equivalent to the python expression: 'self(*args, **kw)'
    #[inline]
    fn call(&self, args: &PyObject<'p>, kw: Option<&PyObject<'p>>) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        unimplemented!()
    }
    
    /// Calls a method on the object.
    /// This is equivalent to the python expression: 'self.name(*args, **kw)'
    #[inline]
    fn call_method(&self, name: &str, args: &PyObject<'p>, kw: Option<&PyObject<'p>>)
      -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        try!(self.getattr(name)).call(args, kw)
    }
    
    /// Retrieves the hash code of the object.
    /// This is equivalent to the python expression: 'hash(self)'
    #[inline]
    fn hash(&self) -> PyResult<'p, libc::c_long> {
        let v = unsafe { ffi::PyObject_Hash(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.python()))
        } else {
            Ok(v)
        }
    }
    
    /// Returns whether the object is considered to be true.
    /// This is equivalent to the python expression: 'not not self'
    #[inline]
    fn is_true(&self) -> PyResult<'p, bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.python()))
        } else {
            Ok(v != 0)
        }
    }
    
    /// Returns the length of the sequence or mapping.
    /// This is equivalent to the python expression: 'len(self)'
    #[inline]
    fn len(&self) -> PyResult<'p, Py_ssize_t> {
        let v = unsafe { ffi::PyObject_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.python()))
        } else {
            Ok(v)
        }
    }
    
    /// This is equivalent to the python expression: 'self[key]'
    #[inline]
    fn get_item<'k, Sized? K: ToPyObject<'p, 'k>>(&self, key: &'k K) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
           let py = self.python();
        let key = try!(key.to_py_object(py));
        unsafe {
            err::result_from_owned_ptr(py,
                ffi::PyObject_GetItem(self.as_ptr(), as_ptr(&key)))
        }
    }

    /// Sets an item value.
    /// This is equivalent to the Python expression 'self[key] = value'.
    #[inline]
    fn set_item<'k, 'v, Sized? K: ToPyObject<'p, 'k>, Sized? V: ToPyObject<'p, 'v>>
            (&self, key: &'k K, value: &'v V) -> PyResult<'p, ()> {
        let py = self.python();
        let key = try!(key.to_py_object(py));
        let value = try!(value.to_py_object(py));
        unsafe {
            err::error_on_minusone(py,
                ffi::PyObject_SetItem(self.as_ptr(), as_ptr(&key), as_ptr(&value)))
        }
    }

    /// Deletes an item.
    /// This is equivalent to the Python expression 'del self[key]'.
    #[inline]
    fn del_item<'k, Sized? K: ToPyObject<'p, 'k>>(&self, key: &'k K) -> PyResult<'p, ()> {
        let py = self.python();
        let key = try!(key.to_py_object(py));
        unsafe {
            err::error_on_minusone(py,
                ffi::PyObject_DelItem(self.as_ptr(), as_ptr(&key)))
        }
    }
    
    /// Takes an object and returns an iterator for it.
    /// This is typically a new iterator but if the argument
    /// is an iterator, this returns itself.
    #[inline]
    fn iter(&self) -> PyResult<'p, PyPtr<'p, PyObject<'p>>> {
        unsafe {
            err::result_from_owned_ptr(self.python(), ffi::PyObject_GetIter(self.as_ptr()))
        }
    }
    
    /// Retrieves the next item from an iterator.
    /// Returns None when the iterator is exhausted.
    #[inline]
    fn iter_next(&self) -> PyResult<'p, Option<PyPtr<'p, PyObject<'p>>>> {
        let py = self.python();
        let r = unsafe { ffi::PyIter_Next(self.as_ptr()) };
        if r.is_null() {
            if PyErr::occurred(py) {
                Err(PyErr::fetch(py))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(unsafe { PyPtr::from_owned_ptr(py, r) }))
        }
    }
}

impl <'p> ObjectProtocol<'p> for PyObject<'p> {}

