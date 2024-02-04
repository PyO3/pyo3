//! Fundamental properties of objects tied to the Python interpreter.
//!
//! The Python interpreter is not threadsafe. To protect the Python interpreter in multithreaded
//! scenarios there is a global lock, the *global interpreter lock* (hereafter referred to as *GIL*)
//! that must be held to safely interact with Python objects. This is why in PyO3 when you acquire
//! the GIL you get a [`Python`] marker token that carries the *lifetime* of holding the GIL and all
//! borrowed references to Python objects carry this lifetime as well. This will statically ensure
//! that you can never use Python objects after dropping the lock - if you mess this up it will be
//! caught at compile time and your program will fail to compile.
//!
//! It also supports this pattern that many extension modules employ:
//! - Drop the GIL, so that other Python threads can acquire it and make progress themselves
//! - Do something independently of the Python interpreter, like IO, a long running calculation or
//! awaiting a future
//! - Once that is done, reacquire the GIL
//!
//! That API is provided by [`Python::allow_threads`] and enforced via the [`Ungil`] bound on the
//! closure and the return type. This is done by relying on the [`Send`] auto trait. `Ungil` is
//! defined as the following:
//!
//! ```rust
//! pub unsafe trait Ungil {}
//!
//! unsafe impl<T: Send> Ungil for T {}
//! ```
//!
//! We piggy-back off the `Send` auto trait because it is not possible to implement custom auto
//! traits on stable Rust. This is the solution which enables it for as many types as possible while
//! making the API usable.
//!
//! In practice this API works quite well, but it comes with some drawbacks:
//!
//! ## Drawbacks
//!
//! There is no reason to prevent `!Send` types like [`Rc`] from crossing the closure. After all,
//! [`Python::allow_threads`] just lets other Python threads run - it does not itself launch a new
//! thread.
//!
//! ```rust, compile_fail
//! # #[cfg(feature = "nightly")]
//! # compile_error!("this actually works on nightly")
//! use pyo3::prelude::*;
//! use std::rc::Rc;
//!
//! fn main() {
//!     Python::with_gil(|py| {
//!         let rc = Rc::new(5);
//!
//!         py.allow_threads(|| {
//!             // This would actually be fine...
//!             println!("{:?}", *rc);
//!         });
//!     });
//! }
//! ```
//!
//! Because we are using `Send` for something it's not quite meant for, other code that
//! (correctly) upholds the invariants of [`Send`] can cause problems.
//!
//! [`SendWrapper`] is one of those. Per its documentation:
//!
//! > A wrapper which allows you to move around non-Send-types between threads, as long as you
//! > access the contained value only from within the original thread and make sure that it is
//! > dropped from within the original thread.
//!
//! This will "work" to smuggle Python references across the closure, because we're not actually
//! doing anything with threads:
//!
//! ```rust, no_run
//! use pyo3::prelude::*;
//! use pyo3::types::PyString;
//! use send_wrapper::SendWrapper;
//!
//! Python::with_gil(|py| {
//!     let string = PyString::new_bound(py, "foo");
//!
//!     let wrapped = SendWrapper::new(string);
//!
//!     py.allow_threads(|| {
//! # #[cfg(not(feature = "nightly"))]
//! # {
//!         // 💥 Unsound! 💥
//!         let smuggled: &Bound<'_, PyString> = &*wrapped;
//!         println!("{:?}", smuggled);
//! # }
//!     });
//! });
//! ```
//!
//! For now the answer to that is "don't do that".
//!
//! # A proper implementation using an auto trait
//!
//! However on nightly Rust and when PyO3's `nightly` feature is
//! enabled, `Ungil` is defined as the following:
//!
//! ```rust
//! # #[cfg(FALSE)]
//! # {
//! #![feature(auto_traits, negative_impls)]
//!
//! pub unsafe auto trait Ungil {}
//!
//! // It is unimplemented for the `Python` struct and Python objects.
//! impl !Ungil for Python<'_> {}
//! impl !Ungil for ffi::PyObject {}
//!
//! // `Py` wraps it in  a safe api, so this is OK
//! unsafe impl<T> Ungil for Py<T> {}
//! # }
//! ```
//!
//! With this feature enabled, the above two examples will start working and not working, respectively.
//!
//! [`SendWrapper`]: https://docs.rs/send_wrapper/latest/send_wrapper/struct.SendWrapper.html
//! [`Rc`]: std::rc::Rc
//! [`Py`]: crate::Py
use crate::err::{self, PyDowncastError, PyErr, PyResult};
use crate::gil::{GILGuard, GILPool, SuspendGIL};
use crate::impl_::not_send::NotSend;
use crate::type_object::HasPyGilRef;
use crate::types::{
    PyAny, PyDict, PyEllipsis, PyModule, PyNone, PyNotImplemented, PyString, PyType,
};
use crate::version::PythonVersionInfo;
use crate::{ffi, Borrowed, FromPyPointer, IntoPy, Py, PyObject, PyTypeCheck, PyTypeInfo};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_int;

