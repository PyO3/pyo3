// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::fmt;
use std::cmp::Ordering;
use ffi;
use libc;
use pyptr::{Py, PyPtr};
use python::{Python, PyDowncastInto, ToPythonPointer};
use objects::{PyObject, PyDict, PyString};
use token::PythonObjectWithGilToken;
use conversion::{ToPyObject, ToPyTuple};
use typeob::PyTypeInfo;
use err::{PyErr, PyResult, self};


pub trait ObjectProtocol<'p> {

    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool> where N: ToPyObject;

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    fn getattr<N>(&self, attr_name: N) -> PyResult<PyObject<'p>> where N: ToPyObject;

    /// Sets an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name = value'.
    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
        where N: ToPyObject, V: ToPyObject;

    /// Deletes an attribute.
    /// This is equivalent to the Python expression 'del self.attr_name'.
    fn delattr<N>(&self, attr_name: N) -> PyResult<()> where N: ToPyObject;

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
    fn compare<O>(&self, other: O) -> PyResult<Ordering> where O: ToPyObject;

    /// Compares two Python objects.
    ///
    /// Depending on the value of `compare_op`, equivalent to one of the following Python expressions:
    ///   * CompareOp::Eq: `self == other`
    ///   * CompareOp::Ne: `self != other`
    ///   * CompareOp::Lt: `self < other`
    ///   * CompareOp::Le: `self <= other`
    ///   * CompareOp::Gt: `self > other`
    ///   * CompareOp::Ge: `self >= other`
    fn rich_compare<O>(&self, other: O, compare_op: ::CompareOp)
                       -> PyResult<PyObject<'p>> where O: ToPyObject;

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'repr(self)'.
    fn repr(&self) -> PyResult<PyString<'p>>;

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'str(self)'.
    fn str(&self) -> PyResult<PyString<'p>>;

    /// Determines whether this object is callable.
    fn is_callable(&self) -> bool;

    /// Calls the object.
    /// This is equivalent to the Python expression: 'self(*args, **kwargs)'
    fn call<A>(&self, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject<'p>>
        where A: ToPyTuple;

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
    fn call_method<A>(&self,
                      name: &str, args: A,
                      kwargs: Option<&PyDict>) -> PyResult<PyObject<'p>>
        where A: ToPyTuple;

    /// Retrieves the hash code of the object.
    /// This is equivalent to the Python expression: 'hash(self)'
    fn hash(&self) -> PyResult<::Py_hash_t>;

    /// Returns whether the object is considered to be true.
    /// This is equivalent to the Python expression: 'not not self'
    fn is_true(&self) -> PyResult<bool>;

    /// Returns whether the object is considered to be None.
    /// This is equivalent to the Python expression: 'is None'
    #[inline]
    fn is_none(&self) -> bool;

    /// Returns the length of the sequence or mapping.
    /// This is equivalent to the Python expression: 'len(self)'
    fn len(&self) -> PyResult<usize>;

    /// This is equivalent to the Python expression: 'self[key]'
    fn get_item<K>(&self, key: K) -> PyResult<PyObject<'p>> where K: ToPyObject;

    /// Sets an item value.
    /// This is equivalent to the Python expression 'self[key] = value'.
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
        where K: ToPyObject, V: ToPyObject;

    /// Deletes an item.
    /// This is equivalent to the Python expression 'del self[key]'.
    fn del_item<K>(&self, key: K) -> PyResult<()> where K: ToPyObject;

    // /// Takes an object and returns an iterator for it.
    // /// This is typically a new iterator but if the argument
    // /// is an iterator, this returns itself.
    // fn iter<'a>(&'a self) -> PyResult<Py<'p, ::objects::PyIterator<'a>>>;
}


