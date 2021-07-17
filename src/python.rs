// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::err::{self, PyDowncastError, PyErr, PyResult};
use crate::gil::{self, GILGuard, GILPool};
use crate::type_object::{PyTypeInfo, PyTypeObject};
use crate::types::{PyAny, PyDict, PyModule, PyType};
use crate::{ffi, AsPyPointer, FromPyPointer, IntoPyPointer, PyNativeType, PyObject, PyTryFrom};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int};

/// Represents the major, minor, and patch (if any) versions of this interpreter.
///
/// See [Python::version].
#[derive(Debug)]
pub struct PythonVersionInfo<'p> {
    /// Python major version (e.g. `3`).
    pub major: u8,
    /// Python minor version (e.g. `11`).
    pub minor: u8,
    /// Python patch version (e.g. `0`).
    pub patch: u8,
    /// Python version suffix, if applicable (e.g. `a0`).
    pub suffix: Option<&'p str>,
}

impl<'p> PythonVersionInfo<'p> {
    /// Parses a hard-coded Python interpreter version string (e.g. 3.9.0a4+).
    ///
    /// Panics if the string is ill-formatted.
    fn from_str(version_number_str: &'p str) -> Self {
        fn split_and_parse_number(version_part: &str) -> (u8, Option<&str>) {
            match version_part.find(|c: char| !c.is_ascii_digit()) {
                None => (version_part.parse().unwrap(), None),
                Some(version_part_suffix_start) => {
                    let (version_part, version_part_suffix) =
                        version_part.split_at(version_part_suffix_start);
                    (version_part.parse().unwrap(), Some(version_part_suffix))
                }
            }
        }

        let mut parts = version_number_str.split('.');
        let major_str = parts.next().expect("Python major version missing");
        let minor_str = parts.next().expect("Python minor version missing");
        let patch_str = parts.next();
        assert!(
            parts.next().is_none(),
            "Python version string has too many parts"
        );

        let major = major_str
            .parse()
            .expect("Python major version not an integer");
        let (minor, suffix) = split_and_parse_number(minor_str);
        if suffix.is_some() {
            assert!(patch_str.is_none());
            return PythonVersionInfo {
                major,
                minor,
                patch: 0,
                suffix,
            };
        }

        let (patch, suffix) = patch_str.map(split_and_parse_number).unwrap_or_default();
        PythonVersionInfo {
            major,
            minor,
            patch,
            suffix,
        }
    }
}

impl PartialEq<(u8, u8)> for PythonVersionInfo<'_> {
    fn eq(&self, other: &(u8, u8)) -> bool {
        self.major == other.0 && self.minor == other.1
    }
}

impl PartialEq<(u8, u8, u8)> for PythonVersionInfo<'_> {
    fn eq(&self, other: &(u8, u8, u8)) -> bool {
        self.major == other.0 && self.minor == other.1 && self.patch == other.2
    }
}

impl PartialOrd<(u8, u8)> for PythonVersionInfo<'_> {
    fn partial_cmp(&self, other: &(u8, u8)) -> Option<std::cmp::Ordering> {
        (self.major, self.minor).partial_cmp(other)
    }
}

impl PartialOrd<(u8, u8, u8)> for PythonVersionInfo<'_> {
    fn partial_cmp(&self, other: &(u8, u8, u8)) -> Option<std::cmp::Ordering> {
        (self.major, self.minor, self.patch).partial_cmp(other)
    }
}

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

impl Python<'_> {
    /// Acquires the global interpreter lock, which allows access to the Python runtime. The
    /// provided closure F will be executed with the acquired `Python` marker token.
    ///
    /// If the `auto-initialize` feature is enabled and the Python runtime is not already
    /// initialized, this function will initialize it. See
    /// [prepare_freethreaded_python()](fn.prepare_freethreaded_python.html) for details.
    ///
    /// # Panics
    /// - If the `auto-initialize` feature is not enabled and the Python interpreter is not
    ///   initialized.
    ///
    /// # Examples
    /// ```
    /// use pyo3::prelude::*;
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let x: i32 = py.eval("5", None, None)?.extract()?;
    ///     assert_eq!(x, 5);
    ///     Ok(())
    /// });
    /// ```
    #[inline]
    pub fn with_gil<F, R>(f: F) -> R
    where
        F: for<'p> FnOnce(Python<'p>) -> R,
    {
        f(unsafe { gil::ensure_gil().python() })
    }
}

