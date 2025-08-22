//! Fundamental properties of objects tied to the Python interpreter.
//!
//! The Python interpreter is not thread-safe. To protect the Python interpreter in multithreaded
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
//!   awaiting a future
//! - Once that is done, reacquire the GIL
//!
//! That API is provided by [`Python::detach`] and enforced via the [`Ungil`] bound on the
//! closure and the return type. This is done by relying on the [`Send`] auto trait. `Ungil` is
//! defined as the following:
//!
//! ```rust,no_run
//! # #![allow(dead_code)]
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
//! [`Python::detach`] just lets other Python threads run - it does not itself launch a new
//! thread.
//!
//! ```rust, compile_fail
//! # #[cfg(feature = "nightly")]
//! # compile_error!("this actually works on nightly")
//! use pyo3::prelude::*;
//! use std::rc::Rc;
//!
//! fn main() {
//!     Python::attach(|py| {
//!         let rc = Rc::new(5);
//!
//!         py.detach(|| {
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
//! Python::attach(|py| {
//!     let string = PyString::new(py, "foo");
//!
//!     let wrapped = SendWrapper::new(string);
//!
//!     py.detach(|| {
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
//! ```rust,no_run
//! # #[cfg(any())]
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
use crate::conversion::IntoPyObject;
use crate::err::{self, PyResult};
use crate::internal::state::{AttachGuard, SuspendAttach};
use crate::types::any::PyAnyMethods;
use crate::types::{
    PyAny, PyCode, PyCodeMethods, PyDict, PyEllipsis, PyModule, PyNone, PyNotImplemented, PyString,
    PyType,
};
use crate::version::PythonVersionInfo;
use crate::{ffi, Bound, Py, PyTypeInfo};
use std::ffi::CStr;
use std::marker::PhantomData;

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
/// Python::attach(|py| {
///     let rc = Rc::new(42);
///
///     py.detach(|| {
///         println!("{:?}", rc);
///     });
/// });
/// ```
///
/// This also implies that the interplay between `attach` and `detach` is unsound, for example
/// one can circumvent this protection using the [`send_wrapper`](https://docs.rs/send_wrapper/) crate:
///
/// ```no_run
/// # use pyo3::prelude::*;
/// # use pyo3::types::PyString;
/// use send_wrapper::SendWrapper;
///
/// Python::attach(|py| {
///     let string = PyString::new(py, "foo");
///
///     let wrapped = SendWrapper::new(string);
///
///     py.detach(|| {
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
        /// Python::attach(|py| {
        ///     let string = PyString::new(py, "foo");
        ///
        ///     py.detach(|| {
        ///         println!("{:?}", string);
        ///     });
        /// });
        /// ```
        ///
        /// This applies to the GIL token `Python` itself as well, e.g.
        ///
        /// ```compile_fail
        /// # use pyo3::prelude::*;
        /// Python::attach(|py| {
        ///     py.detach(|| {
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
        /// Python::attach(|py| {
        ///     let string = PyString::new(py, "foo");
        ///
        ///     let wrapped = SendWrapper::new(string);
        ///
        ///     py.detach(|| {
        ///         let sneaky: &PyString = *wrapped;
        ///
        ///         println!("{:?}", sneaky);
        ///     });
        /// });
        /// ```
        ///
        /// This also enables using non-[`Send`] types in `detach`,
        /// at least if they are not also bound to the GIL:
        ///
        /// ```rust
        /// # use pyo3::prelude::*;
        /// use std::rc::Rc;
        ///
        /// Python::attach(|py| {
        ///     let rc = Rc::new(42);
        ///
        ///     py.detach(|| {
        ///         println!("{:?}", rc);
        ///     });
        /// });
        /// ```
        pub unsafe auto trait Ungil {}
    }

    impl !Ungil for crate::Python<'_> {}

    // This means that PyString, PyList, etc all inherit !Ungil from  this.
    impl !Ungil for crate::PyAny {}

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
///   [`Py::clone_ref`](crate::Py::clone_ref).
/// - Its lifetime represents the scope of holding the GIL which can be used to create Rust
///   references that are bound to it, such as [`Bound<'py, PyAny>`].
///
/// Note that there are some caveats to using it that you might need to be aware of. See the
/// [Deadlocks](#deadlocks) and [Releasing and freeing memory](#releasing-and-freeing-memory)
/// paragraphs for more information about that.
///
/// # Obtaining a Python token
///
/// The following are the recommended ways to obtain a [`Python<'py>`] token, in order of preference:
/// - If you already have something with a lifetime bound to the GIL, such as [`Bound<'py, PyAny>`], you can
///   use its `.py()` method to get a token.
/// - In a function or method annotated with [`#[pyfunction]`](crate::pyfunction) or [`#[pymethods]`](crate::pymethods) you can declare it
///   as a parameter, and PyO3 will pass in the token when Python code calls it.
/// - When you need to acquire the GIL yourself, such as when calling Python code from Rust, you
///   should call [`Python::attach`] to do that and pass your code as a closure to it.
///
/// The first two options are zero-cost; [`Python::attach`] requires runtime checking and may need to block
/// to acquire the GIL.
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
/// asynchronous code, e.g. with [`Python::detach`].
///
/// # Releasing and freeing memory
///
/// The [`Python<'py>`] type can be used to create references to variables owned by the Python
/// interpreter, using functions such as [`Python::eval`] and [`PyModule::import`].
#[derive(Copy, Clone)]
pub struct Python<'py>(PhantomData<&'py AttachGuard>, PhantomData<NotSend>);

