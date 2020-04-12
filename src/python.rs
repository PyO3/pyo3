// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::err::{PyDowncastError, PyErr, PyResult};
use crate::ffi;
use crate::gil::{self, GILGuard};
use crate::instance::AsPyRef;
use crate::object::PyObject;
use crate::type_object::{PyDowncastImpl, PyTypeInfo, PyTypeObject};
use crate::types::{PyAny, PyDict, PyModule, PyType};
use crate::{AsPyPointer, FromPyPointer, IntoPyPointer, PyTryFrom};
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_int;
use std::ptr::NonNull;

pub use gil::prepare_freethreaded_python;

/// Marker type that indicates that the GIL is currently held.
///
/// The `Python` struct is a zero-sized marker struct that is required for most Python operations.
/// This is used to indicate that the operation accesses/modifies the Python interpreter state,
/// and thus can only be called if the Python interpreter is initialized and the
/// Python global interpreter lock (GIL) is acquired. The lifetime `'p` represents the lifetime of
/// holding the lock.
///
/// Note that the GIL can be temporarily released by the Python interpreter during a function call
/// (e.g. importing a module), even when you're holding a GILGuard. In general, you don't need to
/// worry about this because the GIL is reacquired before returning to the Rust code:
///
/// ```text
/// `Python` exists   |=====================================|
/// GIL actually held |==========|         |================|
/// Rust code running |=======|                |==|  |======|
/// ```
///
/// This behaviour can cause deadlocks when trying to lock a Rust mutex while
/// holding the GIL:
///
///  * Thread 1 acquires the GIL
///  * Thread 1 locks a mutex
///  * Thread 1 makes a call into the Python interpreter which releases the GIL
///  * Thread 2 acquires the GIL
///  * Thread 2 tries to locks the mutex, blocks
///  * Thread 1's Python interpreter call blocks trying to reacquire the GIL held by thread 2
///
/// To avoid deadlocking, you should release the GIL before trying to lock a mutex, e.g. with
/// [Python::allow_threads].
#[derive(Copy, Clone)]
pub struct Python<'p>(PhantomData<&'p GILGuard>);

impl<'p> Python<'p> {
    /// Retrieves a Python instance under the assumption that the GIL is already
    /// acquired at this point, and stays acquired for the lifetime `'p`.
    ///
    /// Because the output lifetime `'p` is not connected to any input parameter,
    /// care must be taken that the compiler infers an appropriate lifetime for `'p`
    /// when calling this function.
    ///
    /// # Safety
    /// The lifetime `'p` must be shorter than the period you *assume* that you have GIL.
    /// I.e., `Python<'static>` is always *really* unsafe.
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

    /// Temporarily releases the `GIL`, thus allowing other Python threads to run.
    ///
    /// # Example
    /// ```
    /// # use pyo3::prelude::*; use pyo3::types::IntoPyDict; use pyo3::wrap_pyfunction;
    /// use pyo3::exceptions::RuntimeError;
    /// use std::sync::Arc;
    /// use std::thread;
    /// #[pyfunction]
    /// fn parallel_count(py: Python<'_>, strings: Vec<String>, query: String) -> PyResult<usize> {
    ///     let query = query.chars().next().unwrap();
    ///     py.allow_threads(move || {
    ///         let threads: Vec<_> = strings
    ///             .into_iter()
    ///             .map(|s| thread::spawn(move || s.chars().filter(|&c| c == query).count()))
    ///             .collect();
    ///         let mut sum = 0;
    ///         for t in threads {
    ///             sum += t.join().map_err(|_| PyErr::new::<RuntimeError, _>(()))?;
    ///         }
    ///         Ok(sum)
    ///     })
    /// }
    /// let gil = Python::acquire_gil();
    /// let py = gil.python();
    /// let m = PyModule::new(py, "pcount").unwrap();
    /// m.add_wrapped(wrap_pyfunction!(parallel_count)).unwrap();
    /// let locals = [("pcount", m)].into_py_dict(py);
    /// py.run(r#"
    ///    s = ["Flow", "my", "tears", "the", "Policeman", "Said"]
    ///    assert pcount.parallel_count(s, "a") == 3
    /// "#, None, Some(locals));
    /// ```
    ///
    /// **Note:**
    /// PyO3 types that represent objects with a lifetime tied to holding the GIL
    /// cannot be used in the closure.  This includes `&PyAny` and all the
    /// concrete-typed siblings, like `&PyString`.
    ///
    /// You can convert such references to e.g. `PyObject` or `Py<PyString>`,
    /// which makes them independent of the GIL lifetime.  However, you cannot
    /// do much with those without a `Python<'p>` token, for which you'd need to
    /// reacquire the GIL.
    ///
    /// # Example
    /// ```compile_fail
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::PyString;
    /// fn parallel_print(py: Python<'_>) {
    ///     let s = PyString::new(py, "This object should not be shared >_<");
    ///     py.allow_threads(move || {
    ///         println!("{:?}", s); // This causes a compile error.
    ///     });
    /// }
    /// ```
    pub fn allow_threads<T, F>(self, f: F) -> T
    where
        F: Send + FnOnce() -> T,
    {
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
    ///
    /// # Example:
    /// ```
    /// # use pyo3::{types::{PyBytes, PyDict}, prelude::*};
    /// # let gil = pyo3::Python::acquire_gil();
    /// # let py = gil.python();
    /// let result = py.eval("[i * 10 for i in range(5)]", None, None).unwrap();
    /// let res: Vec<i64> = result.extract().unwrap();
    /// assert_eq!(res, vec![0, 10, 20, 30, 40])
    /// ```
    pub fn eval(
        self,
        code: &str,
        globals: Option<&PyDict>,
        locals: Option<&PyDict>,
    ) -> PyResult<&'p PyAny> {
        self.run_code(code, ffi::Py_eval_input, globals, locals)
    }

