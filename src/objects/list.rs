// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;

use err::{self, PyResult};
use ffi::{self, Py_ssize_t};
use instance::PyObjectWithToken;
use object::PyObject;
use objects::PyObjectRef;
use python::{Python, ToPyPointer, IntoPyPointer};
use conversion::{ToPyObject, IntoPyObject, ToBorrowedObject};

/// Represents a Python `list`.
pub struct PyList(PyObject);

pyobject_convert!(PyList);
pyobject_nativetype!(PyList, PyList_Type, PyList_Check);

impl PyList {
    /// Construct a new list with the given elements.
    pub fn new<'p, T: ToPyObject>(py: Python<'p>, elements: &[T]) -> &'p PyList {
        unsafe {
            let ptr = ffi::PyList_New(elements.len() as Py_ssize_t);
            for (i, e) in elements.iter().enumerate() {
                let obj = e.to_object(py).into_ptr();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj);
            }
            py.from_owned_ptr::<PyList>(ptr)
        }
    }

    /// Construct a new empty list.
    pub fn empty(py: Python) -> &PyList {
        unsafe {
            py.from_owned_ptr::<PyList>(ffi::PyList_New(0))
        }
    }

    /// Gets the length of the list.
    pub fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe {
            ffi::PyList_Size(self.as_ptr()) as usize
        }
    }

    /// Check if list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_item(&self, index: isize) -> &PyObjectRef {
        unsafe {
            self.py().from_borrowed_ptr(
                ffi::PyList_GetItem(self.as_ptr(), index as Py_ssize_t))
        }
    }

    /// Gets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_parked_item(&self, index: isize) -> PyObject {
        unsafe {
            PyObject::from_borrowed_ptr(
                self.py(), ffi::PyList_GetItem(self.as_ptr(), index as Py_ssize_t))
        }
    }

    /// Sets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn set_item<I>(&self, index: isize, item: I) -> PyResult<()> where I: ToPyObject {
        unsafe {
            err::error_on_minusone(
                self.py(), ffi::PyList_SetItem(self.as_ptr(), index,
                                               item.to_object(self.py()).into_ptr()))
        }
    }

    /// Appends an item at the list.
    pub fn append<I>(&self, item: I) -> PyResult<()> where I: ToBorrowedObject
    {
        item.with_borrowed_ptr(self.py(), |item| unsafe {
            err::error_on_minusone(
                self.py(), ffi::PyList_Append(self.as_ptr(), item))
        })
    }

    /// Inserts an item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn insert<I>(&self, index: isize, item: I) -> PyResult<()> where I: ToBorrowedObject
    {
        item.with_borrowed_ptr(self.py(), |item| unsafe {
            err::error_on_minusone(
                self.py(), ffi::PyList_Insert(self.as_ptr(), index, item))
        })
    }

    /// Returns an iterator over the tuple items.
    pub fn iter(&self) -> PyListIterator {
        PyListIterator { list: self, index: 0 }
    }
}

/// Used by `PyList::iter()`.
pub struct PyListIterator<'a> {
    list: &'a PyList,
    index: isize,
}

impl<'a> Iterator for PyListIterator<'a> {
    type Item = &'a PyObjectRef;

    #[inline]
    fn next(&mut self) -> Option<&'a PyObjectRef> {
        if self.index < self.list.len() as isize {
            let item = self.list.get_item(self.index);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<'a> std::iter::IntoIterator for &'a PyList {
    type Item = &'a PyObjectRef;
    type IntoIter = PyListIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl <T> ToPyObject for [T] where T: ToPyObject {

    fn to_object<'p>(&self, py: Python<'p>) -> PyObject {
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

    fn to_object<'p>(&self, py: Python<'p>) -> PyObject {
        self.as_slice().to_object(py)
    }

}

impl <T> IntoPyObject for Vec<T> where T: IntoPyObject + ToPyObject {

    fn into_object(self, py: Python) -> PyObject {
        unsafe {
            let ptr = ffi::PyList_New(self.len() as Py_ssize_t);
            for (i, e) in self.into_iter().enumerate() {
                let obj = e.into_object(py).into_ptr();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj);
            }
            PyObject::from_owned_ptr_or_panic(py, ptr)
        }
    }
}

#[cfg(test)]
mod test {
    use instance::AsPyRef;
    use python::Python;
    use conversion::{ToPyObject, PyTryFrom};
    use objects::PyList;
    use objectprotocol::ObjectProtocol;

