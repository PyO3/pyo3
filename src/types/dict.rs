// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::err::{self, PyErr, PyResult};
use crate::types::{PyAny, PyList};
#[cfg(not(PyPy))]
use crate::IntoPyPointer;
use crate::{ffi, AsPyPointer, FromPyObject, IntoPy, PyObject, PyTryFrom, Python, ToPyObject};
use std::collections::{BTreeMap, HashMap};
use std::ptr::NonNull;
use std::{cmp, collections, hash};

use super::PyMapping;

/// Represents a Python `dict`.
#[repr(transparent)]
pub struct PyDict(PyAny);

pyobject_native_type!(
    PyDict,
    ffi::PyDictObject,
    ffi::PyDict_Type,
    #checkfunction=ffi::PyDict_Check
);

impl PyDict {
    /// Creates a new empty dictionary.
    pub fn new(py: Python<'_>) -> &PyDict {
        unsafe { py.from_owned_ptr::<PyDict>(ffi::PyDict_New()) }
    }

    /// Creates a new dictionary from the sequence given.
    ///
    /// The sequence must consist of `(PyObject, PyObject)`. This is
    /// equivalent to `dict([("a", 1), ("b", 2)])`.
    ///
    /// Returns an error on invalid input. In the case of key collisions,
    /// this keeps the last entry seen.
    #[cfg(not(PyPy))]
    pub fn from_sequence(py: Python<'_>, seq: PyObject) -> PyResult<&PyDict> {
        unsafe {
            let dict = py.from_owned_ptr::<PyDict>(ffi::PyDict_New());
            err::error_on_minusone(
                py,
                ffi::PyDict_MergeFromSeq2(dict.into_ptr(), seq.into_ptr(), 1),
            )?;
            Ok(dict)
        }
    }

    /// Returns a new dictionary that contains the same key-value pairs as self.
    ///
    /// This is equivalent to the Python expression `self.copy()`.
    pub fn copy(&self) -> PyResult<&PyDict> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err::<PyDict>(ffi::PyDict_Copy(self.as_ptr()))
        }
    }

    /// Empties an existing dictionary of all key-value pairs.
    pub fn clear(&self) {
        unsafe { ffi::PyDict_Clear(self.as_ptr()) }
    }

    /// Return the number of items in the dictionary.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    pub fn len(&self) -> usize {
        unsafe { ffi::PyDict_Size(self.as_ptr()) as usize }
    }

    /// Checks if the dict is empty, i.e. `len(self) == 0`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Determines if the dictionary contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        unsafe {
            match ffi::PyDict_Contains(self.as_ptr(), key.to_object(self.py()).as_ptr()) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.py())),
            }
        }
    }

    /// Gets an item from the dictionary.
    ///
    /// Returns `None` if the item is not present, or if an error occurs.
    ///
    /// To get a `KeyError` for non-existing keys, use `PyAny::get_item`.
    pub fn get_item<K>(&self, key: K) -> Option<&PyAny>
    where
        K: ToPyObject,
    {
        unsafe {
            let ptr = ffi::PyDict_GetItem(self.as_ptr(), key.to_object(self.py()).as_ptr());
            NonNull::new(ptr).map(|p| {
                // PyDict_GetItem return s borrowed ptr, must make it owned for safety (see #890).
                self.py().from_owned_ptr(ffi::_Py_NewRef(p.as_ptr()))
            })
        }
    }

    /// Sets an item value.
    ///
    /// This is equivalent to the Python statement `self[key] = value`.
    pub fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject,
    {
        let py = self.py();
        unsafe {
            err::error_on_minusone(
                py,
                ffi::PyDict_SetItem(
                    self.as_ptr(),
                    key.to_object(py).as_ptr(),
                    value.to_object(py).as_ptr(),
                ),
            )
        }
    }

    /// Deletes an item.
    ///
    /// This is equivalent to the Python statement `del self[key]`.
    pub fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        let py = self.py();
        unsafe {
            err::error_on_minusone(
                py,
                ffi::PyDict_DelItem(self.as_ptr(), key.to_object(py).as_ptr()),
            )
        }
    }

    /// Returns a list of dict keys.
    ///
    /// This is equivalent to the Python expression `list(dict.keys())`.
    pub fn keys(&self) -> &PyList {
        unsafe {
            self.py()
                .from_owned_ptr::<PyList>(ffi::PyDict_Keys(self.as_ptr()))
        }
    }

    /// Returns a list of dict values.
    ///
    /// This is equivalent to the Python expression `list(dict.values())`.
    pub fn values(&self) -> &PyList {
        unsafe {
            self.py()
                .from_owned_ptr::<PyList>(ffi::PyDict_Values(self.as_ptr()))
        }
    }

    /// Returns a list of dict items.
    ///
    /// This is equivalent to the Python expression `list(dict.items())`.
    pub fn items(&self) -> &PyList {
        unsafe {
            self.py()
                .from_owned_ptr::<PyList>(ffi::PyDict_Items(self.as_ptr()))
        }
    }

    /// Returns an iterator of `(key, value)` pairs in this dictionary.
    ///
    /// Note that it's unsafe to use when the dictionary might be changed by
    /// other code.
    pub fn iter(&self) -> PyDictIterator<'_> {
        PyDictIterator {
            dict: self.as_ref(),
            pos: 0,
        }
    }

    /// Returns `self` cast as a `PyMapping`.
    pub fn as_mapping(&self) -> &PyMapping {
        unsafe { PyMapping::try_from_unchecked(self) }
    }
}

