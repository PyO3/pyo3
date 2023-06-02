use crate::err::{PyDowncastError, PyErr, PyResult};
use crate::sync::GILOnceCell;
use crate::type_object::PyTypeInfo;
use crate::types::{PyAny, PyDict, PySequence, PyType};
use crate::{ffi, AsPyPointer, IntoPyPointer, Py, PyNativeType, PyTryFrom, Python, ToPyObject};

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

    /// Determines if the mapping contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        PyAny::contains(self, key)
    }

    /// Gets the item in self with key `key`.
    ///
    /// Returns an `Err` if the item with specified key is not found, usually `KeyError`.
    ///
    /// This is equivalent to the Python expression `self[key]`.
    #[inline]
    pub fn get_item<K>(&self, key: K) -> PyResult<&PyAny>
    where
        K: ToPyObject,
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
        K: ToPyObject,
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

    /// Register a pyclass as a subclass of `collections.abc.Mapping` (from the Python standard
    /// library). This is equvalent to `collections.abc.Mapping.register(T)` in Python.
    /// This registration is required for a pyclass to be downcastable from `PyAny` to `PyMapping`.
    pub fn register<T: PyTypeInfo>(py: Python<'_>) -> PyResult<()> {
        let ty = T::type_object(py);
        get_mapping_abc(py)?.call_method1("register", (ty,))?;
        Ok(())
    }
}

static MAPPING_ABC: GILOnceCell<Py<PyType>> = GILOnceCell::new();

fn get_mapping_abc(py: Python<'_>) -> PyResult<&PyType> {
    MAPPING_ABC
        .get_or_try_init(py, || {
            py.import("collections.abc")?.getattr("Mapping")?.extract()
        })
        .map(|ty| ty.as_ref(py))
}

impl<'v> PyTryFrom<'v> for PyMapping {
    /// Downcasting to `PyMapping` requires the concrete class to be a subclass (or registered
    /// subclass) of `collections.abc.Mapping` (from the Python standard library) - i.e.
    /// `isinstance(<class>, collections.abc.Mapping) == True`.
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v PyMapping, PyDowncastError<'v>> {
        let value = value.into();

        // Using `is_instance` for `collections.abc.Mapping` is slow, so provide
        // optimized case dict as a well-known mapping
        if PyDict::is_type_of(value)
            || get_mapping_abc(value.py())
                .and_then(|abc| value.is_instance(abc))
                // TODO: surface errors in this chain to the user
                .unwrap_or(false)
        {
            unsafe { return Ok(value.downcast_unchecked()) }
        }

        Err(PyDowncastError::new(value, "Mapping"))
    }

    #[inline]
    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v PyMapping, PyDowncastError<'v>> {
        value.into().downcast()
    }

    #[inline]
    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v PyMapping {
        let ptr = value.into() as *const _ as *const PyMapping;
        &*ptr
    }
}

impl Py<PyMapping> {
    /// Borrows a GIL-bound reference to the PyMapping. By binding to the GIL lifetime, this
    /// allows the GIL-bound reference to not require `Python` for any of its methods.
    pub fn as_ref<'py>(&'py self, _py: Python<'py>) -> &'py PyMapping {
        let any = self.as_ptr() as *const PyAny;
        unsafe { PyNativeType::unchecked_downcast(&*any) }
    }

    /// Similar to [`as_ref`](#method.as_ref), and also consumes this `Py` and registers the
    /// Python object reference in PyO3's object storage. The reference count for the Python
    /// object will not be decreased until the GIL lifetime ends.
    pub fn into_ref(self, py: Python<'_>) -> &PyMapping {
        unsafe { py.from_owned_ptr(self.into_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        exceptions::PyKeyError,
        types::{PyDict, PyTuple},
        Python,
    };

    use super::*;

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            let ob = v.to_object(py);
            let mapping: &PyMapping = ob.downcast(py).unwrap();
            assert_eq!(0, mapping.len().unwrap());
            assert!(mapping.is_empty().unwrap());

            v.insert(7, 32);
            let ob = v.to_object(py);
            let mapping2: &PyMapping = ob.downcast(py).unwrap();
            assert_eq!(1, mapping2.len().unwrap());
            assert!(!mapping2.is_empty().unwrap());
        });
    }

    #[test]
    fn test_contains() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert("key0", 1234);
            let ob = v.to_object(py);
            let mapping: &PyMapping = ob.downcast(py).unwrap();
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
            let ob = v.to_object(py);
            let mapping: &PyMapping = ob.downcast(py).unwrap();
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
            let ob = v.to_object(py);
            let mapping: &PyMapping = ob.downcast(py).unwrap();
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
            let mapping: &PyMapping = ob.downcast(py).unwrap();
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
            let ob = v.to_object(py);
            let mapping: &PyMapping = ob.downcast(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            let mut value_sum = 0;
            for el in mapping.items().unwrap().iter().unwrap() {
                let tuple = el.unwrap().downcast::<PyTuple>().unwrap();
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
            let mapping: &PyMapping = ob.downcast(py).unwrap();
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
            let mapping: &PyMapping = ob.downcast(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut values_sum = 0;
            for el in mapping.values().unwrap().iter().unwrap() {
                values_sum += el.unwrap().extract::<i32>().unwrap();
            }
            assert_eq!(32 + 42 + 123, values_sum);
        });
    }

    #[test]
    fn test_as_ref() {
        Python::with_gil(|py| {
            let mapping: Py<PyMapping> = PyDict::new(py).as_mapping().into();
            let mapping_ref: &PyMapping = mapping.as_ref(py);
            assert_eq!(mapping_ref.len().unwrap(), 0);
        })
    }

    #[test]
    fn test_into_ref() {
        Python::with_gil(|py| {
            let bare_mapping = PyDict::new(py).as_mapping();
            assert_eq!(bare_mapping.get_refcnt(), 1);
            let mapping: Py<PyMapping> = bare_mapping.into();
            assert_eq!(bare_mapping.get_refcnt(), 2);
            let mapping_ref = mapping.into_ref(py);
            assert_eq!(mapping_ref.len().unwrap(), 0);
            assert_eq!(mapping_ref.get_refcnt(), 2);
        })
    }
}
