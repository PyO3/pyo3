#![deny(clippy::undocumented_unsafe_blocks)]

//! Safe bindings for watching changes to Python's current [`contextvars.Context`][1].
//!
//! Context watchers are registered for the current Python interpreter and are invoked whenever
//! the current context changes.
//!
//! [1]: https://docs.python.org/3/library/contextvars.html#contextvars.Context

use crate::err::{error_on_minusone, error_on_minusone_with_result};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::types::any::PyAnyMethods;
use crate::types::PyContext;
use crate::{ffi, Borrowed, PyAny, PyResult, Python};
use core::ffi::c_int;

/// An event passed to a context watcher.
///
/// This enum is non-exhaustive because CPython may add context watcher events in future versions.
#[doc(alias = "PyContextEvent")]
#[non_exhaustive]
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
/// function pointer because [`register_context_watcher!`](crate::register_context_watcher) creates
/// a static, monomorphized trampoline.
#[must_use = "dropping the guard immediately unregisters the context watcher"]
pub struct ContextWatcherGuard<'py> {
    watcher_id: c_int,
    py: Python<'py>,
    active: bool,
}

impl ContextWatcherGuard<'_> {
    /// Removes this watcher from the current Python interpreter.
    ///
    /// Dropping the guard also removes the watcher, but cannot report a failure to the caller.
    #[doc(alias = "PyContext_ClearWatcher")]
    pub fn clear(mut self) -> PyResult<()> {
        self.active = false;

        // SAFETY:
        // - `self.py` proves that the thread is attached to the interpreter for which the watcher
        //   was registered
        // - `watcher_id` was returned by `PyContext_AddWatcher`
        error_on_minusone(self.py, unsafe {
            ffi::PyContext_ClearWatcher(self.watcher_id)
        })
    }
}