pub struct PyDictIterator<'py> {
    dict: &'py PyAny,
    pos: isize,
}

impl<'py> Iterator for PyDictIterator<'py> {
    type Item = (&'py PyAny, &'py PyAny);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut key: *mut ffi::PyObject = std::ptr::null_mut();
            let mut value: *mut ffi::PyObject = std::ptr::null_mut();
            if ffi::PyDict_Next(self.dict.as_ptr(), &mut self.pos, &mut key, &mut value) != 0 {
                let py = self.dict.py();
                // PyDict_Next returns borrowed values; for safety must make them owned (see #890)
                Some((
                    py.from_owned_ptr(ffi::_Py_NewRef(key)),
                    py.from_owned_ptr(ffi::_Py_NewRef(value)),
                ))
            } else {
                None
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.dict.len().unwrap_or_default();
        (
            len.saturating_sub(self.pos as usize),
            Some(len.saturating_sub(self.pos as usize)),
        )
    }
}

impl<'a> std::iter::IntoIterator for &'a PyDict {
    type Item = (&'a PyAny, &'a PyAny);
    type IntoIter = PyDictIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V, H> ToPyObject for collections::HashMap<K, V, H>
where
    K: hash::Hash + cmp::Eq + ToPyObject,
    V: ToPyObject,
    H: hash::BuildHasher,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        IntoPyDict::into_py_dict(self, py).into()
    }
}

impl<K, V> ToPyObject for collections::BTreeMap<K, V>
where
    K: cmp::Eq + ToPyObject,
    V: ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        IntoPyDict::into_py_dict(self, py).into()
    }
}

impl<K, V, H> IntoPy<PyObject> for collections::HashMap<K, V, H>
where
    K: hash::Hash + cmp::Eq + IntoPy<PyObject>,
    V: IntoPy<PyObject>,
    H: hash::BuildHasher,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        let iter = self
            .into_iter()
            .map(|(k, v)| (k.into_py(py), v.into_py(py)));
        IntoPyDict::into_py_dict(iter, py).into()
    }
}

impl<K, V> IntoPy<PyObject> for collections::BTreeMap<K, V>
where
    K: cmp::Eq + IntoPy<PyObject>,
    V: IntoPy<PyObject>,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        let iter = self
            .into_iter()
            .map(|(k, v)| (k.into_py(py), v.into_py(py)));
        IntoPyDict::into_py_dict(iter, py).into()
    }
}

