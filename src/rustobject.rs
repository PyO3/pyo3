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
use objects::{PyObject, PyType, PyString, PyModule, PyDict};
use std::{mem, ops, ptr, marker};
use err::{self, PyResult};

/// A PythonObject that is usable as a base type with PyTypeBuilder::base().
pub trait PythonBaseObject<'p> : PythonObject<'p> {
    /// Gets the size of the object, in bytes.
    fn size() -> usize;
}

impl <'p> PythonBaseObject<'p> for PyObject<'p> {
    #[inline]
    fn size() -> usize {
        mem::size_of::<ffi::PyObject>()
    }
}

/// A python object that contains a rust value of type T,
/// and is derived from base class B.
/// Note that this type effectively acts like `Rc<T>`,
/// except that the reference counting is done by the python runtime.
#[repr(C)]
pub struct PyRustObject<'p, T, B = PyObject<'p>> where T: 'p, B: PythonBaseObject<'p> {
    obj: PyObject<'p>,
    /// The PyRustObject acts like a shared reference to the contained T.
    t: marker::PhantomData<&'p T>,
    b: marker::PhantomData<B>
}

impl <'p, T, B> PyRustObject<'p, T, B> where T: 'p, B: PythonBaseObject<'p> {
    #[inline] // this function can usually be reduced to a compile-time constant
    fn offset() -> usize {
        let align = mem::min_align_of::<T>();
        // round B::size() up to next multiple of align
        (B::size() + align - 1) / align * align
    }

    /// Gets a reference to this object, 
    #[inline]
    pub fn base(&self) -> &B {
        unsafe { B::unchecked_downcast_borrow_from(&self.obj) }
    }

    /// Gets a reference to the rust value stored in this python object.
    #[inline]
    pub fn get(&self) -> &T {
        let offset = PyRustObject::<T, B>::offset() as isize;
        unsafe {
            let ptr = (self.obj.as_ptr() as *mut u8).offset(offset) as *mut T;
            &*ptr
        }
    }
}

impl <'p, T, B> PythonBaseObject<'p> for PyRustObject<'p, T, B> where T: 'p, B: PythonBaseObject<'p> {
    #[inline]
    fn size() -> usize {
        PyRustObject::<T, B>::offset() + mem::size_of::<T>()
    }
}

impl <'p, T, B> Clone for PyRustObject<'p, T, B> where T: 'p, B: PythonBaseObject<'p> {
    #[inline]
    fn clone(&self) -> Self {
        PyRustObject {
            obj: self.obj.clone(), 
            t: marker::PhantomData,
            b: marker::PhantomData
        }
    }
}

impl <'p, T, B> ToPythonPointer for PyRustObject<'p, T, B> where T: 'p, B: PythonBaseObject<'p> {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.obj.as_ptr()
    }

    #[inline]
    fn steal_ptr(self) -> *mut ffi::PyObject {
        self.obj.steal_ptr()
    }
}

impl <'p, T, B> PythonObject<'p> for PyRustObject<'p, T, B> where T: 'p, B: PythonBaseObject<'p> {
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
            t: marker::PhantomData,
            b: marker::PhantomData
        }
    }

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    #[inline]
    unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a PyObject<'p>) -> &'a Self {
        mem::transmute(obj)
    }
}

/// A python class that contains rust values of type T.
/// Serves as a python type object, and can be used to construct
/// `PyRustObject<T>` instances.
#[repr(C)]
pub struct PyRustType<'p, T, B = PyObject<'p>> where T: 'p, B: PythonBaseObject<'p> {
    type_obj: PyType<'p>,
    phantom: marker::PhantomData<&'p (B, T)>
}

impl <'p, T> PyRustType<'p, T, PyObject<'p>> {
    /// Creates a PyRustObject instance from a value.
    pub fn create_instance(&self, val: T) -> PyRustObject<'p, T> {
        let py = self.type_obj.python();
        unsafe {
            let obj = ffi::PyType_GenericAlloc(self.type_obj.as_type_ptr(), 0);
            if obj.is_null() {
                panic!("Out of memory")
            }
            let offset = PyRustObject::<T>::offset() as isize;
            ptr::write((obj as *mut u8).offset(offset) as *mut T, val);
            PyRustObject {
                obj: PyObject::from_owned_ptr(py, obj),
                t: marker::PhantomData,
                b: marker::PhantomData
            }
        }
    }
}

impl <'p, T, B> ops::Deref for PyRustType<'p, T, B> where T: 'p, B: PythonBaseObject<'p> {
    type Target = PyType<'p>;

    #[inline]
    fn deref(&self) -> &PyType<'p> {
        &self.type_obj
    }
}

impl <'p, T: 'p> ToPythonPointer for PyRustType<'p, T> {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.type_obj.as_ptr()
    }

    #[inline]
    fn steal_ptr(self) -> *mut ffi::PyObject {
        self.type_obj.steal_ptr()
    }
}

