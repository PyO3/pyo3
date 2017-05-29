// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use ::pptr;
use ffi;
use pyptr::PyPtr;
use python::{Python, ToPythonPointer};
use conversion::{ToPyObject, IntoPyObject};
use objects::PyObject;
use token::{PyObjectMarker, PythonObjectWithToken}; //, PyList};
use err::{self, PyResult, PyErr};
use std::{mem, collections, hash, cmp};

/// Represents a Python `dict`.
pub struct PyDict<'p>(pptr<'p>);

pyobject_nativetype!(PyDict, PyDict_Check, PyDict_Type);


impl<'p> PyDict<'p> {
    /// Creates a new empty dictionary.
    ///
    /// May panic when running out of memory.
    pub fn new(py: Python<'p>) -> PyDict<'p> {
        unsafe { PyDict(pptr::from_owned_ptr_or_panic(py, ffi::PyDict_New())) }
    }

    /// Construct a new dict with the given raw pointer
    pub fn from_borrowed_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> PyDict<'p> {
        unsafe { PyDict(pptr::from_borrowed_ptr(py, ptr)) }
    }

    /// Return a new dictionary that contains the same key-value pairs as self.
    /// Corresponds to `dict(self)` in Python.
    pub fn copy(&'p self) -> PyResult<PyDict<'p>> {
        unsafe {
            Ok(PyDict(
                pptr::from_owned_ptr_or_err(self.token(), ffi::PyDict_Copy(self.0.as_ptr()))?
            ))
        }
    }

    /// Empty an existing dictionary of all key-value pairs.
    #[inline]
    pub fn clear(&self) {
        unsafe { ffi::PyDict_Clear(self.as_ptr()) }
    }

    /// Return the number of items in the dictionary.
    /// This is equivalent to len(p) on a dictionary.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ffi::PyDict_Size(self.as_ptr()) as usize }
    }

    /// Determine if the dictionary contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool> where K: ToPyObject {
        key.with_borrowed_ptr(self.token(), |key| unsafe {
            match ffi::PyDict_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.token()))
            }
        })
    }

    /// Gets an item from the dictionary.
    /// Returns None if the item is not present, or if an error occurs.
    pub fn get_item<K>(&self, key: K) -> Option<PyObject> where K: ToPyObject {
        key.with_borrowed_ptr(self.token(), |key| unsafe {
            PyObject::from_borrowed_ptr_or_opt(
                self.token(), ffi::PyDict_GetItem(self.as_ptr(), key))
        })
    }

    /// Sets an item value.
    /// This is equivalent to the Python expression `self[key] = value`.
    pub fn set_item<K, V>(&self, key: K, value: V)
                          -> PyResult<()> where K: ToPyObject, V: ToPyObject {
        key.with_borrowed_ptr(
            self.token(), move |key|
            value.with_borrowed_ptr(self.token(), |value| unsafe {
                err::error_on_minusone(
                    self.token(), ffi::PyDict_SetItem(self.as_ptr(), key, value))
            }))
    }

    /// Deletes an item.
    /// This is equivalent to the Python expression `del self[key]`.
    pub fn del_item<K>(&self, key: K) -> PyResult<()> where K: ToPyObject {
        key.with_borrowed_ptr(self.token(), |key| unsafe {
            err::error_on_minusone(
                self.token(), ffi::PyDict_DelItem(self.as_ptr(), key))
        })
    }

    // List of dict items.
    // This is equivalent to the python expression `list(dict.items())`.
    //pub fn items_list(&self) -> Py<PyList> {
    //    unsafe {
    //        Py::from_owned_ptr_or_panic(self.py(), ffi::PyDict_Items(self.as_ptr()))
    //    }
    //}

    /// Returns the list of (key, value) pairs in this dictionary.
    pub fn items(&self) -> Vec<(&PyObject, &PyObject)> {
        // Note that we don't provide an iterator because
        // PyDict_Next() is unsafe to use when the dictionary might be changed
        // by other python code.
        let token = self.token();
        let mut vec = Vec::with_capacity(self.len());
        unsafe {
            let mut pos = 0;
            let mut key: *mut ffi::PyObject = mem::uninitialized();
            let mut value: *mut ffi::PyObject = mem::uninitialized();
            while ffi::PyDict_Next(self.as_ptr(), &mut pos, &mut key, &mut value) != 0 {
                vec.push((token.from_owned_ptr(key), token.from_owned_ptr(value)));
            }
        }
        vec
    }
}

impl <K, V> ToPyObject for collections::HashMap<K, V>
    where K: hash::Hash+cmp::Eq+ToPyObject,
          V: ToPyObject
{
    fn to_object(&self, py: Python) -> PyPtr<PyObjectMarker> {
        let dict = PyDict::new(py);
        for (key, value) in self {
            dict.set_item(key, value).unwrap();
        };
        dict.into_object(py)
    }
}