/// Conversion trait that allows a sequence of tuples to be converted into `PyDict`
/// Primary use case for this trait is `call` and `call_method` methods as keywords argument.
pub trait IntoPyDict {
    /// Converts self into a `PyDict` object pointer. Whether pointer owned or borrowed
    /// depends on implementation.
    fn into_py_dict(self, py: Python<'_>) -> &PyDict;
}

impl<T, I> IntoPyDict for I
where
    T: PyDictItem,
    I: IntoIterator<Item = T>,
{
    fn into_py_dict(self, py: Python<'_>) -> &PyDict {
        let dict = PyDict::new(py);
        for item in self {
            dict.set_item(item.key(), item.value())
                .expect("Failed to set_item on dict");
        }
        dict
    }
}

/// Represents a tuple which can be used as a PyDict item.
pub trait PyDictItem {
    type K: ToPyObject;
    type V: ToPyObject;
    fn key(&self) -> &Self::K;
    fn value(&self) -> &Self::V;
}

impl<K, V> PyDictItem for (K, V)
where
    K: ToPyObject,
    V: ToPyObject,
{
    type K = K;
    type V = V;
    fn key(&self) -> &Self::K {
        &self.0
    }
    fn value(&self) -> &Self::V {
        &self.1
    }
}

impl<K, V> PyDictItem for &(K, V)
where
    K: ToPyObject,
    V: ToPyObject,
{
    type K = K;
    type V = V;
    fn key(&self) -> &Self::K {
        &self.0
    }
    fn value(&self) -> &Self::V {
        &self.1
    }
}

impl<'source, K, V, S> FromPyObject<'source> for HashMap<K, V, S>
where
    K: FromPyObject<'source> + cmp::Eq + hash::Hash,
    V: FromPyObject<'source>,
    S: hash::BuildHasher + Default,
{
    fn extract(ob: &'source PyAny) -> Result<Self, PyErr> {
        let dict = <PyDict as PyTryFrom>::try_from(ob)?;
        let mut ret = HashMap::with_capacity_and_hasher(dict.len(), S::default());
        for (k, v) in dict.iter() {
            ret.insert(K::extract(k)?, V::extract(v)?);
        }
        Ok(ret)
    }
}

impl<'source, K, V> FromPyObject<'source> for BTreeMap<K, V>
where
    K: FromPyObject<'source> + cmp::Ord,
    V: FromPyObject<'source>,
{
    fn extract(ob: &'source PyAny) -> Result<Self, PyErr> {
        let dict = <PyDict as PyTryFrom>::try_from(ob)?;
        let mut ret = BTreeMap::new();
        for (k, v) in dict.iter() {
            ret.insert(K::extract(k)?, V::extract(v)?);
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::conversion::IntoPy;
    use crate::types::dict::IntoPyDict;
    #[cfg(not(PyPy))]
    use crate::types::PyList;
    use crate::types::{PyDict, PyTuple};
    use crate::PyObject;
    use crate::Python;
    use crate::{PyTryFrom, ToPyObject};
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn test_new() {
        Python::with_gil(|py| {
            let dict = [(7, 32)].into_py_dict(py);
            assert_eq!(32, dict.get_item(7i32).unwrap().extract::<i32>().unwrap());
            assert!(dict.get_item(8i32).is_none());
            let map: HashMap<i32, i32> = [(7, 32)].iter().cloned().collect();
            assert_eq!(map, dict.extract().unwrap());
            let map: BTreeMap<i32, i32> = [(7, 32)].iter().cloned().collect();
            assert_eq!(map, dict.extract().unwrap());
        });
    }

    #[test]
    #[cfg(not(PyPy))]
    fn test_from_sequence() {
        Python::with_gil(|py| {
            let items = PyList::new(py, &vec![("a", 1), ("b", 2)]);
            let dict = PyDict::from_sequence(py, items.to_object(py)).unwrap();
            assert_eq!(1, dict.get_item("a").unwrap().extract::<i32>().unwrap());
            assert_eq!(2, dict.get_item("b").unwrap().extract::<i32>().unwrap());
            let map: HashMap<&str, i32> = [("a", 1), ("b", 2)].iter().cloned().collect();
            assert_eq!(map, dict.extract().unwrap());
            let map: BTreeMap<&str, i32> = [("a", 1), ("b", 2)].iter().cloned().collect();
            assert_eq!(map, dict.extract().unwrap());
        });
    }

    #[test]
    #[cfg(not(PyPy))]
    fn test_from_sequence_err() {
        Python::with_gil(|py| {
            let items = PyList::new(py, &vec!["a", "b"]);
            assert!(PyDict::from_sequence(py, items.to_object(py)).is_err());
        });
    }

    #[test]
    fn test_copy() {
        Python::with_gil(|py| {
            let dict = [(7, 32)].into_py_dict(py);

            let ndict = dict.copy().unwrap();
            assert_eq!(32, ndict.get_item(7i32).unwrap().extract::<i32>().unwrap());
            assert!(ndict.get_item(8i32).is_none());
        });
    }

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(0, dict.len());
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict2 = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(1, dict2.len());
        });
    }

