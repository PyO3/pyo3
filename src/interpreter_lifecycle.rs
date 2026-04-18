#[cfg(not(any(PyPy, GraalPy)))]
use crate::{internal::state::AttachGuard, Python};

#[cfg(not(any(PyPy, GraalPy)))]
pub(crate) fn initialize() {
    crate::backend::current::runtime::initialize();
}

/// Executes the provided closure with an embedded Python interpreter.
///
/// This function initializes the Python interpreter, executes the provided closure, and then
/// finalizes the Python interpreter.
///
/// After execution all Python resources are cleaned up, and no further Python APIs can be called.
/// Because many Python modules implemented in C do not support multiple Python interpreters in a
/// single process, it is not safe to call this function more than once. (Many such modules will not
/// initialize correctly on the second run.)
///
/// # Panics
/// - If the Python interpreter is already initialized before calling this function.
///
/// # Safety
/// - This function should only ever be called once per process (usually as part of the `main`
///   function). It is also not thread-safe.
/// - No Python APIs can be used after this function has finished executing.
/// - The return value of the closure must not contain any Python value, _including_ `PyResult`.
///
/// # Examples
///
/// ```rust
/// unsafe {
///     pyo3::with_embedded_python_interpreter(|py| {
///         if let Err(e) = py.run(c"print('Hello World')", None, None) {
///             // We must make sure to not return a `PyErr`!
///             e.print(py);
///         }
///     });
/// }
/// ```
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn with_embedded_python_interpreter<F, R>(f: F) -> R
where
    F: for<'p> FnOnce(Python<'p>) -> R,
{
    assert_eq!(
        crate::backend::current::runtime::is_initialized(),
        false,
        "called `with_embedded_python_interpreter` but a Python interpreter is already running."
    );

    crate::backend::current::runtime::initialize_embedded();

    let result = {
        let guard = unsafe { AttachGuard::attach_unchecked() };
        let py = guard.python();
        // Import the threading module - this ensures that it will associate this thread as the "main"
        // thread, which is important to avoid an `AssertionError` at finalization.
        crate::backend::current::runtime::prepare_embedded_python_main_thread(py);

        // Execute the closure.
        f(py)
    };

    // Finalize the Python interpreter.
    crate::backend::current::runtime::finalize_embedded();

    result
}

/// If PyO3 is currently running `Py_InitializeEx` inside the `Once` guard,
/// block until it completes. Needed because `Py_InitializeEx` sets the
/// `initialized` flag in the interpreter to true before it finishes all its
/// steps (in particular, before it imports `site.py`).
///
/// This must only be called after `Py_IsInitialized()` has returned true.
///
/// If the `Once` was never started (e.g. the interpreter was initialized
/// externally, not through PyO3), `call_once` runs the empty closure and
/// returns — this is fine because `initialize()` checks
/// `Py_IsInitialized()` inside its closure and skips `Py_InitializeEx` if
/// the interpreter is already running. If the `Once` is currently in
/// progress (another thread is inside `initialize()`), `call_once` blocks
/// until it completes.
pub(crate) fn wait_for_initialization() {
    crate::backend::current::runtime::wait_for_initialization();
}

pub(crate) fn ensure_initialized() {
    // Maybe auto-initialize the interpreter:
    //  - If auto-initialize feature set and supported, try to initialize the interpreter.
    //  - If the auto-initialize feature is set but unsupported, emit hard errors only when the
    //    extension-module feature is not activated - extension modules don't care about
    //    auto-initialize so this avoids breaking existing builds.
    //  - Otherwise, just check the interpreter is initialized.
    #[cfg(all(feature = "auto-initialize", not(any(PyPy, GraalPy))))]
    {
        initialize();
    }
    #[cfg(not(all(feature = "auto-initialize", not(any(PyPy, GraalPy)))))]
    {
        // This is a "hack" to make running `cargo test` for PyO3 convenient (i.e. no need
        // to specify `--features auto-initialize` manually). Tests within the crate itself
        // all depend on the auto-initialize feature for conciseness but Cargo does not
        // provide a mechanism to specify required features for tests.
        #[cfg(not(any(PyPy, GraalPy)))]
        if option_env!("CARGO_PRIMARY_PACKAGE").is_some() {
            initialize();
        }

        crate::backend::current::runtime::ensure_initialized_or_panic();
    }
}
