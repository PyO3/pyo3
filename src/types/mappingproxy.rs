// Copyright (c) 2017-present PyO3 Project and Contributors

use super::PyMapping;
use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Bound;
use crate::types::any::PyAnyMethods;
use crate::types::{PyAny, PyIterator, PyList};
use crate::{ffi, Python};

use std::ffi::c_int;

/// Represents a Python `mappingproxy`.
#[repr(transparent)]
pub struct PyMappingProxy(PyAny);

#[inline]
unsafe fn dict_proxy_check(op: *mut ffi::PyObject) -> c_int {
    unsafe { ffi::Py_IS_TYPE(op, std::ptr::addr_of_mut!(ffi::PyDictProxy_Type)) }
}

pyobject_native_type_core!(
    PyMappingProxy,
    pyobject_native_static_type_object!(ffi::PyDictProxy_Type),
    #checkfunction=dict_proxy_check
);

impl PyMappingProxy {
    /// Creates a mappingproxy from an object.
    pub fn new<'py>(
        py: Python<'py>,
        elements: &Bound<'py, PyMapping>,
    ) -> Bound<'py, PyMappingProxy> {
        unsafe {
            ffi::PyDictProxy_New(elements.as_ptr())
                .assume_owned(py)
                .cast_into_unchecked()
        }
    }
}

/// Implementation of functionality for [`PyMappingProxy`].
///
/// These methods are defined for the `Bound<'py, PyMappingProxy>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyMappingProxy")]
pub trait PyMappingProxyMethods<'py, 'a>: crate::sealed::Sealed {
    /// Checks if the mappingproxy is empty, i.e. `len(self) == 0`.
    fn is_empty(&self) -> PyResult<bool>;

    /// Returns a list containing all keys in the mapping.
    fn keys(&self) -> PyResult<Bound<'py, PyList>>;

    /// Returns a list containing all values in the mapping.
    fn values(&self) -> PyResult<Bound<'py, PyList>>;

    /// Returns a list of tuples of all (key, value) pairs in the mapping.
    fn items(&self) -> PyResult<Bound<'py, PyList>>;

    /// Returns `self` cast as a `PyMapping`.
    fn as_mapping(&self) -> &Bound<'py, PyMapping>;

    /// Takes an object and returns an iterator for it. Returns an error if the object is not
    /// iterable.
    fn try_iter(&'a self) -> PyResult<BoundMappingProxyIterator<'py, 'a>>;
}

impl<'py, 'a> PyMappingProxyMethods<'py, 'a> for Bound<'py, PyMappingProxy> {
    fn is_empty(&self) -> PyResult<bool> {
        Ok(self.len()? == 0)
    }

    #[inline]
    fn keys(&self) -> PyResult<Bound<'py, PyList>> {
        unsafe {
            Ok(ffi::PyMapping_Keys(self.as_ptr())
                .assume_owned_or_err(self.py())?
                .cast_into_unchecked())
        }
    }

    #[inline]
    fn values(&self) -> PyResult<Bound<'py, PyList>> {
        unsafe {
            Ok(ffi::PyMapping_Values(self.as_ptr())
                .assume_owned_or_err(self.py())?
                .cast_into_unchecked())
        }
    }

    #[inline]
    fn items(&self) -> PyResult<Bound<'py, PyList>> {
        unsafe {
            Ok(ffi::PyMapping_Items(self.as_ptr())
                .assume_owned_or_err(self.py())?
                .cast_into_unchecked())
        }
    }

    fn as_mapping(&self) -> &Bound<'py, PyMapping> {
        unsafe { self.cast_unchecked() }
    }

    fn try_iter(&'a self) -> PyResult<BoundMappingProxyIterator<'py, 'a>> {
        Ok(BoundMappingProxyIterator {
            iterator: PyIterator::from_object(self)?,
            mappingproxy: self,
        })
    }
}

pub struct BoundMappingProxyIterator<'py, 'a> {
    iterator: Bound<'py, PyIterator>,
    mappingproxy: &'a Bound<'py, PyMappingProxy>,
}