impl<'p> Python<'p> {
    /// Acquires the global interpreter lock, which allows access to the Python runtime.
    ///
    /// If the `auto-initialize` feature is enabled and the Python runtime is not already
    /// initialized, this function will initialize it. See
    /// [prepare_freethreaded_python()](fn.prepare_freethreaded_python.html) for details.
    ///
    /// Most users should not need to use this API directly, and should prefer one of two options:
    /// 1. When implementing `#[pymethods]` or `#[pyfunction]` add a function argument
    ///    `py: Python` to receive access to the GIL context in which the function is running.
    /// 2. Use [`Python::with_gil`](#method.with_gil) to run a closure with the GIL, acquiring
    ///    only if needed.
    ///
    /// **Note:** This return type from this function, `GILGuard`, is implemented as a RAII guard
    /// around the C-API Python_EnsureGIL. This means that multiple `acquire_gil()` calls are
    /// allowed, and will not deadlock. However, `GILGuard`s must be dropped in the reverse order
    /// to acquisition. If PyO3 detects this order is not maintained, it may be forced to begin
    /// an irrecoverable panic.
    ///
    /// # Panics
    /// - If the `auto-initialize` feature is not enabled and the Python interpreter is not
    ///   initialized.
    #[inline]
    pub fn acquire_gil() -> GILGuard {
        GILGuard::acquire()
    }

    /// Temporarily releases the `GIL`, thus allowing other Python threads to run.
    ///
    /// # Examples
    /// ```
    /// # use pyo3::prelude::*; use pyo3::types::IntoPyDict;
    /// use pyo3::exceptions::PyRuntimeError;
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
    ///             sum += t.join().map_err(|_| PyRuntimeError::new_err(()))?;
    ///         }
    ///         Ok(sum)
    ///     })
    /// }
    ///
    /// Python::with_gil(|py| {
    ///     let m = PyModule::new(py, "pcount").unwrap();
    ///     m.add_function(wrap_pyfunction!(parallel_count, m).unwrap()).unwrap();
    ///     let locals = [("pcount", m)].into_py_dict(py);
    ///     pyo3::py_run!(py, *locals, r#"
    ///         s = ["Flow", "my", "tears", "the", "Policeman", "Said"]
    ///         assert pcount.parallel_count(s, "a") == 3
    ///     "#);
    /// });
    /// ```
    ///
    /// **Note:**
    /// PyO3 types that represent objects with a lifetime tied to holding the GIL
    /// cannot be used in the closure.  This includes `&PyAny` and all the
    /// concrete-typed siblings, like `&PyString`.
    ///
    /// This is achieved via the `Send` bound on the closure and the return type. This is slightly
    /// more restrictive than necessary, but it's the most fitting solution available in stable
    /// Rust. In the future this bound may be relaxed by a new "auto-trait", if auto-traits
    /// become a stable feature of the Rust language.
    ///
    /// You can convert such references to e.g. `PyObject` or `Py<PyString>`,
    /// which makes them independent of the GIL lifetime.  However, you cannot
    /// do much with those without a `Python<'p>` token, for which you'd need to
    /// reacquire the GIL.
    ///
    /// # Examples
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
        T: Send,
    {
        // The `Send` bound on the closure prevents the user from
        // transferring the `Python` token into the closure.
        let count = gil::GIL_COUNT.with(|c| c.replace(0));
        let tstate = unsafe { ffi::PyEval_SaveThread() };
        // Unwinding right here corrupts the Python interpreter state and leads to weird
        // crashes such as stack overflows. We will catch the unwind and resume as soon as
        // we've restored the GIL state.
        //
        // Because we will resume unwinding as soon as the GIL state is fixed, we can assert
        // that the closure is unwind safe.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

        // Restore GIL state
        gil::GIL_COUNT.with(|c| c.set(count));
        unsafe {
            ffi::PyEval_RestoreThread(tstate);
        }

        // Now that the GIL state has been safely reset, we can unwind if a panic was caught.
        result.unwrap_or_else(|payload| std::panic::resume_unwind(payload))
    }

