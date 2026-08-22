#![deny(clippy::undocumented_unsafe_blocks)]

//! Types and APIs for Python [`contextvars.Context`][1] objects.
//!
//! On GIL-enabled Python 3.14 and newer, this module also provides safe bindings for watching
//! changes to the current context.
//!
//! [1]: https://docs.python.org/3/library/contextvars.html#contextvars.Context

use crate::{ffi, PyAny};

#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
use crate::{
    err::{error_on_minusone, error_on_minusone_with_result},
    Borrowed, PyResult, Python,
};
#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
use core::ffi::c_int;

/// Represents a Python [`contextvars.Context`][1] object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyContext>`][crate::Py] or [`Bound<'py, PyContext>`][crate::Bound].
///
/// [1]: https://docs.python.org/3/library/contextvars.html#contextvars.Context
#[repr(transparent)]
pub struct PyContext(PyAny);

pyobject_native_type_core!(
    PyContext,
    pyobject_native_static_type_object!(ffi::PyContext_Type),
    "contextvars",
    "Context",
    #module=Some("contextvars"),
    #checkfunction=ffi::PyContext_CheckExact
);

#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
impl PyContext {
    /// Registers a context watcher for the current interpreter.
    ///
    /// Use [`watch_callback!`] to create the callback passed to this method. The returned
    /// [`ContextWatcherGuard`] removes the watcher when dropped.
    ///
    /// Panics and returned [`PyErr`][crate::PyErr] values are reported as unraisable exceptions and
    /// never unwind across the C boundary.
    #[doc(alias = "PyContext_AddWatcher")]
    pub fn add_watcher(
        py: Python<'_>,
        callback: WatchCallback,
    ) -> PyResult<ContextWatcherGuard<'_>> {
        // SAFETY:
        // - `py` proves that the thread is attached
        // - `callback` contains a static C-compatible function
        let watcher_id =
            error_on_minusone_with_result(py, unsafe { ffi::PyContext_AddWatcher(callback.0) })?;

        Ok(ContextWatcherGuard {
            watcher_id,
            py,
            active: true,
        })
    }
}

/// An event passed to a context watcher.
///
/// This enum is non-exhaustive because CPython may add context watcher events in future versions.
#[doc(alias = "PyContextEvent")]
#[derive(Debug)]
#[non_exhaustive]
#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
pub enum ContextEvent<'a, 'py> {
    /// The current context changed.
    ///
    /// The value is the new current context, or `None` when there is no current context.
    Switched(Option<Borrowed<'a, 'py, PyContext>>),

    /// An event which is not known to this version of PyO3.
    Unknown {
        /// The raw CPython event value.
        raw_event: ffi::PyContextEvent,

        /// The event-specific object, if one was provided.
        object: Option<Borrowed<'a, 'py, PyAny>>,
    },
}

/// A guard which keeps a context watcher registered.
///
/// The watcher is registered for the current Python interpreter and is removed when this guard is
/// dropped. Use [`clear`][Self::clear] to remove it explicitly and observe any error returned by
/// CPython.
///
/// This guard is bound to the [`Python`] attachment used to create it. It therefore cannot be sent
/// to another thread, moved outside that attachment, or moved into [`Python::detach`].
///
/// If this guard is forgotten, the watcher remains registered. This does not create a dangling
/// function pointer because [`watch_callback!`] creates a static, monomorphized trampoline.
#[must_use = "dropping the guard immediately unregisters the context watcher"]
#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
pub struct ContextWatcherGuard<'py> {
    watcher_id: c_int,
    py: Python<'py>,
    active: bool,
}

#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
impl ContextWatcherGuard<'_> {
    /// Removes this watcher from the current Python interpreter.
    ///
    /// Dropping the guard also removes the watcher, but cannot report a failure to the caller.
    #[doc(alias = "PyContext_ClearWatcher")]
    pub fn clear(mut self) -> PyResult<()> {
        self.active = false;

        // SAFETY:
        // - `self.py` proves that the thread is attached to an interpreter; PyO3 does not
        //   currently support attaching to more than one interpreter, so this is the interpreter
        //   for which the watcher was registered
        // - `watcher_id` was returned by `PyContext_AddWatcher`
        error_on_minusone(self.py, unsafe {
            ffi::PyContext_ClearWatcher(self.watcher_id)
        })
    }
}