impl<'py> Iterator for BoundMappingProxyIterator<'py, '_> {
    type Item = PyResult<(Bound<'py, PyAny>, Bound<'py, PyAny>)>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|key| match key {
            Ok(key) => match self.mappingproxy.get_item(&key) {
                Ok(value) => Ok((key, value)),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::types::dict::*;
    use crate::Python;
    use crate::{
        exceptions::PyKeyError,
        types::{PyInt, PyTuple},
    };
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_new() {
        Python::attach(|py| {
            let pydict = [(7, 32)].into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, pydict.as_mapping());
            mappingproxy.get_item(7i32).unwrap();
            assert_eq!(
                32,
                mappingproxy
                    .get_item(7i32)
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(mappingproxy
                .get_item(8i32)
                .unwrap_err()
                .is_instance_of::<PyKeyError>(py));
        });
    }

    #[test]
    fn test_len() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            let dict = v.clone().into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());
            assert_eq!(mappingproxy.len().unwrap(), 0);
            v.insert(7, 32);
            let dict2 = v.clone().into_py_dict(py).unwrap();
            let mp2 = PyMappingProxy::new(py, dict2.as_mapping());
            assert_eq!(mp2.len().unwrap(), 1);
        });
    }

    #[test]
    fn test_contains() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let dict = v.clone().into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());
            assert!(mappingproxy.contains(7i32).unwrap());
            assert!(!mappingproxy.contains(8i32).unwrap());
        });
    }

    #[test]
    fn test_get_item() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let dict = v.clone().into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());
            assert_eq!(
                32,
                mappingproxy
                    .get_item(7i32)
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(mappingproxy
                .get_item(8i32)
                .unwrap_err()
                .is_instance_of::<PyKeyError>(py));
        });
    }

    #[test]
    fn test_set_item_refcnt() {
        Python::attach(|py| {
            let cnt;
            {
                let none = py.None();
                cnt = none.get_refcnt(py);
                let dict = [(10, none)].into_py_dict(py).unwrap();
                let _mappingproxy = PyMappingProxy::new(py, dict.as_mapping());
            }
            {
                assert_eq!(cnt, py.None().get_refcnt(py));
            }
        });
    }

    #[test]
    fn test_isempty() {
        Python::attach(|py| {
            let map: HashMap<usize, usize> = HashMap::new();
            let dict = map.into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());
            assert!(mappingproxy.is_empty().unwrap());
        });
    }

    #[test]
    fn test_keys() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = v.into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            for el in mappingproxy.keys().unwrap().try_iter().unwrap() {
                key_sum += el.unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
        });
    }

    #[test]
    fn test_values() {
        Python::attach(|py| {
            let mut v: HashMap<i32, i32> = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = v.into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut values_sum = 0;
            for el in mappingproxy.values().unwrap().try_iter().unwrap() {
                values_sum += el.unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(32 + 42 + 123, values_sum);
        });
    }

    #[test]
    fn test_items() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = v.into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            let mut value_sum = 0;
            for res in mappingproxy.items().unwrap().try_iter().unwrap() {
                let el = res.unwrap();
                let tuple = el.cast::<PyTuple>().unwrap();
                key_sum += tuple.get_item(0).unwrap().extract::<i32>().unwrap();
                value_sum += tuple.get_item(1).unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_iter() {
        Python::attach(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let dict = v.into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());
            let mut key_sum = 0;
            let mut value_sum = 0;
            for res in mappingproxy.try_iter().unwrap() {
                let (key, value) = res.unwrap();
                key_sum += key.extract::<i32>().unwrap();
                value_sum += value.extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_hashmap_into_python() {
        Python::attach(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let dict = map.clone().into_py_dict(py).unwrap();
            let py_map = PyMappingProxy::new(py, dict.as_mapping());

            assert_eq!(py_map.len().unwrap(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_hashmap_into_mappingproxy() {
        Python::attach(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let dict = map.clone().into_py_dict(py).unwrap();
            let py_map = PyMappingProxy::new(py, dict.as_mapping());

            assert_eq!(py_map.len().unwrap(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_btreemap_into_py() {
        Python::attach(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let dict = map.clone().into_py_dict(py).unwrap();
            let py_map = PyMappingProxy::new(py, dict.as_mapping());

            assert_eq!(py_map.len().unwrap(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_btreemap_into_mappingproxy() {
        Python::attach(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let dict = map.clone().into_py_dict(py).unwrap();
            let py_map = PyMappingProxy::new(py, dict.as_mapping());

            assert_eq!(py_map.len().unwrap(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_vec_into_mappingproxy() {
        Python::attach(|py| {
            let vec = vec![("a", 1), ("b", 2), ("c", 3)];
            let dict = vec.clone().into_py_dict(py).unwrap();
            let py_map = PyMappingProxy::new(py, dict.as_mapping());

            assert_eq!(py_map.len().unwrap(), 3);
            assert_eq!(py_map.get_item("b").unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn test_slice_into_mappingproxy() {
        Python::attach(|py| {
            let arr = [("a", 1), ("b", 2), ("c", 3)];

            let dict = arr.into_py_dict(py).unwrap();
            let py_map = PyMappingProxy::new(py, dict.as_mapping());

            assert_eq!(py_map.len().unwrap(), 3);
            assert_eq!(py_map.get_item("b").unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn mappingproxy_as_mapping() {
        Python::attach(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let dict = map.clone().into_py_dict(py).unwrap();
            let py_map = PyMappingProxy::new(py, dict.as_mapping());

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

    #[cfg(not(any(PyPy, GraalPy)))]
    fn abc_mappingproxy(py: Python<'_>) -> Bound<'_, PyMappingProxy> {
        let mut map = HashMap::<&'static str, i32>::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);
        let dict = map.clone().into_py_dict(py).unwrap();
        PyMappingProxy::new(py, dict.as_mapping())
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn mappingproxy_keys_view() {
        Python::attach(|py| {
            let mappingproxy = abc_mappingproxy(py);
            let keys = mappingproxy.call_method0("keys").unwrap();
            assert!(keys.is_instance(&py.get_type::<PyDictKeys>()).unwrap());
        })
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn mappingproxy_values_view() {
        Python::attach(|py| {
            let mappingproxy = abc_mappingproxy(py);
            let values = mappingproxy.call_method0("values").unwrap();
            assert!(values.is_instance(&py.get_type::<PyDictValues>()).unwrap());
        })
    }

    #[test]
    #[cfg(not(any(PyPy, GraalPy)))]
    fn mappingproxy_items_view() {
        Python::attach(|py| {
            let mappingproxy = abc_mappingproxy(py);
            let items = mappingproxy.call_method0("items").unwrap();
            assert!(items.is_instance(&py.get_type::<PyDictItems>()).unwrap());
        })
    }

    #[test]
    fn get_value_from_mappingproxy_of_strings() {
        Python::attach(|py: Python<'_>| {
            let mut map = HashMap::new();
            map.insert("first key".to_string(), "first value".to_string());
            map.insert("second key".to_string(), "second value".to_string());
            map.insert("third key".to_string(), "third value".to_string());

            let dict = map.clone().into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());

            assert_eq!(
                map.into_iter().collect::<Vec<(String, String)>>(),
                mappingproxy
                    .try_iter()
                    .unwrap()
                    .map(|object| {
                        let tuple = object.unwrap();
                        (
                            tuple.0.extract::<String>().unwrap(),
                            tuple.1.extract::<String>().unwrap(),
                        )
                    })
                    .collect::<Vec<(String, String)>>()
            );
        })
    }

    #[test]
    fn get_value_from_mappingproxy_of_integers() {
        Python::attach(|py: Python<'_>| {
            const LEN: usize = 10_000;
            let items: Vec<(usize, usize)> = (1..LEN).map(|i| (i, i - 1)).collect();

            let dict = items.clone().into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());

            assert_eq!(
                items,
                mappingproxy
                    .clone()
                    .try_iter()
                    .unwrap()
                    .map(|object| {
                        let tuple = object.unwrap();
                        (
                            tuple.0.cast::<PyInt>().unwrap().extract::<usize>().unwrap(),
                            tuple.1.cast::<PyInt>().unwrap().extract::<usize>().unwrap(),
                        )
                    })
                    .collect::<Vec<(usize, usize)>>()
            );
            for index in 1..LEN {
                assert_eq!(
                    mappingproxy
                        .clone()
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
        Python::attach(|py| {
            const LEN: usize = 1_000;
            let items = (0..LEN as u64).map(|i| (i, i * 2));

            let dict = items.clone().into_py_dict(py).unwrap();
            let mappingproxy = PyMappingProxy::new(py, dict.as_mapping());

            let mut sum = 0;
            for result in mappingproxy.try_iter().unwrap() {
                let (k, _v) = result.unwrap();
                let i: u64 = k.extract().unwrap();
                sum += i;
            }
            assert_eq!(sum, 499_500);
        })
    }
}
