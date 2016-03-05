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

use libc;
use ffi;
use python::{Python, ToPythonPointer, PythonObject, PyClone};
use conversion::ToPyObject;
use objects::{PyObject, PyType};
use std::{mem, ops, ptr, marker};
use err::{self, PyResult};

// note: because rustobject isn't public, these modules aren't visible
// outside the crate
pub mod typebuilder;
pub mod method;
#[cfg(test)]
mod tests;

/// A PythonObject that is usable as a base type with PyTypeBuilder::base().
pub trait PythonBaseObject : PythonObject {
    /// Gets the size of the object, in bytes.
    fn size() -> usize;

    type InitType;

    /// Allocates a new object (usually by calling ty->tp_alloc),
    /// and initializes it using init_val.
    /// `ty` must be derived from the Self type, and the resulting object
    /// must be of type `ty`.
    unsafe fn alloc(py: Python, ty: &PyType, init_val: Self::InitType) -> PyResult<Self>;

    /// Calls the rust destructor for the object and frees the memory
    /// (usually by calling ptr->ob_type->tp_free).
    /// This function is used as tp_dealloc implementation.
    unsafe fn dealloc(py: Python, ptr: *mut ffi::PyObject);
}

impl PythonBaseObject for PyObject {
    #[inline]
    fn size() -> usize {
        mem::size_of::<ffi::PyObject>()
    }

    type InitType = ();

    unsafe fn alloc(py: Python, ty: &PyType, _init_val: ()) -> PyResult<PyObject> {
        let ptr = ffi::PyType_GenericAlloc(ty.as_type_ptr(), 0);
        err::result_from_owned_ptr(py, ptr)
    }

    unsafe fn dealloc(_py: Python, ptr: *mut ffi::PyObject) {
        // Unfortunately, there is no PyType_GenericFree, so
        // we have to manually un-do the work of PyType_GenericAlloc:
        let ty = ffi::Py_TYPE(ptr);
        if ffi::PyType_IS_GC(ty) != 0 {
            ffi::PyObject_GC_Del(ptr as *mut libc::c_void);
        } else {
            ffi::PyObject_Free(ptr as *mut libc::c_void);
        }
        // For heap types, PyType_GenericAlloc calls INCREF on the type objects,
        // so we need to call DECREF here:
        if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
            ffi::Py_DECREF(ty as *mut ffi::PyObject);
        }
    }
}

/// A Python object that contains a rust value of type T,
/// and is derived from base class B.
/// Note that this type effectively acts like `Rc<T>`,
/// except that the reference counting is done by the Python runtime.
#[repr(C)]
pub struct PyRustObject<T, B = PyObject> where T: 'static + Send, B: PythonBaseObject {
    obj: PyObject,
    /// The PyRustObject acts like a shared reference to the contained T.
    t: marker::PhantomData<&'static (T, B)>
}

impl <T, B> PyRustObject<T, B> where T: 'static + Send, B: PythonBaseObject {
    #[inline] // this function can usually be reduced to a compile-time constant
    fn offset() -> usize {
        let align = mem::align_of::<T>();
        // round B::size() up to next multiple of align
        (B::size() + align - 1) / align * align
    }

    /// Gets a reference to this object, but of the base class type.
    #[inline]
    pub fn as_base(&self) -> &B {
        unsafe { B::unchecked_downcast_borrow_from(&self.obj) }
    }

    /// Gets a reference to this object, but of the base class type.
    #[inline]
    pub fn into_base(self) -> B {
        unsafe { B::unchecked_downcast_from(self.obj) }
    }

    /// Gets a reference to the rust value stored in this Python object.
    #[inline]
    pub fn get<'a>(&'a self, _py: Python<'a>) -> &'a T {
        // We require the `Python` token to access the contained value,
        // because `PyRustObject` is `Sync` even if `T` is `!Sync`.
        let offset = PyRustObject::<T, B>::offset() as isize;
        unsafe {
            let ptr = (self.obj.as_ptr() as *mut u8).offset(offset) as *mut T;
            &*ptr
        }
    }
}

