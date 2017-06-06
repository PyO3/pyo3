// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use ffi;
use std::os::raw::c_char;
use std::ffi::{CStr, CString};

use conversion::{ToPyObject, IntoPyTuple};
use pointers::PyPtr;
use python::{Python, ToPyPointer};
use objects::{PyObject, PyDict, PyType, exc};
use objectprotocol::ObjectProtocol;
use err::{PyResult, PyErr};


/// Represents a Python module object.
pub struct PyModule(PyPtr);

pyobject_convert!(PyModule);
pyobject_nativetype!(PyModule, PyModule_Check, PyModule_Type);


impl<'p> PyModule {
    /// Create a new module object with the `__name__` attribute set to name.
    pub fn new(py: Python, name: &str) -> PyResult<PyModule> {
        let name = CString::new(name).unwrap();
        Ok(PyModule(PyPtr::from_owned_ptr_or_err(
            py, unsafe{ffi::PyModule_New(name.as_ptr())} )?))
    }

    /// Import the Python module with the specified name.
    pub fn import(py: Python, name: &str) -> PyResult<PyModule> {
        let name = CString::new(name).unwrap();
        Ok(PyModule(PyPtr::from_owned_ptr_or_err(
            py, unsafe{ffi::PyImport_ImportModule(name.as_ptr())} )?))
    }

    /// Return the dictionary object that implements module's namespace;
    /// this object is the same as the `__dict__` attribute of the module object.
    pub fn dict(&self, py: Python) -> PyDict {
        unsafe {
            PyDict::from_borrowed_ptr(py, ffi::PyModule_GetDict(self.as_ptr()))
        }
    }

    unsafe fn str_from_ptr<'a>(&'a self, py: Python, ptr: *const c_char) -> PyResult<&'a str> {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            let slice = CStr::from_ptr(ptr).to_bytes();
            match std::str::from_utf8(slice) {
                Ok(s) => Ok(s),
                Err(e) => Err(PyErr::from_instance(
                    py,
                    try!(exc::UnicodeDecodeError::new_utf8(py, slice, e))))
            }
        }
    }

    /// Gets the module name.
    ///
    /// May fail if the module does not have a `__name__` attribute.
    pub fn name<'a>(&'a self, py: Python) -> PyResult<&'a str> {
        unsafe { self.str_from_ptr(py, ffi::PyModule_GetName(self.as_ptr())) }
    }

    /// Gets the module filename.
    ///
    /// May fail if the module does not have a `__file__` attribute.
    pub fn filename<'a>(&'a self, py: Python) -> PyResult<&'a str> {
        unsafe { self.str_from_ptr(py, ffi::PyModule_GetFilename(self.as_ptr())) }
    }

    /// Calls a function in the module.
    /// This is equivalent to the Python expression: `getattr(module, name)(*args, **kwargs)`
    pub fn call<A>(&self, py: Python, name: &str,
                   args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
        where A: IntoPyTuple
    {
        self.getattr(py, name)?.call(py, args, kwargs)
    }

    /// Gets a member from the module.
    /// This is equivalent to the Python expression: `getattr(module, name)`
    pub fn get(&self, py: Python, name: &str) -> PyResult<PyObject>
    {
        self.getattr(py, name)
    }

    /// Adds a member to the module.
    ///
    /// This is a convenience function which can be used from the module's initialization function.
    pub fn add<V>(&self, py: Python, name: &str, value: V) -> PyResult<()> where V: ToPyObject {
        self.setattr(py, name, value)
    }

    /// Adds a new extension type to the module.
    ///
    /// This is a convenience function that initializes the `class`,
    /// sets `new_type.__module__` to this module's name,
    /// and adds the type to this module.
    pub fn add_class<T>(&self, py: Python) -> PyResult<()>
        where T: ::typeob::PyTypeInfo
    {
        let mut ty = <T as ::typeob::PyTypeInfo>::type_object();
        let type_name = <T as ::typeob::PyTypeInfo>::type_name();

        let ty = if (ty.tp_flags & ffi::Py_TPFLAGS_READY) != 0 {
            unsafe { PyType::from_type_ptr(py, ty) }
        } else {
            // automatically initialize the class
            let name = self.name(py)?;
            ::typeob::initialize_type::<T>(py, Some(name), type_name, ty)
                .expect(
                    format!("An error occurred while initializing class {}",
                            <T as ::typeob::PyTypeInfo>::type_name()).as_ref());
            unsafe { PyType::from_type_ptr(py, ty) }
        };

        self.setattr(py, type_name, ty)
    }
}
