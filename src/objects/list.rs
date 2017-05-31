// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use ::pyptr;
use err::{self, PyResult};
use ffi::{self, Py_ssize_t};
use python::{Python, ToPythonPointer, IntoPythonPointer};
use objects::PyObject;
use token::PythonObjectWithGilToken;
use conversion::{ToPyObject, IntoPyObject};

/// Represents a Python `list`.
pub struct PyList<'p>(pyptr<'p>);

pyobject_nativetype!(PyList, PyList_Check, PyList_Type);

impl<'p> PyList<'p> {
    /// Construct a new list with the given elements.
    pub fn new<T: ToPyObject>(py: Python<'p>, elements: &[T]) -> PyList<'p> {
        unsafe {
            let ptr = ffi::PyList_New(elements.len() as Py_ssize_t);
            for (i, e) in elements.iter().enumerate() {
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, e.to_object(py).into_ptr());
            }
            PyList(pyptr::from_owned_ptr_or_panic(py, ptr))
        }
    }

    /// Gets the length of the list.
    #[inline]
    pub fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe {
            ffi::PyList_Size(self.as_ptr()) as usize
        }
    }

    /// Gets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_item(&self, index: isize) -> PyObject<'p> {
        unsafe {
            PyObject::from_borrowed_ptr(
                self.gil(), ffi::PyList_GetItem(self.as_ptr(), index as Py_ssize_t))
        }
    }

    /// Sets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn set_item<I>(&self, index: isize, item: I) -> PyResult<()>
        where I: ToPyObject
    {
        item.with_borrowed_ptr(self.gil(), |item| unsafe {
            err::error_on_minusone(
                self.gil(), ffi::PyList_SetItem(self.as_ptr(), index, item))
        })
    }

    /// Inserts an item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn insert_item<I>(&self, index: isize, item: I) -> PyResult<()>
        where I: ToPyObject
    {
        item.with_borrowed_ptr(self.gil(), |item| unsafe {
            err::error_on_minusone(
                self.gil(), ffi::PyList_Insert(self.as_ptr(), index, item))
        })
    }

    #[inline]
    pub fn iter(&'p self) -> PyListIterator<'p> {
        PyListIterator { list: self, index: 0 }
    }
}

/// Used by `PyList::iter()`.
pub struct PyListIterator<'p> {
    list: &'p PyList<'p>,
    index: isize
}

impl <'p> Iterator for PyListIterator<'p> {
    type Item = PyObject<'p>;

    #[inline]
    fn next(&mut self) -> Option<PyObject<'p>> {
        if self.index < self.list.len() as isize {
            let item = self.list.get_item(self.index);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    // Note: we cannot implement size_hint because the length of the list
    // might change during the iteration.
}

impl <T> ToPyObject for [T] where T: ToPyObject {

    fn to_object<'p>(&self, py: Python<'p>) -> PyObject<'p> {
        unsafe {
            let ptr = ffi::PyList_New(self.len() as Py_ssize_t);
            for (i, e) in self.iter().enumerate() {
                let obj = e.to_object(py).into_ptr();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj);
            }
            PyObject::from_owned_ptr_or_panic(py, ptr)
        }
    }
}

impl <T> ToPyObject for Vec<T> where T: ToPyObject {

    fn to_object<'p>(&self, py: Python<'p>) -> PyObject<'p> {
        self.as_slice().to_object(py)
    }

}

impl <T> IntoPyObject for Vec<T> where T: IntoPyObject {

    fn into_object(self, py: Python) -> ::pptr {
        unsafe {
            let ptr = ffi::PyList_New(self.len() as Py_ssize_t);
            for (i, e) in self.into_iter().enumerate() {
                let obj = e.into_object(py).into_ptr();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj);
            }
            ::pptr::from_owned_ptr_or_panic(ptr)
        }
    }
}

#[cfg(test)]
mod test {
    use python::{Python, PyDowncastInto};
    use conversion::{ToPyObject, IntoPyObject};
    use objects::PyList;

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![1,2,3,4];
        let list = PyList::downcast_into(py, v.to_object(py)).unwrap();
        assert_eq!(4, list.len());
    }

    #[test]
    fn test_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::downcast_into(py, v.to_object(py)).unwrap();
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>().unwrap());
        assert_eq!(5, list.get_item(2).extract::<i32>().unwrap());
        assert_eq!(7, list.get_item(3).extract::<i32>().unwrap());
    }

    #[test]
    fn test_set_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::downcast_into(py, v.to_object(py)).unwrap();
        let val = 42i32.to_object(py);
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        list.set_item(0, val).unwrap();
        assert_eq!(42, list.get_item(0).extract::<i32>().unwrap());
    }

    #[test]
    fn test_insert_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::downcast_into(py, v.to_object(py)).unwrap();
        let val = 42i32.to_object(py);
        assert_eq!(4, list.len());
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        list.insert_item(0, val).unwrap();
        assert_eq!(5, list.len());
        assert_eq!(42, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(2, list.get_item(1).extract::<i32>().unwrap());
    }

    #[test]
    fn test_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::downcast_into(py, v.to_object(py)).unwrap();
        let mut idx = 0;
        for el in list.iter() {
            assert_eq!(v[idx], el.extract::<i32>().unwrap());
            idx += 1;
        }
        assert_eq!(idx, v.len());
    }

    #[test]
    fn test_extract() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::downcast_into(py, v.to_object(py)).unwrap();
        let v2 = list.into_object(py).into_object(py).extract::<Vec<i32>>().unwrap();
        assert_eq!(v, v2);
    }
}
