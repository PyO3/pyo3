// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::{mem, collections, hash, cmp};

use ffi;
use pointers::PyPtr;
use python::{Python, ToPyPointer};
use conversion::{ToPyObject};
use objects::{PyObject, PyList};
use err::{self, PyResult, PyErr};

/// Represents a Python `dict`.
pub struct PyDict(PyPtr);

pyobject_convert!(PyDict);
pyobject_nativetype!(PyDict, PyDict_Type, PyDict_Check);


impl PyDict {
    /// Creates a new empty dictionary.
    ///
    /// May panic when running out of memory.
    pub fn new(_py: Python) -> PyDict {
        unsafe { PyDict(PyPtr::from_owned_ptr_or_panic(ffi::PyDict_New())) }
    }

    /// Construct a new dict with the given raw pointer
    /// Undefined behavior if the pointer is NULL or invalid.
    pub unsafe fn from_borrowed_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyDict {
        PyDict(PyPtr::from_borrowed_ptr(ptr))
    }

    /// Return a new dictionary that contains the same key-value pairs as self.
    /// Corresponds to `dict(self)` in Python.
    pub fn copy(&self, py: Python) -> PyResult<PyDict> {
        unsafe {
            Ok(PyDict(
                PyPtr::from_owned_ptr_or_err(py, ffi::PyDict_Copy(self.0.as_ptr()))?
            ))
        }
    }

    /// Empty an existing dictionary of all key-value pairs.
    #[inline]
    pub fn clear(&self, _py: Python) {
        unsafe { ffi::PyDict_Clear(self.as_ptr()) }
    }

    /// Return the number of items in the dictionary.
    /// This is equivalent to len(p) on a dictionary.
    #[inline]
    pub fn len(&self, _py: Python) -> usize {
        unsafe { ffi::PyDict_Size(self.as_ptr()) as usize }
    }

    /// Determine if the dictionary contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, py: Python, key: K) -> PyResult<bool> where K: ToPyObject {
        key.with_borrowed_ptr(py, |key| unsafe {
            match ffi::PyDict_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(py))
            }
        })
    }

    /// Gets an item from the dictionary.
    /// Returns None if the item is not present, or if an error occurs.
    pub fn get_item<K>(&self, py: Python, key: K) -> Option<PyObject> where K: ToPyObject {
        key.with_borrowed_ptr(py, |key| unsafe {
            PyObject::from_borrowed_ptr_or_opt(
                py, ffi::PyDict_GetItem(self.as_ptr(), key))
        })
    }

    /// Sets an item value.
    /// This is equivalent to the Python expression `self[key] = value`.
    pub fn set_item<K, V>(&self, py: Python, key: K, value: V)
                          -> PyResult<()> where K: ToPyObject, V: ToPyObject {
        key.with_borrowed_ptr(
            py, move |key|
            value.with_borrowed_ptr(py, |value| unsafe {
                err::error_on_minusone(
                    py, ffi::PyDict_SetItem(self.as_ptr(), key, value))
            }))
    }

    /// Deletes an item.
    /// This is equivalent to the Python expression `del self[key]`.
    pub fn del_item<K>(&self, py: Python, key: K) -> PyResult<()> where K: ToPyObject {
        key.with_borrowed_ptr(py, |key| unsafe {
            err::error_on_minusone(
                py, ffi::PyDict_DelItem(self.as_ptr(), key))
        })
    }

    /// List of dict items.
    /// This is equivalent to the python expression `list(dict.items())`.
    pub fn items_list(&self, py: Python) -> PyList {
        unsafe {
            PyObject::from_owned_ptr(
                py, ffi::PyDict_Items(self.as_ptr())).unchecked_cast_into::<PyList>()
        }
    }

    /// Returns the list of (key, value) pairs in this dictionary.
    pub fn items(&self, py: Python) -> Vec<(PyObject, PyObject)> {
        // Note that we don't provide an iterator because
        // PyDict_Next() is unsafe to use when the dictionary might be changed
        // by other python code.
        let mut vec = Vec::with_capacity(self.len(py));
        unsafe {
            let mut pos = 0;
            let mut key: *mut ffi::PyObject = mem::uninitialized();
            let mut value: *mut ffi::PyObject = mem::uninitialized();
            while ffi::PyDict_Next(self.as_ptr(), &mut pos, &mut key, &mut value) != 0 {
                vec.push((PyObject::from_borrowed_ptr(py, key),
                          PyObject::from_borrowed_ptr(py, value)));
            }
        }
        vec
    }
}

impl <K, V> ToPyObject for collections::HashMap<K, V>
    where K: hash::Hash+cmp::Eq+ToPyObject,
          V: ToPyObject
{
    fn to_object(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        for (key, value) in self {
            dict.set_item(py, key, value).expect("Failed to set_item on dict");
        };
        dict.into()
    }
}

impl <K, V> ToPyObject for collections::BTreeMap<K, V>
    where K: cmp::Eq+ToPyObject,
          V: ToPyObject
{
    fn to_object(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        for (key, value) in self {
            dict.set_item(py, key, value).expect("Failed to set_item on dict");
        };
        dict.into()
    }
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};
    use python::Python;
    use conversion::ToPyObject;
    use objects::{PyDict, PyTuple};
    use ::PyDowncastFrom;


    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        let ob = v.to_object(py);
        let dict = PyDict::downcast_from(py, &ob).unwrap();
        assert_eq!(0, dict.len(py));
        v.insert(7, 32);
        let ob = v.to_object(py);
        let dict2 = PyDict::downcast_from(py, &ob).unwrap();
        assert_eq!(1, dict2.len(py));
    }

    #[test]
    fn test_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let ob = v.to_object(py);
        let dict = PyDict::downcast_from(py, &ob).unwrap();
        assert_eq!(true, dict.contains(py, 7i32).unwrap());
        assert_eq!(false, dict.contains(py, 8i32).unwrap());
    }

    #[test]
    fn test_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let ob = v.to_object(py);
        let dict = PyDict::downcast_from(py, &ob).unwrap();
        assert_eq!(32, dict.get_item(py, 7i32).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(None, dict.get_item(py, 8i32));
    }

    #[test]
    fn test_set_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let ob = v.to_object(py);
        let dict = PyDict::downcast_from(py, &ob).unwrap();
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
        let ob = v.to_object(py);
        let dict = PyDict::downcast_from(py, &ob).unwrap();
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
        let ob = v.to_object(py);
        let dict = PyDict::downcast_from(py, &ob).unwrap();
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
        let ob = v.to_object(py);
        let dict = PyDict::downcast_from(py, &ob).unwrap();
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
        let ob = v.to_object(py);
        let dict = PyDict::downcast_from(py, &ob).unwrap();
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
        let ob = v.to_object(py);
        let dict = PyDict::downcast_from(py, &ob).unwrap();
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

        let m = map.to_object(py);
        let py_map = PyDict::downcast_from(py, &m).unwrap();

        assert!(py_map.len(py) == 1);
        assert!( py_map.get_item(py, 1).unwrap().extract::<i32>(py).unwrap() == 1);
    }

    #[test]
    fn test_btreemap_to_python() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut map = BTreeMap::<i32, i32>::new();
        map.insert(1, 1);

        let m = map.to_object(py);
        let py_map = PyDict::downcast_from(py, &m).unwrap();

        assert!(py_map.len(py) == 1);
        assert!( py_map.get_item(py, 1).unwrap().extract::<i32>(py).unwrap() == 1);
    }
}
