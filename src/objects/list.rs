// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use python::{Python, PythonObject, ToPythonPointer, PyClone, PyDrop};
use err::{self, PyErr, PyResult};
use super::object::PyObject;
use ffi::{self, Py_ssize_t};
use conversion::{ToPyObject, FromPyObject};

/// Represents a Python `list`.
pub struct PyList(PyObject);

pyobject_newtype!(PyList, PyList_Check, PyList_Type);

impl PyList {
    /// Construct a new list with the given elements.
    pub fn new(py: Python, elements: &[PyObject]) -> PyList {
        unsafe {
            let ptr = ffi::PyList_New(elements.len() as Py_ssize_t);
            let t = err::result_from_owned_ptr(py, ptr).unwrap().unchecked_cast_into::<PyList>();
            for (i, e) in elements.iter().enumerate() {
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, e.steal_ptr(py));
            }
            t
        }
    }

    /// Gets the length of the list.
    #[inline]
    pub fn len(&self, _py: Python) -> usize {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe {
            ffi::PyList_Size(self.0.as_ptr()) as usize
        }
    }

    /// Gets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_item(&self, py: Python, index: usize) -> PyObject {
        // TODO: do we really want to panic here?
        assert!(index < self.len(py));
        unsafe {
            PyObject::from_borrowed_ptr(py, ffi::PyList_GetItem(self.0.as_ptr(), index as Py_ssize_t))
        }
    }

    /// Sets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn set_item(&self, _py: Python, index: usize, item: PyObject) {
        let r = unsafe { ffi::PyList_SetItem(self.0.as_ptr(), index as Py_ssize_t, item.steal_ptr()) };
        assert!(r == 0);
    }

    /// Inserts an item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn insert_item(&self, _py: Python, index: usize, item: PyObject) {
        let r = unsafe { ffi::PyList_Insert(self.0.as_ptr(), index as Py_ssize_t, item.as_ptr()) };
        assert!(r == 0);
    }

    #[inline]
    pub fn iter<'a, 'p>(&'a self, py: Python<'p>) -> PyListIterator<'a, 'p> {
        PyListIterator { py: py, list: self, index: 0 }
    }
}

/// Used by `PyList::iter()`.
pub struct PyListIterator<'a, 'p> {
    py: Python<'p>,
    list: &'a PyList,
    index: usize
}

impl <'a, 'p> Iterator for PyListIterator<'a, 'p> {
    type Item = PyObject;

    #[inline]
    fn next(&mut self) -> Option<PyObject> {
        if self.index < self.list.len(self.py) {
            let item = self.list.get_item(self.py, self.index);
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
    type ObjectType = PyList;

    fn to_py_object(&self, py: Python) -> PyList {
        unsafe {
            let ptr = ffi::PyList_New(self.len() as Py_ssize_t);
            let t = err::cast_from_owned_ptr_or_panic(py, ptr);
            for (i, e) in self.iter().enumerate() {
                let obj = e.to_py_object(py).into_object();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj.steal_ptr());
            }
            t
        }
    }
}

impl <T> ToPyObject for Vec<T> where T: ToPyObject {
    type ObjectType = PyList;

    fn to_py_object(&self, py: Python) -> PyList {
        self.as_slice().to_py_object(py)
    }

    fn into_py_object(self, py: Python) -> PyList {
        unsafe {
            let ptr = ffi::PyList_New(self.len() as Py_ssize_t);
            let t = err::cast_from_owned_ptr_or_panic(py, ptr);
            for (i, e) in self.into_iter().enumerate() {
                let obj = e.into_py_object(py).into_object();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj.steal_ptr());
            }
            t
        }
    }
}

#[cfg(test)]
mod test {
    use python::{Python, PythonObject};
    use conversion::ToPyObject;
    use objects::PyList;

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![1,2,3,4];
        let list = v.to_py_object(py);
        assert_eq!(4, list.len(py));
    }

    #[test]
    fn test_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = v.to_py_object(py);
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
        let list = v.to_py_object(py);
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
        let list = v.to_py_object(py);
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
        let list = v.to_py_object(py);
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
        let list = v.to_py_object(py);
        let v2 = list.into_object().extract::<Vec<i32>>(py).unwrap();
        assert_eq!(v, v2);
    }
}