impl <T, B> PythonBaseObject for PyRustObject<T, B> where T: 'static + Send, B: PythonBaseObject {
    #[inline]
    fn size() -> usize {
        PyRustObject::<T, B>::offset() + mem::size_of::<T>()
    }

    type InitType = (T, B::InitType);

    unsafe fn alloc(py: Python, ty: &PyType, (val, base_val): Self::InitType) -> PyResult<Self> {
        let obj = try!(B::alloc(py, ty, base_val));
        let offset = PyRustObject::<T, B>::offset() as isize;
        ptr::write((obj.as_object().as_ptr() as *mut u8).offset(offset) as *mut T, val);
        Ok(Self::unchecked_downcast_from(obj.into_object()))
    }

    unsafe fn dealloc(py: Python, obj: *mut ffi::PyObject) {
        let offset = PyRustObject::<T, B>::offset() as isize;
        ptr::read_and_drop((obj as *mut u8).offset(offset) as *mut T);
        B::dealloc(py, obj)
    }
}

impl <T, B> ToPyObject for PyRustObject<T, B> where T: 'static + Send, B: PythonBaseObject {
    type ObjectType = PyObject;

    #[inline]
    fn to_py_object(&self, py: Python) -> PyObject {
        self.obj.clone_ref(py)
    }

    #[inline]
    fn into_py_object(self, _py: Python) -> PyObject {
        self.into_object()
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
      where F: FnOnce(*mut ffi::PyObject) -> R {
        f(self.obj.as_ptr())
    }
}

impl <T, B> PythonObject for PyRustObject<T, B> where T: 'static + Send, B: PythonBaseObject {
    #[inline]
    fn as_object(&self) -> &PyObject {
        &self.obj
    }

    #[inline]
    fn into_object(self) -> PyObject {
        self.obj
    }

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    unsafe fn unchecked_downcast_from(obj: PyObject) -> Self {
        PyRustObject {
            obj: obj,
            t: marker::PhantomData
        }
    }

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a PyObject) -> &'a Self {
        mem::transmute(obj)
    }
}

/// A Python class that contains rust values of type T.
/// Serves as a Python type object, and can be used to construct
/// `PyRustObject<T>` instances.
#[repr(C)]
pub struct PyRustType<T, B = PyObject> where T: 'static + Send, B: PythonBaseObject {
    type_obj: PyType,
    phantom: marker::PhantomData<&'static (B, T)>
}

impl <T, B> PyRustType<T, B> where T: 'static + Send, B: PythonBaseObject {
    /// Creates a PyRustObject instance from a value.
    pub fn create_instance(&self, py: Python, val: T, base_val: B::InitType) -> PyRustObject<T, B> {
        unsafe {
            PythonBaseObject::alloc(py, &self.type_obj, (val, base_val)).unwrap()
        }
    }
}

impl <T, B> ops::Deref for PyRustType<T, B> where T: 'static + Send, B: PythonBaseObject {
    type Target = PyType;

    #[inline]
    fn deref(&self) -> &PyType {
        &self.type_obj
    }
}

impl <T, B> ToPyObject for PyRustType<T, B> where T: 'static + Send, B: PythonBaseObject {
    type ObjectType = PyType;

    #[inline]
    fn to_py_object(&self, py: Python) -> PyType {
        self.type_obj.clone_ref(py)
    }

    #[inline]
    fn into_py_object(self, _py: Python) -> PyType {
        self.type_obj
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
      where F: FnOnce(*mut ffi::PyObject) -> R {
        f(self.as_object().as_ptr())
    }
}

impl <T, B> PythonObject for PyRustType<T, B> where T: 'static + Send, B: PythonBaseObject {
    #[inline]
    fn as_object(&self) -> &PyObject {
        self.type_obj.as_object()
    }

    #[inline]
    fn into_object(self) -> PyObject {
        self.type_obj.into_object()
    }

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    unsafe fn unchecked_downcast_from(obj: PyObject) -> Self {
        PyRustType {
            type_obj: PyType::unchecked_downcast_from(obj),
            phantom: marker::PhantomData
        }
    }

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a PyObject) -> &'a Self {
        mem::transmute(obj)
    }
}

