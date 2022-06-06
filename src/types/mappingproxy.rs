// Copyright (c) 2017-present PyO3 Project and Contributors

use super::PyMapping;
use crate::err::PyResult;
use crate::types::dict::PyDictItem;
#[cfg(all(test, not(PyPy)))]
use crate::types::dict::{PyDictItems, PyDictKeys, PyDictValues};
use crate::types::{IntoPyDict, PyAny, PyDict, PyIterator, PyList, PySequence};
#[cfg(not(PyPy))]
use crate::PyObject;
use crate::{ffi, AsPyPointer, PyErr, PyTryFrom, Python, ToPyObject};
use std::os::raw::c_int;

#[inline]
#[allow(non_snake_case)]
unsafe fn PyDictProxy_Check(object: *mut crate::ffi::PyObject) -> c_int {
    ffi::PyObject_TypeCheck(object, ffi::addr_of_mut_shim!(ffi::PyDictProxy_Type))
}

/// Represents a Python `mappingproxy`.
#[repr(transparent)]
pub struct PyMappingProxy(PyAny);

pyobject_native_type!(
    PyMappingProxy,
    ffi::PyDictProxyObject,
    ffi::PyDictProxy_Type,
    #checkfunction=PyDictProxy_Check
);

impl PyMappingProxy {
    /// Creates a mappingproxy from an object.
    pub fn new(py: Python<'_>, elements: impl IntoPyDict) -> PyResult<&'_ PyMappingProxy> {
        unsafe {
            let dict = elements.into_py_dict(py);
            let proxy = ffi::PyDictProxy_New(dict.as_ptr());
            py.from_owned_ptr_or_err::<PyMappingProxy>(proxy)
        }
    }

    /// Creates a new mappingproxy from the sequence given.
    ///
    /// The sequence must consist of `(PyObject, PyObject)`.
    ///
    /// Returns an error on invalid input. In the case of key collisions,
    /// this keeps the last entry seen.
    #[cfg(not(PyPy))]
    pub fn from_sequence(py: Python<'_>, seq: PyObject) -> PyResult<&PyMappingProxy> {
        unsafe {
            let dict = py.from_owned_ptr::<PyDict>(PyDict::from_sequence(py, seq)?.as_ptr());
            py.from_owned_ptr_or_err(ffi::PyDictProxy_New(dict.as_ptr()))
        }
    }

    /// Returns a new mappingproxy that contains the same key-value pairs as self.
    ///
    /// This is equivalent to the Python expression `self.copy()`.
    pub fn copy(&self) -> PyResult<&PyDict> {
        self.call_method0("copy")
            .and_then(|object| object.downcast().map_err(PyErr::from))
    }

    /// Checks if the mappingproxy is empty, i.e. `len(self) == 0`.
    pub fn is_empty(&self) -> bool {
        self.len().unwrap() == 0
    }

    /// Gets an item from the mappingproxy.
    ///    
    /// Returns `None` if the item is not present, or if an error occurs.
    ///
    /// To get a `KeyError` for non-existing keys, use `PyAny::get_item`.
    pub fn get_item<K>(&self, key: K) -> Option<&PyAny>
    where
        K: ToPyObject,
    {
        PyAny::get_item(self, key).ok()
    }

    /// Returns a list of mappingproxy keys.
    ///
    /// This is equivalent to the Python expression `list(mappingproxy.keys())`.
    pub fn keys(&self) -> &PyList {
        unsafe {
            PySequence::try_from_unchecked(self.call_method0("keys").unwrap())
                .list()
                .unwrap()
        }
    }

    /// Returns a list of mappingproxy values.
    ///
    /// This is equivalent to the Python expression `list(mappingproxy.values())`.
    pub fn values(&self) -> &PyList {
        unsafe {
            PySequence::try_from_unchecked(self.call_method0("values").unwrap())
                .list()
                .unwrap()
        }
    }

    /// Returns a list of mappingproxy items.
    ///
    /// This is equivalent to the Python expression `list(mappingproxy.items())`.
    pub fn items(&self) -> &PyList {
        unsafe {
            PySequence::try_from_unchecked(self.call_method0("items").unwrap())
                .list()
                .unwrap()
        }
    }

    ///
    /// Returns an iterator of `(key, value)` pairs in this mappingproxy.
    pub fn iter(&self) -> PyMappingProxyIterator<'_> {
        IntoIterator::into_iter(self)
    }

    /// Returns `self` cast as a `PyMapping`.
    pub fn as_mapping(&self) -> &PyMapping {
        unsafe { PyMapping::try_from_unchecked(self) }
    }
}

pub struct PyMappingProxyIterator<'py> {
    iterator: &'py PyIterator,
    mappingproxy: &'py PyMappingProxy,
}

