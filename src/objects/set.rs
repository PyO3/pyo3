// Copyright (c) 2017-present PyO3 Project and Contributors
//
use std::{hash, collections};
use ffi;
use python::{Python, PythonObject};
use conversion::ToPyObject;
use objects::{PyObject, PyIterator};
use err::{self, PyResult, PyErr};


/// Represents a Python `set`
pub struct PySet(PyObject);
/// Represents a  Python `frozenset`
pub struct PyFrozenSet(PyObject);

pyobject_newtype!(PySet, PySet_Check, PySet_Type);
pyobject_newtype!(PyFrozenSet, PyFrozenSet_Check, PyFrozenSet_Type);

impl PySet {
    /// Creates a new set.
    ///
    /// May panic when running out of memory.
    pub fn new<T: ToPyObject>(py: Python, elements: &[T]) -> PySet {
        let list = elements.to_py_object(py);
        unsafe {
            err::cast_from_owned_ptr_or_panic(py, ffi::PySet_New(list.as_ptr()))
        }
    }

    /// Remove all elements from the set.
    #[inline]
    pub fn clear(&self, _py: Python) {
        unsafe { ffi::PySet_Clear(self.0.as_ptr()); }
    }

    /// Return the number of items in the set.
    /// This is equivalent to len(p) on a set.
    #[inline]
    pub fn len(&self, _py: Python) -> usize {
        unsafe { ffi::PySet_Size(self.0.as_ptr()) as usize }
    }

    /// Determine if the set contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, py: Python, key: K) -> PyResult<bool> where K: ToPyObject {
        key.with_borrowed_ptr(py, |key| unsafe {
            match ffi::PySet_Contains(self.0.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(py))
            }
        })
    }

    /// Remove element from the set if it is present.
    pub fn discard<K>(&self, py: Python, key: K) where K: ToPyObject {
        key.with_borrowed_ptr(py, |key| unsafe {
            ffi::PySet_Discard(self.0.as_ptr(), key);
        })
    }

    /// Add element to the set.
    pub fn add<K>(&self, py: Python, key: K) -> PyResult<()> where K: ToPyObject {
        key.with_borrowed_ptr(py, move |key| unsafe {
            err::error_on_minusone(py,
                ffi::PySet_Add(self.0.as_ptr(), key))
        })
    }

    /// Remove and return an arbitrary element from the set
    pub fn pop(&self, py: Python) -> Option<PyObject> {
        unsafe {
            PyObject::from_borrowed_ptr_opt(py,
                ffi::PySet_Pop(self.0.as_ptr()))
        }
    }

    #[inline]
    pub fn iter<'p>(&self, py: Python<'p>) -> PyResult<PyIterator<'p>> {
        use objectprotocol::ObjectProtocol;
        self.as_object().iter(py)
    }
}

impl<T> ToPyObject for collections::HashSet<T>
   where T: hash::Hash + Eq + ToPyObject
{
    fn to_py_object(&self, py: Python) -> PyObject {
        let set = PySet::new::<T>(py, &[]);
        for val in self {
            set.add(py, val).unwrap();
        }
        set.into_object()
    }
}

impl<T> ToPyObject for collections::BTreeSet<T>
   where T: hash::Hash + Eq + ToPyObject
{
    fn to_py_object(&self, py: Python) -> PyObject {
        let set = PySet::new::<T>(py, &[]);
        for val in self {
            set.add(py, val).unwrap();
        }
        set.into_object()
    }
}

impl PyFrozenSet {
    /// Creates a new frozenset.
    ///
    /// May panic when running out of memory.
    pub fn new<T: ToPyObject>(py: Python, elements: &[T]) -> PyFrozenSet {
        let list = elements.to_py_object(py);
        unsafe {
            err::cast_from_owned_ptr_or_panic(py, ffi::PyFrozenSet_New(list.as_ptr()))
        }
    }

    /// Return the number of items in the set.
    /// This is equivalent to len(p) on a set.
    #[inline]
    pub fn len(&self, _py: Python) -> usize {
        unsafe { ffi::PySet_Size(self.0.as_ptr()) as usize }
    }

    /// Determine if the set contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, py: Python, key: K) -> PyResult<bool> where K: ToPyObject {
        key.with_borrowed_ptr(py, |key| unsafe {
            match ffi::PySet_Contains(self.0.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(py))
            }
        })
    }

    #[inline]
    pub fn iter<'p>(&self, py: Python<'p>) -> PyResult<PyIterator<'p>> {
        use objectprotocol::ObjectProtocol;
        self.as_object().iter(py)
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashSet};
    use python::{Python, PythonObjectWithCheckedDowncast};
    use conversion::ToPyObject;
    use super::{PySet, PyFrozenSet};

    #[test]
    fn test_set_new() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PySet::new(py, &[1]);
        assert_eq!(1, set.len(py));
    }

    #[test]
    fn test_set_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut v = HashSet::new();
        let set = PySet::downcast_from(py, v.to_py_object(py)).unwrap();
        assert_eq!(0, set.len(py));
        v.insert(7);
        let set2 = PySet::downcast_from(py, v.to_py_object(py)).unwrap();
        assert_eq!(1, set2.len(py));
    }

    #[test]
    fn test_set_clear() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]);
        assert_eq!(1, set.len(py));
        set.clear(py);
        assert_eq!(0, set.len(py));
    }

    #[test]
    fn test_set_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]);
        assert!(set.contains(py, 1).unwrap());
    }

    #[test]
    fn test_set_discard() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]);
        set.discard(py, 2);
        assert_eq!(1, set.len(py));
        set.discard(py, 1);
        assert_eq!(0, set.len(py));
    }

    #[test]
    fn test_set_add() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1, 2]);
        set.add(py, 1).unwrap();  // Add a dupliated element
        assert!(set.contains(py, 1).unwrap());
    }

    #[test]
    fn test_set_pop() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PySet::new(py, &[1]);
        let val = set.pop(py);
        assert!(val.is_some());
        let val2 = set.pop(py);
        assert!(val2.is_none());
    }

    #[test]
    fn test_set_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PySet::new(py, &[1]);
        for el in set.iter(py).unwrap() {
            assert_eq!(1i32, el.unwrap().extract::<i32>(py).unwrap());
        }
    }

    #[test]
    fn test_frozenset_new_and_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PyFrozenSet::new(py, &[1]);
        assert_eq!(1, set.len(py));
    }

    #[test]
    fn test_frozenset_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let set = PyFrozenSet::new(py, &[1]);
        assert!(set.contains(py, 1).unwrap());
    }

    #[test]
    fn test_frozenset_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let set = PyFrozenSet::new(py, &[1]);
        for el in set.iter(py).unwrap() {
            assert_eq!(1i32, el.unwrap().extract::<i32>(py).unwrap());
        }
    }
}
