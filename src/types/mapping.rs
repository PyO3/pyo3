use crate::err::{PyDowncastError, PyResult};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Py2;
use crate::py_result_ext::PyResultExt;
use crate::sync::GILOnceCell;
use crate::type_object::PyTypeInfo;
use crate::types::any::PyAnyMethods;
use crate::types::{PyAny, PyDict, PySequence, PyType};
use crate::{ffi, Py, PyNativeType, PyTypeCheck, Python, ToPyObject};

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
        Py2::borrowed_from_gil_ref(&self).len()
    }

    /// Returns whether the mapping is empty.
    #[inline]
    pub fn is_empty(&self) -> PyResult<bool> {
        Py2::borrowed_from_gil_ref(&self).is_empty()
    }

    /// Determines if the mapping contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).contains(key)
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
        Py2::borrowed_from_gil_ref(&self)
            .get_item(key)
            .map(Py2::into_gil_ref)
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
        Py2::borrowed_from_gil_ref(&self).set_item(key, value)
    }

    /// Deletes the item with key `key`.
    ///
    /// This is equivalent to the Python statement `del self[key]`.
    #[inline]
    pub fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        Py2::borrowed_from_gil_ref(&self).del_item(key)
    }

    /// Returns a sequence containing all keys in the mapping.
    #[inline]
    pub fn keys(&self) -> PyResult<&PySequence> {
        Py2::borrowed_from_gil_ref(&self)
            .keys()
            .map(Py2::into_gil_ref)
    }

    /// Returns a sequence containing all values in the mapping.
    #[inline]
    pub fn values(&self) -> PyResult<&PySequence> {
        Py2::borrowed_from_gil_ref(&self)
            .values()
            .map(Py2::into_gil_ref)
    }

    /// Returns a sequence of tuples of all (key, value) pairs in the mapping.
    #[inline]
    pub fn items(&self) -> PyResult<&PySequence> {
        Py2::borrowed_from_gil_ref(&self)
            .items()
            .map(Py2::into_gil_ref)
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

/// Implementation of functionality for [`PyMapping`].
///
/// These methods are defined for the `Py2<'py, PyMapping>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyMapping")]
pub(crate) trait PyMappingMethods<'py> {
    /// Returns the number of objects in the mapping.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    fn len(&self) -> PyResult<usize>;

    /// Returns whether the mapping is empty.
    fn is_empty(&self) -> PyResult<bool>;

    /// Determines if the mapping contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject;

    /// Gets the item in self with key `key`.
    ///
    /// Returns an `Err` if the item with specified key is not found, usually `KeyError`.
    ///
    /// This is equivalent to the Python expression `self[key]`.
    fn get_item<K>(&self, key: K) -> PyResult<Py2<'py, PyAny>>
    where
        K: ToPyObject;

    /// Sets the item in self with key `key`.
    ///
    /// This is equivalent to the Python expression `self[key] = value`.
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject;

    /// Deletes the item with key `key`.
    ///
    /// This is equivalent to the Python statement `del self[key]`.
    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject;

    /// Returns a sequence containing all keys in the mapping.
    fn keys(&self) -> PyResult<Py2<'py, PySequence>>;

    /// Returns a sequence containing all values in the mapping.
    fn values(&self) -> PyResult<Py2<'py, PySequence>>;

    /// Returns a sequence of tuples of all (key, value) pairs in the mapping.
    fn items(&self) -> PyResult<Py2<'py, PySequence>>;
}

impl<'py> PyMappingMethods<'py> for Py2<'py, PyMapping> {
    #[inline]
    fn len(&self) -> PyResult<usize> {
        let v = unsafe { ffi::PyMapping_Size(self.as_ptr()) };
        crate::err::error_on_minusone(self.py(), v)?;
        Ok(v as usize)
    }

    #[inline]
    fn is_empty(&self) -> PyResult<bool> {
        self.len().map(|l| l == 0)
    }

    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        PyAnyMethods::contains(&**self, key)
    }

    #[inline]
    fn get_item<K>(&self, key: K) -> PyResult<Py2<'py, PyAny>>
    where
        K: ToPyObject,
    {
        PyAnyMethods::get_item(&**self, key)
    }

    #[inline]
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject,
    {
        PyAnyMethods::set_item(&**self, key, value)
    }

    #[inline]
    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        PyAnyMethods::del_item(&**self, key)
    }

    #[inline]
    fn keys(&self) -> PyResult<Py2<'py, PySequence>> {
        unsafe {
            ffi::PyMapping_Keys(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }

    #[inline]
    fn values(&self) -> PyResult<Py2<'py, PySequence>> {
        unsafe {
            ffi::PyMapping_Values(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
    }

    #[inline]
    fn items(&self) -> PyResult<Py2<'py, PySequence>> {
        unsafe {
            ffi::PyMapping_Items(self.as_ptr())
                .assume_owned_or_err(self.py())
                .downcast_into_unchecked()
        }
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

impl PyTypeCheck for PyMapping {
    const NAME: &'static str = "Mapping";

    #[inline]
    fn type_check(object: &PyAny) -> bool {
        // Using `is_instance` for `collections.abc.Mapping` is slow, so provide
        // optimized case dict as a well-known mapping
        PyDict::is_type_of(object)
            || get_mapping_abc(object.py())
                .and_then(|abc| object.is_instance(abc))
                // TODO: surface errors in this chain to the user
                .unwrap_or(false)
    }
}

#[allow(deprecated)]
impl<'v> crate::PyTryFrom<'v> for PyMapping {
    /// Downcasting to `PyMapping` requires the concrete class to be a subclass (or registered
    /// subclass) of `collections.abc.Mapping` (from the Python standard library) - i.e.
    /// `isinstance(<class>, collections.abc.Mapping) == True`.
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v PyMapping, PyDowncastError<'v>> {
        let value = value.into();

        if PyMapping::type_check(value) {
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
    #[allow(deprecated)]
    fn test_mapping_try_from() {
        use crate::PyTryFrom;
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            let _ = <PyMapping as PyTryFrom>::try_from(dict).unwrap();
            let _ = PyMapping::try_from_exact(dict).unwrap();
        });
    }
}