#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
impl Drop for ContextWatcherGuard<'_> {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        self.active = false;

        // A destructor must not replace an exception which was already pending. The Python token
        // stored in the guard proves that this thread is still attached to an interpreter; PyO3
        // does not currently support attaching to more than one interpreter, so this is the same
        // interpreter the watcher was registered on.
        //
        // SAFETY:
        // - the thread is attached, as guaranteed by `self.py`
        // - `PyErr_GetRaisedException` returns an owned reference or NULL
        // - `watcher_id` was returned by `PyContext_AddWatcher`
        // - `PyErr_SetRaisedException` steals the owned reference returned above
        unsafe {
            let pending_exception = ffi::PyErr_GetRaisedException();
            if ffi::PyContext_ClearWatcher(self.watcher_id) == -1 {
                ffi::PyErr_WriteUnraisable(core::ptr::null_mut());
            }

            if !pending_exception.is_null() {
                // Be defensive in case an unraisable hook itself left an exception set.
                ffi::PyErr_Clear();
                ffi::PyErr_SetRaisedException(pending_exception);
            }
        }
    }
}

/// Callback type for context watchers.
///
/// Values of this type are created by [`watch_callback!`].
#[repr(transparent)]
#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
pub struct WatchCallback(ffi::PyContext_WatchCallback);

/// Creates a context watcher callback from a safe Rust function.
///
/// A function path is required because CPython's context watcher callback has no user-data
/// pointer. The macro creates a unique static trampoline for the function, avoiding global callback
/// storage. State can still be shared through safe static synchronization primitives.
#[macro_export]
#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
macro_rules! watch_callback {
    ($callback:path) => {{
        struct Callback;

        impl $crate::types::context::impl_::ContextWatcherCallbackDef for Callback {
            const CALLBACK: $crate::types::context::impl_::ContextWatcherCallback = $callback;
        }

        $crate::types::context::impl_::new_watch_callback::<Callback>()
    }};
}

#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
pub use crate::watch_callback;

/// Implementation details used by [`watch_callback!`].
#[doc(hidden)]
#[cfg(all(Py_3_14, not(Py_GIL_DISABLED)))]
pub mod impl_ {

    use crate::{ffi_ptr_ext::FfiPtrExt, types::PyAnyMethods};

    use super::*;

    /// The safe callback signature accepted by context watcher trampolines.
    pub type ContextWatcherCallback =
        for<'a, 'py> fn(Python<'py>, ContextEvent<'a, 'py>) -> PyResult<()>;

    /// Associates a generated trampoline type with its Rust callback.
    pub trait ContextWatcherCallbackDef {
        /// The Rust callback invoked by the generated C trampoline.
        const CALLBACK: ContextWatcherCallback;
    }

    pub fn new_watch_callback<Callback: ContextWatcherCallbackDef>() -> WatchCallback {
        WatchCallback(context_watcher::<Callback>)
    }

    /// C-compatible trampoline for a context watcher callback.
    ///
    /// # Safety
    ///
    /// - The thread must be attached to Python.
    /// - `object` must follow the contract for the supplied `event`.
    pub unsafe extern "C" fn context_watcher<Callback: ContextWatcherCallbackDef>(
        event: ffi::PyContextEvent,
        object: *mut ffi::PyObject,
    ) -> c_int {
        // A context watcher may be called with an exception already set. Save it before invoking
        // arbitrary Rust code so that safe PyO3 APIs can be used normally inside the callback.
        //
        // SAFETY: the caller guarantees that the thread is attached.
        let pending_exception = unsafe { ffi::PyErr_GetRaisedException() };

        // SAFETY:
        // - the caller guarantees that the thread is attached and `object` follows the contract
        //   for `event`
        // - `trampoline` catches panics and converts callback errors into a Python exception
        // - the callback's higher-ranked signature prevents borrowed event data from escaping
        let result = unsafe {
            crate::impl_::trampoline::trampoline(|py| {
                let event = match event {
                    ffi::Py_CONTEXT_SWITCHED => {
                        let object = object.assume_borrowed(py);

                        if object.is_none() {
                            ContextEvent::Switched(None)
                        } else {
                            ContextEvent::Switched(Some(object.cast_unchecked()))
                        }
                    }

                    raw_event => {
                        let object = object.assume_borrowed_or_opt(py);
                        ContextEvent::Unknown { raw_event, object }
                    }
                };

                (Callback::CALLBACK)(py, event)?;

                // Although normal PyO3 APIs return errors as `PyResult`, `PyErr::restore` can be
                // called directly. Do not allow an Ok return with an exception still set.
                if crate::PyErr::occurred(py) {
                    return Err(crate::PyErr::fetch(py));
                }

                Ok(0)
            })
        };

        if pending_exception.is_null() {
            return result;
        }

        // When an exception was already pending on entry, CPython requires the callback to return
        // 0 with that same exception still set. Report a new callback error ourselves before
        // restoring the original exception.
        //
        // SAFETY:
        // - the thread is attached
        // - `object` is valid for the duration of the callback or NULL
        // - `pending_exception` is an owned reference from `PyErr_GetRaisedException`
        // - `PyErr_SetRaisedException` steals that reference
        unsafe {
            if result == -1 {
                ffi::PyErr_WriteUnraisable(object);
            }

            // Be defensive in case an unraisable hook itself left an exception set.
            ffi::PyErr_Clear();
            ffi::PyErr_SetRaisedException(pending_exception);
        }

        0
    }
}

#[cfg(all(test, Py_3_14, not(Py_GIL_DISABLED)))]
mod tests {
    use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