/// Types that are safe to access while the GIL is not held.
///
/// # Safety
///
/// The type must not carry borrowed Python references or, if it does, not allow access to them if
/// the GIL is not held.
///
/// See the [module-level documentation](self) for more information.
///
/// # Examples
///
/// This tracking is currently imprecise as it relies on the [`Send`] auto trait on stable Rust.
/// For example, an `Rc` smart pointer should be usable without the GIL, but we currently prevent that:
///
/// ```compile_fail
/// # use pyo3::prelude::*;
/// use std::rc::Rc;
///
/// Python::with_gil(|py| {
///     let rc = Rc::new(42);
///
///     py.allow_threads(|| {
///         println!("{:?}", rc);
///     });
/// });
/// ```
///
/// This also implies that the interplay between `with_gil` and `allow_threads` is unsound, for example
/// one can circumvent this protection using the [`send_wrapper`](https://docs.rs/send_wrapper/) crate:
///
/// ```no_run
/// # use pyo3::prelude::*;
/// # use pyo3::types::PyString;
/// use send_wrapper::SendWrapper;
///
/// Python::with_gil(|py| {
///     let string = PyString::new_bound(py, "foo");
///
///     let wrapped = SendWrapper::new(string);
///
///     py.allow_threads(|| {
///         let sneaky: &Bound<'_, PyString> = &*wrapped;
///
///         println!("{:?}", sneaky);
///     });
/// });
/// ```
///
/// Fixing this loophole on stable Rust has significant ergonomic issues, but it is fixed when using
/// nightly Rust and the `nightly` feature, c.f. [#2141](https://github.com/PyO3/pyo3/issues/2141).
#[cfg_attr(docsrs, doc(cfg(all())))] // Hide the cfg flag
#[cfg(not(feature = "nightly"))]
pub unsafe trait Ungil {}

#[cfg_attr(docsrs, doc(cfg(all())))] // Hide the cfg flag
#[cfg(not(feature = "nightly"))]
unsafe impl<T: Send> Ungil for T {}

#[cfg(feature = "nightly")]
mod nightly {
    macro_rules! define {
        ($($tt:tt)*) => { $($tt)* }
    }

    define! {
        /// Types that are safe to access while the GIL is not held.
        ///
        /// # Safety
        ///
        /// The type must not carry borrowed Python references or, if it does, not allow access to them if
        /// the GIL is not held.
        ///
        /// See the [module-level documentation](self) for more information.
        ///
        /// # Examples
        ///
        /// Types which are `Ungil` cannot be used in contexts where the GIL was released, e.g.
        ///
        /// ```compile_fail
        /// # use pyo3::prelude::*;
        /// # use pyo3::types::PyString;
        /// Python::with_gil(|py| {
        ///     let string = PyString::new_bound(py, "foo");
        ///
        ///     py.allow_threads(|| {
        ///         println!("{:?}", string);
        ///     });
        /// });
        /// ```
        ///
        /// This applies to the GIL token `Python` itself as well, e.g.
        ///
        /// ```compile_fail
        /// # use pyo3::prelude::*;
        /// Python::with_gil(|py| {
        ///     py.allow_threads(|| {
        ///         drop(py);
        ///     });
        /// });
        /// ```
        ///
        /// On nightly Rust, this is not based on the [`Send`] auto trait and hence we are able
        /// to prevent incorrectly circumventing it using e.g. the [`send_wrapper`](https://docs.rs/send_wrapper/) crate:
        ///
        /// ```compile_fail
        /// # use pyo3::prelude::*;
        /// # use pyo3::types::PyString;
        /// use send_wrapper::SendWrapper;
        ///
        /// Python::with_gil(|py| {
        ///     let string = PyString::new_bound(py, "foo");
        ///
        ///     let wrapped = SendWrapper::new(string);
        ///
        ///     py.allow_threads(|| {
        ///         let sneaky: &PyString = *wrapped;
        ///
        ///         println!("{:?}", sneaky);
        ///     });
        /// });
        /// ```
        ///
        /// This also enables using non-[`Send`] types in `allow_threads`,
        /// at least if they are not also bound to the GIL:
        ///
        /// ```rust
        /// # use pyo3::prelude::*;
        /// use std::rc::Rc;
        ///
        /// Python::with_gil(|py| {
        ///     let rc = Rc::new(42);
        ///
        ///     py.allow_threads(|| {
        ///         println!("{:?}", rc);
        ///     });
        /// });
        /// ```
        pub unsafe auto trait Ungil {}
    }

