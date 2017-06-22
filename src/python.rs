// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_int;

use ffi;
use typeob::{PyTypeInfo, PyTypeObject, PyObjectAlloc};
use instance::{Py, PyToken};
use pointer::PyObjectPtr;
use objects::{PyObject, PyType, PyBool, PyDict, PyModule};
use err::{PyErr, PyResult, PyDowncastError, ToPyErr};
use pythonrun::{self, GILGuard};


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
pub trait PyDowncastFrom : Sized {

    /// Cast from PyObject to a concrete Python object type.
    fn downcast_from(&PyObject) -> Result<&Self, PyDowncastError>;

    /// Cast from PyObject to a concrete Python object type.
    unsafe fn unchecked_downcast_from(&PyObject) -> &Self;

    /// Cast from PyObject to a concrete Python object type.
    unsafe fn unchecked_mut_downcast_from(&PyObject) -> &mut Self;
}

/// Trait implemented by Python object types that allow a checked downcast.
pub trait PyMutDowncastFrom : Sized {

    /// Cast from PyObject to a concrete Python object type.
    fn downcast_mut_from(&mut PyObject) -> Result<&mut Self, PyDowncastError>;
}

/// Trait implemented by Python object types that allow a checked downcast.
pub trait PyDowncastInto : Sized {

    /// Cast Self to a concrete Python object type.
    fn downcast_into<'p, I>(Python<'p>, I) -> Result<Self, PyDowncastError<'p>>
        where I: ToPyPointer + IntoPyPointer;

    /// Cast from ffi::PyObject to a concrete Python object type.
    fn downcast_into_from_ptr<'p>(py: Python<'p>, ptr: *mut ffi::PyObject)
                                  -> Result<Self, PyDowncastError<'p>>;

    /// Cast from ffi::PyObject to a concrete Python object type.
    fn unchecked_downcast_into<'p, I>(I) -> Self where I: IntoPyPointer;
}

/// This trait allows retrieving the underlying FFI pointer from Python objects.
pub trait ToPyPointer {
    /// Retrieves the underlying FFI pointer (as a borrowed pointer).
    fn as_ptr(&self) -> *mut ffi::PyObject;

}

/// This trait allows retrieving the underlying FFI pointer from Python objects.
pub trait IntoPyPointer {
    /// Retrieves the underlying FFI pointer. Whether pointer owned or borrowed
    /// depends on implementation.
    fn into_ptr(self) -> *mut ffi::PyObject;
}

/// Convert None into a null pointer.
impl<'p, T> ToPyPointer for Option<&'p T> where T: ToPyPointer {
    #[inline]
    default fn as_ptr(&self) -> *mut ffi::PyObject {
        match *self {
            Some(ref t) => t.as_ptr(),
            None => std::ptr::null_mut()
        }
    }
}

/// Convert None into a null pointer.
impl <T> IntoPyPointer for Option<T> where T: IntoPyPointer {
    #[inline]
    fn into_ptr(self) -> *mut ffi::PyObject {
        match self {
            Some(t) => t.into_ptr(),
            None => std::ptr::null_mut()
        }
    }
}

pub trait PyClone {

    fn clone_ref(&self, py: Python) -> Self;

}

impl<T> PyClone for Option<T> where T: PyClone {
    fn clone_ref(&self, py: Python) -> Option<T> {
        match *self {
            Some(ref p) => Some(p.clone_ref(py)),
            None => None,
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
                locals: Option<&PyDict>) -> PyResult<&'p PyObject> {
        self.run_code(code, ffi::Py_eval_input, globals, locals)
    }