    #[cfg(feature = "macros")]
    use alloc::string::ToString;
    use static_assertions::assert_not_impl_any;

    use crate::exceptions::{PyRuntimeError, PyValueError};
    use crate::types::context::impl_::{
        context_watcher, ContextWatcherCallback, ContextWatcherCallbackDef,
    };
    use crate::types::context::ContextEvent;
    use crate::types::PyAnyMethods;
    use crate::{PyErr, Python};

    #[cfg(feature = "macros")]
    use crate::test_utils::UnraisableCapture;

    use super::*;

    #[test]
    fn context_type() {
        Python::attach(|py| {
            let context = py
                .import(c"contextvars")
                .unwrap()
                .getattr(c"Context")
                .unwrap()
                .call0()
                .unwrap();

            assert!(context.is_exact_instance_of::<PyContext>());
            context.cast::<PyContext>().unwrap();
        });
    }

    static SWITCH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SAW_CONTEXT: AtomicBool = AtomicBool::new(false);

    #[allow(clippy::unnecessary_wraps, reason = "context watcher callback")]
    fn record_switch(_py: Python<'_>, event: ContextEvent<'_, '_>) -> PyResult<()> {
        if let ContextEvent::Switched(context) = event {
            SWITCH_COUNT.fetch_add(1, Ordering::Relaxed);
            if let Some(context) = context {
                assert!(context.is_exact_instance_of::<PyContext>());
                SAW_CONTEXT.store(true, Ordering::Relaxed);
            }
        }
        Ok(())
    }

    #[test]
    fn watcher_is_cleared_on_drop() {
        Python::attach(|py| {
            SWITCH_COUNT.store(0, Ordering::Relaxed);
            SAW_CONTEXT.store(false, Ordering::Relaxed);

            let watcher = PyContext::add_watcher(py, watch_callback!(record_switch)).unwrap();
            py.run(
                c"import contextvars\ncontextvars.Context().run(lambda: None)",
                None,
                None,
            )
            .unwrap();

            let count_after_first_run = SWITCH_COUNT.load(Ordering::Relaxed);
            assert!(count_after_first_run >= 2);
            assert!(SAW_CONTEXT.load(Ordering::Relaxed));

            drop(watcher);

            py.run(c"contextvars.Context().run(lambda: None)", None, None)
                .unwrap();
            assert_eq!(SWITCH_COUNT.load(Ordering::Relaxed), count_after_first_run);
        });
    }

