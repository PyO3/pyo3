// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use pyptr::Py;
use python::{Python, ToPythonPointer, IntoPythonPointer,
             PythonToken, PythonObjectWithToken, PythonTokenApi};
use objects::PyObject;
use ffi::{self, Py_ssize_t};
use conversion::{ToPyObject, IntoPyObject};

/// Represents a Python `list`.
pub struct PyList(PythonToken<PyList>);

pyobject_newtype!(PyList, PyList_Check, PyList_Type);

impl PyList {
    /// Construct a new list with the given elements.
    pub fn new<'p, T: ToPyObject>(py: Python<'p>, elements: &[T]) -> Py<'p, PyList> {
        unsafe {
            let ptr = ffi::PyList_New(elements.len() as Py_ssize_t);
            let t = Py::<PyList>::cast_from_owned_ptr_or_panic(py, ptr);
            for (i, e) in elements.iter().enumerate() {
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, e.to_object(py).into_ptr());
            }
            t
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
    pub fn get_item(&self, index: usize) -> &PyObject {
        // TODO: do we really want to panic here?
        assert!(index < self.len());
        unsafe {
            let ptr = ffi::PyList_GetItem(self.as_ptr(), index as Py_ssize_t);
            self.py_token().from_owned(ptr)
        }
    }

    /// Sets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn set_item<'p>(&self, index: usize, item: Py<'p, PyObject>) {
        let r = unsafe { ffi::PyList_SetItem(
            self.as_ptr(), index as Py_ssize_t, item.into_ptr()) };
        assert!(r == 0);
    }

    /// Inserts an item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn insert_item<'p>(&self, index: usize, item: Py<'p, PyObject>) {
        let r = unsafe { ffi::PyList_Insert(self.as_ptr(), index as Py_ssize_t, item.as_ptr()) };
        assert!(r == 0);
    }

    #[inline]
    pub fn iter<'p>(&'p self) -> PyListIterator<'p> {
        PyListIterator { list: self, index: 0 }
    }
}

/// Used by `PyList::iter()`.
pub struct PyListIterator<'p> {
    list: &'p PyList,
    index: usize
}

impl <'p> Iterator for PyListIterator<'p> {
    type Item = Py<'p, PyObject>;

    #[inline]
    fn next(&mut self) -> Option<Py<'p, PyObject>> {
        if self.index < self.list.len() {
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

    fn to_object<'p>(&self, py: Python<'p>) -> Py<'p, PyObject> {
        unsafe {
            let ptr = ffi::PyList_New(self.len() as Py_ssize_t);
            let t = Py::cast_from_owned_ptr_or_panic(py, ptr);
            for (i, e) in self.iter().enumerate() {
                let obj = e.to_object(py).into_ptr();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj);
            }
            t
        }
    }
}

impl <T> ToPyObject for Vec<T> where T: ToPyObject {

    fn to_object<'p>(&self, py: Python<'p>) -> Py<'p, PyObject> {
        self.as_slice().to_object(py)
    }

}

impl <T> IntoPyObject for Vec<T> where T: IntoPyObject {

    fn into_object<'p>(self, py: Python<'p>) -> Py<'p, PyObject> {
        unsafe {
            let ptr = ffi::PyList_New(self.len() as Py_ssize_t);
            let t = Py::from_owned_ptr_or_panic(py, ptr);
            for (i, e) in self.into_iter().enumerate() {
                let obj = e.into_object(py).into_ptr();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj);
            }
            t
        }
    }
}

#[cfg(test)]
mod test {
    use python::{Python, PythonObjectWithCheckedDowncast};
    use conversion::ToPyObject;
    use objects::PyList;

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![1,2,3,4];
        let list = PyList::downcast_from(py, v.to_object(py)).unwrap();
        assert_eq!(4, list.len(py));
    }

    #[test]
    fn test_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::downcast_from(py, v.to_py_object(py)).unwrap();
        assert_eq!(2, list.get_item(py, 0).extract::<i32>(py).unwrap());
        assert_eq!(3, list.get_item(py, 1).extract::<i32>(py).unwrap());
        assert_eq!(5, list.get_item(py, 2).extract::<i32>(py).unwrap());
        assert_eq!(7, list.get_item(py, 3).extract::<i32>(py).unwrap());
    }

    #[test]
    fn test_set_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::downcast_from(py, v.to_py_object(py)).unwrap();
        let val = 42i32.to_py_object(py).into_object();
        assert_eq!(2, list.get_item(py, 0).extract::<i32>(py).unwrap());
        list.set_item(py, 0, val);
        assert_eq!(42, list.get_item(py, 0).extract::<i32>(py).unwrap());
    }

    #[test]
    fn test_insert_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::downcast_from(py, v.to_py_object(py)).unwrap();
        let val = 42i32.to_py_object(py).into_object();
        assert_eq!(4, list.len(py));
        assert_eq!(2, list.get_item(py, 0).extract::<i32>(py).unwrap());
        list.insert_item(py, 0, val);
        assert_eq!(5, list.len(py));
        assert_eq!(42, list.get_item(py, 0).extract::<i32>(py).unwrap());
        assert_eq!(2, list.get_item(py, 1).extract::<i32>(py).unwrap());
    }

    #[test]
    fn test_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::downcast_from(py, v.to_py_object(py)).unwrap();
        let mut idx = 0;
        for el in list.iter(py) {
            assert_eq!(v[idx], el.extract::<i32>(py).unwrap());
            idx += 1;
        }
        assert_eq!(idx, v.len());
    }

    #[test]
    fn test_extract() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::downcast_from(py, v.to_py_object(py)).unwrap();
        let v2 = list.into_object().extract::<Vec<i32>>(py).unwrap();
        assert_eq!(v, v2);
    }
}