impl<'py> Iterator for PyMappingProxyIterator<'py> {
    type Item = (&'py PyAny, &'py PyAny);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator
            .next()
            .map(Result::unwrap)
            .and_then(|key| self.mappingproxy.get_item(key).map(|value| (key, value)))
    }
}

impl<'a> std::iter::IntoIterator for &'a PyMappingProxy {
    type Item = (&'a PyAny, &'a PyAny);
    type IntoIter = PyMappingProxyIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PyMappingProxyIterator {
            iterator: PyIterator::from_object(self.py(), self).unwrap(),
            mappingproxy: self,
        }
    }
}

/// Conversion trait that allows a sequence of tuples to be converted into `PyMappingProxy`
/// Primary use case for this trait is `call` and `call_method` methods as keywords argument.
pub trait IntoPyMappingProxy {
    /// Converts self into a `PyMappingProxy` object pointer. Whether pointer owned or borrowed
    /// depends on implementation.
    fn into_py_mappingproxy(self, py: Python<'_>) -> PyResult<&PyMappingProxy>;
}

impl<T, I> IntoPyMappingProxy for I
where
    T: PyDictItem,
    I: IntoIterator<Item = T>,
{
    fn into_py_mappingproxy(self, py: Python<'_>) -> PyResult<&PyMappingProxy> {
        PyMappingProxy::new(py, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(PyPy))]
    use crate::{types::PyList, PyTypeInfo};
    use crate::{types::PyTuple, Python, ToPyObject};
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_new() {
        Python::with_gil(|py| {
            let mappingproxy = [(7, 32)].into_py_mappingproxy(py).unwrap();
            assert_eq!(
                32,
                mappingproxy
                    .get_item(7i32)
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(mappingproxy.get_item(8i32).is_none());
            let map: HashMap<i32, i32> = [(7, 32)].iter().cloned().collect();
            assert_eq!(map, mappingproxy.extract().unwrap());
            let map: BTreeMap<i32, i32> = [(7, 32)].iter().cloned().collect();
            assert_eq!(map, mappingproxy.extract().unwrap());
        });
    }

    #[test]
    #[cfg(not(PyPy))]
    fn test_from_sequence() {
        Python::with_gil(|py| {
            let items = PyList::new(py, &vec![("a", 1), ("b", 2)]);
            let mappingproxy = PyMappingProxy::from_sequence(py, items.to_object(py)).unwrap();
            assert_eq!(
                1,
                mappingproxy
                    .get_item("a")
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert_eq!(
                2,
                mappingproxy
                    .get_item("b")
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            let map: HashMap<&str, i32> = [("a", 1), ("b", 2)].iter().cloned().collect();
            assert_eq!(map, mappingproxy.extract().unwrap());
            let map: BTreeMap<&str, i32> = [("a", 1), ("b", 2)].iter().cloned().collect();
            assert_eq!(map, mappingproxy.extract().unwrap());
        });
    }

    #[test]
    #[cfg(not(PyPy))]
    fn test_from_sequence_err() {
        Python::with_gil(|py| {
            let items = PyList::new(py, &vec!["a", "b"]);
            assert!(PyMappingProxy::from_sequence(py, items.to_object(py)).is_err());
        });
    }

    #[test]
    fn test_copy() {
        Python::with_gil(|py| {
            let mappingproxy = [(7, 32)].into_py_mappingproxy(py).unwrap();

            let new_dict = mappingproxy.copy().unwrap();
            assert_eq!(
                32,
                new_dict.get_item(7i32).unwrap().extract::<i32>().unwrap()
            );
            assert!(new_dict.get_item(8i32).is_none());
        });
    }

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            let mappingproxy = v.clone().into_py_mappingproxy(py).unwrap();
            assert_eq!(mappingproxy.len().unwrap(), 0);
            v.insert(7, 32);
            let mp2 = v.into_py_mappingproxy(py).unwrap();
            assert_eq!(mp2.len().unwrap(), 1);
        });
    }

    #[test]
    fn test_contains() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let mappingproxy = v.into_py_mappingproxy(py).unwrap();
            assert!(mappingproxy.contains(7i32).unwrap());
            assert!(!mappingproxy.contains(8i32).unwrap());
        });
    }

    #[test]
    fn test_get_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let mappingproxy = v.into_py_mappingproxy(py).unwrap();
            assert_eq!(
                32,
                mappingproxy
                    .get_item(7i32)
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(mappingproxy.get_item(8i32).is_none());
        });
    }

    #[test]
    fn test_set_item_refcnt() {
        Python::with_gil(|py| {
            let cnt;
            {
                let _pool = unsafe { crate::GILPool::new() };
                let none = py.None();
                cnt = none.get_refcnt(py);
                let _mapping_proxy = [(10, none)].into_py_mappingproxy(py);
            }
            {
                assert_eq!(cnt, py.None().get_refcnt(py));
            }
        });
    }

    #[test]
    fn test_items() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let mappingproxy = v.into_py_mappingproxy(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            let mut value_sum = 0;
            for el in mappingproxy.items().iter() {
                let tuple = el.cast_as::<PyTuple>().unwrap();
                key_sum += tuple.get_item(0).unwrap().extract::<i32>().unwrap();
                value_sum += tuple.get_item(1).unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_keys() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let mappingproxy = v.into_py_mappingproxy(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            for el in mappingproxy.keys().iter() {
                key_sum += el.extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
        });
    }

    #[test]
    fn test_values() {
        Python::with_gil(|py| {
            let mut v: HashMap<i32, i32> = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let mappingproxy = v.into_py_mappingproxy(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut values_sum = 0;
            for el in mappingproxy.values().iter() {
                values_sum += el.extract::<i32>().unwrap();
            }
            assert_eq!(32 + 42 + 123, values_sum);
        });
    }

    #[test]
    fn test_iter() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let mappingproxy = v.into_py_mappingproxy(py).unwrap();
            let mut key_sum = 0;
            let mut value_sum = 0;
            for (key, value) in mappingproxy.iter() {
                key_sum += key.extract::<i32>().unwrap();
                value_sum += value.extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_into_iter() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let mappingproxy = v.into_py_mappingproxy(py).unwrap();
            let mut key_sum = 0;
            let mut value_sum = 0;
            for (key, value) in mappingproxy {
                key_sum += key.extract::<i32>().unwrap();
                value_sum += value.extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_hashmap_to_python() {
        Python::with_gil(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.clone().into_py_mappingproxy(py).unwrap();

            assert_eq!(py_map.len().unwrap(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
            assert_eq!(map, py_map.extract().unwrap());
        });
    }

    #[test]
    fn test_btreemap_to_python() {
        Python::with_gil(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.clone().into_py_mappingproxy(py).unwrap();

            assert_eq!(py_map.len().unwrap(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
            assert_eq!(map, py_map.extract().unwrap());
        });
    }

    #[test]
    fn test_hashmap_into_python() {
        Python::with_gil(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_mappingproxy(py).unwrap();

            assert_eq!(py_map.len().unwrap(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_hashmap_into_mappingproxy() {
        Python::with_gil(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_mappingproxy(py).unwrap();

            assert_eq!(py_map.len().unwrap(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_btreemap_into_py() {
        Python::with_gil(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.clone().into_py_mappingproxy(py).unwrap();

            assert_eq!(py_map.len().unwrap(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_btreemap_into_mappingproxy() {
        Python::with_gil(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_mappingproxy(py).unwrap();

            assert_eq!(py_map.len().unwrap(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_vec_into_mappingproxy() {
        Python::with_gil(|py| {
            let vec = vec![("a", 1), ("b", 2), ("c", 3)];
            let py_map = vec.into_py_mappingproxy(py).unwrap();

            assert_eq!(py_map.len().unwrap(), 3);
            assert_eq!(py_map.get_item("b").unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn test_slice_into_mappingproxy() {
        Python::with_gil(|py| {
            let arr = [("a", 1), ("b", 2), ("c", 3)];
            let py_map = arr.into_py_mappingproxy(py).unwrap();

            assert_eq!(py_map.len().unwrap(), 3);
            assert_eq!(py_map.get_item("b").unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn mappingproxy_as_mapping() {
        Python::with_gil(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_mappingproxy(py).unwrap();

            assert_eq!(py_map.as_mapping().len().unwrap(), 1);
            assert_eq!(
                py_map
                    .as_mapping()
                    .get_item(1)
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
        });
    }

    #[cfg(not(PyPy))]
    fn abc_mappingproxy(py: Python<'_>) -> &PyMappingProxy {
        let mut map = HashMap::<&'static str, i32>::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);
        map.into_py_mappingproxy(py).unwrap()
    }

    #[test]
    #[cfg(not(PyPy))]
    fn mappingproxy_keys_view() {
        Python::with_gil(|py| {
            let mappingproxy = abc_mappingproxy(py);
            let keys = mappingproxy.call_method0("keys").unwrap();
            assert!(keys.is_instance(PyDictKeys::type_object(py)).unwrap());
        })
    }

    #[test]
    #[cfg(not(PyPy))]
    fn mappingproxy_values_view() {
        Python::with_gil(|py| {
            let mappingproxy = abc_mappingproxy(py);
            let values = mappingproxy.call_method0("values").unwrap();
            assert!(values.is_instance(PyDictValues::type_object(py)).unwrap());
        })
    }

    #[test]
    #[cfg(not(PyPy))]
    fn mappingproxy_items_view() {
        Python::with_gil(|py| {
            let mappingproxy = abc_mappingproxy(py);
            let items = mappingproxy.call_method0("items").unwrap();
            assert!(items.is_instance(PyDictItems::type_object(py)).unwrap());
        })
    }
}