    /// Evaluates a Python expression in the given context and returns the result.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    ///
    /// # Examples
    /// ```
    /// # use pyo3::{types::{PyBytes, PyDict}, prelude::*};
    /// # Python::with_gil(|py| {
    /// let result = py.eval("[i * 10 for i in range(5)]", None, None).unwrap();
    /// let res: Vec<i64> = result.extract().unwrap();
    /// assert_eq!(res, vec![0, 10, 20, 30, 40])
    /// # });
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
    /// # Examples
    /// ```
    /// use pyo3::{types::{PyBytes, PyDict}, prelude::*};
    /// Python::with_gil(|py| {
    ///     let locals = PyDict::new(py);
    ///     py.run(
    ///         r#"
    /// import base64
    /// s = 'Hello Rust!'
    /// ret = base64.b64encode(s.encode('utf-8'))
    /// "#,
    ///       None,
    ///       Some(locals),
    ///     )
    ///     .unwrap();
    ///     let ret = locals.get_item("ret").unwrap();
    ///     let b64: &PyBytes = ret.downcast().unwrap();
    ///     assert_eq!(b64.as_bytes(), b"SGVsbG8gUnVzdCE=");
    /// });
    /// ```
    ///
    /// You can use [`py_run!`](macro.py_run.html) for a handy alternative of `run`
    /// if you don't need `globals` and unwrapping is OK.
    pub fn run(
        self,
        code: &str,
        globals: Option<&PyDict>,
        locals: Option<&PyDict>,
    ) -> PyResult<()> {
        let res = self.run_code(code, ffi::Py_file_input, globals, locals);
        res.map(|obj| {
            debug_assert!(obj.is_none());
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
                return Err(PyErr::api_call_failed(self));
            }

            let globals = globals
                .map(AsPyPointer::as_ptr)
                .unwrap_or_else(|| ffi::PyModule_GetDict(mptr));
            let locals = locals.map(AsPyPointer::as_ptr).unwrap_or(globals);

            let code_obj = ffi::Py_CompileString(code.as_ptr(), "<string>\0".as_ptr() as _, start);
            if code_obj.is_null() {
                return Err(PyErr::api_call_failed(self));
            }
            let res_ptr = ffi::PyEval_EvalCode(code_obj, globals, locals);

            self.from_owned_ptr_or_err(res_ptr)
        }
    }

    /// Gets the Python type object for type `T`.
    pub fn get_type<T>(self) -> &'p PyType
    where
        T: PyTypeObject,
    {
        T::type_object(self)
    }

    /// Imports the Python module with the specified name.
    pub fn import(self, name: &str) -> PyResult<&'p PyModule> {
        PyModule::import(self, name)
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

