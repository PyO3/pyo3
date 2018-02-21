// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::cmp::Ordering;
use std::os::raw::c_int;

use ffi;
use err::{self, PyErr, PyResult, PyDowncastError};
use python::{Python, ToPyPointer, IntoPyPointer, IntoPyDictPointer};
use object::PyObject;
use objects::{PyObjectRef, PyString, PyIterator, PyType, PyTuple};
use conversion::{ToPyObject, ToBorrowedObject,
                 IntoPyTuple, FromPyObject, PyTryFrom};
use instance::PyObjectWithToken;
use typeob::PyTypeInfo;


/// Python object model helper methods
#[cfg_attr(feature = "cargo-clippy", allow(len_without_is_empty))]
pub trait ObjectProtocol {

    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression 'hasattr(self, attr_name)'.
    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool> where N: ToPyObject;

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name'.
    fn getattr<N>(&self, attr_name: N) -> PyResult<&PyObjectRef> where N: ToPyObject;

    /// Sets an attribute value.
    /// This is equivalent to the Python expression 'self.attr_name = value'.
    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
        where N: ToBorrowedObject, V: ToBorrowedObject;

    /// Deletes an attribute.
    /// This is equivalent to the Python expression 'del self.attr_name'.
    fn delattr<N>(&self, attr_name: N) -> PyResult<()> where N: ToPyObject;

    /// Compares two Python objects.
    ///
    /// On Python 2, this is equivalent to the Python expression 'cmp(self, other)'.
    ///
    /// On Python 3, this is equivalent to:
    /// ```python,ignore
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
    fn call<A, K>(&self, args: A, kwargs: K) -> PyResult<&PyObjectRef>
        where A: IntoPyTuple,
              K: IntoPyDictPointer;

    /// Calls the object.
    /// This is equivalent to the Python expression: 'self()'
    fn call0(&self) -> PyResult<&PyObjectRef>;

    /// Calls the object.
    /// This is equivalent to the Python expression: 'self(*args)'
    fn call1<A>(&self, args: A) -> PyResult<&PyObjectRef> where A: IntoPyTuple;

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name(*args, **kwargs)'
    ///
    /// # Example
    /// ```rust,ignore
    /// let obj = SomePyObject::new();
    /// let args = (arg1, arg2, arg3);
    /// let kwargs = ((key1, value1), (key2, value2));
    /// let pid = obj.call_method("do_something", args, kwargs);
    /// ```
    fn call_method<A, K>(&self, name: &str, args: A, kwargs: K) -> PyResult<&PyObjectRef>
        where A: IntoPyTuple,
              K: IntoPyDictPointer;

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: 'self.name()'
    fn call_method0(&self, name: &str) -> PyResult<&PyObjectRef>;

    /// Calls a method on the object with positional arguments only .
    /// This is equivalent to the Python expression: 'self.name(*args)'
    fn call_method1<A: IntoPyTuple>(&self, name: &str, args: A) -> PyResult<&PyObjectRef>;

    /// Retrieves the hash code of the object.
    /// This is equivalent to the Python expression: 'hash(self)'
    fn hash(&self) -> PyResult<isize>;

    /// Returns whether the object is considered to be true.
    /// This is equivalent to the Python expression: 'not not self'
    fn is_true(&self) -> PyResult<bool>;

    /// Returns whether the object is considered to be None.
    /// This is equivalent to the Python expression: 'is None'
    fn is_none(&self) -> bool;

    /// Returns the length of the sequence or mapping.
    /// This is equivalent to the Python expression: 'len(self)'
    fn len(&self) -> PyResult<usize>;

    /// This is equivalent to the Python expression: 'self[key]'
    fn get_item<K>(&self, key: K) -> PyResult<&PyObjectRef> where K: ToBorrowedObject;

    /// Sets an item value.
    /// This is equivalent to the Python expression 'self[key] = value'.
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
        where K: ToBorrowedObject, V: ToBorrowedObject;

    /// Deletes an item.
    /// This is equivalent to the Python expression 'del self[key]'.
    fn del_item<K>(&self, key: K) -> PyResult<()> where K: ToBorrowedObject;

    /// Takes an object and returns an iterator for it.
    /// This is typically a new iterator but if the argument
    /// is an iterator, this returns itself.
    fn iter(&self) -> PyResult<PyIterator>;

    /// Gets the Python type object for this object's type.
    fn get_type(&self) -> &PyType;

    /// Gets the Python base object for this object.
    fn get_base(&self) -> &<Self as PyTypeInfo>::BaseType where Self: PyTypeInfo;

    /// Gets the Python base object for this object.
    #[cfg_attr(feature = "cargo-clippy", allow(mut_from_ref))]
    fn get_mut_base(&self) -> &mut <Self as PyTypeInfo>::BaseType where Self: PyTypeInfo;

