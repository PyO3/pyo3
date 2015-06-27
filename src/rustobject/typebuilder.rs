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
use super::{PythonBaseObject, PyRustObject, PyRustType};

#[repr(C)]
#[must_use]
pub struct PyRustTypeBuilder<'p, T, B = PyObject<'p>> where T: 'static + Send, B: PythonBaseObject<'p> {
    type_obj: PyType<'p>,
    target_module: Option<PyModule<'p>>,
    ht: *mut ffi::PyHeapTypeObject,
    phantom: marker::PhantomData<&'p (B, T)>
}

pub fn new_typebuilder_for_module<'p, T>(m: &PyModule<'p>, name: &str) -> PyRustTypeBuilder<'p, T>
        where T: 'static + Send {
    let b = PyRustTypeBuilder::new(m.python(), name);
    if let Ok(mod_name) = m.name() {
        b.dict().set_item("__module__", mod_name).ok();
    }
    PyRustTypeBuilder { target_module: Some(m.clone()), .. b }
}

unsafe extern "C" fn disabled_tp_new_callback
    (subtype: *mut ffi::PyTypeObject, args: *mut ffi::PyObject, kwds: *mut ffi::PyObject)
    -> *mut ffi::PyObject {
    ffi::PyErr_SetString(ffi::PyExc_TypeError,
        b"Cannot initialize rust object from python.\0" as *const u8 as *const libc::c_char);
    ptr::null_mut()
}

unsafe extern "C" fn tp_dealloc_callback<'p, T, B>(obj: *mut ffi::PyObject)
        where T: 'static + Send, B: PythonBaseObject<'p> {
    abort_on_panic!({
        PyRustObject::<T, B>::dealloc(obj)
    });
}

impl <'p, T> PyRustTypeBuilder<'p, T> where T: 'static + Send {
    /// Create a new type builder.
    ///
    /// py: proof that the GIL is held by the current thread.
    /// name: name of the new type
    pub fn new(py: Python<'p>, name: &str) -> PyRustTypeBuilder<'p, T> {
        unsafe {
            let obj = (ffi::PyType_Type.tp_alloc.unwrap())(&mut ffi::PyType_Type, 0);
            if obj.is_null() {
                panic!("Out of memory")
            }
            debug_assert!(ffi::Py_REFCNT(obj) == 1);
            let ht = obj as *mut ffi::PyHeapTypeObject;
            (*ht).ht_name = PyString::new(py, name.as_bytes()).steal_ptr();
            (*ht).ht_type.tp_name = ffi::PyString_AS_STRING((*ht).ht_name);
            (*ht).ht_type.tp_flags = ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_HEAPTYPE;
            (*ht).ht_type.tp_new = Some(disabled_tp_new_callback);
            PyRustTypeBuilder {
                type_obj: PyType::unchecked_downcast_from(PyObject::from_owned_ptr(py, obj)),
                target_module: None,
                ht: ht,
                phantom: marker::PhantomData
            }
        }
    }

    /// Sets the base class that this type is inheriting from.
    pub fn base<T2, B2>(self, base_type: &PyRustType<'p, T2, B2>)
        -> PyRustTypeBuilder<'p, T, PyRustObject<'p, T2, B2>>
        where T2: 'static + Send, B2: PythonBaseObject<'p>
    {
        unsafe {
            ffi::Py_XDECREF((*self.ht).ht_type.tp_base as *mut ffi::PyObject);
            (*self.ht).ht_type.tp_base = base_type.as_type_ptr();
            ffi::Py_INCREF(base_type.as_object().as_ptr());
        }
        PyRustTypeBuilder {
            type_obj: self.type_obj,
            target_module: self.target_module,
            ht: self.ht,
            phantom: marker::PhantomData
        }
    }

}

impl <'p, T, B> PyRustTypeBuilder<'p, T, B> where T: 'static + Send, B: PythonBaseObject<'p> {

    /// Retrieves the type dictionary of the type being built.
    pub fn dict(&self) -> PyDict<'p> {
        let py = self.type_obj.python();
        unsafe {
            if (*self.ht).ht_type.tp_dict.is_null() {
                (*self.ht).ht_type.tp_dict = PyDict::new(py).steal_ptr();
            }
            PyDict::unchecked_downcast_from(PyObject::from_borrowed_ptr(py, (*self.ht).ht_type.tp_dict))
        }
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

    /// Adds a new member to the type.
    pub fn add<M>(self, name: &str, val: M) -> Self
            where M: TypeMember<'p, PyRustObject<'p, T, B>> {
        self.dict().set_item(name, val.into_descriptor(&self.type_obj, name)).unwrap();
        self
    }

    /// Finalize construction of the new type.
    pub fn finish(self) -> PyResult<'p, PyRustType<'p, T, B>> {
        let py = self.type_obj.python();
        unsafe {
            (*self.ht).ht_type.tp_basicsize = PyRustObject::<T, B>::size() as ffi::Py_ssize_t;
            (*self.ht).ht_type.tp_dealloc = Some(tp_dealloc_callback::<T, B>);
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

}


/// Represents something that can be added as a member to a python class/type.
///
/// T: type of rust class used for instances of the python class/type.
pub trait TypeMember<'p, T> where T: PythonObject<'p> {
    fn into_descriptor(self, ty: &PyType<'p>, name: &str) -> PyObject<'p>;
}

// TODO: does this cause trouble for coherence?
impl <'p, T, S> TypeMember<'p, T> for S where T: PythonObject<'p>, S: ToPyObject<'p> {
    #[inline]
    fn into_descriptor(self, ty: &PyType<'p>, name: &str) -> PyObject<'p> {
        self.into_py_object(ty.python()).into_object()
    }
}

impl <'p, T> TypeMember<'p, T> for fn(&T) where T: PythonObject<'p> {
    fn into_descriptor(self, ty: &PyType<'p>, name: &str) -> PyObject<'p> {
        unimplemented!()
    }
}