impl<'p, T> ObjectProtocol<'p> for T where T: PythonObjectWithGilToken<'p> + ToPythonPointer {

    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    #[inline]
    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool> where N: ToPyObject {
        attr_name.with_borrowed_ptr(self.gil(), |attr_name| unsafe {
            Ok(ffi::PyObject_HasAttr(self.as_ptr(), attr_name) != 0)
        })
    }

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    #[inline]
        fn getattr<N>(&self, attr_name: N) -> PyResult<PyObject<'p>> where N: ToPyObject
    {
        attr_name.with_borrowed_ptr(self.gil(), |attr_name| unsafe {
            PyObject::from_owned_ptr_or_err(
                self.gil(), ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Sets an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name = value'.
    #[inline]
    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
        where N: ToPyObject, V: ToPyObject
    {
        attr_name.with_borrowed_ptr(
            self.gil(), move |attr_name|
            value.with_borrowed_ptr(self.gil(), |value| unsafe {
                err::error_on_minusone(
                    self.gil(), ffi::PyObject_SetAttr(self.as_ptr(), attr_name, value))
            }))
    }

    /// Deletes an attribute.
    /// This is equivalent to the Python expression 'del self.attr_name'.
    #[inline]
    fn delattr<N>(&self, attr_name: N) -> PyResult<()> where N: ToPyObject {
        attr_name.with_borrowed_ptr(self.gil(), |attr_name| unsafe {
            err::error_on_minusone(self.gil(),
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
    fn compare<O>(&self, other: O) -> PyResult<Ordering> where O: ToPyObject {
        unsafe fn do_compare(py: Python,
                             a: *mut ffi::PyObject,
                             b: *mut ffi::PyObject) -> PyResult<Ordering> {
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

        other.with_borrowed_ptr(self.gil(), |other| unsafe {
            do_compare(self.gil(), self.as_ptr(), other)
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
    fn rich_compare<O>(&self, other: O, compare_op: ::CompareOp)
                       -> PyResult<PyObject<'p>> where O: ToPyObject {
        unsafe {
            other.with_borrowed_ptr(self.gil(), |other| {
                PyObject::from_owned_ptr_or_err(
                    self.gil(), ffi::PyObject_RichCompare(
                        self.as_ptr(), other, compare_op as libc::c_int))
            })
        }
    }

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'repr(self)'.
    #[inline]
    fn repr(&self) -> PyResult<PyString<'p>> {
        Ok(PyString::downcast_from_owned_ptr(
            self.gil(), unsafe{ffi::PyObject_Repr(self.as_ptr())})?)
    }

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'str(self)'.
    #[inline]
    fn str(&self) -> PyResult<PyString<'p>> {
        Ok(PyString::downcast_from_owned_ptr(
            self.gil(), unsafe{ffi::PyObject_Str(self.as_ptr())})?)
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
    fn call<A>(&self, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject<'p>>
        where A: ToPyTuple
    {
        let t = args.to_py_tuple(self.gil());
        unsafe {
            PyObject::from_owned_ptr_or_err(
                self.gil(),
                ffi::PyObject_Call(self.as_ptr(), t.as_ptr(), kwargs.as_ptr()))
        }
    }

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
    #[inline]
    fn call_method<A>(&self,
                      name: &str, args: A,
                      kwargs: Option<&PyDict>) -> PyResult<PyObject<'p>>
        where A: ToPyTuple
    {
        name.with_borrowed_ptr(self.gil(), |name| unsafe {
            let t = args.to_py_tuple(self.gil());
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            PyObject::from_owned_ptr_or_err(
                self.gil(),
                ffi::PyObject_Call(ptr, t.as_ptr(), kwargs.as_ptr()))
        })
    }

    /// Retrieves the hash code of the object.
    /// This is equivalent to the Python expression: 'hash(self)'
    #[inline]
    fn hash(&self) -> PyResult<::Py_hash_t> {
        let v = unsafe { ffi::PyObject_Hash(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.gil()))
        } else {
            Ok(v)
        }
    }

    /// Returns whether the object is considered to be true.
    /// This is equivalent to the Python expression: 'not not self'
    #[inline]
    fn is_true(&self) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.gil()))
        } else {
            Ok(v != 0)
        }
    }

    /// Returns whether the object is considered to be None.
    /// This is equivalent to the Python expression: 'is None'
    #[inline]
    fn is_none(&self) -> bool {
        unsafe { ffi::Py_None() == self.as_ptr() }
    }

    /// Returns the length of the sequence or mapping.
    /// This is equivalent to the Python expression: 'len(self)'
    #[inline]
    fn len(&self) -> PyResult<usize> {
        let v = unsafe { ffi::PyObject_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.gil()))
        } else {
            Ok(v as usize)
        }
    }

    /// This is equivalent to the Python expression: 'self[key]'
    #[inline]
    fn get_item<K>(&self, key: K) -> PyResult<PyObject<'p>> where K: ToPyObject {
        key.with_borrowed_ptr(self.gil(), |key| unsafe {
            PyObject::from_owned_ptr_or_err(
                self.gil(), ffi::PyObject_GetItem(self.as_ptr(), key))
        })
    }

    /// Sets an item value.
    /// This is equivalent to the Python expression 'self[key] = value'.
    #[inline]
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
        where K: ToPyObject, V: ToPyObject
    {
        key.with_borrowed_ptr(
            self.gil(), move |key|
            value.with_borrowed_ptr(self.gil(), |value| unsafe {
                err::error_on_minusone(self.gil(),
                    ffi::PyObject_SetItem(self.as_ptr(), key, value))
            }))
    }

    /// Deletes an item.
    /// This is equivalent to the Python expression 'del self[key]'.
    #[inline]
    fn del_item<K>(&self, key: K) -> PyResult<()> where K: ToPyObject {
        key.with_borrowed_ptr(self.gil(), |key| unsafe {
            err::error_on_minusone(self.gil(),
                ffi::PyObject_DelItem(self.as_ptr(), key))
        })
    }

    // /// Takes an object and returns an iterator for it.
    // /// This is typically a new iterator but if the argument
    // /// is an iterator, this returns itself.
    //#[inline]
    //pub fn iter<'a>(&'a self) -> PyResult<Py<'p, ::objects::PyIterator<'a>>> {
    //    Py::from_owned_ptr_or_err(self.py(), ffi::PyObject_GetIter(self.as_ptr()))
    //}
}


impl<'p, T> fmt::Debug for Py<'p, T> where T: ObjectProtocol<'p> + PyTypeInfo {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: we shouldn't use fmt::Error when repr() fails
        let repr_obj = try!(self.repr().map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy())
    }
}