    /// Casts the PyObject to a concrete Python object type.
    fn cast_as<'a, D>(&'a self) -> Result<&'a D, <D as PyTryFrom>::Error>
        where D: PyTryFrom<Error=PyDowncastError>,
              &'a PyObjectRef: std::convert::From<&'a Self>;

    /// Extracts some type from the Python object.
    /// This is a wrapper function around `FromPyObject::extract()`.
    fn extract<'a, D>(&'a self) -> PyResult<D>
        where D: FromPyObject<'a>,
              &'a PyObjectRef: std::convert::From<&'a Self>;

    /// Returns reference count for python object.
    fn get_refcnt(&self) -> isize;

    /// Gets the Python builtin value `None`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    fn None(&self) -> PyObject;

}


impl<T> ObjectProtocol for T where T: PyObjectWithToken + ToPyPointer {

    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool> where N: ToPyObject {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            Ok(ffi::PyObject_HasAttr(self.as_ptr(), attr_name) != 0)
        })
    }

    fn getattr<N>(&self, attr_name: N) -> PyResult<&PyObjectRef> where N: ToPyObject
    {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            self.py().from_owned_ptr_or_err(
                ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
        where N: ToBorrowedObject, V: ToBorrowedObject
    {
        attr_name.with_borrowed_ptr(
            self.py(), move |attr_name|
            value.with_borrowed_ptr(self.py(), |value| unsafe {
                err::error_on_minusone(
                    self.py(), ffi::PyObject_SetAttr(self.as_ptr(), attr_name, value))
            }))
    }

    fn delattr<N>(&self, attr_name: N) -> PyResult<()> where N: ToPyObject {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            err::error_on_minusone(self.py(),
                ffi::PyObject_DelAttr(self.as_ptr(), attr_name))
        })
    }

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
            Err(::exc::TypeError::new(
                "ObjectProtocol::compare(): All comparisons returned false"))
        }

        other.with_borrowed_ptr(self.py(), |other| unsafe {
            do_compare(self.py(), self.as_ptr(), other)
        })
    }

    fn rich_compare<O>(&self, other: O, compare_op: ::CompareOp)
                       -> PyResult<PyObject> where O: ToPyObject {
        unsafe {
            other.with_borrowed_ptr(self.py(), |other| {
                PyObject::from_owned_ptr_or_err(
                    self.py(), ffi::PyObject_RichCompare(
                        self.as_ptr(), other, compare_op as c_int))
            })
        }
    }

    fn repr(&self) -> PyResult<&PyString> {
        unsafe {
            self.py().from_owned_ptr_or_err(ffi::PyObject_Repr(self.as_ptr()))
        }
    }

    fn str(&self) -> PyResult<&PyString> {
        unsafe {
            self.py().from_owned_ptr_or_err(ffi::PyObject_Str(self.as_ptr()))
        }
    }

    fn is_callable(&self) -> bool {
        unsafe {
            ffi::PyCallable_Check(self.as_ptr()) != 0
        }
    }

    fn call<A, K>(&self, args: A, kwargs: K) -> PyResult<&PyObjectRef>
        where A: IntoPyTuple,
              K: IntoPyDictPointer
    {
        let args = args.into_tuple(self.py()).into_ptr();
        let kw_ptr = kwargs.into_dict_ptr(self.py());
        let result = unsafe {
            self.py().from_owned_ptr_or_err(
                ffi::PyObject_Call(self.as_ptr(), args, kw_ptr))
        };
        self.py().xdecref(args);
        self.py().xdecref(kw_ptr);
        result
    }

    fn call0(&self) -> PyResult<&PyObjectRef>
    {
        let args = PyTuple::empty(self.py()).into_ptr();
        let result = unsafe {
            self.py().from_owned_ptr_or_err(
                ffi::PyObject_Call(self.as_ptr(), args, std::ptr::null_mut()))
        };
        self.py().xdecref(args);
        result
    }

    fn call1<A>(&self, args: A) -> PyResult<&PyObjectRef>
        where A: IntoPyTuple
    {
        let args = args.into_tuple(self.py()).into_ptr();
        let result = unsafe {
            self.py().from_owned_ptr_or_err(
                ffi::PyObject_Call(self.as_ptr(), args, std::ptr::null_mut()))
        };
        self.py().xdecref(args);
        result
    }

    fn call_method<A, K>(&self, name: &str, args: A, kwargs: K)
                      -> PyResult<&PyObjectRef>
        where A: IntoPyTuple,
              K: IntoPyDictPointer
    {
        name.with_borrowed_ptr(self.py(), |name| unsafe {
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            if ptr.is_null() {
                return Err(PyErr::fetch(self.py()))
            }
            let args = args.into_tuple(self.py()).into_ptr();
            let kw_ptr = kwargs.into_dict_ptr(self.py());
            let result = self.py().from_owned_ptr_or_err(
                ffi::PyObject_Call(ptr, args, kw_ptr));
            ffi::Py_DECREF(ptr);
            self.py().xdecref(args);
            self.py().xdecref(kw_ptr);
            result
        })
    }

    fn call_method0(&self, name: &str) -> PyResult<&PyObjectRef>
    {
        name.with_borrowed_ptr(self.py(), |name| unsafe {
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            if ptr.is_null() {
                return Err(PyErr::fetch(self.py()))
            }
            let args = PyTuple::empty(self.py()).into_ptr();
            let result = self.py().from_owned_ptr_or_err(
                ffi::PyObject_Call(ptr, args, std::ptr::null_mut()));
            ffi::Py_DECREF(ptr);
            self.py().xdecref(args);
            result
        })
    }

    fn call_method1<A: IntoPyTuple>(&self, name: &str, args: A) -> PyResult<&PyObjectRef>
    {
        name.with_borrowed_ptr(self.py(), |name| unsafe {
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            if ptr.is_null() {
                return Err(PyErr::fetch(self.py()))
            }
            let args = args.into_tuple(self.py()).into_ptr();
            let result = self.py().from_owned_ptr_or_err(
                ffi::PyObject_Call(ptr, args, std::ptr::null_mut()));
            ffi::Py_DECREF(ptr);
            self.py().xdecref(args);
            result
        })
    }

    fn hash(&self) -> PyResult<isize> {
        let v = unsafe { ffi::PyObject_Hash(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(v)
        }
    }

    fn is_true(&self) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(v != 0)
        }
    }

    fn is_none(&self) -> bool {
        unsafe { ffi::Py_None() == self.as_ptr() }
    }

    fn len(&self) -> PyResult<usize> {
        let v = unsafe { ffi::PyObject_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(v as usize)
        }
    }

    fn get_item<K>(&self, key: K) -> PyResult<&PyObjectRef> where K: ToBorrowedObject {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            self.py().from_owned_ptr_or_err(
                ffi::PyObject_GetItem(self.as_ptr(), key))
        })
    }

    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
        where K: ToBorrowedObject, V: ToBorrowedObject
    {
        key.with_borrowed_ptr(
            self.py(), move |key|
            value.with_borrowed_ptr(self.py(), |value| unsafe {
                err::error_on_minusone(
                    self.py(), ffi::PyObject_SetItem(self.as_ptr(), key, value))
            }))
    }

    fn del_item<K>(&self, key: K) -> PyResult<()> where K: ToBorrowedObject {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            err::error_on_minusone(
                self.py(), ffi::PyObject_DelItem(self.as_ptr(), key))
        })
    }

    fn iter(&self) -> PyResult<PyIterator> {
       Ok(PyIterator::from_object(self.py(), self)?)
    }

    fn get_type(&self) -> &PyType {
        unsafe {
            PyType::from_type_ptr(self.py(), (*self.as_ptr()).ob_type)
        }
    }

    fn get_base(&self) -> &<Self as PyTypeInfo>::BaseType where Self: PyTypeInfo
    {
        unsafe { self.py().from_borrowed_ptr(self.as_ptr()) }
    }

    fn get_mut_base(&self) -> &mut <Self as PyTypeInfo>::BaseType where Self: PyTypeInfo
    {
        unsafe { self.py().mut_from_borrowed_ptr(self.as_ptr()) }
    }

    fn cast_as<'a, D>(&'a self) -> Result<&'a D, <D as PyTryFrom>::Error>
        where D: PyTryFrom<Error=PyDowncastError>,
              &'a PyObjectRef: std::convert::From<&'a Self>
    {
        D::try_from(self.into())
    }

    fn extract<'a, D>(&'a self) -> PyResult<D>
        where D: FromPyObject<'a>,
              &'a PyObjectRef: std::convert::From<&'a T>
    {
        FromPyObject::extract(self.into())
    }

    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    fn None(&self) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(self.py(), ffi::Py_None()) }
    }

    fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) }
    }
}

#[cfg(test)]
mod test {
    use instance::AsPyRef;
    use python::Python;
    use conversion::{ToPyObject, PyTryFrom};
    use objects::PyString;
    use super::*;

    #[test]
    fn test_debug_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        let s = PyString::try_from(v.as_ref(py)).unwrap();
        assert_eq!(format!("{:?}", s), "'Hello\\n'");
    }

    #[test]
    fn test_display_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        let s = PyString::try_from(v.as_ref(py)).unwrap();
        assert_eq!(format!("{}", s), "Hello\n");
    }

    #[test]
    fn test_call_for_non_existing_method() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let a = py.eval("42", None, None).unwrap();
        a.call_method0("__str__").unwrap();  // ok
        assert!(a.call_method("nonexistent_method", (1,), ()).is_err());
        assert!(a.call_method0("nonexistent_method").is_err());
        assert!(a.call_method1("nonexistent_method", (1,)).is_err());
    }
}
