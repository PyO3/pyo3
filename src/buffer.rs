// Copyright (c) 2017 Daniel Grunwald
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

use std::{mem, slice};
use std::ffi::CStr;
use ffi;
use libc;
use err::{self, PyResult};
use python::{Python, PyDrop};
use objects::PyObject;

/// Allows access to the underlying buffer used by a python object such as `bytes`, `bytearray` or `array.array`.
pub struct PyBuffer(Box<ffi::Py_buffer>); // use Box<> because Python expects that the Py_buffer struct has a stable memory address

impl PyBuffer {
    /// Get the underlying buffer from the specified python object.
    pub fn get(py: Python, obj: &PyObject) -> PyResult<PyBuffer> {
        unsafe {
            let mut buf = Box::new(mem::zeroed::<ffi::Py_buffer>());
            err::error_on_minusone(py, ffi::PyObject_GetBuffer(obj.as_ptr(), &mut *buf, ffi::PyBUF_FULL_RO))?;
            Ok(PyBuffer(buf))
        }
    }

    /// Gets the pointer to the start of the buffer memory.
    #[inline]
    pub fn buf_ptr(&self) -> *mut libc::c_void {
        self.0.buf
    }

    /// Gets whether the underlying buffer is read-only.
    #[inline]
    pub fn readonly(&self) -> bool {
        self.0.readonly != 0
    }

    /// Gets the size of a single element, in bytes.
    /// Important exception: when requesting an unformatted buffer, item_size still has the value 
    #[inline]
    pub fn item_size(&self) -> usize {
        self.0.itemsize as usize
    }

    /// Gets the total number of items.
    #[inline]
    pub fn item_count(&self) -> usize {
        (self.0.len as usize) / (self.0.itemsize as usize)
    }

    /// `item_size() * item_count()`.
    /// For contiguous arrays, this is the length of the underlying memory block.
    /// For non-contiguous arrays, it is the length that the logical structure would have if it were copied to a contiguous representation.
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.0.len as usize
    }

    /// Gets the number of dimensions.
    ///
    /// May be 0 to indicate a single scalar value.
    #[inline]
    pub fn dimensions(&self) -> usize {
        self.0.ndim as usize
    }

    /// Returns an array of length `dimensions`. `shape()[i]` is the length of the array in dimension number `i`.
    ///
    /// May return None for single-dimensional arrays or scalar values (`dimensions() <= 1`);
    /// You can call `item_count()` to get the length of the single dimension.
    ///
    /// Despite Python using an array of signed integers, the values are guaranteed to be non-negative.
    /// However, dimensions of length 0 are possible and might need special attention.
    #[inline]
    pub fn shape(&self) -> Option<&[ffi::Py_ssize_t]> {
        unsafe {
            if self.0.shape.is_null() {
                None
            } else {
                Some(slice::from_raw_parts(self.0.shape, self.0.ndim as usize))
            }
        }
    }

    /// Returns an array that holds, for each dimension, the number of bytes to skip to get to the next element in the dimension.
    ///
    /// Stride values can be any integer. For regular arrays, strides are usually positive,
    /// but a consumer MUST be able to handle the case `strides[n] <= 0`.
    ///
    /// If this function returns `None`, the array is C-contiguous.
    #[inline]
    pub fn strides(&self) -> Option<&[ffi::Py_ssize_t]> {
        unsafe {
            if self.0.strides.is_null() {
                None
            } else {
                Some(slice::from_raw_parts(self.0.strides, self.0.ndim as usize))
            }
        }
    }

    #[inline]
    pub fn suboffsets(&self) -> Option<&[ffi::Py_ssize_t]> {
        unsafe {
            if self.0.suboffsets.is_null() {
                None
            } else {
                Some(slice::from_raw_parts(self.0.suboffsets, self.0.ndim as usize))
            }
        }
    }

    /// A NUL terminated string in struct module style syntax describing the contents of a single item.
    #[inline]
    pub fn format(&self) -> &CStr {
        if self.0.format.is_null() {
            cstr!("B")
        } else {
            unsafe { CStr::from_ptr(self.0.format) }
        }
    }

    /// Gets whether the buffer is contiguous in C-style order (last index varies fastest when visiting items in order of memory address).
    #[inline]
    pub fn is_c_contiguous(&self) -> bool {
        unsafe {
            // Python 2.7 is not const-correct, so we need the cast to *mut
            ffi::PyBuffer_IsContiguous(&*self.0 as *const ffi::Py_buffer as *mut ffi::Py_buffer, b'C' as libc::c_char) != 0
        }
    }

    /// Gets whether the buffer is contiguous in C-style order (first index varies fastest when visiting items in order of memory address).
    #[inline]
    pub fn is_fortran_contiguous(&self) -> bool {
        unsafe {
            // Python 2.7 is not const-correct, so we need the cast to *mut
            ffi::PyBuffer_IsContiguous(&*self.0 as *const ffi::Py_buffer as *mut ffi::Py_buffer, b'F' as libc::c_char) != 0
        }
    }
}

impl PyDrop for PyBuffer {
    #[inline]
    fn release_ref(mut self, _py: Python) {
        unsafe { ffi::PyBuffer_Release(&mut *self.0) }
    }
}

impl Drop for PyBuffer {
    fn drop(&mut self) {
        let _gil_guard = Python::acquire_gil();
        unsafe { ffi::PyBuffer_Release(&mut *self.0) }
    }
}


#[cfg(test)]
mod test {
    use std;
    use python::{Python, PythonObject, PyDrop};
    use conversion::ToPyObject;
    use objects::{PySequence, PyList, PyTuple, PyIterator};
    use objectprotocol::ObjectProtocol;
    use super::PyBuffer;

    #[test]
    fn test_bytes_buffer() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let bytes = py.eval("b'abcde'", None, None).unwrap();
        let buffer = PyBuffer::get(py, &bytes).unwrap();
        assert_eq!(buffer.dimensions(), 1);
        assert_eq!(buffer.item_count(), 5);
        assert_eq!(buffer.format().to_str().unwrap(), "B");
        assert_eq!(buffer.shape(), Some::<&[::Py_ssize_t]>(&[5]));
        // single-dimensional buffer is always contiguous
        assert!(buffer.is_c_contiguous());
        assert!(buffer.is_fortran_contiguous());
    }

    #[test]
    #[cfg(feature="python3-sys")] // array.array doesn't implement the buffer protocol in python 2.7
    fn test_array_buffer() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let array = py.import("array").unwrap().as_object().call_method(py, "array", ("f", (1.0, 1.5, 2.0, 2.5)), None).unwrap();
        let buffer = PyBuffer::get(py, &array).unwrap();
        assert_eq!(buffer.dimensions(), 1);
        assert_eq!(buffer.item_count(), 4);
        assert_eq!(buffer.format().to_str().unwrap(), "f");
        assert_eq!(buffer.shape(), Some::<&[::Py_ssize_t]>(&[4]));
    }
}

