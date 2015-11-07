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

use std::{ptr, marker};
use std::ffi::{CStr, CString};
use libc;
use ffi;
use python::{Python, ToPythonPointer, PythonObject, PyClone};
use conversion::ToPyObject;
use objects::{PyObject, PyType, PyString, PyModule, PyDict};
use err::{self, PyResult};
use objectprotocol::ObjectProtocol;
use super::{PythonBaseObject, PyRustObject, PyRustType};

#[repr(C)]
#[must_use]
pub struct PyRustTypeBuilder<'p, T, B = PyObject> where T: 'static + Send, B: PythonBaseObject {
    // In Python 2.7, we can create a new PyHeapTypeObject and fill it.

    /// The python type object under construction.
    #[cfg(feature="python27-sys")]
    type_obj: PyType,
    /// The full PyHeapTypeObject under construction.
    #[cfg(feature="python27-sys")]
    ht: *mut ffi::PyHeapTypeObject,

    // In Python 3.x with PEP 384, we prepare the relevant
    // information and then create the type in `finish()`.

    /// Name of the type to be created
    #[cfg(feature="python3-sys")]
    name: CString,
    /// Flags of the type to be created
    #[cfg(feature="python3-sys")]
    flags: libc::c_uint,
    /// Slots to use when creating the type
    #[cfg(feature="python3-sys")]
    slots: Vec<ffi::PyType_Slot>,
    /// Maintains owned reference for base type object
    #[cfg(feature="python3-sys")]
    tp_base: Option<PyType>,
    /// List of future type members
    #[cfg(feature="python3-sys")]
    members: Vec<(String, Box<TypeMember<PyRustObject<T, B>>>)>,

    /// The documentation string.
    doc_str: Option<CString>,
    /// The module to which the new type should be added.
    target_module: Option<PyModule>,
    /// Whether PyTypeBuilder::base() might be called
    can_change_base: bool,
    py: Python<'p>,
    phantom: marker::PhantomData<&'p (B, T)>
}

pub fn new_typebuilder_for_module<'p, T>(py: Python<'p>, m: &PyModule, name: &str) -> PyRustTypeBuilder<'p, T>
        where T: 'static + Send {
    let b = PyRustTypeBuilder::new(py, name);
    PyRustTypeBuilder { target_module: Some(m.clone_ref(py)), .. b }
}

unsafe extern "C" fn disabled_tp_new_callback
    (_subtype: *mut ffi::PyTypeObject, _args: *mut ffi::PyObject, _kwds: *mut ffi::PyObject)
    -> *mut ffi::PyObject {
    ffi::PyErr_SetString(ffi::PyExc_TypeError,
        b"Cannot initialize rust object from python.\0" as *const u8 as *const libc::c_char);
    ptr::null_mut()
}

unsafe extern "C" fn tp_dealloc_callback<T, B>(obj: *mut ffi::PyObject)
        where T: 'static + Send, B: PythonBaseObject {
    abort_on_panic!({
        let py = Python::assume_gil_acquired();
        PyRustObject::<T, B>::dealloc(py, obj)
    });
}