    impl !Ungil for crate::Python<'_> {}

    // This means that PyString, PyList, etc all inherit !Ungil from  this.
    impl !Ungil for crate::PyAny {}

    // All the borrowing wrappers
    impl<T> !Ungil for crate::PyCell<T> {}
    impl<T> !Ungil for crate::PyRef<'_, T> {}
    impl<T> !Ungil for crate::PyRefMut<'_, T> {}

    // FFI pointees
    impl !Ungil for crate::ffi::PyObject {}
    impl !Ungil for crate::ffi::PyLongObject {}

    impl !Ungil for crate::ffi::PyThreadState {}
    impl !Ungil for crate::ffi::PyInterpreterState {}
    impl !Ungil for crate::ffi::PyWeakReference {}
    impl !Ungil for crate::ffi::PyFrameObject {}
    impl !Ungil for crate::ffi::PyCodeObject {}
    #[cfg(not(Py_LIMITED_API))]
    impl !Ungil for crate::ffi::PyDictKeysObject {}
    #[cfg(not(any(Py_LIMITED_API, Py_3_10)))]
    impl !Ungil for crate::ffi::PyArena {}
}

#[cfg(feature = "nightly")]
pub use nightly::Ungil;

/// A marker token that represents holding the GIL.
///
/// It serves three main purposes:
/// - It provides a global API for the Python interpreter, such as [`Python::eval`].
/// - It can be passed to functions that require a proof of holding the GIL, such as
/// [`Py::clone_ref`].
/// - Its lifetime represents the scope of holding the GIL which can be used to create Rust
/// references that are bound to it, such as `&`[`PyAny`].
///
/// Note that there are some caveats to using it that you might need to be aware of. See the
/// [Deadlocks](#deadlocks) and [Releasing and freeing memory](#releasing-and-freeing-memory)
/// paragraphs for more information about that.
///
/// # Obtaining a Python token
///
/// The following are the recommended ways to obtain a [`Python`] token, in order of preference:
/// - In a function or method annotated with [`#[pyfunction]`](crate::pyfunction) or [`#[pymethods]`](crate::pymethods) you can declare it
/// as a parameter, and PyO3 will pass in the token when Python code calls it.
/// - If you already have something with a lifetime bound to the GIL, such as `&`[`PyAny`], you can
/// use its [`.py()`][PyAny::py] method to get a token.
/// - When you need to acquire the GIL yourself, such as when calling Python code from Rust, you
/// should call [`Python::with_gil`] to do that and pass your code as a closure to it.
///
/// # Deadlocks
///
/// Note that the GIL can be temporarily released by the Python interpreter during a function call
/// (e.g. importing a module). In general, you don't need to worry about this because the GIL is
/// reacquired before returning to the Rust code:
///
/// ```text
/// `Python` exists   |=====================================|
/// GIL actually held |==========|         |================|
/// Rust code running |=======|                |==|  |======|
/// ```
///
/// This behaviour can cause deadlocks when trying to lock a Rust mutex while holding the GIL:
///
///  * Thread 1 acquires the GIL
///  * Thread 1 locks a mutex
///  * Thread 1 makes a call into the Python interpreter which releases the GIL
///  * Thread 2 acquires the GIL
///  * Thread 2 tries to locks the mutex, blocks
///  * Thread 1's Python interpreter call blocks trying to reacquire the GIL held by thread 2
///
/// To avoid deadlocking, you should release the GIL before trying to lock a mutex or `await`ing in
/// asynchronous code, e.g. with [`Python::allow_threads`].
///
/// # Releasing and freeing memory
///
/// The [`Python`] type can be used to create references to variables owned by the Python
/// interpreter, using functions such as [`Python::eval`] and [`PyModule::import`]. These
/// references are tied to a [`GILPool`] whose references are not cleared until it is dropped.
/// This can cause apparent "memory leaks" if it is kept around for a long time.
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::types::PyString;
///
/// # fn main () -> PyResult<()> {
/// Python::with_gil(|py| -> PyResult<()> {
///     for _ in 0..10 {
///         let hello: &PyString = py.eval("\"Hello World!\"", None, None)?.extract()?;
///         println!("Python says: {}", hello.to_str()?);
///         // Normally variables in a loop scope are dropped here, but `hello` is a reference to
///         // something owned by the Python interpreter. Dropping this reference does nothing.
///     }
///     Ok(())
/// })
/// // This is where the `hello`'s reference counts start getting decremented.
/// # }
/// ```
///
/// The variable `hello` is dropped at the end of each loop iteration, but the lifetime of the
/// pointed-to memory is bound to [`Python::with_gil`]'s [`GILPool`] which will not be dropped until
/// the end of [`Python::with_gil`]'s scope. Only then is each `hello`'s Python reference count
/// decreased. This means that at the last line of the example there are 10 copies of `hello` in
/// Python's memory, not just one at a time as we might expect from Rust's [scoping rules].
///
/// See the [Memory Management] chapter of the guide for more information about how PyO3 uses
/// [`GILPool`] to manage memory.
///
/// [scoping rules]: https://doc.rust-lang.org/stable/book/ch04-01-what-is-ownership.html#ownership-rules
/// [`Py::clone_ref`]: crate::Py::clone_ref
/// [Memory Management]: https://pyo3.rs/main/memory.html#gil-bound-memory
#[derive(Copy, Clone)]
pub struct Python<'py>(PhantomData<(&'py GILGuard, NotSend)>);

