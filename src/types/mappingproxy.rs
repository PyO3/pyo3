// Copyright (c) 2017-present PyO3 Project and Contributors

use super::PyMapping;
use crate::err::PyResult;
use crate::types::dict::{IntoPyDict, PyDictItem};
#[cfg(all(test, not(PyPy)))]
use crate::types::dict::{PyDictItems, PyDictKeys, PyDictValues};
use crate::types::{PyAny, PyIterator, PyList, PySequence};
#[cfg(not(PyPy))]
use crate::{ffi, AsPyPointer, PyErr, PyTryFrom, Python, ToPyObject};
use std::os::raw::c_int;

#[allow(non_snake_case)]
unsafe fn PyDictProxy_Check(object: *mut crate::ffi::PyObject) -> c_int {
    ffi::PyObject_TypeCheck(object, std::ptr::addr_of_mut!(ffi::PyDictProxy_Type))
}

/// Represents a Python `mappingproxy`.
#[repr(transparent)]
pub struct PyMappingProxy(PyAny);

pyobject_native_type_core!(
    PyMappingProxy,
    pyobject_native_static_type_object!(ffi::PyDictProxy_Type),
    #checkfunction=PyDictProxy_Check
);

impl PyMappingProxy {
    /// Creates a mappingproxy from an object.
    pub fn new<'py>(py: Python<'py>, elements: &'py PyMapping) -> PyResult<&'py PyMappingProxy> {
        unsafe {
            let proxy = ffi::PyDictProxy_New(elements.as_ptr());
            py.from_owned_ptr_or_err::<PyMappingProxy>(proxy)
        }
    }

    /// Returns a new mappingproxy that contains the same key-value pairs as self.
    ///
    /// This is equivalent to the Python expression `self.copy()`.
    pub fn copy(&self) -> PyResult<&PyMapping> {
        self.call_method0("copy")
            .and_then(|object| object.downcast::<PyMapping>().map_err(PyErr::from))
    }

    /// Checks if the mappingproxy is empty, i.e. `len(self) == 0`.
    pub fn is_empty(&self) -> bool {
        self.len().unwrap_or_default() == 0
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
                .to_list()
                .unwrap()
        }
    }

    /// Returns a list of mappingproxy values.
    ///
    /// This is equivalent to the Python expression `list(mappingproxy.values())`.
    pub fn values(&self) -> &PyList {
        unsafe {
            PySequence::try_from_unchecked(self.call_method0("values").unwrap())
                .to_list()
                .unwrap()
        }
    }

    /// Returns a list of mappingproxy items.
    ///
    /// This is equivalent to the Python expression `list(mappingproxy.items())`.
    pub fn items(&self) -> &PyList {
        unsafe {
            PySequence::try_from_unchecked(self.call_method0("items").unwrap())
                .to_list()
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
            iterator: PyIterator::from_object(self).unwrap(),
            mappingproxy: self,
        }
    }
}

/// Conversion trait that allows a sequence of tuples to be converted into `PyMappingProxy`
/// Primary use case for this trait is `call` and `call_method` methods as keywords argument.
pub trait IntoPyMappingProxy<'py> {
    /// Converts self into a `PyMappingProxy` object pointer. Whether pointer owned or borrowed
    /// depends on implementation.
    fn into_py_mappingproxy(self, py: Python<'py>) -> PyResult<&'py PyMappingProxy>;
}

impl<'py, T, I> IntoPyMappingProxy<'py> for I
where
    T: PyDictItem,
    I: IntoIterator<Item = T>,
{
    fn into_py_mappingproxy(self, py: Python<'py>) -> PyResult<&'py PyMappingProxy> {
        let dict = self.into_py_dict(py);
        PyMappingProxy::new(py, dict.as_mapping())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        exceptions::PyKeyError,
        types::{PyInt, PyString, PyTuple},
    };
    #[cfg(not(PyPy))]
    use crate::{PyTypeInfo, Python};
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
    fn test_copy() {
        Python::with_gil(|py| {
            let mappingproxy = [(7, 32)].into_py_mappingproxy(py).unwrap();

            let new_dict = mappingproxy.copy().unwrap();
            assert_eq!(
                32,
                new_dict.get_item(7i32).unwrap().extract::<i32>().unwrap()
            );
            assert!(new_dict
                .get_item(8i32)
                .unwrap_err()
                .is_instance_of::<PyKeyError>(py));
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
                let tuple = el.downcast::<PyTuple>().unwrap();
                key_sum += tuple.get_item(0).unwrap().extract::<i32>().unwrap();
                value_sum += tuple.get_item(1).unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_isempty() {
        Python::with_gil(|py| {
            let map: HashMap<usize, usize> = HashMap::new();
            let mappingproxy = map.into_py_mappingproxy(py).unwrap();
            assert!(mappingproxy.is_empty());
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

    #[test]
    fn get_value_from_mappingproxy_of_strings() {
        Python::with_gil(|py: Python<'_>| {
            let mut map = HashMap::new();
            map.insert("first key", "first value");
            map.insert("second key", "second value");
            map.insert("third key", "third value");

            let mappingproxy = map.iter().into_py_mappingproxy(py).unwrap();

            assert_eq!(
                map.into_iter().collect::<Vec<(&str, &str)>>(),
                mappingproxy
                    .iter()
                    .map(|object| (
                        object.0.downcast::<PyString>().unwrap().to_str().unwrap(),
                        object.1.downcast::<PyString>().unwrap().to_str().unwrap()
                    ))
                    .collect::<Vec<(&str, &str)>>()
            );
        })
    }

    #[test]
    fn get_value_from_mappingproxy_of_integers() {
        Python::with_gil(|py: Python<'_>| {
            const LEN: usize = 10_000;
            let items: Vec<(usize, usize)> = (1..LEN).map(|i| (i, i - 1)).collect();
            let mappingproxy = items.to_vec().into_py_mappingproxy(py).unwrap();
            assert_eq!(
                items,
                mappingproxy
                    .iter()
                    .map(|object| (
                        object
                            .0
                            .downcast::<PyInt>()
                            .unwrap()
                            .extract::<usize>()
                            .unwrap(),
                        object
                            .1
                            .downcast::<PyInt>()
                            .unwrap()
                            .extract::<usize>()
                            .unwrap()
                    ))
                    .collect::<Vec<(usize, usize)>>()
            );
            for index in 1..LEN {
                assert_eq!(
                    mappingproxy
                        .get_item(index)
                        .unwrap()
                        .extract::<usize>()
                        .unwrap(),
                    index - 1
                );
            }
        })
    }

    #[test]
    fn iter_mappingproxy_nosegv() {
        Python::with_gil(|py| {
            const LEN: usize = 10_000_000;
            let mappingproxy = (0..LEN as u64)
                .map(|i| (i, i * 2))
                .into_py_mappingproxy(py)
                .unwrap();

            let mut sum = 0;
            for (k, _v) in mappingproxy.iter() {
                let i: u64 = k.extract().unwrap();
                sum += i;
            }
            assert_eq!(sum, 49_999_995_000_000);
        })
    }
}
