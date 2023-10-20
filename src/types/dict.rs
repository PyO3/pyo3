use super::PyMapping;
use crate::err::{self, PyErr, PyResult};
use crate::ffi::Py_ssize_t;
use crate::instance::Py2;
use crate::types::any::PyAnyMethods;
use crate::types::{PyAny, PyList};
use crate::{ffi, Python, ToPyObject};

/// Represents a Python `dict`.
#[repr(transparent)]
pub struct PyDict(PyAny);

pyobject_native_type!(
    PyDict,
    ffi::PyDictObject,
    pyobject_native_static_type_object!(ffi::PyDict_Type),
    #checkfunction=ffi::PyDict_Check
);

/// Represents a Python `dict_keys`.
#[cfg(not(PyPy))]
#[repr(transparent)]
pub struct PyDictKeys(PyAny);

#[cfg(not(PyPy))]
pyobject_native_type_core!(
    PyDictKeys,
    pyobject_native_static_type_object!(ffi::PyDictKeys_Type),
    #checkfunction=ffi::PyDictKeys_Check
);

/// Represents a Python `dict_values`.
#[cfg(not(PyPy))]
#[repr(transparent)]
pub struct PyDictValues(PyAny);

#[cfg(not(PyPy))]
pyobject_native_type_core!(
    PyDictValues,
    pyobject_native_static_type_object!(ffi::PyDictValues_Type),
    #checkfunction=ffi::PyDictValues_Check
);

/// Represents a Python `dict_items`.
#[cfg(not(PyPy))]
#[repr(transparent)]
pub struct PyDictItems(PyAny);