impl Python<'_> {
    /// Acquires the global interpreter lock, allowing access to the Python interpreter. The
    /// provided closure `F` will be executed with the acquired `Python` marker token.
    ///
    /// If implementing [`#[pymethods]`](crate::pymethods) or [`#[pyfunction]`](crate::pyfunction),
    /// declare `py: Python` as an argument. PyO3 will pass in the token to grant access to the GIL
    /// context in which the function is running, avoiding the need to call `with_gil`.
    ///
    /// If the [`auto-initialize`] feature is enabled and the Python runtime is not already
    /// initialized, this function will initialize it. See
    #[cfg_attr(
        not(PyPy),
        doc = "[`prepare_freethreaded_python`](crate::prepare_freethreaded_python)"
    )]
    #[cfg_attr(PyPy, doc = "`prepare_freethreaded_python`")]
    /// for details.
    ///
    /// If the current thread does not yet have a Python "thread state" associated with it,
    /// a new one will be automatically created before `F` is executed and destroyed after `F`
    /// completes.
    ///
    /// # Panics
    ///
    /// - If the [`auto-initialize`] feature is not enabled and the Python interpreter is not
    ///   initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let x: i32 = py.eval("5", None, None)?.extract()?;
    ///     assert_eq!(x, 5);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// [`auto-initialize`]: https://pyo3.rs/main/features.html#auto-initialize
    #[inline]
    pub fn with_gil<F, R>(f: F) -> R
    where
        F: for<'py> FnOnce(Python<'py>) -> R,
    {
        let _guard = GILGuard::acquire();

        // SAFETY: Either the GIL was already acquired or we just created a new `GILGuard`.
        f(unsafe { Python::assume_gil_acquired() })
    }

    /// Like [`Python::with_gil`] except Python interpreter state checking is skipped.
    ///
    /// Normally when the GIL is acquired, we check that the Python interpreter is an
    /// appropriate state (e.g. it is fully initialized). This function skips those
    /// checks.
    ///
    /// # Safety
    ///
    /// If [`Python::with_gil`] would succeed, it is safe to call this function.
    ///
    /// In most cases, you should use [`Python::with_gil`].
    ///
    /// A justified scenario for calling this function is during multi-phase interpreter
    /// initialization when [`Python::with_gil`] would fail before
    // this link is only valid on 3.8+not pypy and up.
    #[cfg_attr(
        all(Py_3_8, not(PyPy)),
        doc = "[`_Py_InitializeMain`](crate::ffi::_Py_InitializeMain)"
    )]
    #[cfg_attr(any(not(Py_3_8), PyPy), doc = "`_Py_InitializeMain`")]
    /// is called because the interpreter is only partially initialized.
    ///
    /// Behavior in other scenarios is not documented.
    #[inline]
    pub unsafe fn with_gil_unchecked<F, R>(f: F) -> R
    where
        F: for<'py> FnOnce(Python<'py>) -> R,
    {
        let _guard = GILGuard::acquire_unchecked();

        // SAFETY: Either the GIL was already acquired or we just created a new `GILGuard`.
        f(Python::assume_gil_acquired())
    }
}