/// A marker type that makes the type !Send.
/// Workaround for lack of !Send on stable (<https://github.com/rust-lang/rust/issues/68318>).
struct NotSend(PhantomData<*mut Python<'static>>);

impl Python<'_> {
    /// See [Python::attach]
    #[inline]
    #[track_caller]
    #[deprecated(note = "use `Python::attach` instead", since = "0.26.0")]
    pub fn with_gil<F, R>(f: F) -> R
    where
        F: for<'py> FnOnce(Python<'py>) -> R,
    {
        Self::attach(f)
    }

    /// Acquires the global interpreter lock, allowing access to the Python interpreter. The
    /// provided closure `F` will be executed with the acquired `Python` marker token.
    ///
    /// If implementing [`#[pymethods]`](crate::pymethods) or [`#[pyfunction]`](crate::pyfunction),
    /// declare `py: Python` as an argument. PyO3 will pass in the token to grant access to the GIL
    /// context in which the function is running, avoiding the need to call `attach`.
    ///
    /// If the [`auto-initialize`] feature is enabled and the Python runtime is not already
    /// initialized, this function will initialize it. See
    #[cfg_attr(
        not(any(PyPy, GraalPy)),
        doc = "[`Python::initialize`](crate::marker::Python::initialize)"
    )]
    #[cfg_attr(PyPy, doc = "`Python::initialize")]
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
    /// - If the Python interpreter is in the process of [shutting down].
    /// - If the middle of GC traversal.
    ///
    /// To avoid possible initialization or panics if calling in a context where the Python
    /// interpreter might be unavailable, consider using [`Python::try_attach`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pyo3::prelude::*;
    /// use pyo3::ffi::c_str;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| -> PyResult<()> {
    ///     let x: i32 = py.eval(c_str!("5"), None, None)?.extract()?;
    ///     assert_eq!(x, 5);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// [`auto-initialize`]: https://pyo3.rs/main/features.html#auto-initialize
    /// [shutting down]: https://docs.python.org/3/glossary.html#term-interpreter-shutdown
    #[inline]
    #[track_caller]
    pub fn attach<F, R>(f: F) -> R
    where
        F: for<'py> FnOnce(Python<'py>) -> R,
    {
        let guard = AttachGuard::attach();
        f(guard.python())
    }

    /// Variant of [`Python::attach`] which will return without attaching to the Python
    /// interpreter if the interpreter is in a state where it cannot be attached to:
    /// - in the middle of GC traversal
    /// - in the process of shutting down
    /// - not initialized
    ///
    /// Note that due to the nature of the underlying Python APIs used to implement this,
    /// the behavior is currently provided on a best-effort basis; it is expected that a
    /// future CPython version will introduce APIs which guarantee this behaviour. This
    /// function is still recommended for use in the meanwhile as it provides the best
    /// possible behaviour and should transparently change to an optimal implementation
    /// once such APIs are available.
    #[inline]
    #[track_caller]
    pub fn try_attach<F, R>(f: F) -> Option<R>
    where
        F: for<'py> FnOnce(Python<'py>) -> R,
    {
        let guard = AttachGuard::try_attach().ok()?;
        Some(f(guard.python()))
    }

    /// Prepares the use of Python.
    ///
    /// If the Python interpreter is not already initialized, this function will initialize it with
    /// signal handling disabled (Python will not raise the `KeyboardInterrupt` exception). Python
    /// signal handling depends on the notion of a 'main thread', which must be the thread that
    /// initializes the Python interpreter.
    ///
    /// If the Python interpreter is already initialized, this function has no effect.
    ///
    /// This function is unavailable under PyPy because PyPy cannot be embedded in Rust (or any other
    /// software). Support for this is tracked on the
    /// [PyPy issue tracker](https://github.com/pypy/pypy/issues/3836).
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::initialize();
    /// Python::attach(|py| py.run(pyo3::ffi::c_str!("print('Hello World')"), None, None))
    /// # }
    /// ```
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn initialize() {
        crate::interpreter_lifecycle::initialize();
    }

    /// See [Python::attach_unchecked]
    /// # Safety
    ///
    /// If [`Python::attach`] would succeed, it is safe to call this function.
    #[inline]
    #[track_caller]
    #[deprecated(note = "use `Python::attach_unchecked` instead", since = "0.26.0")]
    pub unsafe fn with_gil_unchecked<F, R>(f: F) -> R
    where
        F: for<'py> FnOnce(Python<'py>) -> R,
    {
        unsafe { Self::attach_unchecked(f) }
    }

    /// Like [`Python::attach`] except Python interpreter state checking is skipped.
    ///
    /// Normally when attaching to the Python interpreter, PyO3 checks that it is in
    /// an appropriate state (e.g. it is fully initialized). This function skips
    /// those checks.
    ///
    /// # Safety
    ///
    /// If [`Python::attach`] would succeed, it is safe to call this function.
    #[inline]
    #[track_caller]
    pub unsafe fn attach_unchecked<F, R>(f: F) -> R
    where
        F: for<'py> FnOnce(Python<'py>) -> R,
    {
        let guard = unsafe { AttachGuard::attach_unchecked() };

        f(guard.python())
    }
}

