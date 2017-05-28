// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_int;

use ffi;
use typeob::{PyTypeInfo, PyTypeObject, PyObjectAlloc};
use objects::{PyObject, PyType, PyBool, PyDict, PyModule};
use err::{PyErr, PyResult};
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

#[derive(Copy, Clone)]
pub struct Token<'p>(PhantomData<&'p GILGuard>);

pub struct PythonToken<T>(PhantomData<T>);


pub trait PythonObjectWithToken : Sized {
    fn token<'p>(&'p self) -> Token<'p>;
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

impl<'p, T> ToPythonPointer for T where T: PyTypeInfo + PythonObjectWithToken {
    #[inline]
    default fn as_ptr(&self) -> *mut ffi::PyObject {
        let offset = <T as PyTypeInfo>::offset();
        unsafe {
            {self as *const _ as *mut u8}.offset(-offset) as *mut ffi::PyObject
        }
    }
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
    fn as_ptr(&self) -> *mut ffi::PyObject {
        match *self {
            Some(ref t) => t.as_ptr(),
            None => std::ptr::null_mut()
        }
    }
}*/

/// Convert None into a null pointer.
impl<'p, T> ToPythonPointer for Option<&'p T> where T: ToPythonPointer {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
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
                locals: Option<&PyDict>) -> PyResult<PyPtr<PyObject>> {
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
                globals: Option<&PyDict>, locals: Option<&PyDict>) -> PyResult<PyPtr<PyObject>> {
        let code = CString::new(code).unwrap();

        unsafe {
            let mptr = ffi::PyImport_AddModule("__main__\0".as_ptr() as *const _);
            if mptr.is_null() {
                return Err(PyErr::fetch(self.token()));
            }

            let mdict = ffi::PyModule_GetDict(mptr);

            let globals = match globals {
                Some(g) => g.as_ptr(),
                None => mdict,
            };

            let locals = match locals {
                Some(l) => l.as_ptr(),
                None => globals
            };

            let res_ptr = ffi::PyRun_StringFlags(code.as_ptr(),
                start, globals, locals, 0 as *mut _);

            PyPtr::from_owned_ptr_or_err(self.token(), res_ptr)
        }
    }

    /// Gets the Python type object for type T.
    pub fn get_type<T>(self) -> PyPtr<PyType> where T: PyTypeObject {
        T::type_object(self.token())
    }

    /// Import the Python module with the specified name.
    pub fn import(self, name : &str) -> PyResult<Py<'p, PyModule>> {
        PyModule::import(self.token(), name)
    }

    pub fn with_token<T, F>(self, f: F) -> PyPtr<T>
        where F: FnOnce(PythonToken<T>) -> T,
              T: PyTypeInfo + PyObjectAlloc<Type=T>
    {
        let value = f(PythonToken(PhantomData));
        if let Ok(ob) = Py::new(self.token(), value) {
            println!("created: {:?}", &ob as *const _);
            ob.into_pptr()
        } else {
            ::err::panic_after_error()
        }
    }
//}

//impl<'p> PythonObjectWithToken<'p> for Python<'p> {
    pub fn token(self) -> Token<'p> {
        Token(PhantomData)
    }
}

impl<T> PythonToken<T> {
    pub fn token(&self) -> Token {
        Token(PhantomData)
    }
}

impl<'p> Token<'p> {
    /// Gets the Python builtin value `None`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn None(self) -> PyPtr<PyObject> {
        unsafe { PyPtr::from_borrowed_ptr(ffi::Py_None()) }
    }

    /// Gets the Python builtin value `True`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn True(self) -> PyPtr<PyBool> {
        unsafe { PyPtr::from_borrowed_ptr(ffi::Py_True()) }
    }

    /// Gets the Python builtin value `False`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn False(self) -> PyPtr<PyBool> {
        unsafe { PyPtr::from_borrowed_ptr(ffi::Py_False()) }
    }

    /// Gets the Python builtin value `NotImplemented`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn NotImplemented(self) -> PyPtr<PyObject> {
        unsafe { PyPtr::from_borrowed_ptr(ffi::Py_NotImplemented()) }
    }

    /// Gets the Python type object for type T.
    #[inline]
    pub fn get_type<U>(self) -> PyPtr<PyType> where U: PyTypeObject {
        U::type_object(self)
    }

    /// Execute closure `F` with Python instance.
    /// Retrieve Python instance under the assumption that the GIL is already acquired
    /// at this point, and stays acquired during closure call.
    pub fn with<F, R>(self, f: F) -> R where F: FnOnce(Python<'p>) -> R
    {
        f(Python(PhantomData))
    }

    /// Convert raw pointer into referece
    #[inline]
    pub unsafe fn from_owned_ptr<P>(self, ptr: *mut ffi::PyObject) -> &'p P
    {
        std::mem::transmute(ptr)
    }

    #[inline]
    pub unsafe fn from_owned_ptr_opt<P>(self, ptr: *mut ffi::PyObject) -> Option<&'p P>
    {
        if ptr.is_null() {
            None
        } else {
            Some(std::mem::transmute(ptr))
        }
    }

    #[inline]
    pub unsafe fn from_owned_ptr_or_panic<P>(self, ptr: *mut ffi::PyObject) -> &'p P
    {
        if ptr.is_null() {
            ::err::panic_after_error();
        } else {
            std::mem::transmute(ptr)
        }
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
        let v: i32 = py.eval("min(1, 2)", None, None).unwrap().extract(py).unwrap();
        assert_eq!(v, 1);

        let d = PyDict::new(py);
        d.set_item(py, "foo", 13).unwrap();

        // Inject our own local namespace
        let v: i32 = py.eval("foo + 29", None, Some(&d)).unwrap().extract(py).unwrap();
        assert_eq!(v, 42);

        // Make sure builtin names are still accessible when using a local namespace
        let v: i32 = py.eval("min(foo, 2)", None, Some(&d)).unwrap().extract(py).unwrap();
        assert_eq!(v, 2);
    }
}
