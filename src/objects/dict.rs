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

use ffi;
use python::{Python, ToPythonPointer, PythonObject};
use conversion::ToPyObject;
use objects::{PyObject, PyList};
use err::{self, PyResult, PyErr};

/// Represents a Python `dict`.
pub struct PyDict<'p>(PyObject<'p>);

pyobject_newtype!(PyDict, PyDict_Check, PyDict_Type);

impl <'p> PyDict<'p> {
    /// Creates a new empty dictionary.
    ///
    /// May panic when running out of memory.
    pub fn new(py: Python<'p>) -> PyDict<'p> {
        unsafe {
            err::cast_from_owned_ptr_or_panic(py, ffi::PyDict_New())
        }
    }

    /// Return a new dictionary that contains the same key-value pairs as self.
    /// Corresponds to `dict(self)` in Python.
    pub fn copy(&self) -> PyResult<'p, PyDict<'p>> {
        let py = self.python();
        unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyDict_Copy(self.as_ptr()))
        }
    }

    /// Empty an existing dictionary of all key-value pairs.
    #[inline]
    pub fn clear(&self) {
        unsafe { ffi::PyDict_Clear(self.as_ptr()) }
    }

    /// Return the number of items in the dictionary.
    /// This is equivalent to len(p) on a dictionary.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ffi::PyDict_Size(self.as_ptr()) as usize }
    }

    /// Determine if the dictionary contains the specified key.
    /// This is equivalent to the Python expression `key in self`.
    pub fn contains<K>(&self, key: K) -> PyResult<'p, bool> where K: ToPyObject<'p> {
        let py = self.python();
        key.with_borrowed_ptr(py, |key| unsafe {
            match ffi::PyDict_Contains(self.as_ptr(), key) {
                1 => Ok(true),
                0 => Ok(false),
                _ => Err(PyErr::fetch(py))
            }
        })
    }

    /// Gets an item from the dictionary.
    /// Returns None if the item is not present, or if an error occurs.
    pub fn get_item<K>(&self, key: K) -> Option<PyObject<'p>> where K: ToPyObject<'p> {
        let py = self.python();
        key.with_borrowed_ptr(py, |key| unsafe {
            PyObject::from_borrowed_ptr_opt(py,
                ffi::PyDict_GetItem(self.as_ptr(), key))
        })
    }

    /// Sets an item value.
    /// This is equivalent to the Python expression `self[key] = value`.
    pub fn set_item<K, V>(&self, key: K, value: V) -> PyResult<'p, ()> where K: ToPyObject<'p>, V: ToPyObject<'p> {
        let py = self.python();
        key.with_borrowed_ptr(py, move |key|
            value.with_borrowed_ptr(py, |value| unsafe {
                err::error_on_minusone(py,
                    ffi::PyDict_SetItem(self.as_ptr(), key, value))
            }))
    }

    /// Deletes an item.
    /// This is equivalent to the Python expression `del self[key]`.
    pub fn del_item<K>(&self, key: K) -> PyResult<'p, ()> where K: ToPyObject<'p> {
        let py = self.python();
        key.with_borrowed_ptr(py, |key| unsafe {
            err::error_on_minusone(py,
                ffi::PyDict_DelItem(self.as_ptr(), key))
        })
    }

    // List of dict items.
    // This is equivalent to the `dict.items()` method.
    pub fn items(&self) -> PyList {
        let py = self.python();
        unsafe {
            let pyobj = PyObject::from_borrowed_ptr(py, ffi::PyDict_Items(self.as_ptr()));
            pyobj.unchecked_cast_into::<PyList>()
        }
    }
}