    /// Executes one or more Python statements in the given context.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    ///
    /// # Example:
    /// ```
    /// use pyo3::{types::{PyBytes, PyDict}, prelude::*};
    /// let gil = pyo3::Python::acquire_gil();
    /// let py = gil.python();
    /// let locals = PyDict::new(py);
    /// py.run(
    ///     r#"
    /// import base64
    /// s = 'Hello Rust!'
    /// ret = base64.b64encode(s.encode('utf-8'))
    /// "#,
    ///    None,
    ///    Some(locals),
    /// ).unwrap();
    /// let ret = locals.get_item("ret").unwrap();
    /// let b64: &PyBytes = ret.downcast().unwrap();
    /// assert_eq!(b64.as_bytes(), b"SGVsbG8gUnVzdCE=");
    /// ```
    pub fn run(
        self,
        code: &str,
        globals: Option<&PyDict>,
        locals: Option<&PyDict>,
    ) -> PyResult<()> {
        let res = self.run_code(code, ffi::Py_file_input, globals, locals);
        res.map(|obj| {
            debug_assert!(crate::ObjectProtocol::is_none(obj));
        })
    }

    /// Runs code in the given context.
    ///
    /// `start` indicates the type of input expected: one of `Py_single_input`,
    /// `Py_file_input`, or `Py_eval_input`.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    fn run_code(
        self,
        code: &str,
        start: c_int,
        globals: Option<&PyDict>,
        locals: Option<&PyDict>,
    ) -> PyResult<&'p PyAny> {
        let code = CString::new(code)?;
        unsafe {
            let mptr = ffi::PyImport_AddModule("__main__\0".as_ptr() as *const _);
            if mptr.is_null() {
                return Err(PyErr::fetch(self));
            }

            let globals = globals
                .map(AsPyPointer::as_ptr)
                .unwrap_or_else(|| ffi::PyModule_GetDict(mptr));
            let locals = locals.map(AsPyPointer::as_ptr).unwrap_or(globals);

            let res_ptr = ffi::PyRun_StringFlags(
                code.as_ptr(),
                start,
                globals,
                locals,
                ::std::ptr::null_mut(),
            );

            self.from_owned_ptr_or_err(res_ptr)
        }
    }

    /// Gets the Python type object for type `T`.
    pub fn get_type<T>(self) -> &'p PyType
    where
        T: PyTypeObject,
    {
        unsafe { self.from_borrowed_ptr(T::type_object().into_ptr()) }
    }

    /// Imports the Python module with the specified name.
    pub fn import(self, name: &str) -> PyResult<&'p PyModule> {
        PyModule::import(self, name)
    }

    /// Checks whether `obj` is an instance of type `T`.
    ///
    /// This is equivalent to the Python `isinstance` function.
    pub fn is_instance<T: PyTypeObject, V: AsPyPointer>(self, obj: &V) -> PyResult<bool> {
        T::type_object().as_ref(self).is_instance(obj)
    }

    /// Checks whether type `T` is subclass of type `U`.
    ///
    /// This is equivalent to the Python `issubclass` function.
    pub fn is_subclass<T, U>(self) -> PyResult<bool>
    where
        T: PyTypeObject,
        U: PyTypeObject,
    {
        T::type_object().as_ref(self).is_subclass::<U>()
    }

    /// Gets the Python builtin value `None`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn None(self) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(self, ffi::Py_None()) }
    }

    /// Gets the Python builtin value `NotImplemented`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn NotImplemented(self) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(self, ffi::Py_NotImplemented()) }
    }
}

impl<'p> Python<'p> {
    /// Registers the object in the release pool, and tries to downcast to specific type.
    pub fn checked_cast_as<T>(self, obj: PyObject) -> Result<&'p T, PyDowncastError>
    where
        T: PyTryFrom<'p>,
    {
        let obj = unsafe { gil::register_owned(self, obj.into_nonnull()) };
        <T as PyTryFrom>::try_from(obj)
    }

