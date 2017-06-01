// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use ffi;
use std::os::raw::c_char;
use std::ffi::{CStr, CString};

use conversion::{ToPyObject, ToPyTuple};
use pointers::{Ptr, PyPtr};
use python::{ToPyPointer, Python};
use token::PyObjectWithGilToken;
use objects::{PyObject, PyDict, PyType, exc};
use objectprotocol::ObjectProtocol;
use err::{PyResult, PyErr};


/// Represents a Python module object.
pub struct PyModule<'p>(Ptr<'p>);
pub struct PyModulePtr(PyPtr);

pyobject_convert!(PyModule);
pyobject_nativetype!(PyModule, PyModule_Check, PyModule_Type, PyModulePtr);


impl<'p> PyModule<'p> {
    /// Create a new module object with the `__name__` attribute set to name.
    pub fn new(py: Python<'p>, name: &str) -> PyResult<PyModule<'p>> {
        let name = CString::new(name).unwrap();
        Ok(PyModule(Ptr::from_owned_ptr_or_err(
            py, unsafe{ffi::PyModule_New(name.as_ptr())} )?))
    }

    /// Import the Python module with the specified name.
    pub fn import(py: Python<'p>, name: &str) -> PyResult<PyModule<'p>> {
        let name = CString::new(name).unwrap();
        Ok(PyModule(Ptr::from_owned_ptr_or_err(
            py, unsafe{ffi::PyImport_ImportModule(name.as_ptr())} )?))
    }

    /// Return the dictionary object that implements module's namespace;
    /// this object is the same as the `__dict__` attribute of the module object.
    pub fn dict(&self) -> PyDict<'p> {
        unsafe {
            PyDict::from_borrowed_ptr(self.gil(), ffi::PyModule_GetDict(self.as_ptr()))
        }
    }

    unsafe fn str_from_ptr<'a>(&'a self, ptr: *const c_char) -> PyResult<&'a str> {
        if ptr.is_null() {
            Err(PyErr::fetch(self.gil()))
        } else {
            let slice = CStr::from_ptr(ptr).to_bytes();
            match std::str::from_utf8(slice) {
                Ok(s) => Ok(s),
                Err(e) => Err(PyErr::from_instance(
                    self.gil(),
                    try!(exc::UnicodeDecodeError::new_utf8(self.gil(), slice, e))))
            }
        }
    }

    /// Gets the module name.
    ///
    /// May fail if the module does not have a `__name__` attribute.
    pub fn name<'a>(&'a self) -> PyResult<&'a str> {
        unsafe { self.str_from_ptr(ffi::PyModule_GetName(self.as_ptr())) }
    }

    /// Gets the module filename.
    ///
    /// May fail if the module does not have a `__file__` attribute.
    pub fn filename<'a>(&'a self) -> PyResult<&'a str> {
        unsafe { self.str_from_ptr(ffi::PyModule_GetFilename(self.as_ptr())) }
    }

    /// Calls a function in the module.
    /// This is equivalent to the Python expression: `getattr(module, name)(*args, **kwargs)`
    pub fn call<A>(&self, name: &str, args: A, kwargs: Option<&PyDict>)
                   -> PyResult<PyObject<'p>>
        where A: ToPyTuple
    {
        self.getattr(name)?.call(args, kwargs)
    }

    /// Gets a member from the module.
    /// This is equivalent to the Python expression: `getattr(module, name)`
    pub fn get(&self, name: &str) -> PyResult<PyObject> {
        self.getattr(name)
    }

    /// Adds a member to the module.
    ///
    /// This is a convenience function which can be used from the module's initialization function.
    pub fn add<V>(&self, name: &str, value: V) -> PyResult<()> where V: ToPyObject {
        self.setattr(name, value)
    }

    /// Adds a new extension type to the module.
    ///
    /// This is a convenience function that initializes the `class`,
    /// sets `new_type.__module__` to this module's name,
    /// and adds the type to this module.
    pub fn add_class<T>(&self) -> PyResult<()>
        where T: ::typeob::PyTypeInfo
    {
        let mut ty = <T as ::typeob::PyTypeInfo>::type_object();
        let type_name = <T as ::typeob::PyTypeInfo>::type_name();

        let ty = if (ty.tp_flags & ffi::Py_TPFLAGS_READY) != 0 {
            unsafe { PyType::from_type_ptr(self.gil(), ty) }
        } else {
            // automatically initialize the class
            let name = self.name()?;
            ::typeob::initialize_type::<T>(self.gil(), Some(name), type_name, ty)
                .expect(
                    format!("An error occurred while initializing class {}",
                            <T as ::typeob::PyTypeInfo>::type_name()).as_ref());
            unsafe { PyType::from_type_ptr(self.gil(), ty) }
        };

        self.setattr(type_name, ty)
    }
}
