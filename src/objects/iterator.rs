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

use python::{Python, PythonObject, ToPythonPointer, PythonObjectDowncastError};
use conversion::ToPyObject;
use objects::PyObject;
use err::{PyErr, PyResult};
use ffi;

/// A python iterator object.
///
/// Unlike other python objects, this class includes a `Python<'p>` token
/// so that PyIterator can implement the rust `Iterator` trait.
pub struct PyIterator<'p> {
    py: Python<'p>,
    iter: PyObject,
}

impl <'p> PyIterator<'p> {
    /// Constructs a PyIterator from a Python iterator object.
    pub fn from_object(py: Python<'p>, obj: PyObject) -> Result<PyIterator<'p>, PythonObjectDowncastError<'p>> {
        if unsafe { ffi::PyIter_Check(obj.as_ptr()) != 0 } {
            Ok(PyIterator { py: py, iter: obj })
        } else {
            Err(PythonObjectDowncastError(py))
        }
    }

    /// Gets the Python iterator object.
    #[inline]
    pub fn as_object(&self) -> &PyObject {
        &self.iter
    }

    /// Gets the Python iterator object.
    #[inline]
    pub fn into_object(self) -> PyObject {
        self.iter
    }
}

impl <'p> Iterator for PyIterator<'p> {
    type Item = PyResult<PyObject>;

    /// Retrieves the next item from an iterator.
    /// Returns `None` when the iterator is exhausted.
    /// If an exception occurs, returns `Some(Err(..))`.
    /// Further next() calls after an exception occurs are likely
    /// to repeatedly result in the same exception.
    fn next(&mut self) -> Option<PyResult<PyObject>> {
        let py = self.py;
        match unsafe { PyObject::from_owned_ptr_opt(py, ffi::PyIter_Next(self.iter.as_ptr())) } {
            Some(obj) => Some(Ok(obj)),
            None => {
                if PyErr::occurred(py) {
                    Some(Err(PyErr::fetch(py)))
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use python::{Python, PythonObject};
    use conversion::ToPyObject;
    use objectprotocol::ObjectProtocol;

    #[test]
    fn vec_iter() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let obj = vec![10, 20].to_py_object(py).into_object();
        let mut it = obj.iter(py).unwrap();
        assert_eq!(10, it.next().unwrap().unwrap().extract(py).unwrap());
        assert_eq!(20, it.next().unwrap().unwrap().extract(py).unwrap());
        assert!(it.next().is_none());
    }
}