impl <'p, T> PyRustTypeBuilder<'p, T> where T: 'static + Send {
    /// Create a new type builder.
    ///
    /// py: proof that the GIL is held by the current thread.
    /// name: name of the new type
    #[cfg(feature="python27-sys")]
    pub fn new(py: Python<'p>, name: &str) -> PyRustTypeBuilder<'p, T> {
        unsafe {
            let obj = (ffi::PyType_Type.tp_alloc.unwrap())(&mut ffi::PyType_Type, 0);
            if obj.is_null() {
                panic!("Out of memory")
            }
            debug_assert!(ffi::Py_REFCNT(obj) == 1);
            let ht = obj as *mut ffi::PyHeapTypeObject;
            // flags must be set first, before the GC traverses the object
            (*ht).ht_type.tp_flags = ffi::Py_TPFLAGS_DEFAULT | ffi::Py_TPFLAGS_HEAPTYPE;
            (*ht).ht_name = PyString::new(py, name.as_bytes()).steal_ptr(py);
            (*ht).ht_type.tp_name = ffi::PyString_AS_STRING((*ht).ht_name);
            (*ht).ht_type.tp_new = Some(disabled_tp_new_callback);
            PyRustTypeBuilder {
                type_obj: PyType::unchecked_downcast_from(PyObject::from_owned_ptr(py, obj)),
                doc_str: None,
                target_module: None,
                ht: ht,
                can_change_base: true,
                py: py,
                phantom: marker::PhantomData
            }
        }
    }

    /// Create a new type builder.
    ///
    /// py: proof that the GIL is held by the current thread.
    /// name: name of the new type
    #[cfg(feature="python3-sys")]
    pub fn new(py: Python<'p>, name: &str) -> PyRustTypeBuilder<'p, T> {
        PyRustTypeBuilder {
            name: CString::new(name).unwrap(),
            flags: ffi::Py_TPFLAGS_DEFAULT as libc::c_uint,
            slots: Vec::new(),
            tp_base: None,
            members: Vec::new(),
            target_module: None,
            doc_str: None,
            can_change_base: true,
            py: py,
            phantom: marker::PhantomData
        }
    }

    /// Sets the base class that this type is inheriting from.
    #[cfg(feature="python27-sys")]
    pub fn base<T2, B2>(self, base_type: &PyRustType<T2, B2>)
        -> PyRustTypeBuilder<'p, T, PyRustObject<T2, B2>>
        where T2: 'static + Send, B2: PythonBaseObject
    {
        assert!(self.can_change_base,
            "base() must be called before any members are added to the type");
        unsafe {
            ffi::Py_XDECREF((*self.ht).ht_type.tp_base as *mut ffi::PyObject);
            (*self.ht).ht_type.tp_base = base_type.as_type_ptr();
            ffi::Py_INCREF(base_type.as_object().as_ptr());
        }
        PyRustTypeBuilder {
            type_obj: self.type_obj,
            doc_str: self.doc_str,
            target_module: self.target_module,
            ht: self.ht,
            can_change_base: false,
            py: self.py,
            phantom: marker::PhantomData
        }
    }

    /// Sets the base class that this type is inheriting from.
    #[cfg(feature="python3-sys")]
    pub fn base<T2, B2>(self, base_type: &PyRustType<T2, B2>)
        -> PyRustTypeBuilder<'p, T, PyRustObject<T2, B2>>
        where T2: 'static + Send, B2: PythonBaseObject
    {
        // Ensure we can't change the base after any callbacks are registered.
        assert!(self.can_change_base && self.members.is_empty(),
            "base() must be called before any members are added to the type");
        let base_type_obj: &PyType = base_type;
        PyRustTypeBuilder {
            name: self.name,
            flags: self.flags,
            slots: self.slots,
            tp_base: Some(base_type_obj.clone_ref(self.py)),
            members: Vec::new(),
            target_module: self.target_module,
            doc_str: self.doc_str,
            can_change_base: false,
            py: self.py,
            phantom: marker::PhantomData
        }
    }
}

impl <'p, T, B> PyRustTypeBuilder<'p, T, B> where T: 'static + Send, B: PythonBaseObject {

    /// Retrieves the type dictionary of the type being built.
    #[cfg(feature="python27-sys")]
    fn dict(&self) -> PyDict {
        unsafe {
            if (*self.ht).ht_type.tp_dict.is_null() {
                (*self.ht).ht_type.tp_dict = PyDict::new(self.py).steal_ptr(self.py);
            }
            PyDict::unchecked_downcast_from(PyObject::from_borrowed_ptr(self.py, (*self.ht).ht_type.tp_dict))
        }
    }

    /// Set the doc string on the type being built.
    pub fn doc(self, doc_str: &str) -> Self {
        PyRustTypeBuilder { doc_str: Some(CString::new(doc_str).unwrap()), .. self }
    }

    /// Adds a new member to the type.
    #[cfg(feature="python27-sys")]
    pub fn add<M>(mut self, name: &str, val: M) -> Self
            where M: TypeMember<PyRustObject<T, B>> {
        self.can_change_base = false;
        self.dict().set_item(self.py, name, val.to_descriptor(self.py, &self.type_obj, name)).unwrap();
        self
    }

    /// Adds a new member to the type.
    #[cfg(feature="python3-sys")]
    pub fn add<M>(mut self, name: &str, val: M) -> Self
            where M: TypeMember<PyRustObject<T, B>> {
        self.can_change_base = false;
        self.members.push((name.to_owned(), val.into_box(self.py)));
        self
    }

    /// Finalize construction of the new type.
    #[cfg(feature="python27-sys")]
    pub fn finish(self) -> PyResult<PyRustType<T, B>> {
        let py = self.py;
        unsafe {
            (*self.ht).ht_type.tp_basicsize = PyRustObject::<T, B>::size() as ffi::Py_ssize_t;
            (*self.ht).ht_type.tp_dealloc = Some(tp_dealloc_callback::<T, B>);
            if let Some(s) = self.doc_str {
                (*self.ht).ht_type.tp_doc = copy_str_to_py_malloc_heap(&s);
            }
            try!(err::error_on_minusone(py, ffi::PyType_Ready(self.type_obj.as_type_ptr())))
        }
        if let Some(m) = self.target_module {
            // Set module name for new type
            if let Ok(mod_name) = m.name(py) {
                try!(self.type_obj.as_object().setattr(py, "__module__", mod_name));
            }
            // Register the new type in the target module
            let name = unsafe { PyObject::from_borrowed_ptr(py, (*self.ht).ht_name) };
            try!(m.dict(py).set_item(py, name, self.type_obj.as_object()));
        }
        Ok(PyRustType {
            type_obj: self.type_obj,
            phantom: marker::PhantomData
        })
    }

    /// Finalize construction of the new type.
    #[cfg(feature="python3-sys")]
    pub fn finish(mut self) -> PyResult<PyRustType<T, B>> {
        // push some more slots
        self.slots.push(ffi::PyType_Slot {
            slot: ffi::Py_tp_dealloc,
            pfunc: tp_dealloc_callback::<T, B> as ffi::destructor as *mut libc::c_void
        });
        if let Some(s) = self.doc_str {
            self.slots.push(ffi::PyType_Slot {
                slot: ffi::Py_tp_doc,
                pfunc: copy_str_to_py_malloc_heap(&s) as *mut libc::c_void
            });
        }
        if let Some(base_type) = self.tp_base {
            self.slots.push(ffi::PyType_Slot {
                slot: ffi::Py_tp_base,
                pfunc: base_type.as_type_ptr() as *mut libc::c_void
            });
        }

        let type_obj = try!(unsafe { create_type_from_slots(
            self.py, &self.name, PyRustObject::<T, B>::size(),
            self.flags, &mut self.slots) });
        for (name, member) in self.members {
            let descr = member.to_descriptor(self.py, &type_obj, &name);
            try!(type_obj.as_object().setattr(self.py, name, descr));
        }
        if let Some(m) = self.target_module {
            // Set module name for new type
            if let Ok(mod_name) = m.name(self.py) {
                try!(type_obj.as_object().setattr(self.py, "__module__", mod_name));
            }
            // Register the new type in the target module
            unsafe {
                try!(err::error_on_minusone(self.py,
                    ffi::PyDict_SetItemString(
                        m.dict(self.py).as_object().as_ptr(), 
                        self.name.as_ptr(),
                        type_obj.as_object().as_ptr())
                ));
            }
        }
        Ok(PyRustType {
            type_obj: type_obj,
            phantom: marker::PhantomData
        })
    }

}