impl<'py> Python<'py> {
    /// See [Python::detach]
    #[inline]
    #[deprecated(note = "use `Python::detach` instead", since = "0.26.0")]
    pub fn allow_threads<T, F>(self, f: F) -> T
    where
        F: Ungil + FnOnce() -> T,
        T: Ungil,
    {
        self.detach(f)
    }

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
    ///     py.detach(move || {
    ///         // An example of an "expensive" Rust calculation
    ///         let sum = numbers.iter().sum();
    ///
    ///         Ok(sum)
    ///     })
    /// }
    /// #
    /// # fn main() -> PyResult<()> {
    /// #     Python::attach(|py| -> PyResult<()> {
    /// #         let fun = pyo3::wrap_pyfunction!(sum_numbers, py)?;
    /// #         let res = fun.call1((vec![1_u32, 2, 3],))?;
    /// #         assert_eq!(res.extract::<u32>()?, 6_u32);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    ///
    /// Please see the [Parallelism] chapter of the guide for a thorough discussion of using
    /// [`Python::detach`] in this manner.
    ///
    /// # Example: Passing borrowed Python references into the closure is not allowed
    ///
    /// ```compile_fail
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyString;
    ///
    /// fn parallel_print(py: Python<'_>) {
    ///     let s = PyString::new(py, "This object cannot be accessed without holding the GIL >_<");
    ///     py.detach(move || {
    ///         println!("{:?}", s); // This causes a compile error.
    ///     });
    /// }
    /// ```
    ///
    /// [`Py`]: crate::Py
    /// [`PyString`]: crate::types::PyString
    /// [auto-traits]: https://doc.rust-lang.org/nightly/unstable-book/language-features/auto-traits.html
    /// [Parallelism]: https://pyo3.rs/main/parallelism.html
    pub fn detach<T, F>(self, f: F) -> T
    where
        F: Ungil + FnOnce() -> T,
        T: Ungil,
    {
        // Use a guard pattern to handle reacquiring the GIL,
        // so that the GIL will be reacquired even if `f` panics.
        // The `Send` bound on the closure prevents the user from
        // transferring the `Python` token into the closure.
        let _guard = unsafe { SuspendAttach::new() };
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
    /// # use pyo3::ffi::c_str;
    /// # Python::attach(|py| {
    /// let result = py.eval(c_str!("[i * 10 for i in range(5)]"), None, None).unwrap();
    /// let res: Vec<i64> = result.extract().unwrap();
    /// assert_eq!(res, vec![0, 10, 20, 30, 40])
    /// # });
    /// ```
    pub fn eval(
        self,
        code: &CStr,
        globals: Option<&Bound<'py, PyDict>>,
        locals: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let code = PyCode::compile(
            self,
            code,
            ffi::c_str!("<string>"),
            crate::types::PyCodeInput::Eval,
        )?;
        code.run(globals, locals)
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
    ///     ffi::c_str,
    /// };
    /// Python::attach(|py| {
    ///     let locals = PyDict::new(py);
    ///     py.run(c_str!(
    ///         r#"
    /// import base64
    /// s = 'Hello Rust!'
    /// ret = base64.b64encode(s.encode('utf-8'))
    /// "#),
    ///         None,
    ///         Some(&locals),
    ///     )
    ///     .unwrap();
    ///     let ret = locals.get_item("ret").unwrap().unwrap();
    ///     let b64 = ret.cast::<PyBytes>().unwrap();
    ///     assert_eq!(b64.as_bytes(), b"SGVsbG8gUnVzdCE=");
    /// });
    /// ```
    ///
    /// You can use [`py_run!`](macro.py_run.html) for a handy alternative of `run`
    /// if you don't need `globals` and unwrapping is OK.
    pub fn run(
        self,
        code: &CStr,
        globals: Option<&Bound<'py, PyDict>>,
        locals: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<()> {
        let code = PyCode::compile(
            self,
            code,
            ffi::c_str!("<string>"),
            crate::types::PyCodeInput::File,
        )?;
        code.run(globals, locals).map(|obj| {
            debug_assert!(obj.is_none());
        })
    }

    /// Gets the Python type object for type `T`.
    #[inline]
    pub fn get_type<T>(self) -> Bound<'py, PyType>
    where
        T: PyTypeInfo,
    {
        T::type_object(self)
    }

    /// Imports the Python module with the specified name.
    pub fn import<N>(self, name: N) -> PyResult<Bound<'py, PyModule>>
    where
        N: IntoPyObject<'py, Target = PyString>,
    {
        PyModule::import(self, name)
    }

    /// Gets the Python builtin value `None`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn None(self) -> Py<PyAny> {
        PyNone::get(self).to_owned().into_any().unbind()
    }

    /// Gets the Python builtin value `Ellipsis`, or `...`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn Ellipsis(self) -> Py<PyAny> {
        PyEllipsis::get(self).to_owned().into_any().unbind()
    }

    /// Gets the Python builtin value `NotImplemented`.
    #[allow(non_snake_case)] // the Python keyword starts with uppercase
    #[inline]
    pub fn NotImplemented(self) -> Py<PyAny> {
        PyNotImplemented::get(self).to_owned().into_any().unbind()
    }

    /// Gets the running Python interpreter version as a string.
    ///
    /// # Examples
    /// ```rust
    /// # use pyo3::Python;
    /// Python::attach(|py| {
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
    /// Python::attach(|py| {
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

    /// Lets the Python interpreter check and handle any pending signals. This will invoke the
    /// corresponding signal handlers registered in Python (if any).
    ///
    /// Returns `Err(`[`PyErr`](crate::PyErr)`)` if any signal handler raises an exception.
    ///
    /// These signals include `SIGINT` (normally raised by CTRL + C), which by default raises
    /// `KeyboardInterrupt`. For this reason it is good practice to call this function regularly
    /// as part of long-running Rust functions so that users can cancel it.
    ///
    /// # Example
    ///
    /// ```rust,no_run
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
}

impl<'unbound> Python<'unbound> {
    /// Deprecated version of [`Python::assume_attached`]
    ///
    /// # Safety
    /// See [`Python::assume_attached`]
    #[inline]
    #[deprecated(since = "0.26.0", note = "use `Python::assume_attached` instead")]
    pub unsafe fn assume_gil_acquired() -> Python<'unbound> {
        unsafe { Self::assume_attached() }
    }
    /// Unsafely creates a Python token with an unbounded lifetime.
    ///
    /// Many of PyO3 APIs use [`Python<'_>`] as proof that the calling thread is attached to the
    /// interpreter, but this function can be used to call them unsafely.
    ///
    /// # Safety
    ///
    /// - This token and any borrowed Python references derived from it can only be safely used
    ///   whilst the currently executing thread is actually attached to the interpreter.
    /// - This function creates a token with an *unbounded* lifetime. Safe code can assume that
    ///   holding a [`Python<'py>`] token means the thread is attached and stays attached for the
    ///   lifetime `'py`. If you let it or borrowed Python references escape to safe code you are
    ///   responsible for bounding the lifetime `'unbound` appropriately. For more on unbounded
    ///   lifetimes, see the [nomicon].
    ///
    /// [nomicon]: https://doc.rust-lang.org/nomicon/unbounded-lifetimes.html
    #[inline]
    pub unsafe fn assume_attached() -> Python<'unbound> {
        Python(PhantomData, PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        internal::state::ForbidAttaching,
        types::{IntoPyDict, PyList},
    };

