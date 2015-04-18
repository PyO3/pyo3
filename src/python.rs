use std;
use std::marker::PhantomData;
use ffi;
use objects::{PyObject, PyType, PyBool, PyModule};
use err::PyResult;
use pythonrun::GILGuard;

// Dummy struct representing the global state in the python interpreter.
struct PythonInterpreterState;
impl !Sync for PythonInterpreterState {}

/// The 'Python' struct is a zero-size marker struct that is required for most python operations.
/// This is used to indicate that the operation accesses/modifies the python interpreter state,
/// and thus can only be called if the python interpreter is initialized and the GIL is acquired.
/// The lifetime 'p represents the lifetime of the python interpreter.
/// For example, python constants like None have the type "&'p PyObject<'p>".
/// You can imagine the GIL to be a giant "Mutex<AllPythonState>". This makes 'p the lifetime of the
/// python state protected by that mutex.
#[derive(Copy, Clone)]
pub struct Python<'p>(PhantomData<&'p PythonInterpreterState>);

// Trait for converting from Self to *mut ffi::PyObject
pub trait ToPythonPointer {
    /// Retrieves the underlying FFI pointer (as a borrowed pointer).
    fn as_ptr(&self) -> *mut ffi::PyObject;
    
    /// Destructures the input object, moving out the ownership of the underlying FFI pointer.
    fn steal_ptr(self) -> *mut ffi::PyObject;
}

/// Trait implemented by all python object types.
pub trait PythonObject<'p> : 'p + Clone + ToPythonPointer {
    /// Casts the python object to PyObject.
    fn as_object(&self) -> &PyObject<'p>;
    
    /// Casts the python object to PyObject.
    fn into_object(self) -> PyObject<'p>;

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    unsafe fn unchecked_downcast_from(PyObject<'p>) -> Self;

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    unsafe fn unchecked_downcast_borrow_from<'a>(&'a PyObject<'p>) -> &'a Self;

    /// Retrieve python instance from an existing python object.
    #[inline]
    fn python(&self) -> Python<'p> {
        self.as_object().python()
    }
}

// Marker type that indicates an error while downcasting 
pub struct PythonObjectDowncastError<'p>(pub Python<'p>);

/// Trait implemented by python object types that allow a checked downcast.
pub trait PythonObjectWithCheckedDowncast<'p> : PythonObject<'p> {
    /// Cast from PyObject to a concrete python object type.
    fn downcast_from(PyObject<'p>) -> Result<Self, PythonObjectDowncastError<'p>>;
    
    /// Cast from PyObject to a concrete python object type.
    fn downcast_borrow_from<'a>(&'a PyObject<'p>) -> Result<&'a Self, PythonObjectDowncastError<'p>>;
}

/// Trait implemented by python object types that have a corresponding type object.
pub trait PythonObjectWithTypeObject<'p> : PythonObjectWithCheckedDowncast<'p> {
    /// Retrieves the type object for this python object type.
    fn type_object(Python<'p>) -> PyType<'p>;
}

/// ToPythonPointer for borrowed python pointers.
impl <'a, 'p, T> ToPythonPointer for &'a T where T: PythonObject<'p> {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        (**self).as_ptr()
    }
    
    #[inline]
    fn steal_ptr(self) -> *mut ffi::PyObject {
        (*self).clone().steal_ptr()
    }
}


/// Convert None into a null pointer.
impl <T> ToPythonPointer for Option<T> where T: ToPythonPointer {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        match *self {
            Some(ref t) => t.as_ptr(),
            None => std::ptr::null_mut()
        }
    }
    
    #[inline]
    fn steal_ptr(self) -> *mut ffi::PyObject {
        match self {
            Some(t) => t.steal_ptr(),
            None => std::ptr::null_mut()
        }
    }
}

impl<'p> Python<'p> {
    /// Retrieve python instance under the assumption that the GIL is already acquired at this point,
    /// and stays acquired for the lifetime 'p
    #[inline]
    pub unsafe fn assume_gil_acquired() -> Python<'p> {
        Python(PhantomData)
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
    
    /// Gets the python builtin value `None`.
    #[allow(non_snake_case)] // the python keyword starts with uppercase
    #[inline]
    pub fn None(self) -> PyObject<'p> {
        unsafe { PyObject::from_borrowed_ptr(self, ffi::Py_None()) }
    }
    
    /// Gets the python builtin value `True`.
    #[allow(non_snake_case)] // the python keyword starts with uppercase
    #[inline]
    pub fn True(self) -> PyBool<'p> {
        unsafe { PyObject::from_borrowed_ptr(self, ffi::Py_True()).unchecked_cast_into::<PyBool>() }
    }
    
    /// Gets the python builtin value `False`.
    #[allow(non_snake_case)] // the python keyword starts with uppercase
    #[inline]
    pub fn False(self) -> PyBool<'p> {
        unsafe { PyObject::from_borrowed_ptr(self, ffi::Py_False()).unchecked_cast_into::<PyBool>() }
    }

    /// Gets the python type object for type T.
    pub fn get_type<T>(self) -> PyType<'p> where T: PythonObjectWithTypeObject<'p> {
        T::type_object(self)
    }

    /// Import the python module with the specified name.
    pub fn import(self, name : &str) -> PyResult<'p, PyModule<'p>> {
        PyModule::import(self, name)
    }
}

