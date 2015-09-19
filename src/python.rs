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
use std::ffi::CString;
use std::marker::PhantomData;
use libc::c_int;
use ffi;
use objects::{PyObject, PyType, PyBool, PyDict, PyModule};
use err::{self, PyErr, PyResult};
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

/// This trait allows retrieving the underlying FFI pointer from Python objects.
pub trait ToPythonPointer {
    /// Retrieves the underlying FFI pointer (as a borrowed pointer).
    fn as_ptr(&self) -> *mut ffi::PyObject;

    /// Retrieves the underlying FFI pointer as a "stolen pointer".
    fn steal_ptr(self) -> *mut ffi::PyObject;
}

/// Trait implemented by all Python object types.
pub trait PythonObject<'p> : 'p + Clone + ::conversion::ToPyObject<'p> {
    /// Casts the Python object to PyObject.
    fn as_object(&self) -> &PyObject<'p>;

    /// Casts the Python object to PyObject.
    fn into_object(self) -> PyObject<'p>;

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    unsafe fn unchecked_downcast_from(PyObject<'p>) -> Self;

    /// Unchecked downcast from PyObject to Self.
    /// Undefined behavior if the input object does not have the expected type.
    unsafe fn unchecked_downcast_borrow_from<'a>(&'a PyObject<'p>) -> &'a Self;

    /// Retrieve Python instance from an existing Python object.
    #[inline]
    fn python(&self) -> Python<'p> {
        self.as_object().python()
    }
}

// Marker type that indicates an error while downcasting
pub struct PythonObjectDowncastError<'p>(pub Python<'p>);

/// Trait implemented by Python object types that allow a checked downcast.
pub trait PythonObjectWithCheckedDowncast<'p> : PythonObject<'p> {
    /// Cast from PyObject to a concrete Python object type.
    fn downcast_from(PyObject<'p>) -> Result<Self, PythonObjectDowncastError<'p>>;
    
    /// Cast from PyObject to a concrete Python object type.
    fn downcast_borrow_from<'a>(&'a PyObject<'p>) -> Result<&'a Self, PythonObjectDowncastError<'p>>;
}

/// Trait implemented by Python object types that have a corresponding type object.
pub trait PythonObjectWithTypeObject<'p> : PythonObjectWithCheckedDowncast<'p> {
    /// Retrieves the type object for this Python object type.
    fn type_object(Python<'p>) -> PyType<'p>;
}

/// ToPythonPointer for borrowed Python pointers.
impl <'a, 'p, T> ToPythonPointer for &'a T where T: PythonObject<'p> {
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.as_object().as_ptr()
    }
    
    #[inline]
    fn steal_ptr(self) -> *mut ffi::PyObject {
        self.as_object().clone().steal_ptr()
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

    /// Releases the GIL and allows the use of Python on other threads.
    /// Unsafe because we do not ensure that existing references to Python objects
    /// are not accessed within the closure.
    pub unsafe fn allow_threads<T, F>(self, f: F) -> T where F : FnOnce() -> T {
        // TODO: we should use a type with destructor to be panic-safe, and avoid the unnecessary closure
        let save = ffi::PyEval_SaveThread();
        let result = f();
        ffi::PyEval_RestoreThread(save);
        result
    }

    /// Evaluates a Python expression in the given context and returns the result.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    pub fn eval(self, code: &str, globals: Option<&PyDict<'p>>,
                locals: Option<&PyDict<'p>>) -> PyResult<'p, PyObject<'p>> {
        self.run_code(code, ffi::Py_eval_input, globals, locals)
    }

    /// Executes one or more Python statements in the given context.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    pub fn run(self, code: &str, globals: Option<&PyDict<'p>>,
                locals: Option<&PyDict<'p>>) -> PyResult<'p, ()> {
        try!(self.run_code(code, ffi::Py_file_input, globals, locals));
        Ok(())
    }

    /// Runs code in the given context.
    /// `start` indicates the type of input expected:
    /// one of `Py_single_input`, `Py_file_input`, or `Py_eval_input`.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    fn run_code(self, code: &str, start: c_int,
                globals: Option<&PyDict<'p>>, locals: Option<&PyDict<'p>>)
                -> PyResult<'p, PyObject<'p>> {
        let code = CString::new(code).unwrap();

        unsafe {
            let mptr = ffi::PyImport_AddModule("__main__\0".as_ptr() as *const _);

            if mptr.is_null() {
                return Err(PyErr::fetch(self));
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

            err::result_from_owned_ptr(self, res_ptr)
        }
    }

    /// Gets the Python builtin value `None`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn None(self) -> PyObject<'p> {
        unsafe { PyObject::from_borrowed_ptr(self, ffi::Py_None()) }
    }

    /// Gets the Python builtin value `True`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn True(self) -> PyBool<'p> {
        unsafe { PyObject::from_borrowed_ptr(self, ffi::Py_True()).unchecked_cast_into::<PyBool>() }
    }

    /// Gets the Python builtin value `False`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn False(self) -> PyBool<'p> {
        unsafe { PyObject::from_borrowed_ptr(self, ffi::Py_False()).unchecked_cast_into::<PyBool>() }
    }

    /// Gets the Python type object for type T.
    pub fn get_type<T>(self) -> PyType<'p> where T: PythonObjectWithTypeObject<'p> {
        T::type_object(self)
    }

    /// Import the Python module with the specified name.
    pub fn import(self, name : &str) -> PyResult<'p, PyModule<'p>> {
        PyModule::import(self, name)
    }
}

impl <'p> std::fmt::Debug for PythonObjectDowncastError<'p> {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str("PythonObjectDowncastError")
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

        // Inject our own local namespace
        let v: i32 = py.eval("foo + 29", None, Some(&d)).unwrap().extract().unwrap();

        assert_eq!(v, 42);
    }
}
