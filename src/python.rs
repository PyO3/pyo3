// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_int;

use ffi;
use typeob::{PyTypeInfo, PyTypeObject, PyObjectAlloc};
use token::{PyObjectMarker, PythonToken};
use objects::{PyObject, PyType, PyBool, PyDict, PyModule};
use err::{PyErr, PyResult, PyDowncastError};
use pyptr::{Py, PyPtr};
use pythonrun::GILGuard;


/// Marker type that indicates that the GIL is currently held.
///
/// The 'Python' struct is a zero-size marker struct that is required for most Python operations.
/// This is used to indicate that the operation accesses/modifies the Python interpreter state,
/// and thus can only be called if the Python interpreter is initialized and the
/// Python global interpreter lock (GIL) is acquired.
/// The lifetime `'p` represents the lifetime of the Python interpreter.
///
/// You can imagine the GIL to be a giant `Mutex<PythonInterpreterState>`.
/// The type `Python<'p>` then acts like a reference `&'p PythonInterpreterState`.
#[derive(Copy, Clone)]
pub struct Python<'p>(PhantomData<&'p GILGuard>);


/// Trait implemented by Python object types that allow a checked downcast.
pub trait PyDowncastFrom<'p> : Sized {

    /// Cast from PyObject to a concrete Python object type.
    fn downcast_from(&'p PyObject<'p>) -> Result<&'p Self, PyDowncastError<'p>>;

}