fn copy_str_to_py_malloc_heap(s: &CStr) -> *mut libc::c_char {
    copy_to_py_malloc_heap(s.to_bytes_with_nul()) as *mut libc::c_char
}

fn copy_to_py_malloc_heap(s: &[u8]) -> *mut u8 {
    unsafe {
        let p = ffi::PyObject_Malloc(s.len() as libc::size_t) as *mut u8;
        if p.is_null() {
            panic!("Out of memory")
        }
        ptr::copy_nonoverlapping(s.as_ptr(), p, s.len());
        p
    }
}

#[cfg(feature="python3-sys")]
unsafe fn create_type_from_slots<'p>(
    py: Python<'p>,
    name: &CStr,
    basicsize: usize,
    flags: libc::c_uint,
    slots: &mut Vec<ffi::PyType_Slot>
) -> PyResult<PyType>
{
    // ensure the necessary slots are set:
    if !slots.iter().any(|s| s.slot == ffi::Py_tp_new) {
        slots.push(ffi::PyType_Slot {
            slot: ffi::Py_tp_new,
            pfunc: disabled_tp_new_callback as ffi::newfunc as *mut libc::c_void
        });
    }
    slots.push(ffi::PyType_Slot::default()); // sentinel
    let mut spec = ffi::PyType_Spec {
        name: name.as_ptr(),
        basicsize: basicsize as libc::c_int,
        itemsize: 0,
        flags: flags,
        slots: slots.as_mut_ptr()
    };
    err::result_cast_from_owned_ptr(py,
        ffi::PyType_FromSpec(&mut spec))
}

/// Represents something that can be added as a member to a Python class/type.
///
/// T: type of rust class used for instances of the Python class/type.
pub trait TypeMember<T> where T: PythonObject {
    /// Convert the type member into a python object
    /// that can be stored in the type dict.
    fn to_descriptor(&self, py: Python, ty: &PyType, name: &str) -> PyObject;

    /// Put the type member into a box with lifetime `'p` so that
    /// it can be used at a later point in time.
    ///
    /// `PyRustTypeBuilder:add()` may use this function to store the member,
    /// with `into_descriptor()` being called from the `finish()` method.
    fn into_box(self, py: Python) -> Box<TypeMember<T>>;
}

// TODO: does this cause trouble for coherence?

impl <T, S> TypeMember<T> for S where T: PythonObject, S: ToPyObject {
    #[inline]
    fn to_descriptor(&self, py: Python, _ty: &PyType, _name: &str) -> PyObject {
        self.to_py_object(py).into_object()
    }

    #[inline]
    fn into_box(self, py: Python) -> Box<TypeMember<T>> {
        Box::new(self.into_py_object(py).into_object())
    }
}