    /// Gets the running Python interpreter version as a string.
    ///
    /// This is a wrapper around the ffi call Py_GetVersion.
    ///
    /// # Examples
    /// ```rust
    /// # use pyo3::Python;
    /// Python::with_gil(|py| {
    ///     // The full string could be, for example:
    ///     // "3.0a5+ (py3k:63103M, May 12 2008, 00:53:55) \n[GCC 4.2.3]"
    ///     assert!(py.version().starts_with("3."));
    /// });
    /// ```
    pub fn version(self) -> &'p str {
        unsafe {
            CStr::from_ptr(ffi::Py_GetVersion() as *const c_char)
                .to_str()
                .expect("Python version string not UTF-8")
        }
    }

    /// Gets the running Python interpreter version as a struct similar to
    /// `sys.version_info`.
    ///
    /// # Examples
    /// ```rust
    /// # use pyo3::Python;
    /// Python::with_gil(|py| {
    ///     // PyO3 supports Python 3.6 and up.
    ///     assert!(py.version_info() >= (3, 6));
    ///     assert!(py.version_info() >= (3, 6, 0));
    /// });
    /// ```
    pub fn version_info(self) -> PythonVersionInfo<'p> {
        let version_str = self.version();

        // Portion of the version string returned by Py_GetVersion up to the first space is the
        // version number.
        let version_number_str = version_str.split(' ').next().unwrap_or(version_str);

        PythonVersionInfo::from_str(version_number_str)
    }

    /// Registers the object in the release pool, and tries to downcast to specific type.
    pub fn checked_cast_as<T>(self, obj: PyObject) -> Result<&'p T, PyDowncastError<'p>>
    where
        T: PyTryFrom<'p>,
    {
        let any: &PyAny = unsafe { self.from_owned_ptr(obj.into_ptr()) };
        <T as PyTryFrom>::try_from(any)
    }

    /// Registers the object in the release pool, and does an unchecked downcast
    /// to the specific type.
    ///
    /// # Safety
    ///
    /// Callers must ensure that ensure that the cast is valid.
    pub unsafe fn cast_as<T>(self, obj: PyObject) -> &'p T
    where
        T: PyNativeType + PyTypeInfo,
    {
        let any: &PyAny = self.from_owned_ptr(obj.into_ptr());
        T::unchecked_downcast(any)
    }

    /// Registers the object pointer in the release pool,
    /// and does an unchecked downcast to the specific type.
    ///
    /// # Safety
    ///
    /// Callers must ensure that ensure that the cast is valid.
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
    ///
    /// # Safety
    ///
    /// Callers must ensure that ensure that the cast is valid.
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
    ///
    /// # Safety
    ///
    /// Callers must ensure that ensure that the cast is valid.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_owned_ptr_or_opt<T>(self, ptr: *mut ffi::PyObject) -> Option<&'p T>
    where
        T: FromPyPointer<'p>,
    {
        FromPyPointer::from_owned_ptr_or_opt(self, ptr)
    }

    /// Does an unchecked downcast to the specific type.
    ///
    /// Panics if the pointer is NULL.
    ///
    /// # Safety
    ///
    /// Callers must ensure that ensure that the cast is valid.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_borrowed_ptr<T>(self, ptr: *mut ffi::PyObject) -> &'p T
    where
        T: FromPyPointer<'p>,
    {
        FromPyPointer::from_borrowed_ptr(self, ptr)
    }

    /// Does an unchecked downcast to the specific type.
    ///
    /// Returns `Err(PyErr)` if the pointer is NULL.
    ///
    /// # Safety
    ///
    /// Callers must ensure that ensure that the cast is valid.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_borrowed_ptr_or_err<T>(self, ptr: *mut ffi::PyObject) -> PyResult<&'p T>
    where
        T: FromPyPointer<'p>,
    {
        FromPyPointer::from_borrowed_ptr_or_err(self, ptr)
    }

    /// Does an unchecked downcast to the specific type.
    ///
    /// Returns `None` if the pointer is NULL.
    ///
    /// # Safety
    ///
    /// Callers must ensure that ensure that the cast is valid.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_borrowed_ptr_or_opt<T>(self, ptr: *mut ffi::PyObject) -> Option<&'p T>
    where
        T: FromPyPointer<'p>,
    {
        FromPyPointer::from_borrowed_ptr_or_opt(self, ptr)
    }

    /// Lets the Python interpreter check and handle any pending signals. This will invoke the
    /// corresponding signal handlers registered in Python (if any).
    ///
    /// Returns `Err(PyErr)` if any signal handler raises an exception.
    ///
    /// These signals include `SIGINT` (normally raised by CTRL + C), which by default raises
    /// `KeyboardInterrupt`. For this reason it is good practice to call this function regularly
    /// as part of long-running Rust functions so that users can cancel it.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// # fn main(){
    /// #[pyfunction]
    /// fn loop_forever(py: Python) -> PyResult<()> {
    ///     loop {
    ///         // As this loop is infinite it should check for signals every once in a while.
    ///         // Using `?` causes any `PyErr` (potentially containing `KeyboardInterrupt`) to break out of the loop.
    ///         py.check_signals()?;
    ///
    ///         // do work here
    ///         # break Ok(()) // don't actually loop forever
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// # Note
    /// This function calls [`PyErr_CheckSignals()`][1] which in turn may call signal handlers.
    /// As Python's [`signal`][2] API allows users to define custom signal handlers, calling this
    /// function allows arbitary Python code inside signal handlers to run.
    ///
    /// [1]: https://docs.python.org/3/c-api/exceptions.html?highlight=pyerr_checksignals#c.PyErr_CheckSignals
    /// [2]: https://docs.python.org/3/library/signal.html
    pub fn check_signals(self) -> PyResult<()> {
        let v = unsafe { ffi::PyErr_CheckSignals() };
        err::error_on_minusone(self, v)
    }

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

    /// Create a new pool for managing PyO3's owned references.
    ///
    /// When this `GILPool` is dropped, all PyO3 owned references created after this `GILPool` will
    /// all have their Python reference counts decremented, potentially allowing Python to drop
    /// the corresponding Python objects.
    ///
    /// Typical usage of PyO3 will not need this API, as `Python::acquire_gil` automatically
    /// creates a `GILPool` where appropriate.
    ///
    /// Advanced uses of PyO3 which perform long-running tasks which never free the GIL may need
    /// to use this API to clear memory, as PyO3 usually does not clear memory until the GIL is
    /// released.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use pyo3::prelude::*;
    /// Python::with_gil(|py| {
    ///     // Some long-running process like a webserver, which never releases the GIL.
    ///     loop {
    ///         // Create a new pool, so that PyO3 can clear memory at the end of the loop.
    ///         let pool = unsafe { py.new_pool() };
    ///
    ///         // It is recommended to *always* immediately set py to the pool's Python, to help
    ///         // avoid creating references with invalid lifetimes.
    ///         let py = unsafe { pool.python() };
    ///
    ///         // do stuff...
    /// #       break;  // Exit the loop so that doctest terminates!
    ///     }
    /// });
    /// ```
    ///
    /// # Safety
    ///
    /// Extreme care must be taken when using this API, as misuse can lead to accessing invalid
    /// memory. In addition, the caller is responsible for guaranteeing that the GIL remains held
    /// for the entire lifetime of the returned `GILPool`.
    ///
    /// Two best practices are required when using this API:
    /// - From the moment `new_pool()` is called, only the `Python` token from the returned
    ///   `GILPool` (accessible using `.python()`) should be used in PyO3 APIs. All other older
    ///   `Python` tokens with longer lifetimes are unsafe to use until the `GILPool` is dropped,
    ///   because they can be used to create PyO3 owned references which have lifetimes which
    ///   outlive the `GILPool`.
    /// - Similarly, methods on existing owned references will implicitly refer back to the
    ///   `Python` token which that reference was originally created with. If the returned values
    ///   from these methods are owned references they will inherit the same lifetime. As a result,
    ///   Rust's lifetime rules may allow them to outlive the `GILPool`, even though this is not
    ///   safe for reasons discussed above. Care must be taken to never access these return values
    ///   after the `GILPool` is dropped, unless they are converted to `Py<T>` *before* the pool
    ///   is dropped.
    #[inline]
    pub unsafe fn new_pool(self) -> GILPool {
        GILPool::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{IntoPyDict, PyList};

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
    fn test_allow_threads_panics_safely() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let result = std::panic::catch_unwind(|| unsafe {
            let py = Python::assume_gil_acquired();
            py.allow_threads(|| {
                panic!("There was a panic!");
            });
        });

        // Check panic was caught
        assert!(result.is_err());

        // If allow_threads is implemented correctly, this thread still owns the GIL here
        // so the following Python calls should not cause crashes.
        let list = PyList::new(py, &[1, 2, 3, 4]);
        assert_eq!(list.extract::<Vec<i32>>().unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_python_version_info() {
        Python::with_gil(|py| {
            let version = py.version_info();
            #[cfg(Py_3_6)]
            assert!(version >= (3, 6));
            #[cfg(Py_3_6)]
            assert!(version >= (3, 6, 0));
            #[cfg(Py_3_7)]
            assert!(version >= (3, 7));
            #[cfg(Py_3_7)]
            assert!(version >= (3, 7, 0));
            #[cfg(Py_3_8)]
            assert!(version >= (3, 8));
            #[cfg(Py_3_8)]
            assert!(version >= (3, 8, 0));
            #[cfg(Py_3_9)]
            assert!(version >= (3, 9));
            #[cfg(Py_3_9)]
            assert!(version >= (3, 9, 0));
        });
    }

    #[test]
    fn test_python_version_info_parse() {
        assert!(PythonVersionInfo::from_str("3.5.0a1") >= (3, 5, 0));
        assert!(PythonVersionInfo::from_str("3.5+") >= (3, 5, 0));
        assert!(PythonVersionInfo::from_str("3.5+") == (3, 5, 0));
        assert!(PythonVersionInfo::from_str("3.5+") != (3, 5, 1));
        assert!(PythonVersionInfo::from_str("3.5.2a1+") < (3, 5, 3));
        assert!(PythonVersionInfo::from_str("3.5.2a1+") == (3, 5, 2));
        assert!(PythonVersionInfo::from_str("3.5.2a1+") == (3, 5));
        assert!(PythonVersionInfo::from_str("3.5+") == (3, 5));
        assert!(PythonVersionInfo::from_str("3.5.2a1+") < (3, 6));
        assert!(PythonVersionInfo::from_str("3.5.2a1+") > (3, 4));
    }
}