impl<'p, T> fmt::Display for Py<'p, T> where T: ObjectProtocol<'p> + PyTypeInfo {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // TODO: we shouldn't use fmt::Error when str() fails
        let str_obj = try!(self.str().map_err(|_| fmt::Error));
        f.write_str(&str_obj.to_string_lossy())
    }
}

impl<'p, T> fmt::Debug for PyPtr<T> where T: ObjectProtocol<'p> + PyTypeInfo {
    default fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // TODO: we shouldn't use fmt::Error when repr() fails
        let r = self.as_ref(py);
        let repr_obj = try!(r.repr().map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy())
    }
}

impl<'p, T> fmt::Display for PyPtr<T> where T: ObjectProtocol<'p> + PyTypeInfo {
    default fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // TODO: we shouldn't use fmt::Error when repr() fails
        let r = self.as_ref(py);
        let repr_obj = try!(r.as_ref().str().map_err(|_| fmt::Error));
        f.write_str(&repr_obj.to_string_lossy())
    }
}

#[cfg(test)]
mod test {
    use python::{Python};
    use conversion::ToPyObject;
    //use objects::{PyTuple}; //PyList,
    use super::ObjectProtocol;

    #[test]
    fn test_debug_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        assert_eq!(format!("{:?}", v), "'Hello\\n'");
    }

    #[test]
    fn test_display_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        assert_eq!(format!("{}", v), "Hello\n");
    }

    #[test]
    fn test_compare() {
        use std::cmp::Ordering;
        let gil = Python::acquire_gil();
        let py = gil.python();
        let one = 1i32.to_object(py).into_object(py);
        assert_eq!(one.compare(1).unwrap(), Ordering::Equal);
        assert_eq!(one.compare(2).unwrap(), Ordering::Less);
        assert_eq!(one.compare(0).unwrap(), Ordering::Greater);
    }
}
