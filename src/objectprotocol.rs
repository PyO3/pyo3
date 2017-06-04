// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use libc;
use std::cmp::Ordering;

use ffi;
use err::{PyErr, PyResult, self};
use python::{Python, PyDowncastInto, ToPyPointer};
use objects::{PyObject, PyDict, PyString, PyIterator};
use conversion::{ToPyObject, ToPyTuple};


pub trait ObjectProtocol {

    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    fn hasattr<N>(&self, py: Python, attr_name: N) -> PyResult<bool> where N: ToPyObject;

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    fn getattr<N>(&self, py: Python, attr_name: N) -> PyResult<PyObject> where N: ToPyObject;

    /// Sets an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name = value'.
    fn setattr<N, V>(&self, py: Python, attr_name: N, value: V) -> PyResult<()>
        where N: ToPyObject, V: ToPyObject;

    /// Deletes an attribute.
    /// This is equivalent to the Python expression 'del self.attr_name'.
    fn delattr<N>(&self, py: Python, attr_name: N) -> PyResult<()> where N: ToPyObject;

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
    fn compare<O>(&self, py: Python, other: O) -> PyResult<Ordering> where O: ToPyObject;

    /// Compares two Python objects.
    ///
    /// Depending on the value of `compare_op`, equivalent to one of the following Python expressions:
    ///   * CompareOp::Eq: `self == other`
    ///   * CompareOp::Ne: `self != other`
    ///   * CompareOp::Lt: `self < other`
    ///   * CompareOp::Le: `self <= other`
    ///   * CompareOp::Gt: `self > other`
    ///   * CompareOp::Ge: `self >= other`
    fn rich_compare<O>(&self, py: Python, other: O, compare_op: ::CompareOp)
                       -> PyResult<PyObject> where O: ToPyObject;

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'repr(self)'.
    fn repr(&self, py: Python) -> PyResult<PyString>;

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'str(self)'.
    fn str(&self, py: Python) -> PyResult<PyString>;

    /// Determines whether this object is callable.
    fn is_callable(&self, py: Python) -> bool;

    /// Calls the object.
    /// This is equivalent to the Python expression: 'self(*args, **kwargs)'
    fn call<A>(&self, py: Python, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: ToPyTuple;

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
    fn call_method<A>(&self, py: Python,
                      name: &str, args: A,
                      kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: ToPyTuple;

    /// Retrieves the hash code of the object.
    /// This is equivalent to the Python expression: 'hash(self)'
    fn hash(&self, py: Python) -> PyResult<::Py_hash_t>;

    /// Returns whether the object is considered to be true.
    /// This is equivalent to the Python expression: 'not not self'
    fn is_true(&self, py: Python) -> PyResult<bool>;

    /// Returns whether the object is considered to be None.
    /// This is equivalent to the Python expression: 'is None'
    #[inline]
    fn is_none(&self, py: Python) -> bool;

    /// Returns the length of the sequence or mapping.
    /// This is equivalent to the Python expression: 'len(self)'
    fn len(&self, py: Python) -> PyResult<usize>;

    /// This is equivalent to the Python expression: 'self[key]'
    fn get_item<K>(&self, py: Python, key: K) -> PyResult<PyObject> where K: ToPyObject;

    /// Sets an item value.
    /// This is equivalent to the Python expression 'self[key] = value'.
    fn set_item<K, V>(&self, py: Python, key: K, value: V) -> PyResult<()>
        where K: ToPyObject, V: ToPyObject;

    /// Deletes an item.
    /// This is equivalent to the Python expression 'del self[key]'.
    fn del_item<K>(&self, py: Python, key: K) -> PyResult<()> where K: ToPyObject;

    /// Takes an object and returns an iterator for it.
    /// This is typically a new iterator but if the argument
    /// is an iterator, this returns itself.
    fn iter<'p>(&self, py: Python<'p>) -> PyResult<PyIterator<'p>>;

    fn get_refcnt(&self) -> isize;
}


impl<T> ObjectProtocol for T where T: ToPyPointer {

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
        fn getattr<N>(&self, py: Python, attr_name: N) -> PyResult<PyObject> where N: ToPyObject
    {
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            PyObject::from_owned_ptr_or_err(
                py, ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Sets an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name = value'.
    #[inline]
    fn setattr<N, V>(&self, py: Python, attr_name: N, value: V) -> PyResult<()>
        where N: ToPyObject, V: ToPyObject
    {
        attr_name.with_borrowed_ptr(
            py, move |attr_name|
            value.with_borrowed_ptr(py, |value| unsafe {
                err::error_on_minusone(
                    py, ffi::PyObject_SetAttr(self.as_ptr(), attr_name, value))
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
    fn rich_compare<O>(&self, py: Python, other: O, compare_op: ::CompareOp)
                       -> PyResult<PyObject> where O: ToPyObject {
        unsafe {
            other.with_borrowed_ptr(py, |other| {
                PyObject::from_owned_ptr_or_err(
                    py, ffi::PyObject_RichCompare(
                        self.as_ptr(), other, compare_op as libc::c_int))
            })
        }
    }

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'repr(self)'.
    #[inline]
    fn repr(&self, py: Python) -> PyResult<PyString> {
        Ok(PyString::downcast_from_ptr(
            py, unsafe{ffi::PyObject_Repr(self.as_ptr())})?)
    }

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'str(self)'.
    #[inline]
    fn str(&self, py: Python) -> PyResult<PyString> {
        Ok(PyString::downcast_from_ptr(
            py, unsafe{ffi::PyObject_Str(self.as_ptr())})?)
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
        let t = args.to_py_tuple(py);
        unsafe {
            PyObject::from_owned_ptr_or_err(
                py,
                ffi::PyObject_Call(self.as_ptr(), t.as_ptr(), kwargs.as_ptr()))
        }
    }

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
    #[inline]
    fn call_method<A>(&self, py: Python,
                      name: &str, args: A,
                      kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: ToPyTuple
    {
        name.with_borrowed_ptr(py, |name| unsafe {
            let t = args.to_py_tuple(py);
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            PyObject::from_owned_ptr_or_err(
                py,
                ffi::PyObject_Call(ptr, t.as_ptr(), kwargs.as_ptr()))
        })
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

    /// Returns whether the object is considered to be None.
    /// This is equivalent to the Python expression: 'is None'
    #[inline]
    fn is_none(&self, _py: Python) -> bool {
        unsafe { ffi::Py_None() == self.as_ptr() }
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
            PyObject::from_owned_ptr_or_err(
                py, ffi::PyObject_GetItem(self.as_ptr(), key))
        })
    }

    /// Sets an item value.
    /// This is equivalent to the Python expression 'self[key] = value'.
    #[inline]
    fn set_item<K, V>(&self, py: Python, key: K, value: V) -> PyResult<()>
        where K: ToPyObject, V: ToPyObject
    {
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
    fn iter<'p>(&self, py: Python<'p>) -> PyResult<PyIterator<'p>> {
        unsafe {
            let ptr = PyObject::from_owned_ptr_or_err(
                py, ffi::PyObject_GetIter(self.as_ptr()))?;
            PyIterator::from_object(py, ptr).map_err(|e| e.into())
        }
    }

    fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) }
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
        let one = 1i32.to_object(py);
        assert_eq!(one.compare(py, 1).unwrap(), Ordering::Equal);
        assert_eq!(one.compare(py, 2).unwrap(), Ordering::Less);
        assert_eq!(one.compare(py, 0).unwrap(), Ordering::Greater);
    }
}