    #[test]
    fn test_eval() {
        Python::attach(|py| {
            // Make sure builtin names are accessible
            let v: i32 = py
                .eval(ffi::c_str!("min(1, 2)"), None, None)
                .map_err(|e| e.display(py))
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, 1);

            let d = [("foo", 13)].into_py_dict(py).unwrap();

            // Inject our own global namespace
            let v: i32 = py
                .eval(ffi::c_str!("foo + 29"), Some(&d), None)
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, 42);

            // Inject our own local namespace
            let v: i32 = py
                .eval(ffi::c_str!("foo + 29"), None, Some(&d))
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, 42);

            // Make sure builtin names are still accessible when using a local namespace
            let v: i32 = py
                .eval(ffi::c_str!("min(foo, 2)"), None, Some(&d))
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(v, 2);
        });
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))] // We are building wasm Python with pthreads disabled
    fn test_detach_releases_and_acquires_gil() {
        Python::attach(|py| {
            let b = std::sync::Arc::new(std::sync::Barrier::new(2));

            let b2 = b.clone();
            std::thread::spawn(move || Python::attach(|_| b2.wait()));

            py.detach(|| {
                // If `detach` does not release the GIL, this will deadlock because
                // the thread spawned above will never be able to acquire the GIL.
                b.wait();
            });

            unsafe {
                // If the GIL is not reacquired at the end of `detach`, this call
                // will crash the Python interpreter.
                let tstate = ffi::PyEval_SaveThread();
                ffi::PyEval_RestoreThread(tstate);
            }
        });
    }

    #[test]
    fn test_detach_panics_safely() {
        Python::attach(|py| {
            let result = std::panic::catch_unwind(|| unsafe {
                let py = Python::assume_attached();
                py.detach(|| {
                    panic!("There was a panic!");
                });
            });

            // Check panic was caught
            assert!(result.is_err());

            // If `detach` is implemented correctly, this thread still owns the GIL here
            // so the following Python calls should not cause crashes.
            let list = PyList::new(py, [1, 2, 3, 4]).unwrap();
            assert_eq!(list.extract::<Vec<i32>>().unwrap(), vec![1, 2, 3, 4]);
        });
    }

    #[cfg(not(pyo3_disable_reference_pool))]
    #[test]
    fn test_detach_pass_stuff_in() {
        let list = Python::attach(|py| PyList::new(py, vec!["foo", "bar"]).unwrap().unbind());
        let mut v = vec![1, 2, 3];
        let a = std::sync::Arc::new(String::from("foo"));

        Python::attach(|py| {
            py.detach(|| {
                drop((list, &mut v, a));
            });
        });
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_acquire_gil() {
        use std::ffi::c_int;

        const GIL_NOT_HELD: c_int = 0;
        const GIL_HELD: c_int = 1;

        // Before starting the interpreter the state of calling `PyGILState_Check`
        // seems to be undefined, so let's ensure that Python is up.
        #[cfg(not(any(PyPy, GraalPy)))]
        Python::initialize();

        let state = unsafe { crate::ffi::PyGILState_Check() };
        assert_eq!(state, GIL_NOT_HELD);

        Python::attach(|_| {
            let state = unsafe { crate::ffi::PyGILState_Check() };
            assert_eq!(state, GIL_HELD);
        });

        let state = unsafe { crate::ffi::PyGILState_Check() };
        assert_eq!(state, GIL_NOT_HELD);
    }

    #[test]
    fn test_ellipsis() {
        Python::attach(|py| {
            assert_eq!(py.Ellipsis().to_string(), "Ellipsis");

            let v = py
                .eval(ffi::c_str!("..."), None, None)
                .map_err(|e| e.display(py))
                .unwrap();

            assert!(v.eq(py.Ellipsis()).unwrap());
        });
    }

    #[test]
    fn test_py_run_inserts_globals() {
        use crate::types::dict::PyDictMethods;

        Python::attach(|py| {
            let namespace = PyDict::new(py);
            py.run(
                ffi::c_str!("class Foo: pass\na = int(3)"),
                Some(&namespace),
                Some(&namespace),
            )
            .unwrap();
            assert!(matches!(namespace.get_item("Foo"), Ok(Some(..))));
            assert!(matches!(namespace.get_item("a"), Ok(Some(..))));
            // 3.9 and older did not automatically insert __builtins__ if it wasn't inserted "by hand"
            #[cfg(not(Py_3_10))]
            assert!(matches!(namespace.get_item("__builtins__"), Ok(Some(..))));
        })
    }

    #[cfg(feature = "macros")]
    #[test]
    fn test_py_run_inserts_globals_2() {
        use std::ffi::CString;

        #[crate::pyclass(crate = "crate")]
        #[derive(Clone)]
        struct CodeRunner {
            code: CString,
        }

        impl CodeRunner {
            fn reproducer(&mut self, py: Python<'_>) -> PyResult<()> {
                let variables = PyDict::new(py);
                variables.set_item("cls", crate::Py::new(py, self.clone())?)?;

                py.run(self.code.as_c_str(), Some(&variables), None)?;
                Ok(())
            }
        }

        #[crate::pymethods(crate = "crate")]
        impl CodeRunner {
            fn func(&mut self, py: Python<'_>) -> PyResult<()> {
                py.import("math")?;
                Ok(())
            }
        }

        let mut runner = CodeRunner {
            code: CString::new(
                r#"
cls.func()
"#
                .to_string(),
            )
            .unwrap(),
        };

        Python::attach(|py| {
            runner.reproducer(py).unwrap();
        });
    }

    #[test]
    fn python_is_zst() {
        assert_eq!(std::mem::size_of::<Python<'_>>(), 0);
    }

    #[test]
    fn test_try_attach_fail_during_gc() {
        Python::attach(|_| {
            assert!(Python::try_attach(|_| {}).is_some());

            let guard = ForbidAttaching::during_traverse();
            assert!(Python::try_attach(|_| {}).is_none());
            drop(guard);

            assert!(Python::try_attach(|_| {}).is_some());
        })
    }

    #[test]
    fn test_try_attach_ok_when_detached() {
        Python::attach(|py| {
            py.detach(|| {
                assert!(Python::try_attach(|_| {}).is_some());
            });
        });
    }
}