impl<'py> Python<'py> {
    /// Temporarily releases the GIL, thus allowing other Python threads to run. The GIL will be
    /// reacquired when `F`'s scope ends.
    ///
    /// If you don't need to touch the Python
    /// interpreter for some time and have other Python threads around, this will let you run
    /// Rust-only code while letting those other Python threads make progress.
    ///
    /// Only types that implement [`Ungil`] can cross the closure. See the
    /// [module level documentation](self) for more information.
    ///
    /// If you need to pass Python objects into the closure you can use [`Py`]`<T>`to create a
    /// reference independent of the GIL lifetime. However, you cannot do much with those without a
    /// [`Python`] token, for which you'd need to reacquire the GIL.
    ///
    /// # Example: Releasing the GIL while running a computation in Rust-only code
    ///
    /// ```
    /// use pyo3::prelude::*;
    ///
    /// #[pyfunction]
    /// fn sum_numbers(py: Python<'_>, numbers: Vec<u32>) -> PyResult<u32> {
    ///     // We release the GIL here so any other Python threads get a chance to run.
    ///     py.allow_threads(move || {
    ///         // An example of an "expensive" Rust calculation
    ///         let sum = numbers.iter().sum();
    ///
    ///         Ok(sum)
    ///     })
    /// }
    /// #
    /// # fn main() -> PyResult<()> {
    /// #     Python::with_gil(|py| -> PyResult<()> {
    /// #         let fun = pyo3::wrap_pyfunction!(sum_numbers, py)?;
    /// #         let res = fun.call1((vec![1_u32, 2, 3],))?;
    /// #         assert_eq!(res.extract::<u32>()?, 6_u32);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    ///
    /// Please see the [Parallelism] chapter of the guide for a thorough discussion of using
    /// [`Python::allow_threads`] in this manner.
    ///
    /// # Example: Passing borrowed Python references into the closure is not allowed
    ///
    /// ```compile_fail
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyString;
    ///
    /// fn parallel_print(py: Python<'_>) {
    ///     let s = PyString::new_bound(py, "This object cannot be accessed without holding the GIL >_<");
    ///     py.allow_threads(move || {
    ///         println!("{:?}", s); // This causes a compile error.
    ///     });
    /// }
    /// ```
    ///
    /// [`Py`]: crate::Py
    /// [`PyString`]: crate::types::PyString
    /// [auto-traits]: https://doc.rust-lang.org/nightly/unstable-book/language-features/auto-traits.html
    /// [Parallelism]: https://pyo3.rs/main/parallelism.html
    pub fn allow_threads<T, F>(self, f: F) -> T
    where
        F: Ungil + FnOnce() -> T,
        T: Ungil,
    {
        // Use a guard pattern to handle reacquiring the GIL,
        // so that the GIL will be reacquired even if `f` panics.
        // The `Send` bound on the closure prevents the user from
        // transferring the `Python` token into the closure.
        let _guard = unsafe { SuspendGIL::new() };
        f()
    }

