// Copyright (c) 2017-present PyO3 Project and Contributors
//

use std::{hash, collections};
use ffi;
use python::{Python, ToPyPointer};
use object::PyObject;
use conversion::{ToPyObject, ToBorrowedObject};
use instance::{AsPyRef, Py, PyObjectWithToken};
use err::{self, PyResult, PyErr};


/// Represents a Python `set`
pub struct PySet(PyObject);

/// Represents a  Python `frozenset`
pub struct PyFrozenSet(PyObject);

pyobject_convert!(PySet);
pyobject_convert!(PyFrozenSet);
pyobject_nativetype!(PySet, PySet_Type, PySet_Check);
pyobject_nativetype!(PyFrozenSet, PyFrozenSet_Type, PyFrozenSet_Check);

impl PySet {
    /// Creates a new set.
    ///
    /// May panic when running out of memory.
    pub fn new<T: ToPyObject>(py: Python, elements: &[T]) -> Py<PySet> {
        let list = elements.to_object(py);
        unsafe {
            Py::from_owned_ptr_or_panic(ffi::PySet_New(list.as_ptr()))
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

    /// Check if set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Determine if the set contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<bool> where K: ToPyObject {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            match ffi::PySet_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.py()))
            }
        })
    }

    /// Remove element from the set if it is present.
    pub fn discard<K>(&self, key: K) where K: ToPyObject {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            ffi::PySet_Discard(self.as_ptr(), key);
        })
    }

    /// Add element to the set.
    pub fn add<K>(&self, key: K) -> PyResult<()> where K: ToPyObject {
        key.with_borrowed_ptr(self.py(), move |key| unsafe {
            err::error_on_minusone(self.py(), ffi::PySet_Add(self.as_ptr(), key))
        })
    }

    /// Remove and return an arbitrary element from the set
    pub fn pop(&self) -> Option<PyObject> {
        unsafe {
            PyObject::from_owned_ptr_or_opt(self.py(), ffi::PySet_Pop(self.as_ptr()))
        }
    }
}

impl<T> ToPyObject for collections::HashSet<T>
    where T: hash::Hash + Eq + ToPyObject
{
    fn to_object(&self, py: Python) -> PyObject {
        let set = PySet::new::<T>(py, &[]);
        {
            let s = set.as_ref(py);
            for val in self {
                s.add(val).expect("Failed to add to set");
            }
        }
        set.into()
    }
}

impl<T> ToPyObject for collections::BTreeSet<T>
    where T: hash::Hash + Eq + ToPyObject
{
    fn to_object(&self, py: Python) -> PyObject {
        let set = PySet::new::<T>(py, &[]);
        {
            let s = set.as_ref(py);
            for val in self {
                s.add(val).expect("Failed to add to set");
            }
        }
        set.into()
    }
}

impl PyFrozenSet {
    /// Creates a new frozenset.
    ///
    /// May panic when running out of memory.
    pub fn new<T: ToPyObject>(py: Python, elements: &[T]) -> Py<PyFrozenSet> {
        let list = elements.to_object(py);
        unsafe {
            Py::from_owned_ptr_or_panic(ffi::PyFrozenSet_New(list.as_ptr()))
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
    pub fn contains<K>(&self, key: K) -> PyResult<bool> where K: ToBorrowedObject {
        key.with_borrowed_ptr(self.py(), |key| unsafe {
            match ffi::PySet_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(self.py()))
            }
        })
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashSet};
    use super::{PySet, PyFrozenSet};
    use python::Python;
    use conversion::{ToPyObject, PyTryFrom};
    use objectprotocol::ObjectProtocol;
    use instance::AsPyRef;

    #[test]
    fn test_set_new() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PySet::new(py, &[1]);
        assert_eq!(1, set.as_ref(py).len());
    }

    #[test]
    fn test_set_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut v = HashSet::new();
        let ob = v.to_object(py);
        let set = PySet::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(0, set.len());
        v.insert(7);
        let ob = v.to_object(py);
        let set2 = PySet::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(1, set2.len());
    }

    #[test]
    fn test_set_clear() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = PySet::new(py, &[1]);
        let set = ob.as_ref(py);
        assert_eq!(1, set.len());
        set.clear();
        assert_eq!(0, set.len());
    }

    #[test]
    fn test_set_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]);
        assert!(set.as_ref(py).contains(1).unwrap());
    }

    #[test]
    fn test_set_discard() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = PySet::new(py, &[1]);
        let set = ob.as_ref(py);
        set.discard(2);
        assert_eq!(1, set.len());
        set.discard(1);
        assert_eq!(0, set.len());
    }

    #[test]
    fn test_set_add() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = PySet::new(py, &[1, 2]);
        let set = ob.as_ref(py);
        set.add(1).unwrap();  // Add a dupliated element
        assert!(set.contains(1).unwrap());
    }

    #[test]
    fn test_set_pop() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = PySet::new(py, &[1]);
        let set = ob.as_ref(py);
        let val = set.pop();
        assert!(val.is_some());
        let val2 = set.pop();
        assert!(val2.is_none());
    }

    #[test]
    fn test_set_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let ob = PySet::new(py, &[1]);
        let set = ob.as_ref(py);
        for el in set.iter().unwrap() {
            assert_eq!(1i32, el.unwrap().extract::<i32>().unwrap());
        }
    }

    #[test]
    fn test_frozenset_new_and_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let ob = PyFrozenSet::new(py, &[1]);
        let set = ob.as_ref(py);
        assert_eq!(1, set.len());
    }

    #[test]
    fn test_frozenset_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = PyFrozenSet::new(py, &[1]);
        let set = ob.as_ref(py);
        assert!(set.contains(1).unwrap());
    }

    #[test]
    fn test_frozenset_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let ob = PyFrozenSet::new(py, &[1]);
        let set = ob.as_ref(py);
        for el in set.iter().unwrap() {
            assert_eq!(1i32, el.unwrap().extract::<i32>().unwrap());
        }
    }
}