    /// Registers the object in the release pool, and does an unchecked downcast
    /// to the specific type.
    pub unsafe fn cast_as<T>(self, obj: PyObject) -> &'p T
    where
        T: PyDowncastImpl + PyTypeInfo,
    {
        let obj = gil::register_owned(self, obj.into_nonnull());
        T::unchecked_downcast(obj)
    }

    /// Registers the object pointer in the release pool.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_borrowed_ptr_to_obj(self, ptr: *mut ffi::PyObject) -> &'p PyAny {
        match NonNull::new(ptr) {
            Some(p) => gil::register_borrowed(self, p),
            None => crate::err::panic_after_error(),
        }
    }

    /// Registers the object pointer in the release pool,
    /// and does an unchecked downcast to the specific type.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_owned_ptr<T>(self, ptr: *mut ffi::PyObject) -> &'p T
    where
        T: FromPyPointer<'p>,
    {
        FromPyPointer::from_owned_ptr(self, ptr)
    }

    /// Registers the owned object pointer in the release pool.
    ///
    /// Returns `Err(PyErr)` if the pointer is NULL.
    /// Does an unchecked downcast to the specific type.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_owned_ptr_or_err<T>(self, ptr: *mut ffi::PyObject) -> PyResult<&'p T>
    where
        T: FromPyPointer<'p>,
    {
        FromPyPointer::from_owned_ptr_or_err(self, ptr)
    }

    /// Registers the owned object pointer in release pool.
    ///
    /// Returns `None` if the pointer is NULL.
    /// Does an unchecked downcast to the specific type.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_owned_ptr_or_opt<T>(self, ptr: *mut ffi::PyObject) -> Option<&'p T>
    where
        T: FromPyPointer<'p>,
    {
        FromPyPointer::from_owned_ptr_or_opt(self, ptr)
    }

    /// Registers the borrowed object pointer in the release pool.
    ///
    /// Panics if the pointer is NULL.
    /// Does an unchecked downcast to the specific type.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_borrowed_ptr<T>(self, ptr: *mut ffi::PyObject) -> &'p T
    where
        T: FromPyPointer<'p>,
    {
        FromPyPointer::from_borrowed_ptr(self, ptr)
    }

    /// Registers the borrowed object pointer in the release pool.
    ///
    /// Returns `Err(PyErr)` if the pointer is NULL.
    /// Does an unchecked downcast to the specific type.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_borrowed_ptr_or_err<T>(self, ptr: *mut ffi::PyObject) -> PyResult<&'p T>
    where
        T: FromPyPointer<'p>,
    {
        FromPyPointer::from_borrowed_ptr_or_err(self, ptr)
    }

    /// Registers the borrowed object pointer in the release pool.
    ///
    /// Returns `None` if the pointer is NULL.
    /// Does an unchecked downcast to the specific type.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_borrowed_ptr_or_opt<T>(self, ptr: *mut ffi::PyObject) -> Option<&'p T>
    where
        T: FromPyPointer<'p>,
    {
        FromPyPointer::from_borrowed_ptr_or_opt(self, ptr)
    }

    #[doc(hidden)]
    /// Passes value ownership to `Python` object and get reference back.
    /// Value get cleaned up on the GIL release.
    pub fn register_any<T: 'static>(self, ob: T) -> &'p T {
        unsafe { gil::register_any(ob) }
    }

    /// Releases a PyObject reference.
    #[inline]
    pub fn release<T>(self, ob: T)
    where
        T: IntoPyPointer,
    {
        unsafe {
            let ptr = ob.into_ptr();
            if !ptr.is_null() {
                ffi::Py_DECREF(ptr);
            }
        }
    }

    /// Releases a `ffi::PyObject` pointer.
    #[inline]
    pub fn xdecref<T: IntoPyPointer>(self, ptr: T) {
        unsafe { ffi::Py_XDECREF(ptr.into_ptr()) };
    }
}

#[cfg(test)]
mod test {
    use crate::objectprotocol::ObjectProtocol;
    use crate::types::{IntoPyDict, PyAny, PyBool, PyInt, PyList};
    use crate::Python;

    #[test]
    fn test_eval() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Make sure builtin names are accessible
        let v: i32 = py
            .eval("min(1, 2)", None, None)
            .map_err(|e| e.print(py))
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(v, 1);

        let d = [("foo", 13)].into_py_dict(py);

        // Inject our own global namespace
        let v: i32 = py
            .eval("foo + 29", Some(d), None)
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(v, 42);

        // Inject our own local namespace
        let v: i32 = py
            .eval("foo + 29", None, Some(d))
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(v, 42);

        // Make sure builtin names are still accessible when using a local namespace
        let v: i32 = py
            .eval("min(foo, 2)", None, Some(d))
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(v, 2);
    }

    #[test]
    fn test_is_instance() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(py
            .is_instance::<PyBool, PyAny>(PyBool::new(py, true).into())
            .unwrap());
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
