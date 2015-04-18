use std;
use ffi;
use libc::c_char;
use python::{Python, PythonObject, ToPythonPointer};
use conversion::ToPyObject;
use objects::{PyObject, PyType, PyDict, exc};
use err::{self, PyResult, PyErr};
use std::ffi::{CStr, CString};

pyobject_newtype!(PyModule, PyModule_Check, PyModule_Type);

impl <'p> PyModule<'p> {
    /// Create a new module object with the __name__ attribute set to name.
    pub fn new(py: Python<'p>, name: &str) -> PyResult<'p, PyModule<'p>> {
        let name = CString::new(name).unwrap();
        unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyModule_New(name.as_ptr()))
        }
    }

    /// Import the python module with the specified name.
    pub fn import(py: Python<'p>, name: &str) -> PyResult<'p, PyModule<'p>> {
        let name = CString::new(name).unwrap();
        unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyImport_ImportModule(name.as_ptr()))
        }
    }

    // Helper method for module_initializer!() macro, do not use directly!
    #[doc(hidden)]
    pub fn _init<F, R>(py: Python<'p>, name: &CStr, init: F) -> PyResult<'p, R>
      where F: FnOnce(Python<'p>, PyModule<'p>) -> PyResult<'p, R> {
        let module = try!(unsafe {
            err::result_from_borrowed_ptr(py, ffi::Py_InitModule(name.as_ptr(), std::ptr::null_mut()))
        });
        let module = try!(module.cast_into::<PyModule>());
        init(py, module)
    }

    /// Return the dictionary object that implements module‘s namespace;
    /// this object is the same as the __dict__ attribute of the module object.
    pub fn dict(&self) -> PyDict<'p> {
        let py = self.python();
        unsafe {
            let r = PyObject::from_borrowed_ptr(py, ffi::PyModule_GetDict(self.as_ptr()));
            r.unchecked_cast_into::<PyDict>()
        }
    }

    unsafe fn str_from_ptr<'a>(&'a self, ptr: *mut c_char) -> PyResult<'p, &'a str> {
        let py = self.python();
        if ptr == std::ptr::null_mut() {
            Err(PyErr::fetch(py))
        } else {
            let ptr = ptr as *const c_char;
            let slice = CStr::from_ptr(ptr).to_bytes();
            match std::str::from_utf8(slice) {
                Ok(s) => Ok(std::mem::copy_lifetime(self, s)),
                Err(e) => Err(PyErr::new(try!(exc::UnicodeDecodeError::new_utf8(py, slice, e))))
            }
        }
    }

    /// Gets the module name.
    ///
    /// May fail if the module does not have a __name__ attribute.
    pub fn name<'a>(&'a self) -> PyResult<'p, &'a str> {
        unsafe { self.str_from_ptr(ffi::PyModule_GetName(self.as_ptr())) }
    }
    
    /// Gets the module filename.
    ///
    /// May fail if the module does not have a __file__ attribute.
    pub fn filename<'a>(&'a self) -> PyResult<'p, &'a str> {
        unsafe { self.str_from_ptr(ffi::PyModule_GetFilename(self.as_ptr())) }
    }

    /// Gets a member from the module.
    pub fn get(&self, name: &str) -> PyResult<'p, PyObject<'p>> {
        use objectprotocol::ObjectProtocol;
        self.as_object().getattr(name)
    }

    /// Adds a member to the module.
    ///
    /// This is a convenience function which can be used from the module's initialization function.
    pub fn add<V>(&self, name: &str, value: V) -> PyResult<'p, ()> where V: ToPyObject<'p> {
        let py = self.python();
        let name = CString::new(name).unwrap();
        let value = try!(value.into_py_object(py));
        let r = unsafe { ffi::PyModule_AddObject(self.as_ptr(), name.as_ptr(), value.steal_ptr()) };
        err::error_on_minusone(py, r)
    }
}