    static EXPLICIT_CLEAR_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[allow(clippy::unnecessary_wraps, reason = "context watcher callback")]
    fn record_explicit_clear(_py: Python<'_>, event: ContextEvent<'_, '_>) -> PyResult<()> {
        if matches!(event, ContextEvent::Switched(_)) {
            EXPLICIT_CLEAR_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    #[test]
    fn watcher_can_be_cleared_explicitly() {
        Python::attach(|py| {
            EXPLICIT_CLEAR_COUNT.store(0, Ordering::Relaxed);

            let watcher =
                PyContext::add_watcher(py, watch_callback!(record_explicit_clear)).unwrap();
            watcher.clear().unwrap();

            py.run(
                c"import contextvars\ncontextvars.Context().run(lambda: None)",
                None,
                None,
            )
            .unwrap();
            assert_eq!(EXPLICIT_CLEAR_COUNT.load(Ordering::Relaxed), 0);
        });
    }

    static DUPLICATE_CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[allow(clippy::unnecessary_wraps, reason = "context watcher callback")]
    fn record_duplicate_callback(_py: Python<'_>, event: ContextEvent<'_, '_>) -> PyResult<()> {
        if matches!(event, ContextEvent::Switched(_)) {
            DUPLICATE_CALLBACK_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    #[test]
    fn multiple_watchers_can_register_the_same_callback() {
        Python::attach(|py| {
            DUPLICATE_CALLBACK_COUNT.store(0, Ordering::Relaxed);

            let first =
                PyContext::add_watcher(py, watch_callback!(record_duplicate_callback)).unwrap();
            let second =
                PyContext::add_watcher(py, watch_callback!(record_duplicate_callback)).unwrap();

            py.run(
                c"import contextvars\ncontextvars.Context().run(lambda: None)",
                None,
                None,
            )
            .unwrap();
            assert!(DUPLICATE_CALLBACK_COUNT.load(Ordering::Relaxed) >= 4);

            drop(first);
            let count_with_both = DUPLICATE_CALLBACK_COUNT.load(Ordering::Relaxed);
            py.run(c"contextvars.Context().run(lambda: None)", None, None)
                .unwrap();
            assert!(DUPLICATE_CALLBACK_COUNT.load(Ordering::Relaxed) >= count_with_both + 2);

            drop(second);
            let count_after_drop = DUPLICATE_CALLBACK_COUNT.load(Ordering::Relaxed);
            py.run(c"contextvars.Context().run(lambda: None)", None, None)
                .unwrap();
            assert_eq!(
                DUPLICATE_CALLBACK_COUNT.load(Ordering::Relaxed),
                count_after_drop
            );
        });
    }

    #[test]
    fn dropping_watcher_preserves_a_pending_exception() {
        Python::attach(|py| {
            let watcher =
                PyContext::add_watcher(py, watch_callback!(record_explicit_clear)).unwrap();
            PyValueError::new_err("original error").restore(py);

            drop(watcher);

            let error = PyErr::fetch(py);
            assert!(error.is_instance_of::<PyValueError>(py));
        });
    }

    fn fail_callback(_py: Python<'_>, _event: ContextEvent<'_, '_>) -> PyResult<()> {
        Err(PyRuntimeError::new_err("watcher failed"))
    }

    struct FailingCallback;

    impl ContextWatcherCallbackDef for FailingCallback {
        const CALLBACK: ContextWatcherCallback = fail_callback;
    }

    #[test]
    fn callback_error_is_returned_without_a_pending_exception() {
        Python::attach(|py| {
            // SAFETY: the thread is attached and None is valid for Py_CONTEXT_SWITCHED.
            let result = unsafe {
                context_watcher::<FailingCallback>(ffi::Py_CONTEXT_SWITCHED, ffi::Py_None())
            };

            assert_eq!(result, -1);
            let error = PyErr::fetch(py);
            assert!(error.is_instance_of::<PyRuntimeError>(py));
        });
    }

    #[test]
    #[cfg(feature = "macros")]
    fn callback_error_preserves_a_pending_exception() {
        Python::attach(|py| {
            UnraisableCapture::enter(py, |capture| {
                PyValueError::new_err("original error").restore(py);

                // SAFETY: the thread is attached and None is valid for Py_CONTEXT_SWITCHED.
                let result = unsafe {
                    context_watcher::<FailingCallback>(ffi::Py_CONTEXT_SWITCHED, ffi::Py_None())
                };

                assert_eq!(result, 0);

                let original_error = PyErr::fetch(py);
                assert!(original_error.is_instance_of::<PyValueError>(py));
                assert_eq!(original_error.to_string(), "ValueError: original error");

                let (watcher_error, object) =
                    capture.take_capture().expect("missing unraisable error");
                assert!(watcher_error.is_instance_of::<PyRuntimeError>(py));
                assert!(object.is_none());
            });
        });
    }

    #[test]
    #[cfg(feature = "macros")]
    fn registered_callback_errors_are_unraisable() {
        Python::attach(|py| {
            UnraisableCapture::enter(py, |capture| {
                let watcher = PyContext::add_watcher(py, watch_callback!(fail_callback)).unwrap();

                py.run(
                    c"import contextvars\ncontextvars.Context().run(lambda: None)",
                    None,
                    None,
                )
                .unwrap();

                let (watcher_error, _) = capture.take_capture().expect("missing unraisable error");
                assert!(watcher_error.is_instance_of::<PyRuntimeError>(py));

                drop(watcher);
            });
        });
    }

    fn panic_callback(_py: Python<'_>, _event: ContextEvent<'_, '_>) -> PyResult<()> {
        panic!("context watcher panic")
    }

    struct PanickingCallback;

    impl ContextWatcherCallbackDef for PanickingCallback {
        const CALLBACK: ContextWatcherCallback = panic_callback;
    }

    #[test]
    fn callback_panic_does_not_cross_ffi_boundary() {
        Python::attach(|py| {
            // SAFETY: the thread is attached and None is valid for Py_CONTEXT_SWITCHED.
            let result = unsafe {
                context_watcher::<PanickingCallback>(ffi::Py_CONTEXT_SWITCHED, ffi::Py_None())
            };

            assert_eq!(result, -1);
            assert!(PyErr::occurred(py));

            // SAFETY: the test has observed and intentionally discards the panic exception.
            unsafe { ffi::PyErr_Clear() };
        });
    }

    static UNKNOWN_EVENT: AtomicU32 = AtomicU32::new(0);

    #[allow(clippy::unnecessary_wraps, reason = "context watcher callback")]
    fn record_unknown(_py: Python<'_>, event: ContextEvent<'_, '_>) -> PyResult<()> {
        if let ContextEvent::Unknown { raw_event, object } = event {
            UNKNOWN_EVENT.store(raw_event, Ordering::Relaxed);
            assert!(object.is_none());
        }
        Ok(())
    }

    struct UnknownCallback;

    impl ContextWatcherCallbackDef for UnknownCallback {
        const CALLBACK: ContextWatcherCallback = record_unknown;
    }

    #[test]
    fn unknown_events_are_forwarded() {
        const FUTURE_EVENT: ffi::PyContextEvent = 123;

        Python::attach(|_py| {
            UNKNOWN_EVENT.store(0, Ordering::Relaxed);

            // SAFETY: the thread is attached and unknown events accept a null object.
            let result =
                unsafe { context_watcher::<UnknownCallback>(FUTURE_EVENT, core::ptr::null_mut()) };

            assert_eq!(result, 0);
            assert_eq!(UNKNOWN_EVENT.load(Ordering::Relaxed), FUTURE_EVENT);
        });
    }

    #[test]
    fn watcher_guard_is_not_send_or_sync() {
        assert_not_impl_any!(super::ContextWatcherGuard<'_>: Send, Sync);
    }

    static FORGOTTEN_GUARD_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[allow(clippy::unnecessary_wraps, reason = "context watcher callback")]
    fn record_forgotten_guard(_py: Python<'_>, event: ContextEvent<'_, '_>) -> PyResult<()> {
        if matches!(event, ContextEvent::Switched(_)) {
            FORGOTTEN_GUARD_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    #[test]
    fn forgotten_guard() {
        Python::attach(|py| {
            FORGOTTEN_GUARD_COUNT.store(0, Ordering::Relaxed);
            let watcher =
                PyContext::add_watcher(py, watch_callback!(record_forgotten_guard)).unwrap();
            core::mem::forget(watcher);

            py.run(
                c"import contextvars; contextvars.Context().run(lambda: None)",
                None,
                None,
            )
            .unwrap();

            assert!(FORGOTTEN_GUARD_COUNT.load(Ordering::Relaxed) > 0);
        });
    }
}