    #[test]
    fn test_new() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::new(py, &v);
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>().unwrap());
        assert_eq!(5, list.get_item(2).extract::<i32>().unwrap());
        assert_eq!(7, list.get_item(3).extract::<i32>().unwrap());
    }

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![1,2,3,4];
        let ob = v.to_object(py);
        let list = PyList::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(4, list.len());
    }

    #[test]
    fn test_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>().unwrap());
        assert_eq!(5, list.get_item(2).extract::<i32>().unwrap());
        assert_eq!(7, list.get_item(3).extract::<i32>().unwrap());
    }

    #[test]
    fn test_get_parked_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(2, list.get_parked_item(0).extract::<i32>(py).unwrap());
        assert_eq!(3, list.get_parked_item(1).extract::<i32>(py).unwrap());
        assert_eq!(5, list.get_parked_item(2).extract::<i32>(py).unwrap());
        assert_eq!(7, list.get_parked_item(3).extract::<i32>(py).unwrap());
    }

    #[test]
    fn test_set_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::try_from(ob.as_ref(py)).unwrap();
        let val = 42i32.to_object(py);
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        list.set_item(0, val).unwrap();
        assert_eq!(42, list.get_item(0).extract::<i32>().unwrap());
    }

    #[test]
    fn test_set_item_refcnt() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let cnt;
        {
            let _pool = ::GILPool::new();
            let v = vec![2];
            let ob = v.to_object(py);
            let list = PyList::try_from(ob.as_ref(py)).unwrap();
            let none = py.None();
            cnt = none.get_refcnt();
            list.set_item(0, none).unwrap();
        }

        assert_eq!(cnt, py.None().get_refcnt());
    }

    #[test]
    fn test_insert() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::try_from(ob.as_ref(py)).unwrap();
        let val = 42i32.to_object(py);
        assert_eq!(4, list.len());
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        list.insert(0, val).unwrap();
        assert_eq!(5, list.len());
        assert_eq!(42, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(2, list.get_item(1).extract::<i32>().unwrap());
    }

    #[test]
    fn test_insert_refcnt() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let cnt;
        {
            let _pool = ::GILPool::new();
            let list = PyList::empty(py);
            let none = py.None();
            cnt = none.get_refcnt();
            list.insert(0, none).unwrap();
        }

        assert_eq!(cnt, py.None().get_refcnt());
    }

    #[test]
    fn test_append() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2];
        let ob = v.to_object(py);
        let list = PyList::try_from(ob.as_ref(py)).unwrap();
        list.append(3).unwrap();
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>().unwrap());
    }

    #[test]
    fn test_append_refcnt() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let cnt;
        {
            let _pool = ::GILPool::new();
            let list = PyList::empty(py);
            let none = py.None();
            cnt = none.get_refcnt();
            list.append(none).unwrap();
        }
        assert_eq!(cnt, py.None().get_refcnt());
    }

    #[test]
    fn test_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::try_from(ob.as_ref(py)).unwrap();
        let mut idx = 0;
        for el in list.iter() {
            assert_eq!(v[idx], el.extract::<i32>().unwrap());
            idx += 1;
        }
        assert_eq!(idx, v.len());
    }

    #[test]
    fn test_into_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![1, 2, 3, 4];
        let ob = v.to_object(py);
        let list = PyList::try_from(ob.as_ref(py)).unwrap();
        let mut i = 0;
        for el in list {
            i += 1;
            assert_eq!(i, el.extract::<i32>().unwrap());
        }
    }

    #[test]
    fn test_extract() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::try_from(ob.as_ref(py)).unwrap();
        let v2 = list.as_ref().extract::<Vec<i32>>().unwrap();
        assert_eq!(v, v2);
    }
}
