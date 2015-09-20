// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::fmt;
use std::cmp::Ordering;
use ffi;
use libc;
use python::{PythonObject, ToPythonPointer};
use objects::{PyObject, PyTuple, PyDict, PyString};
use conversion::ToPyObject;
use err::{PyErr, PyResult, self};

/// Trait that contains methods 
pub trait ObjectProtocol<'p> : PythonObject<'p> {
    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    #[inline]
    fn hasattr<N>(&self, attr_name: N) -> PyResult<'p, bool> where N: ToPyObject<'p> {
        attr_name.with_borrowed_ptr(self.python(), |attr_name| unsafe {
            Ok(ffi::PyObject_HasAttr(self.as_ptr(), attr_name) != 0)
        })
    }

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    #[inline]
    fn getattr<N>(&self, attr_name: N) -> PyResult<'p, PyObject<'p>> where N: ToPyObject<'p> {
        let py = self.python();
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            err::result_from_owned_ptr(py,
                ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Sets an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name = value'.
    #[inline]
    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<'p, ()>
        where N: ToPyObject<'p>, V: ToPyObject<'p>
    {
        let py = self.python();
        attr_name.with_borrowed_ptr(py, move |attr_name|
            value.with_borrowed_ptr(py, |value| unsafe {
                err::error_on_minusone(py,
                    ffi::PyObject_SetAttr(self.as_ptr(), attr_name, value))
            }))
    }

    /// Deletes an attribute.
    /// This is equivalent to the Python expression 'del self.attr_name'.
    #[inline]
    fn delattr<N>(&self, attr_name: N) -> PyResult<'p, ()> where N: ToPyObject<'p> {
        let py = self.python();
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            err::error_on_minusone(py,
                ffi::PyObject_DelAttr(self.as_ptr(), attr_name))
        })
    }

    /// Compares two Python objects.
    /// This is equivalent to the Python expression 'cmp(self, other)'.
    #[cfg(feature="python27-sys")]
    fn compare<O>(&self, other: O) -> PyResult<'p, Ordering> where O: ToPyObject<'p> {
        let py = self.python();
        other.with_borrowed_ptr(py, |other| unsafe {
            let mut result : libc::c_int = -1;
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
    /// This is equivalent to the Python expression 'repr(self)'.
    #[inline]
    fn repr(&self) -> PyResult<'p, PyString<'p>> {
        unsafe {
            err::result_cast_from_owned_ptr(self.python(), ffi::PyObject_Repr(self.as_ptr()))
        }
    }

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'str(self)'.
    #[inline]
    fn str(&self) -> PyResult<'p, PyString<'p>> {
        unsafe {
            err::result_cast_from_owned_ptr(self.python(), ffi::PyObject_Str(self.as_ptr()))
        }
    }

    /// Compute the unicode string representation of self.
    /// This is equivalent to the Python expression 'unistr(self)'.
    #[inline]
    #[cfg(feature="python27-sys")]
    fn unistr(&self) -> PyResult<'p, PyUnicode<'p>> {
        unsafe {
            err::result_cast_from_owned_ptr(self.python(), ffi::PyObject_Unicode(self.as_ptr()))
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
    /// This is equivalent to the Python expression: 'self(*args, **kwargs)'
    #[inline]
    fn call<A>(&self, args: A, kwargs: Option<&PyDict<'p>>) -> PyResult<'p, PyObject<'p>>
      where A: ToPyObject<'p, ObjectType=PyTuple<'p>> {
        let py = self.python();
        args.with_borrowed_ptr(py, |args| unsafe {
            err::result_from_owned_ptr(py, ffi::PyObject_Call(self.as_ptr(), args, kwargs.as_ptr()))
        })
    }

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
    #[inline]
    fn call_method<A>(&self, name: &str, args: A, kwargs: Option<&PyDict<'p>>) -> PyResult<'p, PyObject<'p>>
      where A: ToPyObject<'p, ObjectType=PyTuple<'p>> {
        try!(self.getattr(name)).call(args, kwargs)
    }

    /// Retrieves the hash code of the object.
    /// This is equivalent to the Python expression: 'hash(self)'
    #[inline]
    fn hash(&self) -> PyResult<'p, ::Py_hash_t> {
        let v = unsafe { ffi::PyObject_Hash(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.python()))
        } else {
            Ok(v)
        }
    }

    /// Returns whether the object is considered to be true.
    /// This is equivalent to the Python expression: 'not not self'
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
    /// This is equivalent to the Python expression: 'len(self)'
    #[inline]
    fn len(&self) -> PyResult<'p, usize> {
        let v = unsafe { ffi::PyObject_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.python()))
        } else {
            Ok(v as usize)
        }
    }

    /// This is equivalent to the Python expression: 'self[key]'
    #[inline]
    fn get_item<K>(&self, key: K) -> PyResult<'p, PyObject<'p>> where K: ToPyObject<'p> {
        let py = self.python();
        key.with_borrowed_ptr(py, |key| unsafe {
            err::result_from_owned_ptr(py,
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
                err::error_on_minusone(py,
                    ffi::PyObject_SetItem(self.as_ptr(), key, value))
            }))
    }

    /// Deletes an item.
    /// This is equivalent to the Python expression 'del self[key]'.
    #[inline]
    fn del_item<K>(&self, key: K) -> PyResult<'p, ()> where K: ToPyObject<'p> {
        let py = self.python();
        key.with_borrowed_ptr(py, |key| unsafe {
            err::error_on_minusone(py,
                ffi::PyObject_DelItem(self.as_ptr(), key))
        })
    }

    /// Takes an object and returns an iterator for it.
    /// This is typically a new iterator but if the argument
    /// is an iterator, this returns itself.
    #[cfg(feature="python27-sys")]
    #[inline]
    fn iter(&self) -> PyResult<'p, ::objects::PyIterator<'p>> {
        unsafe {
            err::result_cast_from_owned_ptr(self.python(), ffi::PyObject_GetIter(self.as_ptr()))
        }
    }
}

impl <'p> ObjectProtocol<'p> for PyObject<'p> {}

impl <'p> fmt::Debug for PyObject<'p> {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use objectprotocol::ObjectProtocol;
        let repr_obj = try!(self.repr().map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy())
    }
}

impl <'p> fmt::Display for PyObject<'p> {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use objectprotocol::ObjectProtocol;
        let str_obj = try!(self.str().map_err(|_| fmt::Error));
        f.write_str(&str_obj.to_string_lossy())
    }
}

#[cfg(test)]
mod test {
    use std;
    use python::{Python, PythonObject};
    use conversion::ToPyObject;
    use objects::{PySequence, PyList, PyTuple};

    #[test]
    fn test_debug_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_py_object(py).into_object();
        assert_eq!(format!("{:?}", v), "'Hello\\n'");
    }

    #[test]
    fn test_display_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_py_object(py).into_object();
        assert_eq!(format!("{}", v), "Hello\n");
    }
}

