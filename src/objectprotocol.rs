use std;
use std::{fmt, string};
use std::borrow::Cow;
use std::cmp::Ordering;
use ffi;
use libc;
use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, ToPythonPointer};
use objects::{PyObject, PyTuple, PyDict};
use conversion::ToPyObject;
use err::{PyErr, PyResult, result_from_owned_ptr, error_on_minusone};

/// Trait that contains methods 
pub trait ObjectProtocol<'p> : PythonObject<'p> {
    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    #[inline]
    fn hasattr<N: ?Sized>(&self, attr_name: &N) -> PyResult<'p, bool> where N: ToPyObject<'p> {
        attr_name.with_borrowed_ptr(self.python(), |attr_name| unsafe {
            Ok(ffi::PyObject_HasAttr(self.as_ptr(), attr_name) != 0)
        })
    }
    
    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    #[inline]
    fn getattr<N: ?Sized>(&self, attr_name: &N) -> PyResult<'p, PyObject<'p>> where N: ToPyObject<'p> {
        let py = self.python();
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            result_from_owned_ptr(py,
                ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Sets an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name = value'.
    #[inline]
    fn setattr<N: ?Sized, V: ?Sized>(&self, attr_name: &N, value: &V) -> PyResult<'p, ()>
        where N: ToPyObject<'p>, V: ToPyObject<'p>
    {
        let py = self.python();
        attr_name.with_borrowed_ptr(py, move |attr_name|
            value.with_borrowed_ptr(py, |value| unsafe {
                error_on_minusone(py,
                    ffi::PyObject_SetAttr(self.as_ptr(), attr_name, value))
            }))
    }

    /// Deletes an attribute.
    /// This is equivalent to the Python expression 'del self.attr_name'.
    #[inline]
    fn delattr<N: ?Sized>(&self, attr_name: &N) -> PyResult<'p, ()> where N: ToPyObject<'p> {
        let py = self.python();
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            error_on_minusone(py,
                ffi::PyObject_DelAttr(self.as_ptr(), attr_name))
        })
    }

    /// Compares two python objects.
    /// This is equivalent to the python expression 'cmp(self, other)'.
    #[inline]
    fn compare<O: ?Sized>(&self, other: &O) -> PyResult<'p, Ordering> where O: ToPyObject<'p> {
        let py = self.python();
        other.with_borrowed_ptr(py, |other| unsafe {
            let mut result : libc::c_int = std::mem::uninitialized();
            try!(error_on_minusone(py,
                ffi::PyObject_Cmp(self.as_ptr(), other, &mut result)));
            Ok(if result < 0 {
                Ordering::Less
            } else if result > 0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            })
        })
    }

    /// Compute the string representation of self.
    /// This is equivalent to the python expression 'repr(self)'.
    #[inline]
    fn repr(&self) -> PyResult<'p, PyObject<'p>> {
        unsafe {
            result_from_owned_ptr(self.python(), ffi::PyObject_Repr(self.as_ptr()))
        }
    }
    
    /// Compute the string representation of self.
    /// This is equivalent to the python expression 'str(self)'.
    #[inline]
    fn str(&self) -> PyResult<'p, PyObject<'p>> {
        unsafe {
            result_from_owned_ptr(self.python(), ffi::PyObject_Str(self.as_ptr()))
        }
    }
    
    /// Compute the unicode string representation of self.
    /// This is equivalent to the python expression 'unistr(self)'.
    #[inline]
    fn unistr(&self) -> PyResult<'p, PyObject<'p>> {
        unsafe {
            result_from_owned_ptr(self.python(), ffi::PyObject_Unicode(self.as_ptr()))
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
    /// This is equivalent to the python expression: 'self(*args, **kwargs)'
    #[inline]
    fn call<A: ?Sized>(&self, args: &A, kwargs: Option<&PyDict<'p>>) -> PyResult<'p, PyObject<'p>>
      where A: ToPyObject<'p, ObjectType=PyTuple<'p>> {
        let py = self.python();
        args.with_borrowed_ptr(py, |args| unsafe {
            result_from_owned_ptr(py, ffi::PyObject_Call(self.as_ptr(), args, kwargs.as_ptr()))
        })
    }
    
    /// Calls a method on the object.
    /// This is equivalent to the python expression: 'self.name(*args, **kwargs)'
    #[inline]
    fn call_method<A: ?Sized>(&self, name: &str, args: &A, kwargs: Option<&PyDict<'p>>) -> PyResult<'p, PyObject<'p>>
      where A: ToPyObject<'p, ObjectType=PyTuple<'p>> {
        try!(self.getattr(name)).call(args, kwargs)
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
    fn len(&self) -> PyResult<'p, ffi::Py_ssize_t> {
        let v = unsafe { ffi::PyObject_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.python()))
        } else {
            Ok(v)
        }
    }
    
    /// This is equivalent to the python expression: 'self[key]'
    #[inline]
    fn get_item<K>(&self, key: K) -> PyResult<'p, PyObject<'p>> where K: ToPyObject<'p> {
        let py = self.python();
        key.with_borrowed_ptr(py, |key| unsafe {
            result_from_owned_ptr(py,
                ffi::PyObject_GetItem(self.as_ptr(), key))
        })
    }

    /// Sets an item value.
    /// This is equivalent to the Python expression 'self[key] = value'.
    #[inline]
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<'p, ()> where K: ToPyObject<'p>, V: ToPyObject<'p> {
        let py = self.python();
        key.with_borrowed_ptr(py, move |key|
            value.with_borrowed_ptr(py, |value| unsafe {
                error_on_minusone(py,
                    ffi::PyObject_SetItem(self.as_ptr(), key, value))
            }))
    }

    /// Deletes an item.
    /// This is equivalent to the Python expression 'del self[key]'.
    #[inline]
    fn del_item<K>(&self, key: K) -> PyResult<'p, ()> where K: ToPyObject<'p> {
        let py = self.python();
        key.with_borrowed_ptr(py, |key| unsafe {
            error_on_minusone(py,
                ffi::PyObject_DelItem(self.as_ptr(), key))
        })
    }
    /*
    /// Takes an object and returns an iterator for it.
    /// This is typically a new iterator but if the argument
    /// is an iterator, this returns itself.
    #[inline]
    fn iter(&self) -> PyResult<'p, PyPtr<'p, PyIterator<'p>>> {
        let it = try!(unsafe {
            result_from_owned_ptr(self.python(), ffi::PyObject_GetIter(self.as_ptr()))
        });
        it.downcast_into()
    }*/
}

impl <'p> ObjectProtocol<'p> for PyObject<'p> {}

/*
pub struct PyIterator<'p>(PyObject<'p>);

impl <'p> PythonObject<'p> for PyIterator<'p> {
    #[inline]
    fn as_object<'a>(&'a self) -> &'a PyObject<'p> {
        &self.0
    }
    
    #[inline]
    unsafe fn unchecked_downcast_from<'a>(o: &'a PyObject<'p>) -> &'a PyIterator<'p> {
        std::mem::transmute(o)
    }
}

*/

impl <'p> fmt::Debug for PyObject<'p> {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use objectprotocol::ObjectProtocol;
        let repr_obj = try!(self.str().map_err(|_| fmt::Error));
        let repr = try!(repr_obj.extract::<Cow<str>>().map_err(|_| fmt::Error));
        f.write_str(&*repr)
    }
}

impl <'p> fmt::Display for PyObject<'p> {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use objectprotocol::ObjectProtocol;
        let repr_obj = try!(self.repr().map_err(|_| fmt::Error));
        let repr = try!(repr_obj.extract::<Cow<str>>().map_err(|_| fmt::Error));
        f.write_str(&*repr)
    }
}


