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

use python::{Python, PythonObject, ToPythonPointer};
use objects::PyObject;
use err::{PyErr, PyResult};
use ffi;

pub struct PyIterator(PyObject);

pyobject_newtype!(PyIterator, PyIter_Check);

impl PyIterator {
    /// Retrieves the next item from an iterator.
    /// Returns `None` when the iterator is exhausted.
    pub fn iter_next(&self, py: Python) -> PyResult<Option<PyObject>> {
        match unsafe { PyObject::from_owned_ptr_opt(py, ffi::PyIter_Next(self.as_ptr())) } {
            Some(obj) => Ok(Some(obj)),
            None => {
                if PyErr::occurred(py) {
                    Err(PyErr::fetch(py))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