    /// Executes one or more Python statements in the given context.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    pub fn run(self, code: &str, globals: Option<&PyDict>,
                locals: Option<&PyDict>) -> PyResult<()> {
        let _ = self.run_code(code, ffi::Py_file_input, globals, locals)?;
        Ok(())
    }

    /// Runs code in the given context.
    /// `start` indicates the type of input expected:
    /// one of `Py_single_input`, `Py_file_input`, or `Py_eval_input`.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    fn run_code(self, code: &str, start: c_int,
                globals: Option<&PyDict>, locals: Option<&PyDict>) -> PyResult<&'p PyObject> {
        let code = CString::new(code).map_err(|e| e.to_pyerr(self))?;

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

            self.cast_from_ptr_or_err(res_ptr)
        }
    }

    /// Gets the Python type object for type T.
    pub fn get_type<T>(self) -> &'p PyType where T: PyTypeObject {
        T::type_object(self)
    }

    /// Import the Python module with the specified name.
    pub fn import(self, name : &str) -> PyResult<&'p PyModule> {
        PyModule::import(self, name)
    }

    /// Gets the Python builtin value `None`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn None(self) -> PyObjectPtr {
        unsafe { PyObjectPtr::from_borrowed_ptr(self, ffi::Py_None()) }
    }

    /// Gets the Python builtin value `True`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn True(self) -> &'p PyBool {
        PyBool::new(self, true)
    }

    /// Gets the Python builtin value `False`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn False(self) -> &'p PyBool {
        PyBool::new(self, false)
    }

    /// Gets the Python builtin value `NotImplemented`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn NotImplemented(self) -> PyObjectPtr {
        unsafe { PyObjectPtr::from_borrowed_ptr(self, ffi::Py_NotImplemented()) }
    }

    /// Create new python object and move T instance under python management
    #[inline]
    pub fn init<T, F>(self, f: F) -> PyResult<Py<T>>
        where F: FnOnce(PyToken) -> T,
              T: PyTypeInfo + PyObjectAlloc<T>
    {
        Py::new(self, f)
    }

    /// Release PyObject reference
    #[inline]
    pub fn release<T>(self, ob: T) where T: IntoPyPointer {
        unsafe {
            let ptr = ob.into_ptr();
            if !ptr.is_null() {
                ffi::Py_DECREF(ptr);
            }
        }
    }
    #[inline]
    pub fn release_res<T>(self, res: PyResult<T>) where T: IntoPyPointer {
        match res {
            Ok(ob) => unsafe {ffi::Py_DECREF(ob.into_ptr())},
            Err(e) => e.release(self)
        }
    }

    /// Check whether `obj` is an instance of type `T` like Python `isinstance` function
    pub fn is_instance<T: PyTypeObject, V: ToPyPointer>(self, obj: &V) -> PyResult<bool> {
        T::type_object(self).is_instance(obj)
    }

    /// Check whether type `T` is subclass of type `U` like Python `issubclass` function
    pub fn is_subclass<T, U>(self) -> PyResult<bool>
        where T: PyTypeObject,
            U: PyTypeObject
    {
        T::type_object(self).is_subclass::<U>()
    }
}

impl<'p> Python<'p> {

    pub fn checked_cast_as<D>(self, obj: PyObjectPtr) -> Result<&'p D, PyDowncastError<'p>>
        where D: PyDowncastFrom
    {
        unsafe {
            let p = pythonrun::register(self, obj);
            <D as PyDowncastFrom>::downcast_from(p)
        }
    }

    pub unsafe fn cast_as<D>(self, obj: PyObjectPtr) -> &'p D
        where D: PyDowncastFrom
    {
        let p = pythonrun::register(self, obj);
        <D as PyDowncastFrom>::unchecked_downcast_from(p)
    }

    pub unsafe fn cast_from_ptr<D>(self, ptr: *mut ffi::PyObject) -> &'p D
        where D: PyDowncastFrom
    {
        let obj = PyObjectPtr::from_owned_ptr_or_panic(self, ptr);
        let p = pythonrun::register(self, obj);
        <D as PyDowncastFrom>::unchecked_downcast_from(p)
    }

    pub fn cast_from_ptr_or_err<D>(self, ptr: *mut ffi::PyObject) -> PyResult<&'p D>
        where D: PyDowncastFrom
    {
        let obj = PyObjectPtr::from_owned_ptr_or_err(self, ptr)?;
        unsafe {
            let p = pythonrun::register(self, obj);
            Ok(<D as PyDowncastFrom>::unchecked_downcast_from(p))
        }
    }