#[cfg(not(PyPy))]
pyobject_native_type_core!(
    PyDictItems,
    pyobject_native_static_type_object!(ffi::PyDictItems_Type),
    #checkfunction=ffi::PyDictItems_Check
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
    pub fn from_sequence(seq: &PyAny) -> PyResult<&PyDict> {
        let py = seq.py();
        let dict = Self::new(py);
        err::error_on_minusone(py, unsafe {
            ffi::PyDict_MergeFromSeq2(dict.into_ptr(), seq.into_ptr(), 1)
        })?;
        Ok(dict)
    }

    /// Returns a new dictionary that contains the same key-value pairs as self.
    ///
    /// This is equivalent to the Python expression `self.copy()`.
    pub fn copy(&self) -> PyResult<&PyDict> {
        Py2::borrowed_from_gil_ref(&self)
            .copy()
            .map(Py2::into_gil_ref)
    }

    /// Empties an existing dictionary of all key-value pairs.
    pub fn clear(&self) {
        Py2::borrowed_from_gil_ref(&self).clear()
    }

    /// Return the number of items in the dictionary.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    pub fn len(&self) -> usize {
        Py2::<PyDict>::borrowed_from_gil_ref(&self).len()
    }

    /// Checks if the dict is empty, i.e. `len(self) == 0`.
    pub fn is_empty(&self) -> bool {
        Py2::<PyDict>::borrowed_from_gil_ref(&self).is_empty()
    }

    /// Determines if the dictionary contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        Py2::<PyDict>::borrowed_from_gil_ref(&self).contains(key)
    }

    /// Gets an item from the dictionary.
    ///
    /// Returns `Ok(None)` if the item is not present. To get a `KeyError` for
    /// non-existing keys, use [`PyAny::get_item`].
    ///
    /// Returns `Err(PyErr)` if Python magic methods `__hash__` or `__eq__` used in dictionary
    /// lookup raise an exception, for example if the key `K` is not hashable. Usually it is
    /// best to bubble this error up to the caller using the `?` operator.
    ///
    /// # Examples
    ///
    /// The following example calls `get_item` for the dictionary `{"a": 1}` with various
    /// keys.
    /// - `get_item("a")` returns `Ok(Some(...))`, with the `PyAny` being a reference to the Python
    ///   int `1`.
    /// - `get_item("b")` returns `Ok(None)`, because "b" is not in the dictionary.
    /// - `get_item(dict)` returns an `Err(PyErr)`. The error will be a `TypeError` because a dict is not
    ///   hashable.
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyDict, IntoPyDict};
    /// use pyo3::exceptions::{PyTypeError, PyKeyError};
    ///
    /// # fn main() {
    /// # let _ =
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let dict: &PyDict = [("a", 1)].into_py_dict(py);
    ///     // `a` is in the dictionary, with value 1
    ///     assert!(dict.get_item("a")?.map_or(Ok(false), |x| x.eq(1))?);
    ///     // `b` is not in the dictionary
    ///     assert!(dict.get_item("b")?.is_none());
    ///     // `dict` is not hashable, so this returns an error
    ///     assert!(dict.get_item(dict).unwrap_err().is_instance_of::<PyTypeError>(py));
    ///
    ///     // `PyAny::get_item("b")` will raise a `KeyError` instead of returning `None`
    ///     let any: &PyAny = dict.as_ref();
    ///     assert!(any.get_item("b").unwrap_err().is_instance_of::<PyKeyError>(py));
    ///     Ok(())
    /// });
    /// # }
    /// ```
    pub fn get_item<K>(&self, key: K) -> PyResult<Option<&PyAny>>
    where
        K: ToPyObject,
    {
        match Py2::<PyDict>::borrowed_from_gil_ref(&self).get_item(key) {
            Ok(Some(item)) => Ok(Some(item.into_gil_ref())),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Deprecated version of `get_item`.
    #[deprecated(
        since = "0.20.0",
        note = "this is now equivalent to `PyDict::get_item`"
    )]
    #[inline]
    pub fn get_item_with_error<K>(&self, key: K) -> PyResult<Option<&PyAny>>
    where
        K: ToPyObject,
    {
        self.get_item(key)
    }

    /// Sets an item value.
    ///
    /// This is equivalent to the Python statement `self[key] = value`.
    pub fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject,
    {
        Py2::<PyDict>::borrowed_from_gil_ref(&self).set_item(key, value)
    }

    /// Deletes an item.
    ///
    /// This is equivalent to the Python statement `del self[key]`.
    pub fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        Py2::<PyDict>::borrowed_from_gil_ref(&self).del_item(key)
    }

    /// Returns a list of dict keys.
    ///
    /// This is equivalent to the Python expression `list(dict.keys())`.
    pub fn keys(&self) -> &PyList {
        Py2::borrowed_from_gil_ref(&self).keys().into_gil_ref()
    }

    /// Returns a list of dict values.
    ///
    /// This is equivalent to the Python expression `list(dict.values())`.
    pub fn values(&self) -> &PyList {
        Py2::borrowed_from_gil_ref(&self).values().into_gil_ref()
    }

    /// Returns a list of dict items.
    ///
    /// This is equivalent to the Python expression `list(dict.items())`.
    pub fn items(&self) -> &PyList {
        Py2::borrowed_from_gil_ref(&self).items().into_gil_ref()
    }

    /// Returns an iterator of `(key, value)` pairs in this dictionary.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the dictionary is mutated during iteration, it will panic.
    /// It is allowed to modify values as you iterate over the dictionary, but only
    /// so long as the set of keys does not change.
    pub fn iter(&self) -> PyDictIterator<'_> {
        PyDictIterator(Py2::<PyDict>::borrowed_from_gil_ref(&self).iter())
    }

    /// Returns `self` cast as a `PyMapping`.
    pub fn as_mapping(&self) -> &PyMapping {
        unsafe { self.downcast_unchecked() }
    }

    /// Update this dictionary with the key/value pairs from another.
    ///
    /// This is equivalent to the Python expression `self.update(other)`. If `other` is a `PyDict`, you may want
    /// to use `self.update(other.as_mapping())`, note: `PyDict::as_mapping` is a zero-cost conversion.
    pub fn update(&self, other: &PyMapping) -> PyResult<()> {
        Py2::borrowed_from_gil_ref(&self)
            // this dance is necessary because PyMapping doesn't implement PyTypeInfo so
            // no borrowed_from_gil_ref for Py2<'py, PyMapping>
            .update(unsafe { Py2::borrowed_from_gil_ref(&other.as_ref()).downcast_unchecked() })
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
        Py2::borrowed_from_gil_ref(&self)
            // this dance is necessary because PyMapping doesn't implement PyTypeInfo so
            // no borrowed_from_gil_ref for Py2<'py, PyMapping>
            .update_if_missing(unsafe {
                Py2::borrowed_from_gil_ref(&other.as_ref()).downcast_unchecked()
            })
    }
}

