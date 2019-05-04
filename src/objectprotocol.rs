// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::class::basic::CompareOp;
use crate::err::{self, PyDowncastError, PyErr, PyResult};
use crate::exceptions::TypeError;
use crate::ffi;
use crate::instance::PyNativeType;
use crate::object::PyObject;
use crate::type_object::PyTypeInfo;
use crate::types::{PyAny, PyDict, PyIterator, PyString, PyTuple, PyType};
use crate::AsPyPointer;
use crate::IntoPyPointer;
use crate::Py;
use crate::Python;
use crate::{FromPyObject, IntoPy, PyTryFrom, ToBorrowedObject, ToPyObject};
use std::cmp::Ordering;
use std::os::raw::c_int;

/// Python object model helper methods
pub trait ObjectProtocol {
    /// Determines whether this object has the given attribute.
    /// This is equivalent to the Python expression `hasattr(self, attr_name)`.
    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool>
    where
        N: ToPyObject;

    /// Retrieves an attribute value.
    /// This is equivalent to the Python expression `self.attr_name`.
    fn getattr<N>(&self, attr_name: N) -> PyResult<&PyAny>
    where
        N: ToPyObject;

    /// Sets an attribute value.
    /// This is equivalent to the Python expression `self.attr_name = value`.
    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
    where
        N: ToBorrowedObject,
        V: ToBorrowedObject;

    /// Deletes an attribute.
    /// This is equivalent to the Python expression `del self.attr_name`.
    fn delattr<N>(&self, attr_name: N) -> PyResult<()>
    where
        N: ToPyObject;

    /// Compares two Python objects.
    ///
    /// This is equivalent to:
    /// ```python
    /// if self == other:
    ///     return Equal
    /// elif a < b:
    ///     return Less
    /// elif a > b:
    ///     return Greater
    /// else:
    ///     raise TypeError("ObjectProtocol::compare(): All comparisons returned false")
    /// ```
    fn compare<O>(&self, other: O) -> PyResult<Ordering>
    where
        O: ToPyObject;

    /// Compares two Python objects.
    ///
    /// Depending on the value of `compare_op`, equivalent to one of the following Python expressions:
    ///   * CompareOp::Eq: `self == other`
    ///   * CompareOp::Ne: `self != other`
    ///   * CompareOp::Lt: `self < other`
    ///   * CompareOp::Le: `self <= other`
    ///   * CompareOp::Gt: `self > other`
    ///   * CompareOp::Ge: `self >= other`
    fn rich_compare<O>(&self, other: O, compare_op: CompareOp) -> PyResult<PyObject>
    where
        O: ToPyObject;

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression `repr(self)`.
    fn repr(&self) -> PyResult<&PyString>;

    /// Compute the string representation of self.
    /// This is equivalent to the Python expression `str(self)`.
    fn str(&self) -> PyResult<&PyString>;

    /// Determines whether this object is callable.
    fn is_callable(&self) -> bool;

    /// Calls the object.
    /// This is equivalent to the Python expression: `self(*args, **kwargs)`.
    fn call(&self, args: impl IntoPy<Py<PyTuple>>, kwargs: Option<&PyDict>) -> PyResult<&PyAny>;

    /// Calls the object.
    /// This is equivalent to the Python expression: `self()`.
    fn call0(&self) -> PyResult<&PyAny>;

    /// Calls the object.
    /// This is equivalent to the Python expression: `self(*args)`.
    fn call1(&self, args: impl IntoPy<Py<PyTuple>>) -> PyResult<&PyAny>;

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: `self.name(*args, **kwargs)`.
    ///
    /// # Example
    /// ```rust
    /// # use pyo3::prelude::*;
    /// use pyo3::types::IntoPyDict;
    ///
    /// let gil = Python::acquire_gil();
    /// let py = gil.python();
    /// let list = vec![3, 6, 5, 4, 7].to_object(py);
    /// let dict = vec![("reverse", true)].into_py_dict(py);
    /// list.call_method(py, "sort", (), Some(dict)).unwrap();
    /// assert_eq!(list.extract::<Vec<i32>>(py).unwrap(), vec![7, 6, 5, 4, 3]);
    /// ```
    fn call_method(
        &self,
        name: &str,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<&PyAny>;

    /// Calls a method on the object.
    /// This is equivalent to the Python expression: `self.name()`.
    fn call_method0(&self, name: &str) -> PyResult<&PyAny>;

    /// Calls a method on the object with positional arguments only.
    /// This is equivalent to the Python expression: `self.name(*args)`.
    fn call_method1(&self, name: &str, args: impl IntoPy<Py<PyTuple>>) -> PyResult<&PyAny>;

    /// Retrieves the hash code of the object.
    /// This is equivalent to the Python expression: `hash(self)`.
    fn hash(&self) -> PyResult<isize>;

    /// Returns whether the object is considered to be true.
    /// This is equivalent to the Python expression: `not not self`.
    fn is_true(&self) -> PyResult<bool>;

    /// Returns whether the object is considered to be None.
    /// This is equivalent to the Python expression: `is None`.
    fn is_none(&self) -> bool;

    /// Returns the length of the sequence or mapping.
    /// This is equivalent to the Python expression: `len(self)`.
    fn len(&self) -> PyResult<usize>;

    /// Returns true if the sequence or mapping has a length of 0.
    /// This is equivalent to the Python expression: `len(self) == 0`.
    fn is_empty(&self) -> PyResult<bool>;

    /// This is equivalent to the Python expression: `self[key]`.
    fn get_item<K>(&self, key: K) -> PyResult<&PyAny>
    where
        K: ToBorrowedObject;

    /// Sets an item value.
    /// This is equivalent to the Python expression `self[key] = value`.
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToBorrowedObject,
        V: ToBorrowedObject;

    /// Deletes an item.
    /// This is equivalent to the Python expression `del self[key]`.
    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToBorrowedObject;

    /// Takes an object and returns an iterator for it.
    /// This is typically a new iterator but if the argument
    /// is an iterator, this returns itself.
    fn iter(&self) -> PyResult<PyIterator>;

    /// Gets the Python type object for this object's type.
    fn get_type(&self) -> &PyType;

    /// Gets the Python type pointer for this object.
    fn get_type_ptr(&self) -> *mut ffi::PyTypeObject;

    /// Gets the Python base object for this object.
    fn get_base(&self) -> &<Self as PyTypeInfo>::BaseType
    where
        Self: PyTypeInfo;

    /// Gets the Python base object for this object.

    fn get_mut_base(&mut self) -> &mut <Self as PyTypeInfo>::BaseType
    where
        Self: PyTypeInfo;

    /// Casts the PyObject to a concrete Python object type.
    fn cast_as<'a, D>(&'a self) -> Result<&'a D, PyDowncastError>
    where
        D: PyTryFrom<'a>,
        &'a PyAny: std::convert::From<&'a Self>;

