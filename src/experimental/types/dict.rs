// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::err::{self, PyErr, PyResult};
use crate::experimental::types::PyAny;
use crate::ffi::Py_ssize_t;
use crate::types::{PyList, PyMapping};
use crate::{ffi, AsPyPointer, Python, ToPyObject};

/// Represents a Python `dict`.
#[repr(transparent)]
#[derive(Clone)]
pub struct PyDict<'py>(PyAny<'py>);

pyobject_native_type_experimental!(
    impl<'py> PyDict<'py>,
    ffi::PyDictObject,
    ffi::PyDict_Type,
    #checkfunction=ffi::PyDict_Check
);

/// Represents a Python `dict_keys`.
#[cfg(not(PyPy))]
#[repr(transparent)]
#[derive(Clone)]
pub struct PyDictKeys<'py>(PyAny<'py>);

#[cfg(not(PyPy))]
pyobject_native_type_core_experimental!(
    impl<'py> PyDictKeys<'py>,
    ffi::PyDictKeys_Type,
    #checkfunction=ffi::PyDictKeys_Check
);

/// Represents a Python `dict_values`.
#[cfg(not(PyPy))]
#[repr(transparent)]
#[derive(Clone)]
pub struct PyDictValues<'py>(PyAny<'py>);

#[cfg(not(PyPy))]
pyobject_native_type_core_experimental!(
    impl<'py> PyDictValues<'py>,
    ffi::PyDictValues_Type,
    #checkfunction=ffi::PyDictValues_Check
);

/// Represents a Python `dict_items`.
#[cfg(not(PyPy))]
#[repr(transparent)]
#[derive(Clone)]
pub struct PyDictItems<'py>(PyAny<'py>);

#[cfg(not(PyPy))]
pyobject_native_type_core_experimental!(
    impl<'py> PyDictItems<'py>,
    ffi::PyDictItems_Type,
    #checkfunction=ffi::PyDictItems_Check
);

impl<'py> PyDict<'py> {
    /// Creates a new empty dictionary.
    pub fn new(py: Python<'py>) -> Self {
        unsafe { Self(PyAny::from_owned_ptr_or_panic(py, ffi::PyDict_New())) }
    }

    /// Creates a new dictionary from the sequence given.
    ///
    /// The sequence must consist of `(PyObject, PyObject)`. This is
    /// equivalent to `dict([("a", 1), ("b", 2)])`.
    ///
    /// Returns an error on invalid input. In the case of key collisions,
    /// this keeps the last entry seen.
    #[cfg(not(PyPy))]
    pub fn from_sequence(seq: &PyAny<'py>) -> PyResult<Self> {
        let py = seq.py();
        let dict = Self::new(py);
        unsafe {
            err::error_on_minusone(
                py,
                ffi::PyDict_MergeFromSeq2(dict.as_ptr(), seq.as_ptr(), 1),
            )?;
        }
        Ok(dict)
    }

    /// Returns a new dictionary that contains the same key-value pairs as self.
    ///
    /// This is equivalent to the Python expression `self.copy()`.
    pub fn copy(&self) -> PyResult<Self> {
        unsafe { Self::from_owned_ptr_or_err(self.py(), ffi::PyDict_Copy(self.as_ptr())) }
    }

    /// Empties an existing dictionary of all key-value pairs.
    pub fn clear(&self) {
        unsafe { ffi::PyDict_Clear(self.as_ptr()) }
    }