/// Implementation of functionality for [`PyDict`].
///
/// These methods are defined for the `Py2<'py, PyDict>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyDict")]
pub(crate) trait PyDictMethods<'py> {
    /// Returns a new dictionary that contains the same key-value pairs as self.
    ///
    /// This is equivalent to the Python expression `self.copy()`.
    fn copy(&self) -> PyResult<Py2<'py, PyDict>>;

    /// Empties an existing dictionary of all key-value pairs.
    fn clear(&self);

    /// Return the number of items in the dictionary.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    fn len(&self) -> usize;

    /// Checks if the dict is empty, i.e. `len(self) == 0`.
    fn is_empty(&self) -> bool;

    /// Determines if the dictionary contains the specified key.
    ///
    /// This is equivalent to the Python expression `key in self`.
    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject;

    /// Gets an item from the dictionary.
    ///
    /// Returns `None` if the item is not present, or if an error occurs.
    ///
    /// To get a `KeyError` for non-existing keys, use `PyAny::get_item`.
    fn get_item<K>(&self, key: K) -> PyResult<Option<Py2<'py, PyAny>>>
    where
        K: ToPyObject;

    /// Sets an item value.
    ///
    /// This is equivalent to the Python statement `self[key] = value`.
    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject;

    /// Deletes an item.
    ///
    /// This is equivalent to the Python statement `del self[key]`.
    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject;

    /// Returns a list of dict keys.
    ///
    /// This is equivalent to the Python expression `list(dict.keys())`.
    fn keys(&self) -> Py2<'py, PyList>;

    /// Returns a list of dict values.
    ///
    /// This is equivalent to the Python expression `list(dict.values())`.
    fn values(&self) -> Py2<'py, PyList>;

    /// Returns a list of dict items.
    ///
    /// This is equivalent to the Python expression `list(dict.items())`.
    fn items(&self) -> Py2<'py, PyList>;

    /// Returns an iterator of `(key, value)` pairs in this dictionary.
    ///
    /// # Panics
    ///
    /// If PyO3 detects that the dictionary is mutated during iteration, it will panic.
    /// It is allowed to modify values as you iterate over the dictionary, but only
    /// so long as the set of keys does not change.
    fn iter(&self) -> PyDictIterator2<'py>;

    /// Returns `self` cast as a `PyMapping`.
    fn as_mapping(&self) -> &Py2<'py, PyMapping>;

    /// Update this dictionary with the key/value pairs from another.
    ///
    /// This is equivalent to the Python expression `self.update(other)`. If `other` is a `PyDict`, you may want
    /// to use `self.update(other.as_mapping())`, note: `PyDict::as_mapping` is a zero-cost conversion.
    fn update(&self, other: &Py2<'_, PyMapping>) -> PyResult<()>;

    /// Add key/value pairs from another dictionary to this one only when they do not exist in this.
    ///
    /// This is equivalent to the Python expression `self.update({k: v for k, v in other.items() if k not in self})`.
    /// If `other` is a `PyDict`, you may want to use `self.update_if_missing(other.as_mapping())`,
    /// note: `PyDict::as_mapping` is a zero-cost conversion.
    ///
    /// This method uses [`PyDict_Merge`](https://docs.python.org/3/c-api/dict.html#c.PyDict_Merge) internally,
    /// so should have the same performance as `update`.
    fn update_if_missing(&self, other: &Py2<'_, PyMapping>) -> PyResult<()>;
}

impl<'py> PyDictMethods<'py> for Py2<'py, PyDict> {
    fn copy(&self) -> PyResult<Py2<'py, PyDict>> {
        unsafe {
            Py2::from_owned_ptr_or_err(self.py(), ffi::PyDict_Copy(self.as_ptr()))
                .map(|copy| copy.downcast_into_unchecked())
        }
    }

    fn clear(&self) {
        unsafe { ffi::PyDict_Clear(self.as_ptr()) }
    }

    fn len(&self) -> usize {
        dict_len(self) as usize
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        fn inner(dict: &Py2<'_, PyDict>, key: Py2<'_, PyAny>) -> PyResult<bool> {
            match unsafe { ffi::PyDict_Contains(dict.as_ptr(), key.as_ptr()) } {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(dict.py())),
            }
        }

        let py = self.py();
        inner(self, key.to_object(py).attach_into(py))
    }

    fn get_item<K>(&self, key: K) -> PyResult<Option<Py2<'py, PyAny>>>
    where
        K: ToPyObject,
    {
        fn inner<'py>(
            dict: &Py2<'py, PyDict>,
            key: Py2<'_, PyAny>,
        ) -> PyResult<Option<Py2<'py, PyAny>>> {
            let py = dict.py();
            // PyDict_GetItemWithError returns a borrowed ptr, must make it owned for safety (see #890).
            // Py2::from_borrowed_ptr_or_opt will take ownership in this way.
            match unsafe {
                Py2::from_borrowed_ptr_or_opt(
                    py,
                    ffi::PyDict_GetItemWithError(dict.as_ptr(), key.as_ptr()),
                )
            } {
                some @ Some(_) => Ok(some),
                None => PyErr::take(py).map(Err).transpose(),
            }
        }

        let py = self.py();
        inner(self, key.to_object(py).attach_into(py))
    }

    fn set_item<K, V>(&self, key: K, value: V) -> PyResult<()>
    where
        K: ToPyObject,
        V: ToPyObject,
    {
        fn inner(
            dict: &Py2<'_, PyDict>,
            key: Py2<'_, PyAny>,
            value: Py2<'_, PyAny>,
        ) -> PyResult<()> {
            err::error_on_minusone(dict.py(), unsafe {
                ffi::PyDict_SetItem(dict.as_ptr(), key.as_ptr(), value.as_ptr())
            })
        }

        let py = self.py();
        inner(
            self,
            key.to_object(py).attach_into(py),
            value.to_object(py).attach_into(py),
        )
    }

    fn del_item<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        fn inner(dict: &Py2<'_, PyDict>, key: Py2<'_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(dict.py(), unsafe {
                ffi::PyDict_DelItem(dict.as_ptr(), key.as_ptr())
            })
        }

        let py = self.py();
        inner(self, key.to_object(py).attach_into(py))
    }

    fn keys(&self) -> Py2<'py, PyList> {
        unsafe {
            Py2::from_owned_ptr(self.py(), ffi::PyDict_Keys(self.as_ptr()))
                .downcast_into_unchecked()
        }
    }

    fn values(&self) -> Py2<'py, PyList> {
        unsafe {
            Py2::from_owned_ptr(self.py(), ffi::PyDict_Values(self.as_ptr()))
                .downcast_into_unchecked()
        }
    }

    fn items(&self) -> Py2<'py, PyList> {
        unsafe {
            Py2::from_owned_ptr(self.py(), ffi::PyDict_Items(self.as_ptr()))
                .downcast_into_unchecked()
        }
    }

    fn iter(&self) -> PyDictIterator2<'py> {
        PyDictIterator2::new(self.clone())
    }

    fn as_mapping(&self) -> &Py2<'py, PyMapping> {
        unsafe { self.downcast_unchecked() }
    }

    fn update(&self, other: &Py2<'_, PyMapping>) -> PyResult<()> {
        err::error_on_minusone(self.py(), unsafe {
            ffi::PyDict_Update(self.as_ptr(), other.as_ptr())
        })
    }

    fn update_if_missing(&self, other: &Py2<'_, PyMapping>) -> PyResult<()> {
        err::error_on_minusone(self.py(), unsafe {
            ffi::PyDict_Merge(self.as_ptr(), other.as_ptr(), 0)
        })
    }
}

