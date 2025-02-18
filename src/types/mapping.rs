use crate::conversion::IntoPyObject;
use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Bound;
use crate::py_result_ext::PyResultExt;
use crate::sync::GILOnceCell;
use crate::type_object::PyTypeInfo;
use crate::types::any::PyAnyMethods;
use crate::types::{PyAny, PyDict, PyList, PyType};
use crate::{ffi, Py, PyTypeCheck, Python};

/// Represents a reference to a Python object supporting the mapping protocol.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyMapping>`][crate::Py] or [`Bound<'py, PyMapping>`][Bound].
#[repr(transparent)]
pub struct PyMapping(PyAny);
pyobject_native_type_named!(PyMapping);

impl<'py> PyMapping {
    /// Register a pyclass as a subclass of `collections.abc.Mapping` (from the Python standard
    /// library). This is equivalent to `collections.abc.Mapping.register(T)` in Python.
    /// This registration is required for a pyclass to be downcastable from `PyAny` to `PyMapping`.
    pub fn register<T: PyTypeInfo>(py: Python<'py>) -> PyResult<()> {
        let ty = T::type_object(py);
        get_mapping_abc(py)?.call_method1("register", (ty,))?;
        Ok(())
    }

    /// Returns the number of objects in the mapping.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    #[inline]
    pub fn len(self: &Bound<'py, Self>) -> PyResult<usize> {
        let v = unsafe { ffi::PyMapping_Size(self.as_ptr()) };
        crate::err::error_on_minusone(self.py(), v)?;
        Ok(v as usize)
    }

    /// Returns whether the mapping is empty.
    #[inline]
    pub fn is_empty(self: &Bound<'py, Self>) -> PyResult<bool> {
        self.len().map(|l| l == 0)
    }

    /// Determines if the mapping contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(self: &Bound<'py, Self>, key: K) -> PyResult<bool>
    where
        K: IntoPyObject<'py>,
    {
        PyAnyMethods::contains(self, key)
    }

    /// Gets the item in self with key `key`.
    ///
    /// Returns an `Err` if the item with specified key is not found, usually `KeyError`.
    ///
    /// This is equivalent to the Python expression `self[key]`.
    #[inline]
    pub fn get_item<K>(self: &Bound<'py, Self>, key: K) -> PyResult<Bound<'py, PyAny>>
    where
        K: IntoPyObject<'py>,
    {
        PyAnyMethods::get_item(self, key)
    }

    /// Sets the item in self with key `key`.
    ///
    /// This is equivalent to the Python expression `self[key] = value`.
    #[inline]
    pub fn set_item<K, V>(self: &Bound<'py, Self>, key: K, value: V) -> PyResult<()>
    where
        K: IntoPyObject<'py>,
        V: IntoPyObject<'py>,
    {
        PyAnyMethods::set_item(self, key, value)
    }

    /// Deletes the item with key `key`.
    ///
    /// This is equivalent to the Python statement `del self[key]`.
    #[inline]
    pub fn del_item<K>(self: &Bound<'py, Self>, key: K) -> PyResult<()>
    where
        K: IntoPyObject<'py>,
    {
        PyAnyMethods::del_item(self, key)
    }

    /// Returns a list containing all keys in the mapping.
    #[inline]
    pub fn keys(self: &Bound<'py, Self>) -> PyResult<Bound<'py, PyList>> {
        unsafe {
            ffi::PyMapping_Keys(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }

    /// Returns a list containing all values in the mapping.
    #[inline]
    pub fn values(self: &Bound<'py, Self>) -> PyResult<Bound<'py, PyList>> {
        unsafe {
            ffi::PyMapping_Values(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }

    /// Returns a list of all (key, value) pairs in the mapping.
    #[inline]
    pub fn items(self: &Bound<'py, Self>) -> PyResult<Bound<'py, PyList>> {
        unsafe {
            ffi::PyMapping_Items(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }
}

fn get_mapping_abc(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    static MAPPING_ABC: GILOnceCell<Py<PyType>> = GILOnceCell::new();

    MAPPING_ABC.import(py, "collections.abc", "Mapping")
}

impl PyTypeCheck for PyMapping {
    const NAME: &'static str = "Mapping";

    #[inline]
    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        // Using `is_instance` for `collections.abc.Mapping` is slow, so provide
        // optimized case dict as a well-known mapping
        PyDict::is_type_of(object)
            || get_mapping_abc(object.py())
                .and_then(|abc| object.is_instance(abc.as_any()))
                .unwrap_or_else(|err| {
                    err.write_unraisable(object.py(), Some(object));
                    false
                })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{exceptions::PyKeyError, types::PyTuple};

    use super::*;
    use crate::conversion::IntoPyObject;

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let mut v = HashMap::<i32, i32>::new();
            let ob = (&v).into_pyobject(py).unwrap();
            let mapping = ob.downcast::<PyMapping>().unwrap();
            assert_eq!(0, mapping.len().unwrap());
            assert!(mapping.is_empty().unwrap());

            v.insert(7, 32);
            let ob = v.into_pyobject(py).unwrap();
            let mapping2 = ob.downcast::<PyMapping>().unwrap();
            assert_eq!(1, mapping2.len().unwrap());
            assert!(!mapping2.is_empty().unwrap());
        });
    }

    #[test]
    fn test_contains() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert("key0", 1234);
            let ob = v.into_pyobject(py).unwrap();
            let mapping = ob.downcast::<PyMapping>().unwrap();
            mapping.set_item("key1", "foo").unwrap();

            assert!(mapping.contains("key0").unwrap());
            assert!(mapping.contains("key1").unwrap());
            assert!(!mapping.contains("key2").unwrap());
        });
    }

    #[test]
    fn test_get_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.into_pyobject(py).unwrap();
            let mapping = ob.downcast::<PyMapping>().unwrap();
            assert_eq!(
                32,
                mapping.get_item(7i32).unwrap().extract::<i32>().unwrap()
            );
            assert!(mapping
                .get_item(8i32)
                .unwrap_err()
                .is_instance_of::<PyKeyError>(py));
        });
    }

    #[test]
    fn test_set_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.into_pyobject(py).unwrap();
            let mapping = ob.downcast::<PyMapping>().unwrap();
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
            let ob = v.into_pyobject(py).unwrap();
            let mapping = ob.downcast::<PyMapping>().unwrap();
            assert!(mapping.del_item(7i32).is_ok());
            assert_eq!(0, mapping.len().unwrap());
            assert!(mapping
                .get_item(7i32)
                .unwrap_err()
                .is_instance_of::<PyKeyError>(py));
        });
    }

    #[test]
    fn test_items() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.into_pyobject(py).unwrap();
            let mapping = ob.downcast::<PyMapping>().unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            let mut value_sum = 0;
            for el in mapping.items().unwrap().try_iter().unwrap() {
                let tuple = el.unwrap().downcast_into::<PyTuple>().unwrap();
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
            let ob = v.into_pyobject(py).unwrap();
            let mapping = ob.downcast::<PyMapping>().unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            for el in mapping.keys().unwrap().try_iter().unwrap() {
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
            let ob = v.into_pyobject(py).unwrap();
            let mapping = ob.downcast::<PyMapping>().unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut values_sum = 0;
            for el in mapping.values().unwrap().try_iter().unwrap() {
                values_sum += el.unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(32 + 42 + 123, values_sum);
        });
    }
}