impl <K, V> ToPyObject for collections::BTreeMap<K, V>
    where K: cmp::Eq+ToPyObject,
          V: ToPyObject
{
    fn to_object(&self, py: Python) -> PyPtr<PyObjectMarker> {
        let dict = PyDict::new(py);
        for (key, value) in self {
            dict.set_item(key, value).unwrap();
        };
        dict.into_object(py)
    }
}

#[cfg(test)]
mod test {
    use python::{Python, PythonObjectWithCheckedDowncast};
    use conversion::ToPyObject;
    use objects::{PyDict, PyTuple};
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        let dict = PyDict::downcast_from(py, v.to_py_object(py)).unwrap();
        assert_eq!(0, dict.len(py));
        v.insert(7, 32);
        let dict2 = PyDict::downcast_from(py, v.to_py_object(py)).unwrap();
        assert_eq!(1, dict2.len(py));
    }

    #[test]
    fn test_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let dict = PyDict::downcast_from(py, v.to_py_object(py)).unwrap();
        assert_eq!(true, dict.contains(py, 7i32).unwrap());
        assert_eq!(false, dict.contains(py, 8i32).unwrap());
    }

    #[test]
    fn test_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let dict = PyDict::downcast_from(py, v.to_py_object(py)).unwrap();
        assert_eq!(32, dict.get_item(py, 7i32).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(None, dict.get_item(py, 8i32));
    }

    #[test]
    fn test_set_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let dict = PyDict::downcast_from(py, v.to_py_object(py)).unwrap();
        assert!(dict.set_item(py, 7i32, 42i32).is_ok()); // change
        assert!(dict.set_item(py, 8i32, 123i32).is_ok()); // insert
        assert_eq!(42i32, dict.get_item(py, 7i32).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(123i32, dict.get_item(py, 8i32).unwrap().extract::<i32>(py).unwrap());
    }

    #[test]
    fn test_set_item_does_not_update_original_object() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let dict = PyDict::downcast_from(py, v.to_py_object(py)).unwrap();
        assert!(dict.set_item(py, 7i32, 42i32).is_ok()); // change
        assert!(dict.set_item(py, 8i32, 123i32).is_ok()); // insert
        assert_eq!(32i32, *v.get(&7i32).unwrap()); // not updated!
        assert_eq!(None, v.get(&8i32));
    }


    #[test]
    fn test_del_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let dict = PyDict::downcast_from(py, v.to_py_object(py)).unwrap();
        assert!(dict.del_item(py, 7i32).is_ok());
        assert_eq!(0, dict.len(py));
        assert_eq!(None, dict.get_item(py, 7i32));
    }

    #[test]
    fn test_del_item_does_not_update_original_object() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let dict = PyDict::downcast_from(py, v.to_py_object(py)).unwrap();
        assert!(dict.del_item(py, 7i32).is_ok()); // change
        assert_eq!(32i32, *v.get(&7i32).unwrap()); // not updated!
    }

    #[test]
    fn test_items_list() {
    let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        v.insert(8, 42);
        v.insert(9, 123);
        let dict = PyDict::downcast_from(py, v.to_py_object(py)).unwrap();
        // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
        let mut key_sum = 0;
        let mut value_sum = 0;
        for el in dict.items_list(py).iter(py) {
            let tuple = el.cast_into::<PyTuple>(py).unwrap();
            key_sum += tuple.get_item(py, 0).extract::<i32>(py).unwrap();
            value_sum += tuple.get_item(py, 1).extract::<i32>(py).unwrap();
        }
        assert_eq!(7 + 8 + 9, key_sum);
        assert_eq!(32 + 42 + 123, value_sum);
    }

    #[test]
    fn test_items() {
    let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        v.insert(8, 42);
        v.insert(9, 123);
        let dict = PyDict::downcast_from(py, v.to_py_object(py)).unwrap();
        // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
        let mut key_sum = 0;
        let mut value_sum = 0;
        for (key, value) in dict.items(py) {
            key_sum += key.extract::<i32>(py).unwrap();
            value_sum += value.extract::<i32>(py).unwrap();
        }
        assert_eq!(7 + 8 + 9, key_sum);
        assert_eq!(32 + 42 + 123, value_sum);
    }

    #[test]
    fn test_hashmap_to_python() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut map = HashMap::<i32, i32>::new();
        map.insert(1, 1);

        let py_map = PyDict::downcast_from(py, map.to_py_object(py)).unwrap();

        assert!(py_map.len(py) == 1);
        assert!( py_map.get_item(py, 1).unwrap().extract::<i32>(py).unwrap() == 1);
    }

    fn test_btreemap_to_python() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut map = BTreeMap::<i32, i32>::new();
        map.insert(1, 1);

        let py_map = PyDict::downcast_from(py, map.to_py_object(py)).unwrap();

        assert!(py_map.len(py) == 1);
        assert!( py_map.get_item(py, 1).unwrap().extract::<i32>(py).unwrap() == 1);
    }
}