    /// Evaluates a Python expression in the given context and returns the result.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    ///
    /// If `globals` doesn't contain `__builtins__`, default `__builtins__`
    /// will be added automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
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
    ) -> PyResult<&'py PyAny> {
        self.run_code(code, ffi::Py_eval_input, globals, locals)
    }

    /// Executes one or more Python statements in the given context.
    ///
    /// If `globals` is `None`, it defaults to Python module `__main__`.
    /// If `locals` is `None`, it defaults to the value of `globals`.
    ///
    /// If `globals` doesn't contain `__builtins__`, default `__builtins__`
    /// will be added automatically.
    ///
    /// # Examples
    /// ```
    /// use pyo3::{
    ///     prelude::*,
    ///     types::{PyBytes, PyDict},
    /// };
    /// Python::with_gil(|py| {
    ///     let locals = PyDict::new(py);
    ///     py.run(
    ///         r#"
    /// import base64
    /// s = 'Hello Rust!'
    /// ret = base64.b64encode(s.encode('utf-8'))
    /// "#,
    ///         None,
    ///         Some(locals),
    ///     )
    ///     .unwrap();
    ///     let ret = locals.get_item("ret").unwrap().unwrap();
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
    ) -> PyResult<&'py PyAny> {
        let code = CString::new(code)?;
        unsafe {
            let mptr = ffi::PyImport_AddModule("__main__\0".as_ptr() as *const _);
            if mptr.is_null() {
                return Err(PyErr::fetch(self));
            }

            let globals = globals
                .map(|dict| dict.as_ptr())
                .unwrap_or_else(|| ffi::PyModule_GetDict(mptr));
            let locals = locals.map(|dict| dict.as_ptr()).unwrap_or(globals);

            // If `globals` don't provide `__builtins__`, most of the code will fail if Python
            // version is <3.10. That's probably not what user intended, so insert `__builtins__`
            // for them.
            //
            // See also:
            // - https://github.com/python/cpython/pull/24564 (the same fix in CPython 3.10)
            // - https://github.com/PyO3/pyo3/issues/3370
            let builtins_s = crate::intern!(self, "__builtins__").as_ptr();
            let has_builtins = ffi::PyDict_Contains(globals, builtins_s);
            if has_builtins == -1 {
                return Err(PyErr::fetch(self));
            }
            if has_builtins == 0 {
                // Inherit current builtins.
                let builtins = ffi::PyEval_GetBuiltins();

                // `PyDict_SetItem` doesn't take ownership of `builtins`, but `PyEval_GetBuiltins`
                // seems to return a borrowed reference, so no leak here.
                if ffi::PyDict_SetItem(globals, builtins_s, builtins) == -1 {
                    return Err(PyErr::fetch(self));
                }
            }

            let code_obj = ffi::Py_CompileString(code.as_ptr(), "<string>\0".as_ptr() as _, start);
            if code_obj.is_null() {
                return Err(PyErr::fetch(self));
            }
            let res_ptr = ffi::PyEval_EvalCode(code_obj, globals, locals);
            ffi::Py_DECREF(code_obj);

            self.from_owned_ptr_or_err(res_ptr)
        }
    }

    /// Gets the Python type object for type `T`.
    #[inline]
    pub fn get_type<T>(self) -> &'py PyType
    where
        T: PyTypeInfo,
    {
        T::type_object(self)
    }

    /// Imports the Python module with the specified name.
    pub fn import<N>(self, name: N) -> PyResult<&'py PyModule>
    where
        N: IntoPy<Py<PyString>>,
    {
        PyModule::import(self, name)
    }

    /// Gets the Python builtin value `None`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn None(self) -> PyObject {
        PyNone::get(self).into()
    }

    /// Gets the Python builtin value `Ellipsis`, or `...`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn Ellipsis(self) -> PyObject {
        PyEllipsis::get(self).into()
    }

    /// Gets the Python builtin value `NotImplemented`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn NotImplemented(self) -> PyObject {
        PyNotImplemented::get(self).into()
    }

    /// Gets the running Python interpreter version as a string.
    ///
    /// # Examples
    /// ```rust
    /// # use pyo3::Python;
    /// Python::with_gil(|py| {
    ///     // The full string could be, for example:
    ///     // "3.10.0 (tags/v3.10.0:b494f59, Oct  4 2021, 19:00:18) [MSC v.1929 64 bit (AMD64)]"
    ///     assert!(py.version().starts_with("3."));
    /// });
    /// ```
    pub fn version(self) -> &'py str {
        unsafe {
            CStr::from_ptr(ffi::Py_GetVersion())
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
    ///     // PyO3 supports Python 3.7 and up.
    ///     assert!(py.version_info() >= (3, 7));
    ///     assert!(py.version_info() >= (3, 7, 0));
    /// });
    /// ```
    pub fn version_info(self) -> PythonVersionInfo<'py> {
        let version_str = self.version();

        // Portion of the version string returned by Py_GetVersion up to the first space is the
        // version number.
        let version_number_str = version_str.split(' ').next().unwrap_or(version_str);

        PythonVersionInfo::from_str(version_number_str).unwrap()
    }

    /// Registers the object in the release pool, and tries to downcast to specific type.
    pub fn checked_cast_as<T>(self, obj: PyObject) -> Result<&'py T, PyDowncastError<'py>>
    where
        T: PyTypeCheck<AsRefTarget = T>,
    {
        obj.into_ref(self).downcast()
    }

    /// Registers the object in the release pool, and does an unchecked downcast
    /// to the specific type.
    ///
    /// # Safety
    ///
    /// Callers must ensure that ensure that the cast is valid.
    pub unsafe fn cast_as<T>(self, obj: PyObject) -> &'py T
    where
        T: HasPyGilRef<AsRefTarget = T>,
    {
        obj.into_ref(self).downcast_unchecked()
    }

    /// Registers the object pointer in the release pool,
    /// and does an unchecked downcast to the specific type.
    ///
    /// # Safety
    ///
    /// Callers must ensure that ensure that the cast is valid.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn from_owned_ptr<T>(self, ptr: *mut ffi::PyObject) -> &'py T
    where
        T: FromPyPointer<'py>,
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
    pub unsafe fn from_owned_ptr_or_err<T>(self, ptr: *mut ffi::PyObject) -> PyResult<&'py T>
    where
        T: FromPyPointer<'py>,
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
    pub unsafe fn from_owned_ptr_or_opt<T>(self, ptr: *mut ffi::PyObject) -> Option<&'py T>
    where
        T: FromPyPointer<'py>,
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
    pub unsafe fn from_borrowed_ptr<T>(self, ptr: *mut ffi::PyObject) -> &'py T
    where
        T: FromPyPointer<'py>,
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
    pub unsafe fn from_borrowed_ptr_or_err<T>(self, ptr: *mut ffi::PyObject) -> PyResult<&'py T>
    where
        T: FromPyPointer<'py>,
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
    pub unsafe fn from_borrowed_ptr_or_opt<T>(self, ptr: *mut ffi::PyObject) -> Option<&'py T>
    where
        T: FromPyPointer<'py>,
    {
        FromPyPointer::from_borrowed_ptr_or_opt(self, ptr)
    }

    /// Lets the Python interpreter check and handle any pending signals. This will invoke the
    /// corresponding signal handlers registered in Python (if any).
    ///
    /// Returns `Err(`[`PyErr`]`)` if any signal handler raises an exception.
    ///
    /// These signals include `SIGINT` (normally raised by CTRL + C), which by default raises
    /// `KeyboardInterrupt`. For this reason it is good practice to call this function regularly
    /// as part of long-running Rust functions so that users can cancel it.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #![allow(dead_code)] // this example is quite impractical to test
    /// use pyo3::prelude::*;
    ///
    /// # fn main() {
    /// #[pyfunction]
    /// fn loop_forever(py: Python<'_>) -> PyResult<()> {
    ///     loop {
    ///         // As this loop is infinite it should check for signals every once in a while.
    ///         // Using `?` causes any `PyErr` (potentially containing `KeyboardInterrupt`)
    ///         // to break out of the loop.
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
    ///
    /// This function calls [`PyErr_CheckSignals()`][1] which in turn may call signal handlers.
    /// As Python's [`signal`][2] API allows users to define custom signal handlers, calling this
    /// function allows arbitrary Python code inside signal handlers to run.
    ///
    /// If the function is called from a non-main thread, or under a non-main Python interpreter,
    /// it does nothing yet still returns `Ok(())`.
    ///
    /// [1]: https://docs.python.org/3/c-api/exceptions.html?highlight=pyerr_checksignals#c.PyErr_CheckSignals
    /// [2]: https://docs.python.org/3/library/signal.html
    pub fn check_signals(self) -> PyResult<()> {
        err::error_on_minusone(self, unsafe { ffi::PyErr_CheckSignals() })
    }

    /// Create a new pool for managing PyO3's owned references.
    ///
    /// When this `GILPool` is dropped, all PyO3 owned references created after this `GILPool` will
    /// all have their Python reference counts decremented, potentially allowing Python to drop
    /// the corresponding Python objects.
    ///
    /// Typical usage of PyO3 will not need this API, as [`Python::with_gil`] automatically creates
    /// a `GILPool` where appropriate.
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
    ///         let py = pool.python();
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
    ///   `GILPool` (accessible using [`.python()`]) should be used in PyO3 APIs. All other older
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
    ///
    /// [`.python()`]: crate::GILPool::python
    #[inline]
    pub unsafe fn new_pool(self) -> GILPool {
        GILPool::new()
    }
}

