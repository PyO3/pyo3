// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::err::{PyDowncastError, PyErr, PyResult};
use crate::types::{PyAny, PySequence};
use crate::AsPyPointer;
use crate::{ffi, ToPyObject};
use crate::{PyTryFrom, ToBorrowedObject};

/// Represents a reference to a Python object supporting the mapping protocol.
#[repr(transparent)]
pub struct PyMapping(PyAny);
pyobject_native_type_named!(PyMapping);
pyobject_native_type_extract!(PyMapping);

impl PyMapping {
    /// Returns the number of objects in the mapping.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    #[inline]
    pub fn len(&self) -> PyResult<usize> {
        let v = unsafe { ffi::PyMapping_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(v as usize)
        }
    }

    /// Returns whether the mapping is empty.
    #[inline]
    pub fn is_empty(&self) -> PyResult<bool> {
        self.len().map(|l| l == 0)
    }

    /// Gets the item in self with key `key`.
    ///
    /// Returns an `Err` if the item with specified key is not found, usually `KeyError`.
    ///
    /// This is equivalent to the Python expression `self[key]`.
    #[inline]
    pub fn get_item<K>(&self, key: K) -> PyResult<&PyAny>
    where
        K: ToBorrowedObject,
    {
        PyAny::get_item(self, key)
    }

    /// Sets the item in self with key `key`.
    ///
    /// This is equivalent to the Python expression `self[key] = value`.
    #[inline]
    pub fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject,
    {
        PyAny::set_item(self, key, value)
    }

    /// Deletes the item with key `key`.
    ///
    /// This is equivalent to the Python statement `del self[key]`.
    #[inline]
    pub fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToBorrowedObject,
    {
        PyAny::del_item(self, key)
    }

    /// Returns a sequence containing all keys in the mapping.
    #[inline]
    pub fn keys(&self) -> PyResult<&PySequence> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyMapping_Keys(self.as_ptr()))
        }
    }

    /// Returns a sequence containing all values in the mapping.
    #[inline]
    pub fn values(&self) -> PyResult<&PySequence> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyMapping_Values(self.as_ptr()))
        }
    }

    /// Returns a sequence of tuples of all (key, value) pairs in the mapping.
    #[inline]
    pub fn items(&self) -> PyResult<&PySequence> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PyMapping_Items(self.as_ptr()))
        }
    }
}

impl<'v> PyTryFrom<'v> for PyMapping {
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v PyMapping, PyDowncastError<'v>> {
        let value = value.into();
        unsafe {
            if ffi::PyMapping_Check(value.as_ptr()) != 0 {
                Ok(<PyMapping as PyTryFrom>::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value, "Mapping"))
            }
        }
    }

    #[inline]
    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v PyMapping, PyDowncastError<'v>> {
        <PyMapping as PyTryFrom>::try_from(value)
    }

    #[inline]
    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v PyMapping {
        let ptr = value.into() as *const _ as *const PyMapping;
        &*ptr
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{exceptions::PyKeyError, types::PyTuple, Python};

    use super::*;

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            let ob = v.to_object(py);
            let mapping = <PyMapping as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(0, mapping.len().unwrap());
            assert!(mapping.is_empty().unwrap());

            v.insert(7, 32);
            let ob = v.to_object(py);
            let mapping2 = <PyMapping as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(1, mapping2.len().unwrap());
            assert!(!mapping2.is_empty().unwrap());
        });
    }

    #[test]
    fn test_get_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let mapping = <PyMapping as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(
                32,
                mapping.get_item(7i32).unwrap().extract::<i32>().unwrap()
            );
            assert!(mapping
                .get_item(8i32)
                .unwrap_err()
                .is_instance::<PyKeyError>(py));
        });
    }

    #[test]
    fn test_set_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let mapping = <PyMapping as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert!(mapping.set_item(7i32, 42i32).is_ok()); // change
            assert!(mapping.set_item(8i32, 123i32).is_ok()); // insert
            assert_eq!(
                42i32,
                mapping.get_item(7i32).unwrap().extract::<i32>().unwrap()
            );
            assert_eq!(
                123i32,
                mapping.get_item(8i32).unwrap().extract::<i32>().unwrap()
            );
        });
    }

    #[test]
    fn test_del_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let mapping = <PyMapping as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert!(mapping.del_item(7i32).is_ok());
            assert_eq!(0, mapping.len().unwrap());
            assert!(mapping
                .get_item(7i32)
                .unwrap_err()
                .is_instance::<PyKeyError>(py));
        });
    }

    #[test]
    fn test_items() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let mapping = <PyMapping as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            let mut value_sum = 0;
            for el in mapping.items().unwrap().iter().unwrap() {
                let tuple = el.unwrap().cast_as::<PyTuple>().unwrap();
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
            let ob = v.to_object(py);
            let mapping = <PyMapping as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            for el in mapping.keys().unwrap().iter().unwrap() {
                key_sum += el.unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
        });
    }

    #[test]
    fn test_values() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let mapping = <PyMapping as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut values_sum = 0;
            for el in mapping.values().unwrap().iter().unwrap() {
                values_sum += el.unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(32 + 42 + 123, values_sum);
        });
    }
}