/// Trait implemented by Python object types that allow a checked downcast.
pub trait PyDowncastInto<'p> : Sized {

    /// Cast from PyObject to a concrete Python object type.
    fn downcast_into<I>(Python<'p>, I)
                        -> Result<Self, PyDowncastError<'p>>
        where I: ToPythonPointer + IntoPythonPointer;

    /// Cast from ffi::PyObject to a concrete Python object type.
    fn downcast_from_owned_ptr(py: Python<'p>, ptr: *mut ffi::PyObject)
                               -> Result<Self, PyDowncastError<'p>>;
}


pub trait PyClone : Sized {
    fn clone_ref(&self) -> PyPtr<Self>;
}

impl<T> PyClone for T where T: ToPythonPointer {
    #[inline]
    fn clone_ref(&self) -> PyPtr<T> {
        unsafe {
            let ptr = <T as ToPythonPointer>::as_ptr(self);
            PyPtr::from_borrowed_ptr(ptr)
        }
    }
}


/// This trait allows retrieving the underlying FFI pointer from Python objects.
pub trait ToPythonPointer {
    /// Retrieves the underlying FFI pointer (as a borrowed pointer).
    fn as_ptr(&self) -> *mut ffi::PyObject;

}

/// This trait allows retrieving the underlying FFI pointer from Python objects.
pub trait IntoPythonPointer {
    /// Retrieves the underlying FFI pointer. Whether pointer owned or borrowed
    /// depends on implementation.
    fn into_ptr(self) -> *mut ffi::PyObject;
}


/// Convert None into a null pointer.
/*impl <T> ToPythonPointer for Option<T> where T: ToPythonPointer {
    #[inline]
    default fn as_ptr(&self) -> *mut ffi::PyObject {
        match *self {
            Some(ref t) => t.as_ptr(),
            None => std::ptr::null_mut()
        }
    }
}*/

/// Convert None into a null pointer.
impl<'p, T> ToPythonPointer for Option<&'p T> where T: ToPythonPointer {
    #[inline]
    default fn as_ptr(&self) -> *mut ffi::PyObject {
        match *self {
            Some(ref t) => t.as_ptr(),
            None => std::ptr::null_mut()
        }
    }
}

/// Convert None into a null pointer.
impl <T> IntoPythonPointer for Option<T> where T: IntoPythonPointer {
    #[inline]
    fn into_ptr(self) -> *mut ffi::PyObject {
        match self {
            Some(t) => t.into_ptr(),
            None => std::ptr::null_mut()
        }
    }
}


impl<'p> Python<'p> {
    /// Retrieve Python instance under the assumption that the GIL is already acquired at this point,
    /// and stays acquired for the lifetime `'p`.
    ///
    /// Because the output lifetime `'p` is not connected to any input parameter,
    /// care must be taken that the compiler infers an appropriate lifetime for `'p`
    /// when calling this function.
    #[inline]
    pub unsafe fn assume_gil_acquired() -> Python<'p> {
        Python(PhantomData)
    }

    /// Acquires the global interpreter lock, which allows access to the Python runtime.
    ///
    /// If the Python runtime is not already initialized, this function will initialize it.
    /// See [prepare_freethreaded_python()](fn.prepare_freethreaded_python.html) for details.
    #[inline]
    pub fn acquire_gil() -> GILGuard {
        GILGuard::acquire()
    }

    /// Temporarily releases the GIL, thus allowing other Python threads to run.
    pub fn allow_threads<T, F>(self, f: F) -> T where F : Send + FnOnce() -> T {
        // The `Send` bound on the closure prevents the user from
        // transferring the `Python` token into the closure.
        unsafe {
            let save = ffi::PyEval_SaveThread();
            let result = f();
            ffi::PyEval_RestoreThread(save);
            result
        }
    }

    /// Evaluates a Python expression in the given context and returns the result.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    pub fn eval(self, code: &str, globals: Option<&PyDict>,
                locals: Option<&PyDict>) -> PyResult<PyObject<'p>> {
        self.run_code(code, ffi::Py_eval_input, globals, locals)
    }

    /// Executes one or more Python statements in the given context.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    pub fn run(self, code: &str, globals: Option<&PyDict>,
                locals: Option<&PyDict>) -> PyResult<()> {
        self.run_code(code, ffi::Py_file_input, globals, locals)?;
        Ok(())
    }

    /// Runs code in the given context.
    /// `start` indicates the type of input expected:
    /// one of `Py_single_input`, `Py_file_input`, or `Py_eval_input`.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    fn run_code(self, code: &str, start: c_int,
                globals: Option<&PyDict>, locals: Option<&PyDict>) -> PyResult<PyObject<'p>> {
        let code = CString::new(code).unwrap();

        unsafe {
            let mptr = ffi::PyImport_AddModule("__main__\0".as_ptr() as *const _);
            if mptr.is_null() {
                return Err(PyErr::fetch(self));
            }

            let globals = globals.map(|g| g.as_ptr())
                                 .unwrap_or_else(|| ffi::PyModule_GetDict(mptr));
            let locals = locals.map(|l| l.as_ptr()).unwrap_or(globals);

            let res_ptr = ffi::PyRun_StringFlags(code.as_ptr(),
                start, globals, locals, 0 as *mut _);

            PyObject::from_owned_ptr_or_err(self, res_ptr)
        }
    }

    /// Gets the Python type object for type T.
    pub fn get_type<T>(self) -> PyType<'p> where T: PyTypeObject {
        T::type_object(self)
    }

    /// Import the Python module with the specified name.
    pub fn import(self, name : &str) -> PyResult<PyModule<'p>> {
        PyModule::import(self, name)
    }

    pub fn with_token<T, F>(self, f: F) -> Py<'p, T>
        where F: FnOnce(PythonToken<T>) -> T,
              T: PyTypeInfo + PyObjectAlloc<Type=T>
    {
        ::token::with_token(self, f)
    }

    /// Gets the Python builtin value `None`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn None(self) -> PyPtr<PyObjectMarker> {
        unsafe { PyPtr::from_borrowed_ptr(ffi::Py_None()) }
    }

    /// Gets the Python builtin value `True`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn True(self) -> PyBool<'p> {
        PyBool::new(self, true)
    }

    /// Gets the Python builtin value `False`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn False(self) -> PyBool<'p> {
        PyBool::new(self, false)
    }

    /// Gets the Python builtin value `NotImplemented`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn NotImplemented(self) -> PyPtr<PyObjectMarker> {
        unsafe { PyPtr::from_borrowed_ptr(ffi::Py_NotImplemented()) }
    }

    /// Execute closure `F` with Python instance.
    /// Retrieve Python instance under the assumption that the GIL is already acquired
    /// at this point, and stays acquired during closure call.
    pub fn with<F, R>(self, f: F) -> R where F: FnOnce(Python<'p>) -> R
    {
        f(Python(PhantomData))
    }
}

#[cfg(test)]
mod test {
    use {Python, PyDict};

    #[test]
    fn test_eval() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Make sure builtin names are accessible
        let v: i32 = py.eval("min(1, 2)", None, None).unwrap().extract().unwrap();
        assert_eq!(v, 1);

        let d = PyDict::new(py);
        d.set_item("foo", 13).unwrap();

        // Inject our own global namespace
        let v: i32 = py.eval("foo + 29", Some(&d), None).unwrap().extract(py).unwrap();
        assert_eq!(v, 42);

        // Inject our own local namespace
        let v: i32 = py.eval("foo + 29", None, Some(&d)).unwrap().extract().unwrap();
        assert_eq!(v, 42);

        // Make sure builtin names are still accessible when using a local namespace
        let v: i32 = py.eval("min(foo, 2)", None, Some(&d)).unwrap().extract().unwrap();
        assert_eq!(v, 2);
    }
}
