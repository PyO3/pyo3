use std;
use std::kinds::marker::{NoSend, InvariantLifetime};
use std::ptr;
use ffi;
use objects::{PyObject, PyType, PyBool};
use pythonrun::GILGuard;

/// The 'Python' struct is a zero-size marker struct that is required for most python operations.
/// This is used to indicate that the operation accesses/modifies the python interpreter state,
/// and thus can only be called if the python interpreter is initialized and the GIL is acquired.
/// The lifetime 'p represents the lifetime of the python interpreter.
/// For example, python constants like None have the type "&'p PyObject<'p>".
/// You can imagine the GIL to be a giant "Mutex<AllPythonState>". This makes 'p the lifetime of the
/// python state protected by that mutex.
#[derive(Copy)]
pub struct Python<'p>(NoSend, InvariantLifetime<'p>);

/// Trait implemented by all python object types.
pub trait PythonObject<'p> : 'p {
    /// Casts the python object to PyObject.
    fn as_object(&self) -> &PyObject<'p>;

    /// Unsafe downcast from &PyObject to &Self.
    /// Undefined behavior if the input object does not have the expected type.
    unsafe fn unchecked_downcast_from<'a>(&'a PyObject<'p>) -> &'a Self;

    /// Retrieves the underlying FFI pointer associated with this python object.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.as_object().as_ptr()
    }

    /// Retrieve python instance from an existing python object.
    #[inline]
    fn python(&self) -> Python<'p> {
        self.as_object().python()
    }
}

/// Trait implemented by python object types that allow a checked downcast.
pub trait PythonObjectWithCheckedDowncast<'p> : PythonObject<'p> {
    /// Upcast from PyObject to a concrete python object type.
    /// Returns None if the python object is not of the specified type.
    fn downcast_from<'a>(&'a PyObject<'p>) -> Option<&'a Self>;
}

/// Trait implemented by python object types that have a corresponding type object.
pub trait PythonObjectWithTypeObject<'p> : PythonObjectWithCheckedDowncast<'p> {
    /// Retrieves the type object for this python object type.
    /// Option<&Self> is necessary until UFCS is implemented.
    fn type_object(Python<'p>, Option<&Self>) -> &'p PyType<'p>;
}

impl<'p> Python<'p> {
    /// Retrieve python instance under the assumption that the GIL is already acquired at this point,
    /// and stays acquired for the lifetime 'p
    #[inline]
    pub unsafe fn assume_gil_acquired() -> Python<'p> {
        Python(NoSend, InvariantLifetime)
    }
    
    /// Acquires the global interpreter lock, which allows access to the Python runtime.
    /// If the python runtime is not already initialized, this function will initialize it.
    /// Note that in this case, the python runtime will not have any main thread, and will
    /// not deliver signals like KeyboardInterrupt.
    #[inline]
    pub fn acquire_gil() -> GILGuard {
        GILGuard::acquire()
    }

    /// Releases the GIL and allows the use of python on other threads.
    /// Unsafe because we do not ensure that existing references to python objects
    /// are not accessed within the closure.
    pub unsafe fn allow_threads<T, F>(self, f: F) -> T where F : FnOnce() -> T {
        // TODO: we should use a type with destructor to be panic-safe, and avoid the unnecessary closure
        let save = ffi::PyEval_SaveThread();
        let result = f();
        ffi::PyEval_RestoreThread(save);
        result
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
    pub fn True(self) -> &'p PyBool<'p> {
        unsafe { PythonObject::unchecked_downcast_from(PyObject::from_ptr(self, ffi::Py_True())) }
    }
    
    /// Retrieves a reference to the 'False' constant value.
    #[allow(non_snake_case)] // the python keyword starts with uppercase
    #[inline]
    pub fn False(self) -> &'p PyBool<'p> {
        unsafe { PythonObject::unchecked_downcast_from(PyObject::from_ptr(self, ffi::Py_False())) }
    }
    
    /// Retrieves a reference to the type object for type T.
    #[inline]
    pub fn get_type<T>(self) -> &'p PyType<'p> where T: PythonObjectWithTypeObject<'p> {
        let none : Option<&T> = None;
        PythonObjectWithTypeObject::type_object(self, none)
    }
}

