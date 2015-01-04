use std;
use std::kinds::marker::{NoSend, NoCopy, InvariantLifetime};
use std::ptr;
use ffi;
use std::c_str::CString;
use object::{PythonObject, PyObject};
use typeobject::PyType;

/// The 'Python' struct is a zero-size marker struct that is required for most python operations.
/// This is used to indicate that the operation accesses/modifies the python interpreter state,
/// and thus can only be called if the python interpreter is initialized and the GIL is acquired.
/// The lifetime 'p represents the lifetime of the python interpreter.
/// For example, python constants like None have the type "&'p PyObject<'p>".
/// You can imagine the GIL to be a giant "Mutex<AllPythonState>". This makes 'p the lifetime of the
/// python state protected by that mutex.
#[derive(Copy)]
pub struct Python<'p>(NoSend, InvariantLifetime<'p>);

impl<'p> Python<'p> {
    /// Retrieve python instance under the assumption that the GIL is already acquired at this point,
    /// and stays acquired for the lifetime 'p.
    pub unsafe fn assume_gil_acquired() -> Python<'p> {
        Python(NoSend, InvariantLifetime)
    }
    
    /// Retrieves a reference to the special 'None' value.
    #[allow(non_snake_case)] // the python keyword starts with uppercase
    #[inline]
    pub fn None(self) -> &'p PyObject<'p> {
        unsafe { PyObject::from_ptr(self, ffi::Py_None()) }
    }
    
    /// Retrieves a reference to the 'True' constant value.
    #[allow(non_snake_case)] // the python keyword starts with uppercase
    #[inline]
    pub fn True(self) -> &'p PyObject<'p> {
        unsafe { PyObject::from_ptr(self, ffi::Py_True()) }
    }
    
    /// Retrieves a reference to the 'False' constant value.
    #[allow(non_snake_case)] // the python keyword starts with uppercase
    #[inline]
    pub fn False(self) -> &'p PyObject<'p> {
        unsafe { PyObject::from_ptr(self, ffi::Py_False()) }
    }
    
    /// Retrieves a reference to the type object for type T.
    #[inline]
    pub fn get_type<T>(self) -> &'p PyType<'p> where T: PythonObject<'p> {
        let none : Option<&T> = None;
        PythonObject::type_object(self, none)
    }
    
    /// Acquires the global interpreter lock, which allows access to the Python runtime.
    /// If the python runtime is not already initialized, this function will initialize it.
    /// Note that in this case, the python runtime will not have any main thread, and will
    /// not deliver signals like KeyboardInterrupt.
    pub fn acquire_gil() -> GILGuard {
        ::pythonrun::prepare_freethreaded_python();
        let gstate = unsafe { ffi::PyGILState_Ensure() }; // acquire GIL
        GILGuard { gstate: gstate, marker: NoSend }
    }

    /// Releases the GIL and allows the use of python on other threads.
    /// Unsafe because we do not ensure that existing references to python objects
    /// are not accessed within the closure.
    pub unsafe fn allow_threads<T, F>(self, f: F) -> T where F : FnOnce() -> T {
        let save = ffi::PyEval_SaveThread();
        let result = f();
        ffi::PyEval_RestoreThread(save);
        result
    }
}

/// RAII type that represents an acquired GIL.
#[must_use]
pub struct GILGuard {
    gstate: ffi::PyGILState_STATE,
    marker: NoSend
}

impl Drop for GILGuard {
    fn drop(&mut self) {
        unsafe { ffi::PyGILState_Release(self.gstate) }
    }
}

impl GILGuard {
    pub fn python<'p>(&'p self) -> Python<'p> {
        unsafe { Python::assume_gil_acquired() }
    }
}

