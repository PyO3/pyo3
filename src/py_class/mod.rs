// Copyright (c) 2016 Daniel Grunwald
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

mod py_class;
mod slots;

use std::mem;
use python::{self, Python};
use objects::{PyObject, PyType};
use err::PyResult;
use ffi;

/// Trait implemented by the types produced by the `py_class!()` macro.
pub trait PythonObjectFromPyClassMacro : python::PythonObjectWithTypeObject {
    fn initialize(py: Python) -> PyResult<PyType>;
}

#[inline]
#[doc(hidden)]
pub fn data_offset<T>(base_size: usize) -> usize {
    let align = mem::align_of::<T>();
    // round base_size up to next multiple of align
    (base_size + align - 1) / align * align
}

#[inline]
#[doc(hidden)]
pub fn data_new_size<T>(base_size: usize) -> usize {
    data_offset::<T>(base_size) + mem::size_of::<T>()
}

#[inline]
#[doc(hidden)]
pub unsafe fn data_get<'a, T>(_py: Python<'a>, obj: &'a PyObject, offset: usize) -> &'a T {
    let ptr = (obj.as_ptr() as *mut u8).offset(offset as isize) as *const T;
    &*ptr
}

#[inline]
#[doc(hidden)]
pub fn is_ready(_py: Python, ty: &ffi::PyTypeObject) -> bool {
    (ty.tp_flags & ffi::Py_TPFLAGS_READY) != 0
}