impl Python<'_> {
    /// Creates a scope using a new pool for managing PyO3's owned references.
    ///
    /// This is a safe alterantive to [`new_pool`][Self::new_pool] as
    /// it limits the closure to using the new GIL token at the cost of
    /// being unable to capture existing GIL-bound references.
    ///
    /// Note that on stable Rust, this API suffers from the same the `SendWrapper` loophole
    /// as [`allow_threads`][Self::allow_threads], c.f. the documentation of the [`Ungil`] trait,
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use pyo3::prelude::*;
    /// Python::with_gil(|py| {
    ///     // Some long-running process like a webserver, which never releases the GIL.
    ///     loop {
    ///         // Create a new scope, so that PyO3 can clear memory at the end of the loop.
    ///         py.with_pool(|py| {
    ///             // do stuff...
    ///         });
    /// #       break;  // Exit the loop so that doctest terminates!
    ///     }
    /// });
    /// ```
    ///
    /// The `Ungil` bound on the closure does prevent hanging on to existing GIL-bound references
    ///
    /// ```compile_fail
    /// # #![allow(deprecated)]
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::PyString;
    ///
    /// Python::with_gil(|py| {
    ///     let old_str = PyString::new(py, "a message from the past");
    ///
    ///     py.with_pool(|_py| {
    ///         print!("{:?}", old_str);
    ///     });
    /// });
    /// ```
    ///
    /// or continuing to use the old GIL token
    ///
    /// ```compile_fail
    /// # use pyo3::prelude::*;
    ///
    /// Python::with_gil(|old_py| {
    ///     old_py.with_pool(|_new_py| {
    ///         let _none = old_py.None();
    ///     });
    /// });
    /// ```
    #[inline]
    pub fn with_pool<F, R>(&self, f: F) -> R
    where
        F: for<'py> FnOnce(Python<'py>) -> R + Ungil,
    {
        // SAFETY: The closure is `Ungil`,
        // i.e. it does not capture any GIL-bound references
        // and accesses only the newly created GIL token.
        let pool = unsafe { GILPool::new() };

        f(pool.python())
    }
}

