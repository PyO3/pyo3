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
use python::{Python, ToPythonPointer, PythonObject};
use conversion::ToPyObject;
use objects::{PyObject, PyType, PyString, PyModule, PyDict};
use std::{mem, ops, ptr, marker};
use err::{self, PyResult};

pub mod typebuilder;
pub mod method;
#[cfg(test)]
mod tests;

/// A PythonObject that is usable as a base type with PyTypeBuilder::base().
pub trait PythonBaseObject<'p> : PythonObject<'p> {
    /// Gets the size of the object, in bytes.
    fn size() -> usize;

    type InitType : 'p;

    /// Allocates a new object (usually by calling ty->tp_alloc),
    /// and initializes it using init_val.
    /// `ty` must be derived from the Self type, and the resulting object
    /// must be of type `ty`.
    unsafe fn alloc(ty: &PyType<'p>, init_val: Self::InitType) -> PyResult<'p, Self>;

    /// Calls the rust destructor for the object and frees the memory
    /// (usually by calling ptr->ob_type->tp_free).
    /// This function is used as tp_dealloc implementation.
    unsafe fn dealloc(ptr: *mut ffi::PyObject);
}

impl <'p> PythonBaseObject<'p> for PyObject<'p> {
    #[inline]
    fn size() -> usize {
        mem::size_of::<ffi::PyObject>()
    }

    type InitType = ();

    unsafe fn alloc(ty: &PyType<'p>, init_val: ()) -> PyResult<'p, PyObject<'p>> {
        let py = ty.python();
        let ptr = ((*ty.as_type_ptr()).tp_alloc.unwrap())(ty.as_type_ptr(), 0);
        err::result_from_owned_ptr(py, ptr)
    }

    unsafe fn dealloc(ptr: *mut ffi::PyObject) {
        let ty = ffi::Py_TYPE(ptr);
        ((*ty).tp_free.unwrap())(ptr as *mut libc::c_void);
        // For heap types, tp_alloc calls INCREF on the type objects,
        // but tp_free points directly to the memory deallocator and does not call DECREF.
        // So we'll do that manually here:
        if ((*ty).tp_flags & ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
            ffi::Py_DECREF(ty as *mut ffi::PyObject);
        }
    }
}

/// A Python object that contains a rust value of type T,
/// and is derived from base class B.
/// Note that this type effectively acts like `Rc<T>`,
/// except that the reference counting is done by the Python runtime.
#[repr(C)]
pub struct PyRustObject<'p, T, B = PyObject<'p>> where T: 'static, B: PythonBaseObject<'p> {
    obj: PyObject<'p>,
    /// The PyRustObject acts like a shared reference to the contained T.
    t: marker::PhantomData<&'p (T, B)>
}

impl <'p, T, B> PyRustObject<'p, T, B> where T: 'static + Send, B: PythonBaseObject<'p> {
    #[inline] // this function can usually be reduced to a compile-time constant
    fn offset() -> usize {
        let align = mem::align_of::<T>();
        // round B::size() up to next multiple of align
        (B::size() + align - 1) / align * align
    }

    /// Gets a reference to this object, but of the base class type.
    #[inline]
    pub fn base(&self) -> &B {
        unsafe { B::unchecked_downcast_borrow_from(&self.obj) }
    }

    /// Gets a reference to the rust value stored in this Python object.
    #[inline]
    pub fn get(&self) -> &T {
        let offset = PyRustObject::<T, B>::offset() as isize;
        unsafe {
            let ptr = (self.obj.as_ptr() as *mut u8).offset(offset) as *mut T;
            &*ptr
        }
    }
}