    #[test]
    fn test_contains() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert!(dict.contains(7i32).unwrap());
            assert!(!dict.contains(8i32).unwrap());
        });
    }

    #[test]
    fn test_get_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(32, dict.get_item(7i32).unwrap().extract::<i32>().unwrap());
            assert!(dict.get_item(8i32).is_none());
        });
    }

    #[test]
    fn test_set_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert!(dict.set_item(7i32, 42i32).is_ok()); // change
            assert!(dict.set_item(8i32, 123i32).is_ok()); // insert
            assert_eq!(
                42i32,
                dict.get_item(7i32).unwrap().extract::<i32>().unwrap()
            );
            assert_eq!(
                123i32,
                dict.get_item(8i32).unwrap().extract::<i32>().unwrap()
            );
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
                let _dict = [(10, none)].into_py_dict(py);
            }
            {
                assert_eq!(cnt, py.None().get_refcnt(py));
            }
        });
    }

    #[test]
    fn test_set_item_does_not_update_original_object() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert!(dict.set_item(7i32, 42i32).is_ok()); // change
            assert!(dict.set_item(8i32, 123i32).is_ok()); // insert
            assert_eq!(32i32, v[&7i32]); // not updated!
            assert_eq!(None, v.get(&8i32));
        });
    }

    #[test]
    fn test_del_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert!(dict.del_item(7i32).is_ok());
            assert_eq!(0, dict.len());
            assert!(dict.get_item(7i32).is_none());
        });
    }

    #[test]
    fn test_del_item_does_not_update_original_object() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert!(dict.del_item(7i32).is_ok()); // change
            assert_eq!(32i32, *v.get(&7i32).unwrap()); // not updated!
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
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            let mut value_sum = 0;
            for el in dict.items().iter() {
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
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            for el in dict.keys().iter() {
                key_sum += el.extract::<i32>().unwrap();
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
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut values_sum = 0;
            for el in dict.values().iter() {
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
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            let mut key_sum = 0;
            let mut value_sum = 0;
            for (key, value) in dict.iter() {
                key_sum += key.extract::<i32>().unwrap();
                value_sum += value.extract::<i32>().unwrap();
            }
            assert_eq!(7 + 8 + 9, key_sum);
            assert_eq!(32 + 42 + 123, value_sum);
        });
    }

    #[test]
    fn test_iter_size_hint() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();

            let mut iter = dict.iter();
            assert_eq!(iter.size_hint(), (v.len(), Some(v.len())));
            iter.next();
            assert_eq!(iter.size_hint(), (v.len() - 1, Some(v.len() - 1)));

            // Exhust iterator.
            for _ in &mut iter {}

            assert_eq!(iter.size_hint(), (0, Some(0)));
        });
    }

    #[test]
    fn test_into_iter() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);
            let ob = v.to_object(py);
            let dict = <PyDict as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            let mut key_sum = 0;
            let mut value_sum = 0;
            for (key, value) in dict {
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

            let m = map.to_object(py);
            let py_map = <PyDict as PyTryFrom>::try_from(m.as_ref(py)).unwrap();

            assert!(py_map.len() == 1);
            assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
            assert_eq!(map, py_map.extract().unwrap());
        });
    }

    #[test]
    fn test_btreemap_to_python() {
        Python::with_gil(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let m = map.to_object(py);
            let py_map = <PyDict as PyTryFrom>::try_from(m.as_ref(py)).unwrap();

            assert!(py_map.len() == 1);
            assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
            assert_eq!(map, py_map.extract().unwrap());
        });
    }

    #[test]
    fn test_hashmap_into_python() {
        Python::with_gil(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let m: PyObject = map.into_py(py);
            let py_map = <PyDict as PyTryFrom>::try_from(m.as_ref(py)).unwrap();

            assert!(py_map.len() == 1);
            assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
        });
    }

    #[test]
    fn test_hashmap_into_dict() {
        Python::with_gil(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict(py);

            assert_eq!(py_map.len(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_btreemap_into_py() {
        Python::with_gil(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let m: PyObject = map.into_py(py);
            let py_map = <PyDict as PyTryFrom>::try_from(m.as_ref(py)).unwrap();

            assert!(py_map.len() == 1);
            assert!(py_map.get_item(1).unwrap().extract::<i32>().unwrap() == 1);
        });
    }

    #[test]
    fn test_btreemap_into_dict() {
        Python::with_gil(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict(py);

            assert_eq!(py_map.len(), 1);
            assert_eq!(py_map.get_item(1).unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_vec_into_dict() {
        Python::with_gil(|py| {
            let vec = vec![("a", 1), ("b", 2), ("c", 3)];
            let py_map = vec.into_py_dict(py);

            assert_eq!(py_map.len(), 3);
            assert_eq!(py_map.get_item("b").unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn test_slice_into_dict() {
        Python::with_gil(|py| {
            let arr = [("a", 1), ("b", 2), ("c", 3)];
            let py_map = arr.into_py_dict(py);

            assert_eq!(py_map.len(), 3);
            assert_eq!(py_map.get_item("b").unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn dict_as_mapping() {
        Python::with_gil(|py| {
            let mut map = HashMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict(py);

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
}
