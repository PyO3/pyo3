// Copyright (c) 2017-present PyO3 Project and Contributors
//

use crate::err::{self, PyErr, PyResult};
use crate::ffi;
use crate::instance::PyNativeType;
use crate::object::PyObject;
use crate::AsPyPointer;
use crate::Python;
use crate::{ToBorrowedObject, ToPyObject};
use std::{collections, hash};

/// Represents a Python `set`
#[repr(transparent)]
pub struct PySet(PyObject);

/// Represents a  Python `frozenset`
#[repr(transparent)]
pub struct PyFrozenSet(PyObject);

pyobject_native_type!(PySet, ffi::PySet_Type, Some("builtins"), ffi::PySet_Check);
pyobject_native_type!(PyFrozenSet, ffi::PyFrozenSet_Type, ffi::PyFrozenSet_Check);

impl PySet {
    /// Creates a new set.
    pub fn new<'p, T: ToPyObject>(py: Python<'p>, elements: &[T]) -> PyResult<&'p PySet> {
        let list = elements.to_object(py);
        unsafe { py.from_owned_ptr_or_err(ffi::PySet_New(list.as_ptr())) }
    }

    /// Remove all elements from the set.
    #[inline]
    pub fn clear(&self) {
        unsafe {
            ffi::PySet_Clear(self.as_ptr());
        }
    }

    /// Return the number of items in the set.
    /// This is equivalent to len(p) on a set.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ffi::PySet_Size(self.as_ptr()) as usize }
    }

    /// Check if set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Determine if the set contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToPyObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            match ffi::PySet_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.py())),
            }
        })
    }

    /// Remove element from the set if it is present.
    pub fn discard<K>(&self, key: K)
    where
        K: ToPyObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            ffi::PySet_Discard(self.as_ptr(), key);
        })
    }

    /// Add element to the set.
    pub fn add<K>(&self, key: K) -> PyResult<()>
    where
        K: ToPyObject,
    {
        key.with_borrowed_ptr(self.py(), move |key| unsafe {
            err::error_on_minusone(self.py(), ffi::PySet_Add(self.as_ptr(), key))
        })
    }

    /// Remove and return an arbitrary element from the set
    pub fn pop(&self) -> Option<PyObject> {
        unsafe { PyObject::from_owned_ptr_or_opt(self.py(), ffi::PySet_Pop(self.as_ptr())) }
    }
}

impl<T> ToPyObject for collections::HashSet<T>
where
    T: hash::Hash + Eq + ToPyObject,
{
    fn to_object(&self, py: Python) -> PyObject {
        let set = PySet::new::<T>(py, &[]).expect("Failed to construct empty set");
        {
            for val in self {
                set.add(val).expect("Failed to add to set");
            }
        }
        set.into()
    }
}

impl<T> ToPyObject for collections::BTreeSet<T>
where
    T: hash::Hash + Eq + ToPyObject,
{
    fn to_object(&self, py: Python) -> PyObject {
        let set = PySet::new::<T>(py, &[]).expect("Failed to construct empty set");
        {
            for val in self {
                set.add(val).expect("Failed to add to set");
            }
        }
        set.into()
    }
}

impl PyFrozenSet {
    /// Creates a new frozenset.
    ///
    /// May panic when running out of memory.
    pub fn new<'p, T: ToPyObject>(py: Python<'p>, elements: &[T]) -> PyResult<&'p PyFrozenSet> {
        let list = elements.to_object(py);
        unsafe { py.from_owned_ptr_or_err(ffi::PyFrozenSet_New(list.as_ptr())) }
    }

    /// Return the number of items in the set.
    /// This is equivalent to len(p) on a set.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ffi::PySet_Size(self.as_ptr()) as usize }
    }

    /// Check if set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Determine if the set contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool>
    where
        K: ToBorrowedObject,
    {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            match ffi::PySet_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.py())),
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::{PyFrozenSet, PySet};
    use crate::instance::AsPyRef;
    use crate::objectprotocol::ObjectProtocol;
    use crate::Python;
    use crate::{PyTryFrom, ToPyObject};
    use std::collections::HashSet;

    #[test]
    fn test_set_new() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PySet::new(py, &[1]).unwrap();
        assert_eq!(1, set.len());

        let v = vec![1];
        assert!(PySet::new(py, &[v]).is_err());
    }

    #[test]
    fn test_set_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut v = HashSet::new();
        let ob = v.to_object(py);
        let set = <PySet as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(0, set.len());
        v.insert(7);
        let ob = v.to_object(py);
        let set2 = <PySet as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(1, set2.len());
    }

    #[test]
    fn test_set_clear() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]).unwrap();
        assert_eq!(1, set.len());
        set.clear();
        assert_eq!(0, set.len());
    }

    #[test]
    fn test_set_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]).unwrap();
        assert!(set.contains(1).unwrap());
    }

    #[test]
    fn test_set_discard() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]).unwrap();
        set.discard(2);
        assert_eq!(1, set.len());
        set.discard(1);
        assert_eq!(0, set.len());
    }

    #[test]
    fn test_set_add() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1, 2]).unwrap();
        set.add(1).unwrap(); // Add a dupliated element
        assert!(set.contains(1).unwrap());
    }

    #[test]
    fn test_set_pop() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]).unwrap();
        let val = set.pop();
        assert!(val.is_some());
        let val2 = set.pop();
        assert!(val2.is_none());
    }

    #[test]
    fn test_set_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PySet::new(py, &[1]).unwrap();
        for el in set.iter().unwrap() {
            assert_eq!(1i32, el.unwrap().extract::<i32>().unwrap());
        }
    }

    #[test]
    fn test_frozenset_new_and_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PyFrozenSet::new(py, &[1]).unwrap();
        assert_eq!(1, set.len());

        let v = vec![1];
        assert!(PyFrozenSet::new(py, &[v]).is_err());
    }

    #[test]
    fn test_frozenset_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PyFrozenSet::new(py, &[1]).unwrap();
        assert!(set.contains(1).unwrap());
    }

    #[test]
    fn test_frozenset_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PyFrozenSet::new(py, &[1]).unwrap();
        for el in set.iter().unwrap() {
            assert_eq!(1i32, el.unwrap().extract::<i32>().unwrap());
        }
    }
}
