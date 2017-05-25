// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use ffi;
use std::os::raw::c_char;
use std::ffi::{CStr, CString};

use pyptr::Py;
use python::{AsPy, Python, ToPythonPointer};
use objects::{PyObject, PyDict, PyType, exc};
use err::{PyResult, PyErr};

/// Represents a Python module object.
pub struct PyModule;

pyobject_newtype!(PyModule, PyModule_Check, PyModule_Type);

impl PyModule {
    /// Create a new module object with the `__name__` attribute set to name.
    pub fn new<'p>(py: Python<'p>, name: &str) -> PyResult<Py<'p, PyModule>> {
        let name = CString::new(name).unwrap();
        unsafe {
            Py::cast_from_owned_nullptr(py, ffi::PyModule_New(name.as_ptr()))
        }
    }

    /// Import the Python module with the specified name.
    pub fn import<'p>(py: Python<'p>, name: &str) -> PyResult<Py<'p, PyModule>> {
        let name = CString::new(name).unwrap();
        unsafe {
            Py::cast_from_owned_nullptr(py, ffi::PyImport_ImportModule(name.as_ptr()))
        }
    }

    /// Return the dictionary object that implements module's namespace;
    /// this object is the same as the `__dict__` attribute of the module object.
    pub fn dict(&self) -> Py<PyDict> {
        unsafe {
            Py::from_borrowed_ptr(self.py(), ffi::PyModule_GetDict(self.as_ptr()))
        }
    }

    unsafe fn str_from_ptr<'a>(&'a self, ptr: *const c_char) -> PyResult<&'a str> {
        if ptr.is_null() {
            Err(PyErr::fetch(self.py()))
        } else {
            let slice = CStr::from_ptr(ptr).to_bytes();
            match std::str::from_utf8(slice) {
                Ok(s) => Ok(s),
                Err(e) => Err(PyErr::from_instance(
                    self.py(), try!(exc::UnicodeDecodeError::new_utf8(self.py(), slice, e))))
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

    /// Adds a new extension type to the module.
    ///
    /// This is a convenience function that initializes the `class`,
    /// sets `new_type.__module__` to this module's name,
    /// and adds the type to this module.
    pub fn add_class<'p, T>(&self) -> PyResult<()>
        where T: ::typeob::PyTypeInfo
    {
        let mut ty = <T as ::typeob::PyTypeInfo>::type_object();
        let type_name = <T as ::typeob::PyTypeInfo>::type_name();

        let ty = if (ty.tp_flags & ffi::Py_TPFLAGS_READY) != 0 {
            unsafe { PyType::from_type_ptr(self.py(), ty) }
        } else {
            // automatically initialize the class
            let name = self.name()?;
            ::typeob::initialize_type::<T>(self.py(), Some(name), type_name, ty)
                .expect(
                    format!("An error occurred while initializing class {}",
                            <T as ::typeob::PyTypeInfo>::type_name()).as_ref());
            unsafe { PyType::from_type_ptr(self.py(), ty) }
        };

        PyObject::from_borrowed_ptr(self.py(), self.as_ptr()).setattr(type_name, ty)
    }
}