    /// Return the number of items in the dictionary.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    pub fn len(&self) -> usize {
        self._len() as usize
    }

    fn _len(&self) -> Py_ssize_t {
        #[cfg(any(not(Py_3_8), PyPy, Py_LIMITED_API))]
        unsafe {
            ffi::PyDict_Size(self.as_ptr())
        }

        #[cfg(all(Py_3_8, not(PyPy), not(Py_LIMITED_API)))]
        unsafe {
            (*self.as_ptr().cast::<ffi::PyDictObject>()).ma_used
        }
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
    pub fn get_item<K>(&self, key: K) -> Option<PyAny<'py>>
    where
        K: ToPyObject,
    {
        let py = self.py();
        unsafe {
            let ptr = ffi::PyDict_GetItem(self.as_ptr(), key.to_object(py).as_ptr());
            PyAny::from_borrowed_ptr_or_opt(py, ptr)
        }
    }

    /// Gets an item from the dictionary,
    ///
    /// returns `Ok(None)` if item is not present, or `Err(PyErr)` if an error occurs.
    ///
    /// To get a `KeyError` for non-existing keys, use `PyAny::get_item_with_error`.
    #[cfg(not(PyPy))]
    pub fn get_item_with_error<K>(&self, key: K) -> PyResult<Option<PyAny<'py>>>
    where
        K: ToPyObject,
    {
        let py = self.py();
        let ptr =
            unsafe { ffi::PyDict_GetItemWithError(self.as_ptr(), key.to_object(py).as_ptr()) };
        if let Some(err) = PyErr::take(py) {
            return Err(err);
        }
        Ok(unsafe { PyAny::from_borrowed_ptr_or_opt(py, ptr) })
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
        dbg!(self, key.to_object(py));
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
    pub fn keys(&'py self) -> &'py PyList {
        unsafe {
            self.py()
                .from_owned_ptr::<PyList>(ffi::PyDict_Keys(self.as_ptr()))
        }
    }

    /// Returns a list of dict values.
    ///
    /// This is equivalent to the Python expression `list(dict.values())`.
    pub fn values(&'py self) -> &'py PyList {
        unsafe {
            self.py()
                .from_owned_ptr::<PyList>(ffi::PyDict_Values(self.as_ptr()))
        }
    }

    /// Returns a list of dict items.
    ///
    /// This is equivalent to the Python expression `list(dict.items())`.
    pub fn items(&'py self) -> &'py PyList {
        unsafe {
            self.py()
                .from_owned_ptr::<PyList>(ffi::PyDict_Items(self.as_ptr()))
        }
    }

    /// Returns an iterator of `(key, value)` pairs in this dictionary.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the dictionary is mutated during iteration, it will panic.
    /// It is allowed to modify values as you iterate over the dictionary, but only
    /// so long as the set of keys does not change.
    pub fn iter(&self) -> PyDictIterator<'py> {
        IntoIterator::into_iter(self.clone())
    }

    /// Returns `self` cast as a `PyMapping`.
    pub fn as_mapping(&'py self) -> &'py PyMapping {
        unsafe { self.as_gil_ref().downcast_unchecked() }
    }

    /// Update this dictionary with the key/value pairs from another.
    ///
    /// This is equivalent to the Python expression `self.update(other)`. If `other` is a `PyDict`, you may want
    /// to use `self.update(other.as_mapping())`, note: `PyDict::as_mapping` is a zero-cost conversion.
    pub fn update(&self, other: &PyMapping) -> PyResult<()> {
        let py = self.py();
        unsafe { err::error_on_minusone(py, ffi::PyDict_Update(self.as_ptr(), other.as_ptr())) }
    }

    /// Add key/value pairs from another dictionary to this one only when they do not exist in this.
    ///
    /// This is equivalent to the Python expression `self.update({k: v for k, v in other.items() if k not in self})`.
    /// If `other` is a `PyDict`, you may want to use `self.update_if_missing(other.as_mapping())`,
    /// note: `PyDict::as_mapping` is a zero-cost conversion.
    ///
    /// This method uses [`PyDict_Merge`](https://docs.python.org/3/c-api/dict.html#c.PyDict_Merge) internally,
    /// so should have the same performance as `update`.
    pub fn update_if_missing(&self, other: &PyMapping) -> PyResult<()> {
        let py = self.py();
        unsafe { err::error_on_minusone(py, ffi::PyDict_Merge(self.as_ptr(), other.as_ptr(), 0)) }
    }

    /// Create a new `PyDict` from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be an owned non-null pointer to a PyDict object.
    #[inline]
    pub(crate) unsafe fn from_owned_ptr_or_err(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        PyAny::from_owned_ptr_or_err(py, ptr).map(Self)
    }

    /// Create a new `PyDict` from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be an owned non-null pointer to a PyDict object.
    #[inline]
    pub(crate) unsafe fn from_owned_ptr_or_panic(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        Self(PyAny::from_owned_ptr_or_panic(py, ptr))
    }
}

/// PyO3 implementation of an iterator for a Python `dict` object.
pub struct PyDictIterator<'py> {
    dict: PyDict<'py>,
    ppos: ffi::Py_ssize_t,
    di_used: ffi::Py_ssize_t,
    len: ffi::Py_ssize_t,
}

impl<'py> Iterator for PyDictIterator<'py> {
    type Item = (PyAny<'py>, PyAny<'py>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ma_used = self.dict._len();

        // These checks are similar to what CPython does.
        //
        // If the dimension of the dict changes e.g. key-value pairs are removed
        // or added during iteration, this will panic next time when `next` is called
        if self.di_used != ma_used {
            self.di_used = -1;
            panic!("dictionary changed size during iteration");
        };

        // If the dict is changed in such a way that the length remains constant
        // then this will panic at the end of iteration - similar to this:
        //
        // d = {"a":1, "b":2, "c": 3}
        //
        // for k, v in d.items():
        //     d[f"{k}_"] = 4
        //     del d[k]
        //     print(k)
        //
        if self.len == -1 {
            self.di_used = -1;
            panic!("dictionary keys changed during iteration");
        };

        let ret = unsafe { self.next_unchecked() };
        if ret.is_some() {
            self.len -= 1
        }
        ret
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'py> ExactSizeIterator for PyDictIterator<'py> {
    fn len(&self) -> usize {
        self.len as usize
    }
}

impl<'py> std::iter::IntoIterator for PyDict<'py> {
    type Item = (PyAny<'py>, PyAny<'py>);
    type IntoIter = PyDictIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self._len();
        PyDictIterator {
            dict: self,
            ppos: 0,
            di_used: len,
            len,
        }
    }
}

impl<'py> std::iter::IntoIterator for &PyDict<'py> {
    type Item = (PyAny<'py>, PyAny<'py>);
    type IntoIter = PyDictIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'py> PyDictIterator<'py> {
    /// Advances the iterator without checking for concurrent modification.
    ///
    /// See [`PyDict_Next`](https://docs.python.org/3/c-api/dict.html#c.PyDict_Next)
    /// for more information.
    unsafe fn next_unchecked(&mut self) -> Option<(PyAny<'py>, PyAny<'py>)> {
        let mut key: *mut ffi::PyObject = std::ptr::null_mut();
        let mut value: *mut ffi::PyObject = std::ptr::null_mut();

        if ffi::PyDict_Next(self.dict.as_ptr(), &mut self.ppos, &mut key, &mut value) != 0 {
            let py = self.dict.py();
            Some((
                PyAny::from_borrowed_ptr_unchecked(py, key),
                PyAny::from_borrowed_ptr_unchecked(py, value),
            ))
        } else {
            None
        }
    }
}

/// Conversion trait that allows a sequence of tuples to be converted into `PyDict`
/// Primary use case for this trait is `call` and `call_method` methods as keywords argument.
pub trait IntoPyDict {
    /// Converts self into a `PyDict` object pointer. Whether pointer owned or borrowed
    /// depends on implementation.
    fn into_py_dict<'py>(self, py: Python<'py>) -> PyDict<'py>;
}

impl<T, I> IntoPyDict for I
where
    T: PyDictItem,
    I: IntoIterator<Item = T>,
{
    fn into_py_dict<'py>(self, py: Python<'py>) -> PyDict<'py> {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(PyPy))]
    use crate::exceptions;
    #[cfg(not(PyPy))]
    use crate::types::PyList;
    use crate::{types::PyTuple, Python, ToPyObject};
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
            let dict = PyDict::from_sequence(PyAny::from_gil_ref(&items.as_ref())).unwrap();
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
            assert!(PyDict::from_sequence(PyAny::from_gil_ref(&items.as_ref())).is_err());
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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
            assert_eq!(0, dict.len());
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict2: &PyDict<'_> = ob.downcast2(py).unwrap();
            assert_eq!(1, dict2.len());
        });
    }

    #[test]
    fn test_contains() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
            assert_eq!(32, dict.get_item(7i32).unwrap().extract::<i32>().unwrap());
            assert!(dict.get_item(8i32).is_none());
        });
    }

    #[test]
    #[cfg(not(PyPy))]
    fn test_get_item_with_error() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
            assert_eq!(
                32,
                dict.get_item_with_error(7i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(dict.get_item_with_error(8i32).unwrap().is_none());
            assert!(dict
                .get_item_with_error(dict)
                .unwrap_err()
                .is_instance_of::<exceptions::PyTypeError>(py));
        });
    }

    #[test]
    fn test_set_item() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
            dbg!(ob.get_refcnt(py), dict.get_refcnt());
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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            let mut value_sum = 0;
            for el in dict.items().iter() {
                let tuple = el.downcast::<PyTuple>().unwrap();
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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
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
    fn test_iter_value_mutated() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);

            let ob = v.to_object(py);
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();

            for (key, value) in dict.iter() {
                dict.set_item(key, value.extract::<i32>().unwrap() + 7)
                    .unwrap();
            }
        });
    }

    #[test]
    #[should_panic]
    fn test_iter_key_mutated() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            for i in 0..10 {
                v.insert(i * 2, i * 2);
            }
            let ob = v.to_object(py);
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();

            for (i, (key, value)) in dict.iter().enumerate() {
                let key = key.extract::<i32>().unwrap();
                let value = value.extract::<i32>().unwrap();

                dict.set_item(key + 1, value + 1).unwrap();

                if i > 1000 {
                    // avoid this test just running out of memory if it fails
                    break;
                };
            }
        });
    }

    #[test]
    #[should_panic]
    fn test_iter_key_mutated_constant_len() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            for i in 0..10 {
                v.insert(i * 2, i * 2);
            }
            let ob = v.to_object(py);
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();

            for (i, (key, value)) in dict.iter().enumerate() {
                let key = key.extract::<i32>().unwrap();
                let value = value.extract::<i32>().unwrap();
                dict.del_item(key).unwrap();
                dict.set_item(key + 1, value + 1).unwrap();

                if i > 1000 {
                    // avoid this test just running out of memory if it fails
                    break;
                };
            }
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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();

            let mut iter = dict.iter();
            assert_eq!(iter.size_hint(), (v.len(), Some(v.len())));
            iter.next();
            assert_eq!(iter.size_hint(), (v.len() - 1, Some(v.len() - 1)));

            // Exhaust iterator.
            for _ in &mut iter {}

            assert_eq!(iter.size_hint(), (0, Some(0)));

            assert!(iter.next().is_none());

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
            let dict: &PyDict<'_> = ob.downcast2(py).unwrap();
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

    #[cfg(not(PyPy))]
    fn abc_dict(py: Python<'_>) -> PyDict<'_> {
        let mut map = HashMap::<&'static str, i32>::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);
        map.into_py_dict(py)
    }

    #[test]
    #[cfg(not(PyPy))]
    fn dict_keys_view() {
        Python::with_gil(|py| {
            let dict = abc_dict(py);
            let keys = dict.call_method0("keys").unwrap();
            assert!(keys
                .is_instance(PyAny::from_gil_ref(
                    &py.get_type::<PyDictKeys<'_>>().as_ref()
                ))
                .unwrap());
        })
    }

    #[test]
    #[cfg(not(PyPy))]
    fn dict_values_view() {
        Python::with_gil(|py| {
            let dict = abc_dict(py);
            let values = dict.call_method0("values").unwrap();
            assert!(values
                .is_instance(PyAny::from_gil_ref(
                    &py.get_type::<PyDictValues<'_>>().as_ref()
                ))
                .unwrap());
        })
    }

    #[test]
    #[cfg(not(PyPy))]
    fn dict_items_view() {
        Python::with_gil(|py| {
            let dict = abc_dict(py);
            let items = dict.call_method0("items").unwrap();
            assert!(items
                .is_instance(PyAny::from_gil_ref(
                    &py.get_type::<PyDictItems<'_>>().as_ref()
                ))
                .unwrap());
        })
    }

    #[test]
    fn dict_update() {
        Python::with_gil(|py| {
            let dict = [("a", 1), ("b", 2), ("c", 3)].into_py_dict(py);
            let other = [("b", 4), ("c", 5), ("d", 6)].into_py_dict(py);
            dict.update(other.as_mapping()).unwrap();
            assert_eq!(dict.len(), 4);
            assert_eq!(dict.get_item("a").unwrap().extract::<i32>().unwrap(), 1);
            assert_eq!(dict.get_item("b").unwrap().extract::<i32>().unwrap(), 4);
            assert_eq!(dict.get_item("c").unwrap().extract::<i32>().unwrap(), 5);
            assert_eq!(dict.get_item("d").unwrap().extract::<i32>().unwrap(), 6);

            assert_eq!(other.len(), 3);
            assert_eq!(other.get_item("b").unwrap().extract::<i32>().unwrap(), 4);
            assert_eq!(other.get_item("c").unwrap().extract::<i32>().unwrap(), 5);
            assert_eq!(other.get_item("d").unwrap().extract::<i32>().unwrap(), 6);
        })
    }

    #[test]
    fn dict_update_if_missing() {
        Python::with_gil(|py| {
            let dict = [("a", 1), ("b", 2), ("c", 3)].into_py_dict(py);
            let other = [("b", 4), ("c", 5), ("d", 6)].into_py_dict(py);
            dict.update_if_missing(other.as_mapping()).unwrap();
            assert_eq!(dict.len(), 4);
            assert_eq!(dict.get_item("a").unwrap().extract::<i32>().unwrap(), 1);
            assert_eq!(dict.get_item("b").unwrap().extract::<i32>().unwrap(), 2);
            assert_eq!(dict.get_item("c").unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(dict.get_item("d").unwrap().extract::<i32>().unwrap(), 6);

            assert_eq!(other.len(), 3);
            assert_eq!(other.get_item("b").unwrap().extract::<i32>().unwrap(), 4);
            assert_eq!(other.get_item("c").unwrap().extract::<i32>().unwrap(), 5);
            assert_eq!(other.get_item("d").unwrap().extract::<i32>().unwrap(), 6);
        })
    }
}