    /// Extracts some type from the Python object.
    /// This is a wrapper function around `FromPyObject::extract()`.
    fn extract<'a, D>(&'a self) -> PyResult<D>
    where
        D: FromPyObject<'a>,
        &'a PyAny: std::convert::From<&'a Self>;

    /// Returns reference count for python object.
    fn get_refcnt(&self) -> isize;

    /// Gets the Python builtin value `None`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    fn None(&self) -> PyObject;
}

impl<T> ObjectProtocol for T
where
    T: PyNativeType + AsPyPointer,
{
    fn hasattr<N>(&self, attr_name: N) -> PyResult<bool>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            Ok(ffi::PyObject_HasAttr(self.as_ptr(), attr_name) != 0)
        })
    }

    fn getattr<N>(&self, attr_name: N) -> PyResult<&PyAny>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    fn setattr<N, V>(&self, attr_name: N, value: V) -> PyResult<()>
    where
        N: ToBorrowedObject,
        V: ToBorrowedObject,
    {
        attr_name.with_borrowed_ptr(self.py(), move |attr_name| {
            value.with_borrowed_ptr(self.py(), |value| unsafe {
                err::error_on_minusone(
                    self.py(),
                    ffi::PyObject_SetAttr(self.as_ptr(), attr_name, value),
                )
            })
        })
    }

    fn delattr<N>(&self, attr_name: N) -> PyResult<()>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(self.py(), |attr_name| unsafe {
            err::error_on_minusone(self.py(), ffi::PyObject_DelAttr(self.as_ptr(), attr_name))
        })
    }

    fn compare<O>(&self, other: O) -> PyResult<Ordering>
    where
        O: ToPyObject,
    {
        unsafe fn do_compare(
            py: Python,
            a: *mut ffi::PyObject,
            b: *mut ffi::PyObject,
        ) -> PyResult<Ordering> {
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
            Err(TypeError::py_err(
                "ObjectProtocol::compare(): All comparisons returned false",
            ))
        }

        other.with_borrowed_ptr(self.py(), |other| unsafe {
            do_compare(self.py(), self.as_ptr(), other)
        })
    }

    fn rich_compare<O>(&self, other: O, compare_op: CompareOp) -> PyResult<PyObject>
    where
        O: ToPyObject,
    {
        unsafe {
            other.with_borrowed_ptr(self.py(), |other| {
                PyObject::from_owned_ptr_or_err(
                    self.py(),
                    ffi::PyObject_RichCompare(self.as_ptr(), other, compare_op as c_int),
                )
            })
        }
    }

    fn repr(&self) -> PyResult<&PyString> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyObject_Repr(self.as_ptr()))
        }
    }

    fn str(&self) -> PyResult<&PyString> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyObject_Str(self.as_ptr()))
        }
    }

    fn is_callable(&self) -> bool {
        unsafe { ffi::PyCallable_Check(self.as_ptr()) != 0 }
    }

    fn call(&self, args: impl IntoPy<Py<PyTuple>>, kwargs: Option<&PyDict>) -> PyResult<&PyAny> {
        let args = args.into_py(self.py()).into_ptr();
        let kwargs = kwargs.into_ptr();
        let result = unsafe {
            let return_value = ffi::PyObject_Call(self.as_ptr(), args, kwargs);
            self.py().from_owned_ptr_or_err(return_value)
        };
        unsafe {
            ffi::Py_XDECREF(args);
            ffi::Py_XDECREF(kwargs);
        }
        result
    }

    fn call0(&self) -> PyResult<&PyAny> {
        self.call((), None)
    }

    fn call1(&self, args: impl IntoPy<Py<PyTuple>>) -> PyResult<&PyAny> {
        self.call(args, None)
    }

    fn call_method(
        &self,
        name: &str,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<&PyAny> {
        name.with_borrowed_ptr(self.py(), |name| unsafe {
            let py = self.py();
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            if ptr.is_null() {
                return Err(PyErr::fetch(py));
            }
            let args = args.into_py(py).into_ptr();
            let kwargs = kwargs.into_ptr();
            let result_ptr = ffi::PyObject_Call(ptr, args, kwargs);
            let result = py.from_owned_ptr_or_err(result_ptr);
            ffi::Py_DECREF(ptr);
            ffi::Py_XDECREF(args);
            ffi::Py_XDECREF(kwargs);
            result
        })
    }

    fn call_method0(&self, name: &str) -> PyResult<&PyAny> {
        self.call_method(name, (), None)
    }

    fn call_method1(&self, name: &str, args: impl IntoPy<Py<PyTuple>>) -> PyResult<&PyAny> {
        self.call_method(name, args, None)
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

    fn is_empty(&self) -> PyResult<bool> {
        self.len().map(|l| l == 0)
    }

    fn get_item<K>(&self, key: K) -> PyResult<&PyAny>
    where
        K: ToBorrowedObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyObject_GetItem(self.as_ptr(), key))
        })
    }

    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToBorrowedObject,
        V: ToBorrowedObject,
    {
        key.with_borrowed_ptr(self.py(), move |key| {
            value.with_borrowed_ptr(self.py(), |value| unsafe {
                err::error_on_minusone(self.py(), ffi::PyObject_SetItem(self.as_ptr(), key, value))
            })
        })
    }

    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToBorrowedObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            err::error_on_minusone(self.py(), ffi::PyObject_DelItem(self.as_ptr(), key))
        })
    }

    fn iter(&self) -> PyResult<PyIterator> {
        Ok(PyIterator::from_object(self.py(), self)?)
    }

    fn get_type(&self) -> &PyType {
        unsafe { PyType::from_type_ptr(self.py(), (*self.as_ptr()).ob_type) }
    }

    #[inline]
    fn get_type_ptr(&self) -> *mut ffi::PyTypeObject {
        unsafe { (*self.as_ptr()).ob_type }
    }

    fn get_base(&self) -> &<Self as PyTypeInfo>::BaseType
    where
        Self: PyTypeInfo,
    {
        unsafe { self.py().from_borrowed_ptr(self.as_ptr()) }
    }

    fn get_mut_base(&mut self) -> &mut <Self as PyTypeInfo>::BaseType
    where
        Self: PyTypeInfo,
    {
        unsafe { self.py().mut_from_borrowed_ptr(self.as_ptr()) }
    }

    fn cast_as<'a, D>(&'a self) -> Result<&'a D, PyDowncastError>
    where
        D: PyTryFrom<'a>,
        &'a PyAny: std::convert::From<&'a Self>,
    {
        D::try_from(self)
    }

    fn extract<'a, D>(&'a self) -> PyResult<D>
    where
        D: FromPyObject<'a>,
        &'a PyAny: std::convert::From<&'a T>,
    {
        FromPyObject::extract(self.into())
    }

    fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.as_ptr()) }
    }

    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    fn None(&self) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(self.py(), ffi::Py_None()) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::instance::AsPyRef;
    use crate::types::{IntoPyDict, PyString};
    use crate::Python;
    use crate::{PyTryFrom, ToPyObject};

    #[test]
    fn test_debug_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        let s = <PyString as PyTryFrom>::try_from(v.as_ref(py)).unwrap();
        assert_eq!(format!("{:?}", s), "'Hello\\n'");
    }

    #[test]
    fn test_display_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "Hello\n".to_object(py);
        let s = <PyString as PyTryFrom>::try_from(v.as_ref(py)).unwrap();
        assert_eq!(format!("{}", s), "Hello\n");
    }

    #[test]
    fn test_call_for_non_existing_method() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let a = py.eval("42", None, None).unwrap();
        a.call_method0("__str__").unwrap(); // ok
        assert!(a.call_method("nonexistent_method", (1,), None).is_err());
        assert!(a.call_method0("nonexistent_method").is_err());
        assert!(a.call_method1("nonexistent_method", (1,)).is_err());
    }

    #[test]
    fn test_call_with_kwargs() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = vec![3, 6, 5, 4, 7].to_object(py);
        let dict = vec![("reverse", true)].into_py_dict(py);
        list.call_method(py, "sort", (), Some(dict)).unwrap();
        assert_eq!(list.extract::<Vec<i32>>(py).unwrap(), vec![7, 6, 5, 4, 3]);
    }

    #[test]
    fn test_type() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = py.eval("42", None, None).unwrap();
        assert_eq!(unsafe { obj.get_type().as_type_ptr() }, obj.get_type_ptr())
    }
}
