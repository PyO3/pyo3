// Copyright (c) 2017-present PyO3 Project and Contributors
//

use std::{hash, collections};
use ffi;
use python::{Python, ToPythonPointer};
use pointers::{Ptr, PyPtr};
use conversion::ToPyObject;
use objects::PyObject;
use err::{self, PyResult, PyErr};
use token::{PythonObjectWithGilToken};


/// Represents a Python `set`
pub struct PySet<'p>(Ptr<'p>);
pub struct PySetPtr(PyPtr);

/// Represents a  Python `frozenset`
pub struct PyFrozenSet<'p>(Ptr<'p>);
pub struct PyFrozenSetPtr(PyPtr);

pyobject_convert!(PySet);
pyobject_nativetype!(PySet, PySet_Check, PySet_Type, PySetPtr);
pyobject_convert!(PyFrozenSet);
pyobject_nativetype!(PyFrozenSet, PyFrozenSet_Check, PyFrozenSet_Type, PyFrozenSetPtr);

impl<'p> PySet<'p> {
    /// Creates a new set.
    ///
    /// May panic when running out of memory.
    pub fn new<T: ToPyObject>(py: Python<'p>, elements: &[T]) -> PySet<'p> {
        let list = elements.to_object(py);
        unsafe {
            let ptr = ffi::PySet_New(list.as_ptr());
            PySet(Ptr::from_owned_ptr_or_panic(py, ptr))
        }
    }

    /// Remove all elements from the set.
    #[inline]
    pub fn clear(&self) {
        unsafe { ffi::PySet_Clear(self.as_ptr()); }
    }

    /// Return the number of items in the set.
    /// This is equivalent to len(p) on a set.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ffi::PySet_Size(self.as_ptr()) as usize }
    }

    /// Determine if the set contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool> where K: ToPyObject {
        key.with_borrowed_ptr(self.gil(), |key| unsafe {
            match ffi::PySet_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.gil()))
            }
        })
    }

    /// Remove element from the set if it is present.
    pub fn discard<K>(&self, key: K) where K: ToPyObject {
        key.with_borrowed_ptr(self.gil(), |key| unsafe {
            ffi::PySet_Discard(self.as_ptr(), key);
        })
    }

    /// Add element to the set.
    pub fn add<K>(&self, key: K) -> PyResult<()> where K: ToPyObject {
        key.with_borrowed_ptr(self.gil(), move |key| unsafe {
            err::error_on_minusone(self.gil(),
                ffi::PySet_Add(self.as_ptr(), key))
        })
    }

    /// Remove and return an arbitrary element from the set
    pub fn pop(&self) -> Option<PyObject<'p>> {
        unsafe {
            PyObject::from_borrowed_ptr_or_opt(self.gil(),
                ffi::PySet_Pop(self.as_ptr()))
        }
    }
}

impl<T> ToPyObject for collections::HashSet<T>
   where T: hash::Hash + Eq + ToPyObject
{
    fn to_object<'p>(&self, py: Python<'p>) -> PyObject<'p> {
        let set = PySet::new::<T>(py, &[]);
        for val in self {
            set.add(val).unwrap();
        }
        set.into()
    }
}

impl<T> ToPyObject for collections::BTreeSet<T>
   where T: hash::Hash + Eq + ToPyObject
{
    fn to_object<'p>(&self, py: Python<'p>) -> PyObject<'p> {
        let set = PySet::new::<T>(py, &[]);
        for val in self {
            set.add(val).unwrap();
        }
        set.into()
    }
}

impl<'p> PyFrozenSet<'p> {
    /// Creates a new frozenset.
    ///
    /// May panic when running out of memory.
    pub fn new<T: ToPyObject>(py: Python<'p>, elements: &[T]) -> PyFrozenSet<'p> {
        let list = elements.to_object(py);
        unsafe {
            let ptr = ffi::PyFrozenSet_New(list.as_ptr());
            PyFrozenSet(Ptr::from_owned_ptr_or_panic(py, ptr))
        }
    }

    /// Return the number of items in the set.
    /// This is equivalent to len(p) on a set.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ffi::PySet_Size(self.as_ptr()) as usize }
    }

    /// Determine if the set contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool> where K: ToPyObject {
        key.with_borrowed_ptr(self.gil(), |key| unsafe {
            match ffi::PySet_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.gil()))
            }
        })
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashSet};
    use python::{Python, PyDowncastInto};
    use conversion::ToPyObject;
    use objectprotocol::ObjectProtocol;
    use super::{PySet, PyFrozenSet};

    #[test]
    fn test_set_new() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PySet::new(py, &[1]);
        assert_eq!(1, set.len());
    }

    #[test]
    fn test_set_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut v = HashSet::new();
        let ob = v.to_object(py);
        let set = PySet::downcast_into(py, ob).unwrap();
        assert_eq!(0, set.len());
        v.insert(7);
        let ob = v.to_object(py);
        let set2 = PySet::downcast_into(py, ob).unwrap();
        assert_eq!(1, set2.len());
    }

    #[test]
    fn test_set_clear() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]);
        assert_eq!(1, set.len());
        set.clear();
        assert_eq!(0, set.len());
    }

    #[test]
    fn test_set_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]);
        assert!(set.contains(1).unwrap());
    }

    #[test]
    fn test_set_discard() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]);
        set.discard(2);
        assert_eq!(1, set.len());
        set.discard(1);
        assert_eq!(0, set.len());
    }

    #[test]
    fn test_set_add() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1, 2]);
        set.add(1).unwrap();  // Add a dupliated element
        assert!(set.contains(1).unwrap());
    }

    #[test]
    fn test_set_pop() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]);
        let val = set.pop();
        assert!(val.is_some());
        let val2 = set.pop();
        assert!(val2.is_none());
    }

    #[test]
    fn test_set_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PySet::new(py, &[1]);
        for el in set.iter().unwrap() {
            assert_eq!(1i32, el.unwrap().extract::<i32>().unwrap());
        }
    }

    #[test]
    fn test_frozenset_new_and_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PyFrozenSet::new(py, &[1]);
        assert_eq!(1, set.len());
    }

    #[test]
    fn test_frozenset_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PyFrozenSet::new(py, &[1]);
        assert!(set.contains(1).unwrap());
    }

    #[test]
    fn test_frozenset_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PyFrozenSet::new(py, &[1]);
        for el in set.iter().unwrap() {
            assert_eq!(1i32, el.unwrap().extract::<i32>().unwrap());
        }
    }
}