impl <'p, T, B> PythonBaseObject<'p> for PyRustObject<'p, T, B> where T: 'static + Send, B: PythonBaseObject<'p> {
    #[inline]
    fn size() -> usize {
        PyRustObject::<T, B>::offset() + mem::size_of::<T>()
    }

    type InitType = (T, B::InitType);

    unsafe fn alloc(ty: &PyType<'p>, (val, base_val): Self::InitType) -> PyResult<'p, Self> {
        let obj = try!(B::alloc(ty, base_val));
        let offset = PyRustObject::<T, B>::offset() as isize;
        ptr::write((obj.as_object().as_ptr() as *mut u8).offset(offset) as *mut T, val);
        Ok(Self::unchecked_downcast_from(obj.into_object()))
    }

    unsafe fn dealloc(obj: *mut ffi::PyObject) {
        let offset = PyRustObject::<T, B>::offset() as isize;
        ptr::read_and_drop((obj as *mut u8).offset(offset) as *mut T);
        B::dealloc(obj)
    }
}

impl <'p, T, B> Clone for PyRustObject<'p, T, B> where T: 'static + Send, B: PythonBaseObject<'p> {
    #[inline]
    fn clone(&self) -> Self {
        PyRustObject {
            obj: self.obj.clone(), 
            t: marker::PhantomData
        }
    }
}

impl <'p, 's, T, B> ToPyObject<'p> for PyRustObject<'s, T, B> where T: 'static + Send, B: PythonBaseObject<'s> {
    type ObjectType = PyObject<'p>;

    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> PyObject<'p> {
        self.as_object().to_py_object(py)
    }

    #[inline]
    fn into_py_object(self, py: Python<'p>) -> PyObject<'p> {
        self.into_object().into_py_object(py)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python<'p>, f: F) -> R
      where F: FnOnce(*mut ffi::PyObject) -> R {
        f(self.as_ptr())
    }
}

impl <'p, T, B> PythonObject<'p> for PyRustObject<'p, T, B> where T: 'static + Send, B: PythonBaseObject<'p> {
    #[inline]
    fn as_object(&self) -> &PyObject<'p> {
        &self.obj
    }

    #[inline]
    fn into_object(self) -> PyObject<'p> {
        self.obj
    }

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    unsafe fn unchecked_downcast_from(obj: PyObject<'p>) -> Self {
        PyRustObject {
            obj: obj,
            t: marker::PhantomData
        }
    }

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a PyObject<'p>) -> &'a Self {
        mem::transmute(obj)
    }
}

/// A Python class that contains rust values of type T.
/// Serves as a Python type object, and can be used to construct
/// `PyRustObject<T>` instances.
#[repr(C)]
pub struct PyRustType<'p, T, B = PyObject<'p>> where T: 'p + Send, B: PythonBaseObject<'p> {
    type_obj: PyType<'p>,
    phantom: marker::PhantomData<&'p (B, T)>
}

impl <'p, T, B> PyRustType<'p, T, B> where T: 'p + Send, B: PythonBaseObject<'p> {
    /// Creates a PyRustObject instance from a value.
    pub fn create_instance(&self, val: T, base_val: B::InitType) -> PyRustObject<'p, T, B> {
        let py = self.type_obj.python();
        unsafe {
            PythonBaseObject::alloc(&self.type_obj, (val, base_val)).unwrap()
        }
    }
}

impl <'p, T, B> ops::Deref for PyRustType<'p, T, B> where T: 'p + Send, B: PythonBaseObject<'p> {
    type Target = PyType<'p>;

    #[inline]
    fn deref(&self) -> &PyType<'p> {
        &self.type_obj
    }
}

impl <'p, T> Clone for PyRustType<'p, T> where T: 'p + Send {
    #[inline]
    fn clone(&self) -> Self {
        PyRustType {
            type_obj: self.type_obj.clone(), 
            phantom: marker::PhantomData
        }
    }
}

impl <'p, T> PythonObject<'p> for PyRustType<'p, T> where T: 'p + Send {
    #[inline]
    fn as_object(&self) -> &PyObject<'p> {
        self.type_obj.as_object()
    }

    #[inline]
    fn into_object(self) -> PyObject<'p> {
        self.type_obj.into_object()
    }

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    unsafe fn unchecked_downcast_from(obj: PyObject<'p>) -> Self {
        PyRustType {
            type_obj: PyType::unchecked_downcast_from(obj),
            phantom: marker::PhantomData
        }
    }

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a PyObject<'p>) -> &'a Self {
        mem::transmute(obj)
    }
}