impl<'unbound> Python<'unbound> {
    /// Unsafely creates a Python token with an unbounded lifetime.
    ///
    /// Many of PyO3 APIs use `Python<'_>` as proof that the GIL is held, but this function can be
    /// used to call them unsafely.
    ///
    /// # Safety
    ///
    /// - This token and any borrowed Python references derived from it can only be safely used
    /// whilst the currently executing thread is actually holding the GIL.
    /// - This function creates a token with an *unbounded* lifetime. Safe code can assume that
    /// holding a `Python<'py>` token means the GIL is and stays acquired for the lifetime `'py`.
    /// If you let it or borrowed Python references escape to safe code you are
    /// responsible for bounding the lifetime `'unbound` appropriately. For more on unbounded
    /// lifetimes, see the [nomicon].
    ///
    /// [nomicon]: https://doc.rust-lang.org/nomicon/unbounded-lifetimes.html
    #[inline]
    pub unsafe fn assume_gil_acquired() -> Python<'unbound> {
        Python(PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{any::PyAnyMethods, IntoPyDict, PyDict, PyList};
    use std::sync::Arc;

    #[test]
    fn test_eval() {
        Python::with_gil(|py| {
            // Make sure builtin names are accessible
            let v: i32 = py
                .eval("min(1, 2)", None, None)
                .map_err(|e| e.display(py))
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
        });
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    fn test_allow_threads_releases_and_acquires_gil() {
        Python::with_gil(|py| {
            let b = std::sync::Arc::new(std::sync::Barrier::new(2));

            let b2 = b.clone();
            std::thread::spawn(move || Python::with_gil(|_| b2.wait()));

            py.allow_threads(|| {
                // If allow_threads does not release the GIL, this will deadlock because
                // the thread spawned above will never be able to acquire the GIL.
                b.wait();
            });

            unsafe {
                // If the GIL is not reacquired at the end of allow_threads, this call
                // will crash the Python interpreter.
                let tstate = ffi::PyEval_SaveThread();
                ffi::PyEval_RestoreThread(tstate);
            }
        });
    }

    #[test]
    fn test_allow_threads_panics_safely() {
        Python::with_gil(|py| {
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
            let list = PyList::new_bound(py, [1, 2, 3, 4]);
            assert_eq!(list.extract::<Vec<i32>>().unwrap(), vec![1, 2, 3, 4]);
        });
    }

    #[test]
    fn test_allow_threads_pass_stuff_in() {
        let list = Python::with_gil(|py| PyList::new_bound(py, vec!["foo", "bar"]).unbind());
        let mut v = vec![1, 2, 3];
        let a = Arc::new(String::from("foo"));

        Python::with_gil(|py| {
            py.allow_threads(|| {
                drop((list, &mut v, a));
            });
        });
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_acquire_gil() {
        const GIL_NOT_HELD: c_int = 0;
        const GIL_HELD: c_int = 1;

        let state = unsafe { crate::ffi::PyGILState_Check() };
        assert_eq!(state, GIL_NOT_HELD);

        Python::with_gil(|_| {
            let state = unsafe { crate::ffi::PyGILState_Check() };
            assert_eq!(state, GIL_HELD);
        });

        let state = unsafe { crate::ffi::PyGILState_Check() };
        assert_eq!(state, GIL_NOT_HELD);
    }

    #[test]
    fn test_ellipsis() {
        Python::with_gil(|py| {
            assert_eq!(py.Ellipsis().to_string(), "Ellipsis");

            let v = py
                .eval("...", None, None)
                .map_err(|e| e.display(py))
                .unwrap();

            assert!(v.eq(py.Ellipsis()).unwrap());
        });
    }

    #[test]
    fn test_py_run_inserts_globals() {
        Python::with_gil(|py| {
            let namespace = PyDict::new(py);
            py.run("class Foo: pass", Some(namespace), Some(namespace))
                .unwrap();
            assert!(matches!(namespace.get_item("Foo"), Ok(Some(..))));
            assert!(matches!(namespace.get_item("__builtins__"), Ok(Some(..))));
        })
    }
}
