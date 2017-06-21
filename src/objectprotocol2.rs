// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::cmp::Ordering;
use std::os::raw::c_int;

use ffi;
use err::{PyErr, PyResult, self};
use python::{Python, ToPyPointer};
use objects::{PyObject, PyDict, PyString, PyIterator, PyType};
use conversion::{ToPyObject, IntoPyTuple};
use token::PyObjectWithToken;


pub trait ObjectProtocol2 {

    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool> where N: ToPyObject;

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    fn getattr<N>(&self, attr_name: N) -> PyResult<PyObject> where N: ToPyObject;

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
    fn rich_compare<O>(&self, other: O, compare_op: ::CompareOp) -> PyResult<PyObject>
        where O: ToPyObject;

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'repr(self)'.
    fn repr(&self) -> PyResult<&PyString>;

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'str(self)'.
    fn str(&self) -> PyResult<&PyString>;

    /// Determines whether this object is callable.
    fn is_callable(&self) -> bool;

    /// Calls the object.
    /// This is equivalent to the Python expression: 'self(*args, **kwargs)'
    fn call<A>(&self, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: IntoPyTuple;

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
    fn call_method<A>(&self, name: &str, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: IntoPyTuple;

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
    fn get_item<K>(&self, key: K) -> PyResult<PyObject> where K: ToPyObject;

    /// Sets an item value.
    /// This is equivalent to the Python expression 'self[key] = value'.
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
        where K: ToPyObject, V: ToPyObject;

    /// Deletes an item.
    /// This is equivalent to the Python expression 'del self[key]'.
    fn del_item<K>(&self, key: K) -> PyResult<()> where K: ToPyObject;

    /// Takes an object and returns an iterator for it.
    /// This is typically a new iterator but if the argument
    /// is an iterator, this returns itself.
    fn iter<'p>(&'p self) -> PyResult<PyIterator<'p>>;

    /// Gets the Python type object for this object's type.
    fn get_type(&self) -> PyType;

    fn get_refcnt(&self) -> isize;

}


impl<'a, T> ObjectProtocol2 for &'a T where T: PyObjectWithToken + ToPyPointer {

    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    #[inline]
    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool> where N: ToPyObject {
        attr_name.with_borrowed_ptr(self.token(), |attr_name| unsafe {
            Ok(ffi::PyObject_HasAttr(self.as_ptr(), attr_name) != 0)
        })
    }

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    #[inline]
    fn getattr<N>(&self, attr_name: N) -> PyResult<PyObject> where N: ToPyObject
    {
        attr_name.with_borrowed_ptr(self.token(), |attr_name| unsafe {
            PyObject::from_owned_ptr_or_err(
                self.token(), ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Sets an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name = value'.
    #[inline]
    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
        where N: ToPyObject, V: ToPyObject
    {
        attr_name.with_borrowed_ptr(
            self.token(), move |attr_name|
            value.with_borrowed_ptr(self.token(), |value| unsafe {
                err::error_on_minusone(
                    self.token(), ffi::PyObject_SetAttr(self.as_ptr(), attr_name, value))
            }))
    }

    /// Deletes an attribute.
    /// This is equivalent to the Python expression 'del self.attr_name'.
    #[inline]
    fn delattr<N>(&self, attr_name: N) -> PyResult<()> where N: ToPyObject {
        attr_name.with_borrowed_ptr(self.token(), |attr_name| unsafe {
            err::error_on_minusone(self.token(),
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
            Err(PyErr::new::<::exc::TypeError, _>(py, "ObjectProtocol::compare(): All comparisons returned false"))
        }

        other.with_borrowed_ptr(self.token(), |other| unsafe {
            do_compare(self.token(), self.as_ptr(), other)
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
                       -> PyResult<PyObject> where O: ToPyObject {
        unsafe {
            other.with_borrowed_ptr(self.token(), |other| {
                PyObject::from_owned_ptr_or_err(
                    self.token(), ffi::PyObject_RichCompare(
                        self.as_ptr(), other, compare_op as c_int))
            })
        }
    }

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'repr(self)'.
    #[inline]
    fn repr(&self) -> PyResult<&PyString> {
        unsafe {
            let py = self.token();
            let obj = PyObject::from_owned_ptr_or_err(
                py, ffi::PyObject_Repr(self.as_ptr()))?;
            Ok(py.unchecked_cast_as::<PyString>(obj))
        }
    }

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression 'str(self)'.
    #[inline]
    fn str(&self) -> PyResult<&PyString> {
        unsafe {
            let py = self.token();
            let obj = PyObject::from_owned_ptr_or_err(
                py, ffi::PyObject_Str(self.as_ptr()))?;
            Ok(py.unchecked_cast_as::<PyString>(obj))
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
    fn call<A>(&self, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: IntoPyTuple
    {
        let t = args.into_tuple(self.token());
        let result = unsafe {
            PyObject::from_borrowed_ptr_or_err(
                self.token(), ffi::PyObject_Call(self.as_ptr(), t.as_ptr(), kwargs.as_ptr()))
        };
        self.token().release(t);
        result
    }

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
    #[inline]
    fn call_method<A>(&self, name: &str, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: IntoPyTuple
    {
        name.with_borrowed_ptr(self.token(), |name| unsafe {
            let t = args.into_tuple(self.token());
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            let result = PyObject::from_borrowed_ptr_or_err(
                self.token(), ffi::PyObject_Call(ptr, t.as_ptr(), kwargs.as_ptr()));
            self.token().release(t);
            result
        })
    }

    /// Retrieves the hash code of the object.
    /// This is equivalent to the Python expression: 'hash(self)'
    #[inline]
    fn hash(&self) -> PyResult<ffi::Py_hash_t> {
        let v = unsafe { ffi::PyObject_Hash(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.token()))
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
            Err(PyErr::fetch(self.token()))
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
            Err(PyErr::fetch(self.token()))
        } else {
            Ok(v as usize)
        }
    }

    /// This is equivalent to the Python expression: 'self[key]'
    #[inline]
    fn get_item<K>(&self, key: K) -> PyResult<PyObject> where K: ToPyObject {
        key.with_borrowed_ptr(self.token(), |key| unsafe {
            PyObject::from_owned_ptr_or_err(
                self.token(), ffi::PyObject_GetItem(self.as_ptr(), key))
        })
    }

    /// Sets an item value.
    /// This is equivalent to the Python expression 'self[key] = value'.
    #[inline]
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
        where K: ToPyObject, V: ToPyObject
    {
        key.with_borrowed_ptr(
            self.token(), move |key|
            value.with_borrowed_ptr(self.token(), |value| unsafe {
                err::error_on_minusone(
                    self.token(), ffi::PyObject_SetItem(self.as_ptr(), key, value))
            }))
    }

    /// Deletes an item.
    /// This is equivalent to the Python expression 'del self[key]'.
    #[inline]
    fn del_item<K>(&self, key: K) -> PyResult<()> where K: ToPyObject {
        key.with_borrowed_ptr(self.token(), |key| unsafe {
            err::error_on_minusone(
                self.token(), ffi::PyObject_DelItem(self.as_ptr(), key))
        })
    }

    /// Takes an object and returns an iterator for it.
    /// This is typically a new iterator but if the argument
    /// is an iterator, this returns itself.
    #[inline]
    fn iter<'p>(&'p self) -> PyResult<PyIterator<'p>> {
        unsafe {
            let ptr = PyObject::from_owned_ptr_or_err(
                self.token(), ffi::PyObject_GetIter(self.as_ptr()))?;
            PyIterator::from_object(self.token(), ptr).map_err(|e| e.into())
        }
    }

    /// Gets the Python type object for this object's type.
    #[inline]
    fn get_type(&self) -> PyType {
        unsafe {
            PyType::from_type_ptr(self.token(), (*self.as_ptr()).ob_type)
        }
    }

    fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) }
    }
}

#[cfg(test)]
mod test {
    use python::{Python, PyDowncastFrom};
    use conversion::ToPyObject;
    use objects::PyString;

    #[test]
    fn test_debug_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        let s = PyString::downcast_from(py, &v).unwrap();
        assert_eq!(format!("{:?}", s), "'Hello\\n'");
    }

    #[test]
    fn test_display_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        let s = PyString::downcast_from(py, &v).unwrap();
        assert_eq!(format!("{}", s), "Hello\n");
    }
}