    pub fn cast_from_ptr_or_opt<D>(self, ptr: *mut ffi::PyObject) -> Option<&'p D>
        where D: PyDowncastFrom
    {
        if ptr.is_null() {
            None
        } else {
            unsafe {
                let obj = PyObjectPtr::from_owned_ptr(self, ptr);
                let p = pythonrun::register(self, obj);
                Some(<D as PyDowncastFrom>::unchecked_downcast_from(p))
            }
        }
    }

    pub unsafe fn cast_from_borrowed_ptr<D>(self, ptr: *mut ffi::PyObject) -> &'p D
        where D: PyDowncastFrom
    {
        let obj = PyObjectPtr::from_borrowed_ptr(self, ptr);
        let p = pythonrun::register(self, obj);
        <D as PyDowncastFrom>::unchecked_downcast_from(p)
    }

    pub unsafe fn cast_from_borrowed_ptr_or_err<D>(self, ptr: *mut ffi::PyObject)
                                                   -> PyResult<&'p D>
        where D: PyDowncastFrom
    {
        let obj = PyObjectPtr::from_borrowed_ptr_or_err(self, ptr)?;
        let p = pythonrun::register(self, obj);
        Ok(<D as PyDowncastFrom>::unchecked_downcast_from(p))
    }

    pub unsafe fn cast_from_borrowed_ptr_or_opt<D>(self, ptr: *mut ffi::PyObject)
                                                   -> Option<&'p D>
        where D: PyDowncastFrom
    {
        if ptr.is_null() {
            None
        } else {
            let obj = PyObjectPtr::from_borrowed_ptr(self, ptr);
            let p = pythonrun::register(self, obj);
            Some(<D as PyDowncastFrom>::unchecked_downcast_from(p))
        }
    }

    pub unsafe fn mut_cast_from_borrowed_ptr<D>(self, ptr: *mut ffi::PyObject) -> &'p mut D
        where D: PyDowncastFrom
    {
        let obj = PyObjectPtr::from_borrowed_ptr(self, ptr);
        let p = pythonrun::register(self, obj);
        <D as PyDowncastFrom>::unchecked_mut_downcast_from(p)
    }

    pub fn track_object(self, obj: PyObjectPtr) -> &'p PyObject
    {
        unsafe { pythonrun::register(self, obj) }
    }
}

#[cfg(test)]
mod test {
    use Python;
    use objectprotocol::ObjectProtocol;
    use objects::{PyObject, PyBool, PyList, PyInt, PyDict};

    #[test]
    fn test_eval() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Make sure builtin names are accessible
        let v: i32 = py.eval("min(1, 2)", None, None)
            .map_err(|e| e.print(py)).unwrap().extract().unwrap();
        assert_eq!(v, 1);

        let d = PyDict::new(py);
        d.set_item("foo", 13).unwrap();

        // Inject our own global namespace
        let v: i32 = py.eval("foo + 29", Some(d), None).unwrap().extract().unwrap();
        assert_eq!(v, 42);

        // Inject our own local namespace
        let v: i32 = py.eval("foo + 29", None, Some(d)).unwrap().extract().unwrap();
        assert_eq!(v, 42);

        // Make sure builtin names are still accessible when using a local namespace
        let v: i32 = py.eval("min(foo, 2)", None, Some(d)).unwrap().extract().unwrap();
        assert_eq!(v, 2);
    }

    #[test]
    fn test_is_instance() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(py.is_instance::<PyBool, PyObject>(py.True().into()).unwrap());
        let list = PyList::new(py, &[1, 2, 3, 4]);
        assert!(!py.is_instance::<PyBool, _>(list.as_ref()).unwrap());
        assert!(py.is_instance::<PyList, _>(list.as_ref()).unwrap());
    }

    #[test]
    fn test_is_subclass() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(py.is_subclass::<PyBool, PyInt>().unwrap());
        assert!(!py.is_subclass::<PyBool, PyList>().unwrap());
    }
}
