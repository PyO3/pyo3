#[cfg(not(any(PyPy, GraalPy)))]
use crate::{ffi, internal::state::AttachGuard, Python};

static START: std::sync::Once = std::sync::Once::new();

#[cfg(not(any(PyPy, GraalPy)))]
pub(crate) fn initialize() {
    // Protect against race conditions when Python is not yet initialized and multiple threads
    // concurrently call 'initialize()'. Note that we do not protect against
    // concurrent initialization of the Python runtime by other users of the Python C API.
    START.call_once_force(|_| unsafe {
        // Use call_once_force because if initialization panics, it's okay to try again.
        if ffi::Py_IsInitialized() == 0 {
            ffi::Py_InitializeEx(0);

            // Release the GIL.
            ffi::PyEval_SaveThread();
        }
    });
}

/// See [Python::initialize]
#[cfg(not(any(PyPy, GraalPy)))]
#[inline]
#[deprecated(note = "use `Python::initialize` instead", since = "0.26.0")]
pub fn prepare_freethreaded_python() {
    initialize();
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
///         if let Err(e) = py.run(pyo3::ffi::c_str!("print('Hello World')"), None, None) {
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
        unsafe { ffi::Py_IsInitialized() },
        0,
        "called `with_embedded_python_interpreter` but a Python interpreter is already running."
    );

    unsafe { ffi::Py_InitializeEx(0) };

    let result = {
        let guard = unsafe { AttachGuard::assume() };
        let py = guard.python();
        // Import the threading module - this ensures that it will associate this thread as the "main"
        // thread, which is important to avoid an `AssertionError` at finalization.
        py.import("threading").unwrap();

        // Execute the closure.
        f(py)
    };

    // Finalize the Python interpreter.
    unsafe { ffi::Py_Finalize() };

    result
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

        START.call_once_force(|_| unsafe {
            // Use call_once_force because if there is a panic because the interpreter is
            // not initialized, it's fine for the user to initialize the interpreter and
            // retry.
            assert_ne!(
                crate::ffi::Py_IsInitialized(),
                0,
                "The Python interpreter is not initialized and the `auto-initialize` \
                        feature is not enabled.\n\n\
                        Consider calling `Python::initialize()` before attempting \
                        to use Python APIs."
            );
        });
    }
}
