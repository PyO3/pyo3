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

use std;
use ffi;
use libc::c_char;
use python::{Python, PythonObject, ToPythonPointer};
use objectprotocol::ObjectProtocol;
use conversion::ToPyObject;
use objects::{PyObject, PyTuple, PyDict, exc};
use err::{self, PyResult, PyErr};
use std::ffi::{CStr, CString};

/// Represents a Python module object.
pub struct PyModule<'p>(PyObject<'p>);

pyobject_newtype!(PyModule, PyModule_Check, PyModule_Type);

impl <'p> PyModule<'p> {
    /// Create a new module object with the `__name__` attribute set to name.
    pub fn new(py: Python<'p>, name: &str) -> PyResult<'p, PyModule<'p>> {
        let name = CString::new(name).unwrap();
        unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyModule_New(name.as_ptr()))
        }
    }

    /// Import the Python module with the specified name.
    pub fn import(py: Python<'p>, name: &str) -> PyResult<'p, PyModule<'p>> {
        let name = CString::new(name).unwrap();
        unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyImport_ImportModule(name.as_ptr()))
        }
    }

    /// Return the dictionary object that implements module's namespace;
    /// this object is the same as the `__dict__` attribute of the module object.
    pub fn dict(&self) -> PyDict<'p> {
        let py = self.python();
        unsafe {
            let r = PyObject::from_borrowed_ptr(py, ffi::PyModule_GetDict(self.as_ptr()));
            r.unchecked_cast_into::<PyDict>()
        }
    }

    unsafe fn str_from_ptr<'a>(&'a self, ptr: *const c_char) -> PyResult<'p, &'a str> {
        let py = self.python();
        if ptr == std::ptr::null() {
            Err(PyErr::fetch(py))
        } else {
            let slice = CStr::from_ptr(ptr).to_bytes();
            match std::str::from_utf8(slice) {
                Ok(s) => Ok(s),
                Err(e) => Err(PyErr::new(try!(exc::UnicodeDecodeError::new_utf8(py, slice, e))))
            }
        }
    }

    /// Gets the module name.
    ///
    /// May fail if the module does not have a `__name__` attribute.
    pub fn name<'a>(&'a self) -> PyResult<'p, &'a str> {
        unsafe { self.str_from_ptr(ffi::PyModule_GetName(self.as_ptr())) }
    }

    /// Gets the module filename.
    ///
    /// May fail if the module does not have a `__file__` attribute.
    pub fn filename<'a>(&'a self) -> PyResult<'p, &'a str> {
        unsafe { self.str_from_ptr(ffi::PyModule_GetFilename(self.as_ptr())) }
    }

    /// Gets a member from the module.
    /// This is equivalent to the Python expression: `getattr(module, name)`
    pub fn get(&self, name: &str) -> PyResult<'p, PyObject<'p>> {
        self.as_object().getattr(name)
    }

    /// Calls a function in the module.
    /// This is equivalent to the Python expression: `getattr(module, name)(*args, **kwargs)`
    pub fn call<A>(&self, name: &str, args: A, kwargs: Option<&PyDict<'p>>) -> PyResult<'p, PyObject<'p>>
      where A: ToPyObject<'p, ObjectType=PyTuple<'p>> {
        try!(self.as_object().getattr(name)).call(args, kwargs)
    }

    /// Adds a member to the module.
    ///
    /// This is a convenience function which can be used from the module's initialization function.
    pub fn add<V>(&self, name: &str, value: V) -> PyResult<'p, ()> where V: ToPyObject<'p> {
        self.dict().set_item(name, value)
    }

    /// Adds a new extension type to the module.
    ///
    /// This is a convenience function that creates a new `PyRustTypeBuilder` and
    /// sets `new_type.__module__` to this module's name.
    /// The new type will be added to this module when `finish()` is called on the builder.
    pub fn add_type<T>(&self, name: &str) -> ::rustobject::typebuilder::PyRustTypeBuilder<'p, T>
            where T: 'p + Send {
        ::rustobject::typebuilder::new_typebuilder_for_module(self, name)
    }
}