impl <'p, T: 'p> Clone for PyRustType<'p, T> {
    #[inline]
    fn clone(&self) -> Self {
        PyRustType {
            type_obj: self.type_obj.clone(), 
            phantom: marker::PhantomData
        }
    }
}

impl <'p, T: 'p> PythonObject<'p> for PyRustType<'p, T> {
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

#[repr(C)]
#[must_use]
pub struct PyRustTypeBuilder<'p, T, B = PyObject<'p>> where T: 'p, B: PythonBaseObject<'p> {
    type_obj: PyType<'p>,
    target_module: Option<PyModule<'p>>,
    ht: *mut ffi::PyHeapTypeObject,
    phantom: marker::PhantomData<&'p (B, T)>
}

pub fn new_typebuilder_for_module<'p, T>(m: &PyModule<'p>, name: &str) -> PyRustTypeBuilder<'p, T> {
    let b = PyRustTypeBuilder::new(m.python(), name);
    if let Ok(mod_name) = m.name() {
        b.dict().set_item("__module__", mod_name).ok();
    }
    PyRustTypeBuilder { target_module: Some(m.clone()), .. b }
}
 
impl <'p, T> PyRustTypeBuilder<'p, T> where T: 'p {
    /// Create a new type builder.
    pub fn new(py: Python<'p>, name: &str) -> PyRustTypeBuilder<'p, T> {
        unsafe {
            let obj = ffi::PyType_GenericAlloc(&mut ffi::PyType_Type, 0);
            if obj.is_null() {
                panic!("Out of memory")
            }
            let ht = obj as *mut ffi::PyHeapTypeObject;
            (*ht).ht_name = PyString::new(py, name.as_bytes()).steal_ptr();
            (*ht).ht_type.tp_name = ffi::PyString_AS_STRING((*ht).ht_name);
            (*ht).ht_type.tp_flags = ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_HEAPTYPE;
            PyRustTypeBuilder {
                type_obj: PyType::unchecked_downcast_from(PyObject::from_owned_ptr(py, obj)),
                target_module: None,
                ht: ht,
                phantom: marker::PhantomData
            }
        }
    }

    pub fn base<T2, B2>(self, base_type: &PyRustType<'p, T2, B2>)
        -> PyRustTypeBuilder<'p, T, PyRustObject<'p, T2, B2>>
        where B2: PythonBaseObject<'p>
    {
        unsafe {
            ffi::Py_XDECREF((*self.ht).ht_type.tp_base as *mut ffi::PyObject);
            (*self.ht).ht_type.tp_base = base_type.as_type_ptr();
            ffi::Py_INCREF(base_type.as_ptr());
        }
        PyRustTypeBuilder {
            type_obj: self.type_obj,
            target_module: self.target_module,
            ht: self.ht,
            phantom: marker::PhantomData
        }
    }

    pub fn dict(&self) -> PyDict<'p> {
        let py = self.type_obj.python();
        unsafe {
            if (*self.ht).ht_type.tp_dict.is_null() {
                (*self.ht).ht_type.tp_dict = PyDict::new(py).steal_ptr();
            }
            PyDict::unchecked_downcast_from(PyObject::from_borrowed_ptr(py, (*self.ht).ht_type.tp_dict))
        }
    }
}

impl <'p, T, B> PyRustTypeBuilder<'p, T, B> where T: 'p, B: PythonBaseObject<'p> {
    pub fn finish(self) -> PyResult<'p, PyRustType<'p, T, B>> {
        let py = self.type_obj.python();
        unsafe {
            (*self.ht).ht_type.tp_basicsize = PyRustObject::<T, B>::size() as ffi::Py_ssize_t;
            try!(err::error_on_minusone(py, ffi::PyType_Ready(self.type_obj.as_type_ptr())))
        }
        if let Some(m) = self.target_module {
            // Register the new type in the target module
            let name = unsafe { PyObject::from_borrowed_ptr(py, (*self.ht).ht_name) };
            try!(m.dict().set_item(name, self.type_obj.as_object()));
        }
        Ok(PyRustType {
            type_obj: self.type_obj,
            phantom: marker::PhantomData
        })
    }

    /// Set the doc string on the type being built.
    pub fn doc(self, doc_str: &str) -> Self {
        unsafe {
            if !(*self.ht).ht_type.tp_doc.is_null() {
                ffi::PyObject_Free((*self.ht).ht_type.tp_doc as *mut libc::c_void);
            }
            // ht_type.tp_doc must be allocated with PyObject_Malloc
            let p = ffi::PyObject_Malloc((doc_str.len() + 1) as libc::size_t);
            (*self.ht).ht_type.tp_doc = p as *const libc::c_char;
            if p.is_null() {
                panic!("Out of memory")
            }
            ptr::copy_nonoverlapping(doc_str.as_ptr(), p as *mut u8, doc_str.len() + 1);
        }
        self
    }
}