fn dict_len(dict: &Py2<'_, PyDict>) -> Py_ssize_t {
    #[cfg(any(not(Py_3_8), PyPy, Py_LIMITED_API))]
    unsafe {
        ffi::PyDict_Size(dict.as_ptr())
    }

    #[cfg(all(Py_3_8, not(PyPy), not(Py_LIMITED_API)))]
    unsafe {
        (*dict.as_ptr().cast::<ffi::PyDictObject>()).ma_used
    }
}

/// PyO3 implementation of an iterator for a Python `dict` object.
pub struct PyDictIterator<'py>(PyDictIterator2<'py>);

impl<'py> Iterator for PyDictIterator<'py> {
    type Item = (&'py PyAny, &'py PyAny);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self.0.next()?;
        Some((key.into_gil_ref(), value.into_gil_ref()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'py> ExactSizeIterator for PyDictIterator<'py> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> std::iter::IntoIterator for &'a PyDict {
    type Item = (&'a PyAny, &'a PyAny);
    type IntoIter = PyDictIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PyDictIterator(PyDictIterator2::new(
            Py2::borrowed_from_gil_ref(&self).clone(),
        ))
    }
}

/// PyO3 implementation of an iterator for a Python `dict` object.
// TODO before making this public, consider a better name.
pub(crate) struct PyDictIterator2<'py> {
    dict: Py2<'py, PyDict>,
    ppos: ffi::Py_ssize_t,
    di_used: ffi::Py_ssize_t,
    len: ffi::Py_ssize_t,
}

