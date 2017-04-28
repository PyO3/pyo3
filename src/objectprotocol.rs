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
use python::{Python, PythonObject, ToPythonPointer};
use objects::{PyObject, PyTuple, PyDict, PyString, ToPyTuple};
use conversion::ToPyObject;
use err::{PyErr, PyResult, self};

/// Trait that contains methods 
pub trait ObjectProtocol : PythonObject {
    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    #[inline]
    fn hasattr<N>(&self, py: Python, attr_name: N) -> PyResult<bool> where N: ToPyObject {
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            Ok(ffi::PyObject_HasAttr(self.as_ptr(), attr_name) != 0)
        })
    }

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    #[inline]
    fn getattr<N>(&self, py: Python, attr_name: N) -> PyResult<PyObject> where N: ToPyObject {
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            err::result_from_owned_ptr(py,
                ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Sets an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name = value'.
    #[inline]
    fn setattr<N, V>(&self, py: Python, attr_name: N, value: V) -> PyResult<()>
        where N: ToPyObject, V: ToPyObject
    {
        attr_name.with_borrowed_ptr(py, move |attr_name|
            value.with_borrowed_ptr(py, |value| unsafe {
                err::error_on_minusone(py,
                    ffi::PyObject_SetAttr(self.as_ptr(), attr_name, value))
            }))
    }

    /// Deletes an attribute.
    /// This is equivalent to the Python expression 'del self.attr_name'.
    #[inline]
    fn delattr<N>(&self, py: Python, attr_name: N) -> PyResult<()> where N: ToPyObject {
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            err::error_on_minusone(py,
                ffi::PyObject_DelAttr(self.as_ptr(), attr_name))
        })
    }

    /// Compares two Python objects.
    ///
    /// On Python 2, this is equivalent to the Python expression 'cmp(self, other)'.
    ///
    /// On Python 3, this is equivalent to:
    /// ```
    /// if self == other:
    ///     return Equal
    /// elif a < b:
    ///     return Less
    /// elif a > b:
    ///     return Greater
    /// else:
    ///     raise TypeError("ObjectProtocol::compare(): All comparisons returned false")
    /// ```
    fn compare<O>(&self, py: Python, other: O) -> PyResult<Ordering> where O: ToPyObject {
        #[cfg(feature="python27-sys")]
        unsafe fn do_compare(py: Python, a: *mut ffi::PyObject, b: *mut ffi::PyObject) -> PyResult<Ordering> {
            let mut result = -1;
            try!(err::error_on_minusone(py,
                ffi::PyObject_Cmp(a, b, &mut result)));
            Ok(if result < 0 {
                Ordering::Less
            } else if result > 0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            })
        }

        #[cfg(feature="python3-sys")]
        unsafe fn do_compare(py: Python, a: *mut ffi::PyObject, b: *mut ffi::PyObject) -> PyResult<Ordering> {
            let result = ffi::PyObject_RichCompareBool(a, b, ffi::Py_EQ);
            if result == 1 {
                return Ok(Ordering::Equal);
            } else if result < 0 {
                return Err(PyErr::fetch(py));
            }
            let result = ffi::PyObject_RichCompareBool(a, b, ffi::Py_LT);
            if result == 1 {
                return Ok(Ordering::Less);
            } else if result < 0 {
                return Err(PyErr::fetch(py));
            }
            let result = ffi::PyObject_RichCompareBool(a, b, ffi::Py_GT);
            if result == 1 {
                return Ok(Ordering::Greater);
            } else if result < 0 {
                return Err(PyErr::fetch(py));
            }
            return Err(PyErr::new::<::exc::TypeError, _>(py, "ObjectProtocol::compare(): All comparisons returned false"));
        }

        other.with_borrowed_ptr(py, |other| unsafe {
            do_compare(py, self.as_ptr(), other)
        })
    }

    /// Compares two Python objects.
    ///
    /// Depending on the value of `compare_op`, equivalent to one of the following Python expressions:
    ///   * CompareOp::Eq: `self == other`
    ///   * CompareOp::Ne: `self != other`
    ///   * CompareOp::Lt: `self < other`
    ///   * CompareOp::Le: `self <= other`
    ///   * CompareOp::Gt: `self > other`
    ///   * CompareOp::Ge: `self >= other`
    fn rich_compare<O>(&self, py: Python, other: O, compare_op: ::CompareOp) -> PyResult<PyObject> where O: ToPyObject {
        other.with_borrowed_ptr(py, |other| unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyObject_RichCompare(self.as_ptr(), other, compare_op as libc::c_int))
        })
    }

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'repr(self)'.
    #[inline]
    fn repr(&self, py: Python) -> PyResult<PyString> {
        unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyObject_Repr(self.as_ptr()))
        }
    }

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'str(self)'.
    #[inline]
    fn str(&self, py: Python) -> PyResult<PyString> {
        unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyObject_Str(self.as_ptr()))
        }
    }

    /// Compute the unicode string representation of self.
    /// This is equivalent to the Python expression 'unistr(self)'.
    #[inline]
    #[cfg(feature="python27-sys")]
    fn unistr(&self, py: Python) -> PyResult<::objects::PyUnicode> {
        unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyObject_Unicode(self.as_ptr()))
        }
    }

    /// Determines whether this object is callable.
    #[inline]
    fn is_callable(&self, _py: Python) -> bool {
        unsafe {
            ffi::PyCallable_Check(self.as_ptr()) != 0
        }
    }

    /// Calls the object.
    /// This is equivalent to the Python expression: 'self(*args, **kwargs)'
    #[inline]
    fn call<A>(&self, py: Python, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: ToPyTuple
    {
        args.with_borrowed_ptr(py, |args| unsafe {
            err::result_from_owned_ptr(py, ffi::PyObject_Call(self.as_ptr(), args, kwargs.as_ptr()))
        })
    }

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
    #[inline]
    fn call_method<A>(&self, py: Python, name: &str, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: ToPyTuple
    {
        try!(self.getattr(py, name)).call(py, args, kwargs)
    }

    /// Retrieves the hash code of the object.
    /// This is equivalent to the Python expression: 'hash(self)'
    #[inline]
    fn hash(&self, py: Python) -> PyResult<::Py_hash_t> {
        let v = unsafe { ffi::PyObject_Hash(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(py))
        } else {
            Ok(v)
        }
    }

    /// Returns whether the object is considered to be true.
    /// This is equivalent to the Python expression: 'not not self'
    #[inline]
    fn is_true(&self, py: Python) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(py))
        } else {
            Ok(v != 0)
        }
    }

    /// Returns the length of the sequence or mapping.
    /// This is equivalent to the Python expression: 'len(self)'
    #[inline]
    fn len(&self, py: Python) -> PyResult<usize> {
        let v = unsafe { ffi::PyObject_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(py))
        } else {
            Ok(v as usize)
        }
    }

    /// This is equivalent to the Python expression: 'self[key]'
    #[inline]
    fn get_item<K>(&self, py: Python, key: K) -> PyResult<PyObject> where K: ToPyObject {
        key.with_borrowed_ptr(py, |key| unsafe {
            err::result_from_owned_ptr(py,
                ffi::PyObject_GetItem(self.as_ptr(), key))
        })
    }

    /// Sets an item value.
    /// This is equivalent to the Python expression 'self[key] = value'.
    #[inline]
    fn set_item<K, V>(&self, py: Python, key: K, value: V) -> PyResult<()> where K: ToPyObject, V: ToPyObject {
        key.with_borrowed_ptr(py, move |key|
            value.with_borrowed_ptr(py, |value| unsafe {
                err::error_on_minusone(py,
                    ffi::PyObject_SetItem(self.as_ptr(), key, value))
            }))
    }

    /// Deletes an item.
    /// This is equivalent to the Python expression 'del self[key]'.
    #[inline]
    fn del_item<K>(&self, py: Python, key: K) -> PyResult<()> where K: ToPyObject {
        key.with_borrowed_ptr(py, |key| unsafe {
            err::error_on_minusone(py,
                ffi::PyObject_DelItem(self.as_ptr(), key))
        })
    }

    /// Takes an object and returns an iterator for it.
    /// This is typically a new iterator but if the argument
    /// is an iterator, this returns itself.
    #[inline]
    fn iter<'p>(&self, py: Python<'p>) -> PyResult<::objects::PyIterator<'p>> {
        let obj = try!(unsafe {
            err::result_from_owned_ptr(py, ffi::PyObject_GetIter(self.as_ptr()))
        });
        Ok(try!(::objects::PyIterator::from_object(py, obj)))
    }
}

impl ObjectProtocol for PyObject {}

impl fmt::Debug for PyObject {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: we shouldn't use fmt::Error when repr() fails
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let repr_obj = try!(self.repr(py).map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy(py))
    }
}

impl fmt::Display for PyObject {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: we shouldn't use fmt::Error when str() fails
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let str_obj = try!(self.str(py).map_err(|_| fmt::Error));
        f.write_str(&str_obj.to_string_lossy(py))
    }
}

#[cfg(test)]
mod test {
    use std;
    use python::{Python, PythonObject};
    use conversion::ToPyObject;
    use objects::{PyList, PyTuple};
    use super::ObjectProtocol;

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

    #[test]
    fn test_compare() {
        use std::cmp::Ordering;
        let gil = Python::acquire_gil();
        let py = gil.python();
        let one = 1i32.to_py_object(py).into_object();
        assert_eq!(one.compare(py, 1).unwrap(), Ordering::Equal);
        assert_eq!(one.compare(py, 2).unwrap(), Ordering::Less);
        assert_eq!(one.compare(py, 0).unwrap(), Ordering::Greater);
    }
}

