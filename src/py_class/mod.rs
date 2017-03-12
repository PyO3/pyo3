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
#[cfg(feature="python27-sys")]
mod py_class_impl2;
#[cfg(feature="python3-sys")]
mod py_class_impl3;
#[doc(hidden)] pub mod slots;
#[doc(hidden)] pub mod members;
pub mod gc;

use libc;
use std::{mem, ptr, cell};
use python::{self, Python, PythonObject};
use objects::{PyObject, PyType, PyModule};
use err::{self, PyResult};
use ffi;

// TODO: consider moving CompareOp to a different module, so that it isn't exported via two paths
#[derive(Debug)]
pub enum CompareOp {
    Lt = ffi::Py_LT as isize,
    Le = ffi::Py_LE as isize,
    Eq = ffi::Py_EQ as isize,
    Ne = ffi::Py_NE as isize,
    Gt = ffi::Py_GT as isize,
    Ge = ffi::Py_GE as isize
}

/// Trait implemented by the types produced by the `py_class!()` macro.
///
/// This is an unstable implementation detail; do not implement manually!
pub trait PythonObjectFromPyClassMacro : python::PythonObjectWithTypeObject {
    /// Initializes the class.
    ///
    /// module_name: the name of the parent module into which the class will be placed.
    fn initialize(py: Python, module_name: Option<&str>) -> PyResult<PyType>;

    /// Initializes the class and adds it to the module.
    fn add_to_module(py: Python, module: &PyModule) -> PyResult<()>;
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
    let ptr = (obj.as_ptr() as *const u8).offset(offset as isize) as *const T;
    &*ptr
}

#[inline]
#[doc(hidden)]
pub unsafe fn data_init<'a, T>(_py: Python<'a>, obj: &'a PyObject, offset: usize, value: T)
    where T: Send + 'static
{
    let ptr = (obj.as_ptr() as *mut u8).offset(offset as isize) as *mut T;
    ptr::write(ptr, value)
}

#[inline]
#[doc(hidden)]
pub unsafe fn data_drop<'a, T>(_py: Python<'a>, obj: *mut ffi::PyObject, offset: usize) {
    let ptr = (obj as *mut u8).offset(offset as isize) as *mut T;
    ptr::drop_in_place(ptr)
}

#[inline]
#[doc(hidden)]
pub fn is_ready(_py: Python, ty: &ffi::PyTypeObject) -> bool {
    (ty.tp_flags & ffi::Py_TPFLAGS_READY) != 0
}

/// A PythonObject that is usable as a base type with the `py_class!()` macro.
pub trait BaseObject : PythonObject {
    /// Gets the size of the object, in bytes.
    fn size() -> usize;

    type InitType;

    /// Allocates a new object (usually by calling ty->tp_alloc),
    /// and initializes it using init_val.
    /// `ty` must be derived from the Self type, and the resulting object
    /// must be of type `ty`.
    unsafe fn alloc(py: Python, ty: &PyType, init_val: Self::InitType) -> PyResult<PyObject>;

    /// Calls the rust destructor for the object and frees the memory
    /// (usually by calling ptr->ob_type->tp_free).
    /// This function is used as tp_dealloc implementation.
    unsafe fn dealloc(py: Python, obj: *mut ffi::PyObject);
}

impl BaseObject for PyObject {
    #[inline]
    fn size() -> usize {
        mem::size_of::<ffi::PyObject>()
    }

    type InitType = ();

    unsafe fn alloc(py: Python, ty: &PyType, _init_val: ()) -> PyResult<PyObject> {
        let ptr = ffi::PyType_GenericAlloc(ty.as_type_ptr(), 0);
        //println!("BaseObject::alloc({:?}) = {:?}", ty.as_type_ptr(), ptr);
        err::result_from_owned_ptr(py, ptr)
    }

    unsafe fn dealloc(_py: Python, obj: *mut ffi::PyObject) {
        //println!("BaseObject::dealloc({:?})", ptr);
        // Unfortunately, there is no PyType_GenericFree, so
        // we have to manually un-do the work of PyType_GenericAlloc:
        let ty = ffi::Py_TYPE(obj);
        if ffi::PyType_IS_GC(ty) != 0 {
            ffi::PyObject_GC_Del(obj as *mut libc::c_void);
        } else {
            ffi::PyObject_Free(obj as *mut libc::c_void);
        }
        // For heap types, PyType_GenericAlloc calls INCREF on the type objects,
        // so we need to call DECREF here:
        if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
            ffi::Py_DECREF(ty as *mut ffi::PyObject);
        }
    }
}