impl<'py> Iterator for PyDictIterator2<'py> {
    type Item = (Py2<'py, PyAny>, Py2<'py, PyAny>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ma_used = dict_len(&self.dict);

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

impl<'py> ExactSizeIterator for PyDictIterator2<'py> {
    fn len(&self) -> usize {
        self.len as usize
    }
}

impl<'py> PyDictIterator2<'py> {
    fn new(dict: Py2<'py, PyDict>) -> Self {
        let len = dict_len(&dict);
        PyDictIterator2 {
            dict,
            ppos: 0,
            di_used: len,
            len,
        }
    }

    /// Advances the iterator without checking for concurrent modification.
    ///
    /// See [`PyDict_Next`](https://docs.python.org/3/c-api/dict.html#c.PyDict_Next)
    /// for more information.
    unsafe fn next_unchecked(&mut self) -> Option<(Py2<'py, PyAny>, Py2<'py, PyAny>)> {
        let mut key: *mut ffi::PyObject = std::ptr::null_mut();
        let mut value: *mut ffi::PyObject = std::ptr::null_mut();

        if ffi::PyDict_Next(self.dict.as_ptr(), &mut self.ppos, &mut key, &mut value) != 0 {
            let py = self.dict.py();
            // PyDict_Next returns borrowed values; for safety must make them owned (see #890)
            Some((
                Py2::from_owned_ptr(py, ffi::_Py_NewRef(key)),
                Py2::from_owned_ptr(py, ffi::_Py_NewRef(value)),
            ))
        } else {
            None
        }
    }
}

impl<'py> std::iter::IntoIterator for &'_ Py2<'py, PyDict> {
    type Item = (Py2<'py, PyAny>, Py2<'py, PyAny>);
    type IntoIter = PyDictIterator2<'py>;

    fn into_iter(self) -> Self::IntoIter {
        PyDictIterator2::new(self.clone())
    }
}

impl<'py> std::iter::IntoIterator for Py2<'py, PyDict> {
    type Item = (Py2<'py, PyAny>, Py2<'py, PyAny>);
    type IntoIter = PyDictIterator2<'py>;

