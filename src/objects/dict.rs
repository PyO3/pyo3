// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::{mem, collections, hash, cmp};

use ffi;
use object::PyObject;
use instance::{Py, PyObjectWithToken};
use python::{Python, ToPyPointer, IntoPyPointer, IntoPyDictPointer};
use conversion::{ToPyObject, ToBorrowedObject, IntoPyObject};
use objects::{PyObjectRef, PyList};
use err::{self, PyResult, PyErr};

/// Represents a Python `dict`.
pub struct PyDict(PyObject);

pyobject_convert!(PyDict);
pyobject_nativetype!(PyDict, PyDict_Type, PyDict_Check);


impl PyDict {
    /// Creates a new empty dictionary.
    ///
    /// May panic when running out of memory.
    pub fn new(py: Python) -> &PyDict {
        unsafe {
            py.from_owned_ptr::<PyDict>(ffi::PyDict_New())
        }
    }

    /// Return a new dictionary that contains the same key-value pairs as self.
    /// Corresponds to `dict(self)` in Python.
    pub fn copy(&self) -> PyResult<&PyDict> {
        unsafe {
            self.py().from_owned_ptr_or_err::<PyDict>(ffi::PyDict_Copy(self.as_ptr()))
        }
    }

    /// Empty an existing dictionary of all key-value pairs.
    pub fn clear(&self) {
        unsafe { ffi::PyDict_Clear(self.as_ptr()) }
    }

    /// Return the number of items in the dictionary.
    /// This is equivalent to len(p) on a dictionary.
    pub fn len(&self) -> usize {
        unsafe { ffi::PyDict_Size(self.as_ptr()) as usize }
    }

    /// Check if dict is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Determine if the dictionary contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool> where K: ToBorrowedObject {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            match ffi::PyDict_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.py()))
            }
        })
    }

    /// Gets an item from the dictionary.
    /// Returns None if the item is not present, or if an error occurs.
    pub fn get_item<K>(&self, key: K) -> Option<&PyObjectRef> where K: ToBorrowedObject {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            self.py().from_borrowed_ptr_or_opt(
                ffi::PyDict_GetItem(self.as_ptr(), key))
        })
    }

    /// Sets an item value.
    /// This is equivalent to the Python expression `self[key] = value`.
    pub fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
        where K: ToPyObject, V: ToPyObject
    {
        key.with_borrowed_ptr(
            self.py(), move |key|
            value.with_borrowed_ptr(self.py(), |value| unsafe {
                err::error_on_minusone(
                    self.py(), ffi::PyDict_SetItem(self.as_ptr(), key, value))
            }))
    }

    /// Deletes an item.
    /// This is equivalent to the Python expression `del self[key]`.
    pub fn del_item<K>(&self, key: K) -> PyResult<()> where K: ToBorrowedObject
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            err::error_on_minusone(
                self.py(), ffi::PyDict_DelItem(self.as_ptr(), key))
        })
    }

    /// List of dict keys.
    /// This is equivalent to the python expression `list(dict.keys())`.
    pub fn keys(&self) -> &PyList {
        unsafe {
            self.py().from_owned_ptr::<PyList>(ffi::PyDict_Keys(self.as_ptr()))
        }
    }

    /// List of dict values.
    /// This is equivalent to the python expression `list(dict.values())`.
    pub fn values(&self) -> &PyList {
        unsafe {
            self.py().from_owned_ptr::<PyList>(ffi::PyDict_Values(self.as_ptr()))
        }
    }

    /// List of dict items.
    /// This is equivalent to the python expression `list(dict.items())`.
    pub fn items(&self) -> &PyList {
        unsafe {
            self.py().from_owned_ptr::<PyList>(ffi::PyDict_Items(self.as_ptr()))
        }
    }

    /// Returns a iterator of (key, value) pairs in this dictionary
    /// Note that it's unsafe to use when the dictionary might be changed
    /// by other python code.
    pub fn iter(&self) -> PyDictIterator {
        PyDictIterator { dict: self, pos: 0 }
    }
}

pub struct PyDictIterator<'a> {
    dict: &'a PyDict,
    pos: isize
}

impl<'a> Iterator for PyDictIterator<'a> {
    type Item = (&'a PyObjectRef, &'a PyObjectRef);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut key: *mut ffi::PyObject = mem::uninitialized();
            let mut value: *mut ffi::PyObject = mem::uninitialized();
            if ffi::PyDict_Next(self.dict.as_ptr(), &mut self.pos, &mut key, &mut value) != 0 {
                let py = self.dict.py();
                Some((py.from_borrowed_ptr(key), py.from_borrowed_ptr(value)))
            } else {
                None
            }
        }
    }
}