impl Drop for ContextWatcherGuard<'_> {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        self.active = false;

        // A destructor must not replace an exception which was already pending. The Python token
        // stored in the guard proves that this thread is still attached to the correct interpreter.
        //
        // SAFETY:
        // - the thread is attached, as guaranteed by `self.py`
        // - `PyErr_GetRaisedException` returns an owned reference or NULL
        // - `watcher_id` was returned by `PyContext_AddWatcher`
        // - `PyErr_SetRaisedException` steals the owned reference returned above
        unsafe {
            let pending_exception = ffi::PyErr_GetRaisedException();
            let result = ffi::PyContext_ClearWatcher(self.watcher_id);

            if result == -1 {
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

/// Registers a safe Rust function as a context watcher for the current interpreter.
///
/// The callback must be a function path and must have this signature:
///
/// ```rust
/// # #![cfg(all(Py_3_14, not(Py_LIMITED_API), not(any(PyPy, GraalPy, RustPython))))]
/// use pyo3::context::ContextEvent;
/// use pyo3::prelude::*;
///
/// fn context_changed(
///     py: Python<'_>,
///     event: ContextEvent<'_, '_>,
/// ) -> PyResult<()> {
///     let _ = py;
///     let _ = event;
///     Ok(())
/// }
///
/// # fn main() -> PyResult<()> {
/// Python::attach(|py| {
///     let _watcher = pyo3::register_context_watcher!(py, context_changed)?;
///     Ok(())
/// })
/// # }
/// ```
///
/// A function path is required because CPython's context watcher callback has no user-data
/// pointer. The macro creates a unique static trampoline for the function, avoiding global callback
/// storage. State can still be shared through safe static synchronization primitives.
///
/// The callback may run concurrently on free-threaded Python builds. Panics and returned
/// [`PyErr`][crate::PyErr] values are reported as unraisable exceptions and never unwind across the
/// C boundary.
#[doc(alias = "PyContext_AddWatcher")]
#[macro_export]
macro_rules! register_context_watcher {
    ($py:expr, $callback:path) => {{
        struct Callback;

        impl $crate::context::impl_::ContextWatcherCallbackDef for Callback {
            const CALLBACK: $crate::context::impl_::ContextWatcherCallback = $callback;
        }

        $crate::context::impl_::register::<Callback>($py)
    }};
}

/// Implementation details used by [`register_context_watcher!`](crate::register_context_watcher).
#[doc(hidden)]
pub mod impl_ {
    use super::*;

    /// The safe callback signature accepted by context watcher trampolines.
    pub type ContextWatcherCallback =
        for<'a, 'py> fn(Python<'py>, ContextEvent<'a, 'py>) -> PyResult<()>;

    /// Associates a generated trampoline type with its Rust callback.
    pub trait ContextWatcherCallbackDef {
        /// The Rust callback invoked by the generated C trampoline.
        const CALLBACK: ContextWatcherCallback;
    }

    /// Registers the trampoline specialized for `Callback`.
    pub fn register<Callback: ContextWatcherCallbackDef>(
        py: Python<'_>,
    ) -> PyResult<ContextWatcherGuard<'_>> {
        // SAFETY:
        // - `py` proves that the thread is attached
        // - `context_watcher::<Callback>` is a static C-compatible function
        let watcher_id = unsafe { ffi::PyContext_AddWatcher(context_watcher::<Callback>) };
        let watcher_id = error_on_minusone_with_result(py, watcher_id)?;

        Ok(ContextWatcherGuard {
            watcher_id,
            py,
            active: true,
        })
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

        // SAFETY: the caller guarantees that the thread is attached. `trampoline` catches all
        // panics and converts callback errors into a Python exception with a -1 return value.
        let result = unsafe {
            crate::impl_::trampoline::trampoline(|py| {
                let borrow_guard = ();
                // SAFETY:
                // - CPython guarantees that `object` follows the contract for `event`
                // - `borrow_guard` limits the resulting borrow to this callback invocation
                let event = event_from_raw(py, event, object, &borrow_guard);

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

    unsafe fn event_from_raw<'a, 'py>(
        py: Python<'py>,
        event: ffi::PyContextEvent,
        object: *mut ffi::PyObject,
        _borrow_guard: &'a (),
    ) -> ContextEvent<'a, 'py> {
        match event {
            ffi::Py_CONTEXT_SWITCHED => {
                // SAFETY: CPython documents a non-null context object or `None` for this event.
                let object = unsafe { object.assume_borrowed(py) };

                if object.is_none() {
                    ContextEvent::Switched(None)
                } else {
                    // SAFETY: CPython guarantees that a non-None object for this event is a
                    // `contextvars.Context`.
                    ContextEvent::Switched(Some(unsafe { object.cast_unchecked() }))
                }
            }
            raw_event => {
                // SAFETY: unknown events may have a NULL object; a non-null object is borrowed for
                // at least the callback duration, which is bounded by `_borrow_guard`.
                let object = unsafe { object.assume_borrowed_or_opt(py) };
                ContextEvent::Unknown { raw_event, object }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::impl_::{context_watcher, ContextWatcherCallback, ContextWatcherCallbackDef};
    use super::ContextEvent;
    use crate::exceptions::{PyRuntimeError, PyValueError};
    use crate::test_utils::UnraisableCapture;
    use crate::types::{PyAnyMethods, PyContext};
    use crate::{ffi, PyErr, PyResult, Python};
    use alloc::string::ToString;
    use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
    use static_assertions::assert_not_impl_any;

    static SWITCH_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SAW_CONTEXT: AtomicBool = AtomicBool::new(false);
    static SAW_NONE: AtomicBool = AtomicBool::new(false);

    #[allow(clippy::unnecessary_wraps, reason = "context watcher callback")]
    fn record_switch(_py: Python<'_>, event: ContextEvent<'_, '_>) -> PyResult<()> {
        if let ContextEvent::Switched(context) = event {
            SWITCH_COUNT.fetch_add(1, Ordering::Relaxed);
            match context {
                Some(context) => {
                    assert!(context.is_exact_instance_of::<PyContext>());
                    SAW_CONTEXT.store(true, Ordering::Relaxed);
                }
                None => SAW_NONE.store(true, Ordering::Relaxed),
            }
        }
        Ok(())
    }

    #[test]
    fn watcher_is_cleared_on_drop() {
        Python::attach(|py| {
            SWITCH_COUNT.store(0, Ordering::Relaxed);
            SAW_CONTEXT.store(false, Ordering::Relaxed);
            SAW_NONE.store(false, Ordering::Relaxed);

            let watcher = crate::register_context_watcher!(py, record_switch).unwrap();
            py.run(
                c"import contextvars\ncontextvars.Context().run(lambda: None)",
                None,
                None,
            )
            .unwrap();

            let count_after_first_run = SWITCH_COUNT.load(Ordering::Relaxed);
            assert!(count_after_first_run >= 2);
            assert!(SAW_CONTEXT.load(Ordering::Relaxed));
            assert!(SAW_NONE.load(Ordering::Relaxed));

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

            let watcher = crate::register_context_watcher!(py, record_explicit_clear).unwrap();
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
    fn registered_callback_errors_are_unraisable() {
        Python::attach(|py| {
            UnraisableCapture::enter(py, |capture| {
                let watcher = crate::register_context_watcher!(py, fail_callback).unwrap();

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
}