    fn into_iter(self) -> Self::IntoIter {
        PyDictIterator2::new(self)
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
            assert_eq!(
                32,
                dict.get_item(7i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(dict.get_item(8i32).unwrap().is_none());
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
            let dict = PyDict::from_sequence(items).unwrap();
            assert_eq!(
                1,
                dict.get_item("a")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert_eq!(
                2,
                dict.get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
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
            assert!(PyDict::from_sequence(items).is_err());
        });
    }

    #[test]
    fn test_copy() {
        Python::with_gil(|py| {
            let dict = [(7, 32)].into_py_dict(py);

            let ndict = dict.copy().unwrap();
            assert_eq!(
                32,
                ndict
                    .get_item(7i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(ndict.get_item(8i32).unwrap().is_none());
        });
    }

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            let ob = v.to_object(py);
            let dict: &PyDict = ob.downcast(py).unwrap();
            assert_eq!(0, dict.len());
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict2: &PyDict = ob.downcast(py).unwrap();
            assert_eq!(1, dict2.len());
        });
    }

    #[test]
    fn test_contains() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict: &PyDict = ob.downcast(py).unwrap();
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
            let dict: &PyDict = ob.downcast(py).unwrap();
            assert_eq!(
                32,
                dict.get_item(7i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert!(dict.get_item(8i32).unwrap().is_none());
        });
    }

    #[test]
    #[allow(deprecated)]
    #[cfg(not(PyPy))]
    fn test_get_item_with_error() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict: &PyDict = ob.downcast(py).unwrap();
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
            let dict: &PyDict = ob.downcast(py).unwrap();
            assert!(dict.set_item(7i32, 42i32).is_ok()); // change
            assert!(dict.set_item(8i32, 123i32).is_ok()); // insert
            assert_eq!(
                42i32,
                dict.get_item(7i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
            assert_eq!(
                123i32,
                dict.get_item(8i32)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap()
            );
        });
    }

    #[test]
    fn test_set_item_refcnt() {
        Python::with_gil(|py| {
            let cnt;
            let obj = py.eval("object()", None, None).unwrap();
            {
                let _pool = unsafe { crate::GILPool::new() };
                cnt = obj.get_refcnt();
                let _dict = [(10, obj)].into_py_dict(py);
            }
            {
                assert_eq!(cnt, obj.get_refcnt());
            }
        });
    }