impl<'a> std::iter::IntoIterator for &'a PyDict {
    type Item = (&'a PyObjectRef, &'a PyObjectRef);
    type IntoIter = PyDictIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoPyDictPointer for &'a PyDict {
    #[must_use]
    fn into_dict_ptr(self, _py: Python) -> *mut ffi::PyObject {
        self.into_ptr()
    }
}

impl IntoPyDictPointer for Py<PyDict> {
    #[must_use]
    fn into_dict_ptr(self, _py: Python) -> *mut ffi::PyObject {
        self.into_ptr()
    }
}

impl <K, V, H> ToPyObject for collections::HashMap<K, V, H>
    where K: hash::Hash+cmp::Eq+ToPyObject,
          V: ToPyObject,
          H: hash::BuildHasher
{
    fn to_object(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        for (key, value) in self {
            dict.set_item(key, value).expect("Failed to set_item on dict");
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
            dict.set_item(key, value).expect("Failed to set_item on dict");
        };
        dict.into()
    }
}

impl <K, V, H> IntoPyObject for collections::HashMap<K, V, H>
    where K: hash::Hash+cmp::Eq+ToPyObject,
          V: ToPyObject,
          H: hash::BuildHasher
{
    fn into_object(self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        for (key, value) in self {
            dict.set_item(key, value).expect("Failed to set_item on dict");
        };
        dict.into()
    }
}

impl <K, V> IntoPyObject for collections::BTreeMap<K, V>
    where K: cmp::Eq+ToPyObject,
          V: ToPyObject
{
    fn into_object(self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        for (key, value) in self {
            dict.set_item(key, value).expect("Failed to set_item on dict");
        };
        dict.into()
    }
}

impl <K, V, H> IntoPyDictPointer for collections::HashMap<K, V, H>
    where K: hash::Hash+cmp::Eq+ToPyObject,
          V: ToPyObject,
          H: hash::BuildHasher
{
    #[must_use]
    fn into_dict_ptr(self, py: Python) -> *mut ffi::PyObject {
        let dict = PyDict::new(py);
        for (key, value) in self {
            dict.set_item(key, value).expect("Failed to set_item on dict");
        };
        dict.into_ptr()
    }
}

impl <K, V> IntoPyDictPointer for collections::BTreeMap<K, V>
    where K: cmp::Eq+ToPyObject,
          V: ToPyObject
{
    #[must_use]
    fn into_dict_ptr(self, py: Python) -> *mut ffi::PyObject {
        let dict = PyDict::new(py);
        for (key, value) in self {
            dict.set_item(key, value).expect("Failed to set_item on dict");
        };
        dict.into_ptr()
    }
}

impl<K: ToPyObject, V: ToPyObject> IntoPyDictPointer for (K, V) {
    default fn into_dict_ptr(self, py: Python) -> *mut ffi::PyObject {
        let dict = PyDict::new(py);
        dict.set_item(self.0, self.1).expect("Failed to set_item on dict");
        dict.into_ptr()
    }
}

macro_rules! dict_conversion ({$length:expr,$(($refN:ident, $n:tt, $T1:ident, $T2:ident)),+} => {
    impl<$($T1: ToPyObject, $T2: ToPyObject),+> IntoPyDictPointer for ($(($T1,$T2),)+) {
        fn into_dict_ptr(self, py: Python) -> *mut ffi::PyObject {
            let dict = PyDict::new(py);
            $(dict.set_item(self.$n.0, self.$n.1).expect("Failed to set_item on dict");)+;
            dict.into_ptr()
        }
    }
});

dict_conversion!(1, (ref0, 0, A1, A2));
dict_conversion!(2, (ref0, 0, A1, A2), (ref1, 1, B1, B2));
dict_conversion!(3, (ref0, 0, A1, A2), (ref1, 1, B1, B2), (ref2, 2, C1, C2));
dict_conversion!(4, (ref0, 0, A1, A2), (ref1, 1, B1, B2), (ref2, 2, C1, C2),
                 (ref3, 3, D1, D2));
dict_conversion!(5, (ref0, 0, A1, A2), (ref1, 1, B1, B2), (ref2, 2, C1, C2),
                 (ref3, 3, D1, D2), (ref4, 4, E1, E2));
dict_conversion!(6, (ref0, 0, A1, A2), (ref1, 1, B1, B2), (ref2, 2, C1, C2),
                 (ref3, 3, D1, D2), (ref4, 4, E1, E2), (ref5, 5, F1, F2));