    #[test]
    fn test_set_item_does_not_update_original_object() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict: &PyDict = ob.downcast(py).unwrap();
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
            let dict: &PyDict = ob.downcast(py).unwrap();
            assert!(dict.del_item(7i32).is_ok());
            assert_eq!(0, dict.len());
            assert!(dict.get_item(7i32).unwrap().is_none());
        });
    }

    #[test]
    fn test_del_item_does_not_update_original_object() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            let ob = v.to_object(py);
            let dict: &PyDict = ob.downcast(py).unwrap();
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
            let dict: &PyDict = ob.downcast(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            let mut value_sum = 0;
            for el in dict.items() {
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
            let dict: &PyDict = ob.downcast(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut key_sum = 0;
            for el in dict.keys() {
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
            let dict: &PyDict = ob.downcast(py).unwrap();
            // Can't just compare against a vector of tuples since we don't have a guaranteed ordering.
            let mut values_sum = 0;
            for el in dict.values() {
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
            let dict: &PyDict = ob.downcast(py).unwrap();
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
    fn test_iter_value_mutated() {
        Python::with_gil(|py| {
            let mut v = HashMap::new();
            v.insert(7, 32);
            v.insert(8, 42);
            v.insert(9, 123);

            let ob = v.to_object(py);
            let dict: &PyDict = ob.downcast(py).unwrap();

            for (key, value) in dict {
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
            let dict: &PyDict = ob.downcast(py).unwrap();

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
            let dict: &PyDict = ob.downcast(py).unwrap();

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
            let dict: &PyDict = ob.downcast(py).unwrap();

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
            let dict: &PyDict = ob.downcast(py).unwrap();
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
            assert_eq!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
        });
    }

    #[test]
    fn test_btreemap_into_dict() {
        Python::with_gil(|py| {
            let mut map = BTreeMap::<i32, i32>::new();
            map.insert(1, 1);

            let py_map = map.into_py_dict(py);

            assert_eq!(py_map.len(), 1);
            assert_eq!(
                py_map
                    .get_item(1)
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
        });
    }

    #[test]
    fn test_vec_into_dict() {
        Python::with_gil(|py| {
            let vec = vec![("a", 1), ("b", 2), ("c", 3)];
            let py_map = vec.into_py_dict(py);

            assert_eq!(py_map.len(), 3);
            assert_eq!(
                py_map
                    .get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                2
            );
        });
    }

    #[test]
    fn test_slice_into_dict() {
        Python::with_gil(|py| {
            let arr = [("a", 1), ("b", 2), ("c", 3)];
            let py_map = arr.into_py_dict(py);

            assert_eq!(py_map.len(), 3);
            assert_eq!(
                py_map
                    .get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                2
            );
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
    fn abc_dict(py: Python<'_>) -> &PyDict {
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
            assert!(keys.is_instance(py.get_type::<PyDictKeys>()).unwrap());
        })
    }

    #[test]
    #[cfg(not(PyPy))]
    fn dict_values_view() {
        Python::with_gil(|py| {
            let dict = abc_dict(py);
            let values = dict.call_method0("values").unwrap();
            assert!(values.is_instance(py.get_type::<PyDictValues>()).unwrap());
        })
    }

    #[test]
    #[cfg(not(PyPy))]
    fn dict_items_view() {
        Python::with_gil(|py| {
            let dict = abc_dict(py);
            let items = dict.call_method0("items").unwrap();
            assert!(items.is_instance(py.get_type::<PyDictItems>()).unwrap());
        })
    }

    #[test]
    fn dict_update() {
        Python::with_gil(|py| {
            let dict = [("a", 1), ("b", 2), ("c", 3)].into_py_dict(py);
            let other = [("b", 4), ("c", 5), ("d", 6)].into_py_dict(py);
            dict.update(other.as_mapping()).unwrap();
            assert_eq!(dict.len(), 4);
            assert_eq!(
                dict.get_item("a")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
            assert_eq!(
                dict.get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                4
            );
            assert_eq!(
                dict.get_item("c")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                5
            );
            assert_eq!(
                dict.get_item("d")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                6
            );

            assert_eq!(other.len(), 3);
            assert_eq!(
                other
                    .get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                4
            );
            assert_eq!(
                other
                    .get_item("c")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                5
            );
            assert_eq!(
                other
                    .get_item("d")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                6
            );
        })
    }

    #[test]
    fn dict_update_if_missing() {
        Python::with_gil(|py| {
            let dict = [("a", 1), ("b", 2), ("c", 3)].into_py_dict(py);
            let other = [("b", 4), ("c", 5), ("d", 6)].into_py_dict(py);
            dict.update_if_missing(other.as_mapping()).unwrap();
            assert_eq!(dict.len(), 4);
            assert_eq!(
                dict.get_item("a")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                1
            );
            assert_eq!(
                dict.get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                2
            );
            assert_eq!(
                dict.get_item("c")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                3
            );
            assert_eq!(
                dict.get_item("d")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                6
            );

            assert_eq!(other.len(), 3);
            assert_eq!(
                other
                    .get_item("b")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                4
            );
            assert_eq!(
                other
                    .get_item("c")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                5
            );
            assert_eq!(
                other
                    .get_item("d")
                    .unwrap()
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                6
            );
        })
    }
}