dict_conversion!(7, (ref0, 0, A1, A2), (ref1, 1, B1, B2), (ref2, 2, C1, C2),
                 (ref3, 3, D1, D2), (ref4, 4, E1, E2), (ref5, 5, F1, F2), (ref6, 6, G1, G2));
dict_conversion!(8, (ref0, 0, A1, A2), (ref1, 1, B1, B2), (ref2, 2, C1, C2),
                 (ref3, 3, D1, D2), (ref4, 4, E1, E2), (ref5, 5, F1, F2), (ref6, 6, G1, G2),
                 (ref7, 7, H1, H2));
dict_conversion!(9, (ref0, 0, A1, A2), (ref1, 1, B1, B2), (ref2, 2, C1, C2),
                 (ref3, 3, D1, D2), (ref4, 4, E1, E2), (ref5, 5, F1, F2), (ref6, 6, G1, G2),
                 (ref7, 7, H1, H2), (ref8, 8, I1, I2));


#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, HashMap};
    use python::{Python, IntoPyDictPointer};
    use instance::AsPyRef;
    use conversion::{PyTryFrom, ToPyObject, IntoPyObject};
    use objects::{PyDict, PyTuple};
    use {PyObject, ObjectProtocol};

    #[test]
    fn test_new() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let dict = PyDict::new(py);
        dict.set_item(7, 32).unwrap();
        assert_eq!(32, dict.get_item(7i32).unwrap().extract::<i32>().unwrap());
        assert_eq!(None, dict.get_item(8i32));
    }

    #[test]
    fn test_copy() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let dict = PyDict::new(py);
        dict.set_item(7, 32).unwrap();

        let ndict = dict.copy().unwrap();
        assert_eq!(32, ndict.get_item(7i32).unwrap().extract::<i32>().unwrap());
        assert_eq!(None, ndict.get_item(8i32));
    }

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        let ob = v.to_object(py);
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(0, dict.len());
        v.insert(7, 32);
        let ob = v.to_object(py);
        let dict2 = PyDict::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(1, dict2.len());
    }

    #[test]
    fn test_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let ob = v.to_object(py);
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(true, dict.contains(7i32).unwrap());
        assert_eq!(false, dict.contains(8i32).unwrap());
    }

    #[test]
    fn test_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let ob = v.to_object(py);
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(32, dict.get_item(7i32).unwrap().extract::<i32>().unwrap());
        assert_eq!(None, dict.get_item(8i32));
    }

    #[test]
    fn test_set_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let ob = v.to_object(py);
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        assert!(dict.set_item(7i32, 42i32).is_ok()); // change
        assert!(dict.set_item(8i32, 123i32).is_ok()); // insert
        assert_eq!(42i32, dict.get_item(7i32).unwrap().extract::<i32>().unwrap());
        assert_eq!(123i32, dict.get_item(8i32).unwrap().extract::<i32>().unwrap());
    }

    #[test]
    fn test_set_item_refcnt() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let cnt;
        {
            let _pool = ::GILPool::new();
            let dict = PyDict::new(py);
            let none = py.None();
            cnt = none.get_refcnt();
            dict.set_item(10, none).unwrap();
        }
        {
            assert_eq!(cnt, py.None().get_refcnt());
        }
    }

    #[test]
    fn test_set_item_does_not_update_original_object() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let ob = v.to_object(py);
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        assert!(dict.set_item(7i32, 42i32).is_ok()); // change
        assert!(dict.set_item(8i32, 123i32).is_ok()); // insert
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
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        assert!(dict.del_item(7i32).is_ok());
        assert_eq!(0, dict.len());
        assert_eq!(None, dict.get_item(7i32));
    }

    #[test]
    fn test_del_item_does_not_update_original_object() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        let ob = v.to_object(py);
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        assert!(dict.del_item(7i32).is_ok()); // change
        assert_eq!(32i32, *v.get(&7i32).unwrap()); // not updated!
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
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
        let mut key_sum = 0;
        let mut value_sum = 0;
        for el in dict.items().iter() {
            let tuple = el.cast_as::<PyTuple>().unwrap();
            key_sum += tuple.get_item(0).extract::<i32>().unwrap();
            value_sum += tuple.get_item(1).extract::<i32>().unwrap();
        }
        assert_eq!(7 + 8 + 9, key_sum);
        assert_eq!(32 + 42 + 123, value_sum);
    }

    #[test]
    fn test_keys() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        v.insert(8, 42);
        v.insert(9, 123);
        let ob = v.to_object(py);
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
        let mut key_sum = 0;
        for el in dict.keys().iter() {
            key_sum += el.extract::<i32>().unwrap();
        }
        assert_eq!(7 + 8 + 9, key_sum);
    }

    #[test]
    fn test_values() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        v.insert(8, 42);
        v.insert(9, 123);
        let ob = v.to_object(py);
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
        let mut values_sum = 0;
        for el in dict.values().iter() {
            values_sum += el.extract::<i32>().unwrap();
        }
        assert_eq!(32 + 42 + 123, values_sum);
    }

    #[test]
    fn test_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        v.insert(8, 42);
        v.insert(9, 123);
        let ob = v.to_object(py);
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        let mut key_sum = 0;
        let mut value_sum = 0;
        for (key, value) in dict.iter() {
            key_sum += key.extract::<i32>().unwrap();
            value_sum += value.extract::<i32>().unwrap();
        }
        assert_eq!(7 + 8 + 9, key_sum);
        assert_eq!(32 + 42 + 123, value_sum);
    }

    #[test]
    fn test_into_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v = HashMap::new();
        v.insert(7, 32);
        v.insert(8, 42);
        v.insert(9, 123);
        let ob = v.to_object(py);
        let dict = PyDict::try_from(ob.as_ref(py)).unwrap();
        let mut key_sum = 0;
        let mut value_sum = 0;
        for (key, value) in dict {
            key_sum += key.extract::<i32>().unwrap();
            value_sum += value.extract::<i32>().unwrap();
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
        let py_map = PyDict::try_from(m.as_ref(py)).unwrap();

        assert!(py_map.len() == 1);
        assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
    }

    #[test]
    fn test_btreemap_to_python() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut map = BTreeMap::<i32, i32>::new();
        map.insert(1, 1);

        let m = map.to_object(py);
        let py_map = PyDict::try_from(m.as_ref(py)).unwrap();

        assert!(py_map.len() == 1);
        assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
    }

    #[test]
    fn test_hashmap_into_python() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut map = HashMap::<i32, i32>::new();
        map.insert(1, 1);

        let m = map.into_object(py);
        let py_map = PyDict::try_from(m.as_ref(py)).unwrap();

        assert!(py_map.len() == 1);
        assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
    }

    #[test]
    fn test_hashmap_into_dict() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut map = HashMap::<i32, i32>::new();
        map.insert(1, 1);

        let m = map.into_dict_ptr(py);
        let ob = unsafe{PyObject::from_owned_ptr(py, m)};
        let py_map = PyDict::try_from(ob.as_ref(py)).unwrap();

        assert!(py_map.len() == 1);
        assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
    }

    #[test]
    fn test_btreemap_into_object() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut map = BTreeMap::<i32, i32>::new();
        map.insert(1, 1);

        let m = map.into_object(py);
        let py_map = PyDict::try_from(m.as_ref(py)).unwrap();

        assert!(py_map.len() == 1);
        assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
    }

    #[test]
    fn test_btreemap_into_dict() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut map = BTreeMap::<i32, i32>::new();
        map.insert(1, 1);

        let m = map.into_dict_ptr(py);
        let ob = unsafe{PyObject::from_owned_ptr(py, m)};
        let py_map = PyDict::try_from(ob.as_ref(py)).unwrap();

        assert!(py_map.len() == 1);
        assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
    }

    #[test]
    fn test_tuple_into_dict() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let m = ((1, 1),).into_dict_ptr(py);
        let ob = unsafe{PyObject::from_owned_ptr(py, m)};
        let py_map = PyDict::try_from(ob.as_ref(py)).unwrap();

        assert!(py_map.len() == 1);
        assert!( py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);

        let m = ((1, 1), (2, 3)).into_dict_ptr(py);
        let ob = unsafe{PyObject::from_owned_ptr(py, m)};
        let py_map = PyDict::try_from(ob.as_ref(py)).unwrap();

        assert!(py_map.len() == 2);
        assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
        assert!(py_map.get_item(2).unwrap().extract::<i32>().unwrap() == 3);
    }

    #[test]
    fn test_simple_tuple_into_dict() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let m = (1, 1).into_dict_ptr(py);
        let ob = unsafe{PyObject::from_owned_ptr(py, m)};
        let py_map = PyDict::try_from(ob.as_ref(py)).unwrap();

        assert!(py_map.len() == 1);
        assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
    }
}
