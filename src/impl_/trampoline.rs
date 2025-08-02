//! Trampolines for various pyfunction and pymethod implementations.
//!
//! They exist to monomorphise std::panic::catch_unwind once into PyO3, rather than inline in every
//! function, thus saving a huge amount of compile-time complexity.

use std::{
    any::Any,
    os::raw::c_int,
    panic::{self, UnwindSafe},
};

use crate::internal::state::AttachGuard;
use crate::{
    ffi, ffi_ptr_ext::FfiPtrExt, impl_::callback::PyCallbackOutput, impl_::panic::PanicTrap,
    impl_::pymethods::IPowModulo, panic::PanicException, types::PyModule, Py, PyResult, Python,
};

#[inline]
pub unsafe fn module_init(
    f: for<'py> unsafe fn(Python<'py>) -> PyResult<Py<PyModule>>,
) -> *mut ffi::PyObject {
    unsafe { trampoline(|py| f(py).map(|module| module.into_ptr())) }
}

#[inline]
#[allow(clippy::used_underscore_binding)]
pub unsafe fn noargs(
    slf: *mut ffi::PyObject,
    _args: *mut ffi::PyObject,
    f: for<'py> unsafe fn(Python<'py>, *mut ffi::PyObject) -> PyResult<*mut ffi::PyObject>,
) -> *mut ffi::PyObject {
    #[cfg(not(GraalPy))] // this is not specified and GraalPy does not pass null here
    debug_assert!(_args.is_null());
    unsafe { trampoline(|py| f(py, slf)) }
}

macro_rules! trampoline {
    (pub fn $name:ident($($arg_names:ident: $arg_types:ty),* $(,)?) -> $ret:ty;) => {
        #[inline]
        pub unsafe fn $name(
            $($arg_names: $arg_types,)*
            f: for<'py> unsafe fn (Python<'py>, $($arg_types),*) -> PyResult<$ret>,
        ) -> $ret {
            unsafe {trampoline(|py| f(py, $($arg_names,)*))}
        }
    }
}

macro_rules! trampolines {
    ($(pub fn $name:ident($($arg_names:ident: $arg_types:ty),* $(,)?) -> $ret:ty);* ;) => {
        $(trampoline!(pub fn $name($($arg_names: $arg_types),*) -> $ret;));*;
    }
}

trampolines!(
    pub fn fastcall_with_keywords(
        slf: *mut ffi::PyObject,
        args: *const *mut ffi::PyObject,
        nargs: ffi::Py_ssize_t,
        kwnames: *mut ffi::PyObject,
    ) -> *mut ffi::PyObject;

    pub fn cfunction_with_keywords(
        slf: *mut ffi::PyObject,
        args: *mut ffi::PyObject,
        kwargs: *mut ffi::PyObject,
    ) -> *mut ffi::PyObject;
);

// Trampolines used by slot methods
trampolines!(
    pub fn getattrofunc(slf: *mut ffi::PyObject, attr: *mut ffi::PyObject) -> *mut ffi::PyObject;

    pub fn setattrofunc(
        slf: *mut ffi::PyObject,
        attr: *mut ffi::PyObject,
        value: *mut ffi::PyObject,
    ) -> c_int;

    pub fn binaryfunc(slf: *mut ffi::PyObject, arg1: *mut ffi::PyObject) -> *mut ffi::PyObject;

    pub fn descrgetfunc(
        slf: *mut ffi::PyObject,
        arg1: *mut ffi::PyObject,
        arg2: *mut ffi::PyObject,
    ) -> *mut ffi::PyObject;

    pub fn getiterfunc(slf: *mut ffi::PyObject) -> *mut ffi::PyObject;

    pub fn hashfunc(slf: *mut ffi::PyObject) -> ffi::Py_hash_t;

    pub fn inquiry(slf: *mut ffi::PyObject) -> c_int;

    pub fn iternextfunc(slf: *mut ffi::PyObject) -> *mut ffi::PyObject;

    pub fn lenfunc(slf: *mut ffi::PyObject) -> ffi::Py_ssize_t;

    pub fn newfunc(
        subtype: *mut ffi::PyTypeObject,
        args: *mut ffi::PyObject,
        kwargs: *mut ffi::PyObject,
    ) -> *mut ffi::PyObject;

    pub fn objobjproc(slf: *mut ffi::PyObject, arg1: *mut ffi::PyObject) -> c_int;

    pub fn reprfunc(slf: *mut ffi::PyObject) -> *mut ffi::PyObject;

    pub fn richcmpfunc(
        slf: *mut ffi::PyObject,
        other: *mut ffi::PyObject,
        op: c_int,
    ) -> *mut ffi::PyObject;

    pub fn ssizeargfunc(arg1: *mut ffi::PyObject, arg2: ffi::Py_ssize_t) -> *mut ffi::PyObject;

    pub fn ternaryfunc(
        slf: *mut ffi::PyObject,
        arg1: *mut ffi::PyObject,
        arg2: *mut ffi::PyObject,
    ) -> *mut ffi::PyObject;

    pub fn unaryfunc(slf: *mut ffi::PyObject) -> *mut ffi::PyObject;
);

#[cfg(any(not(Py_LIMITED_API), Py_3_11))]
trampoline! {
    pub fn getbufferproc(slf: *mut ffi::PyObject, buf: *mut ffi::Py_buffer, flags: c_int) -> c_int;
}

#[cfg(any(not(Py_LIMITED_API), Py_3_11))]
#[inline]
pub unsafe fn releasebufferproc(
    slf: *mut ffi::PyObject,
    buf: *mut ffi::Py_buffer,
    f: for<'py> unsafe fn(Python<'py>, *mut ffi::PyObject, *mut ffi::Py_buffer) -> PyResult<()>,
) {
    unsafe { trampoline_unraisable(|py| f(py, slf, buf), slf) }
}

#[inline]
pub(crate) unsafe fn dealloc(
    slf: *mut ffi::PyObject,
    f: for<'py> unsafe fn(Python<'py>, *mut ffi::PyObject) -> (),
) {
    // After calling tp_dealloc the object is no longer valid,
    // so pass null_mut() to the context.
    //
    // (Note that we don't allow the implementation `f` to fail.)
    unsafe {
        trampoline_unraisable(
            |py| {
                f(py, slf);
                Ok(())
            },
            std::ptr::null_mut(),
        )
    }
}

// Ipowfunc is a unique case where PyO3 has its own type
// to workaround a problem on 3.7 (see IPowModulo type definition).
// Once 3.7 support dropped can just remove this.
trampoline!(
    pub fn ipowfunc(
        arg1: *mut ffi::PyObject,
        arg2: *mut ffi::PyObject,
        arg3: IPowModulo,
    ) -> *mut ffi::PyObject;
);

/// Implementation of trampoline functions, which sets up an AttachGuard and calls F.
///
/// Panics during execution are trapped so that they don't propagate through any
/// outer FFI boundary.
///
/// The thread must already be attached to the interpreter when this is called.
#[inline]
pub(crate) unsafe fn trampoline<F, R>(body: F) -> R
where
    F: for<'py> FnOnce(Python<'py>) -> PyResult<R> + UnwindSafe,
    R: PyCallbackOutput,
{
    let trap = PanicTrap::new("uncaught panic at ffi boundary");

    // SAFETY: This function requires the thread to already be attached.
    let guard = unsafe { AttachGuard::assume() };
    let py = guard.python();
    let out = panic_result_into_callback_output(
        py,
        panic::catch_unwind(move || -> PyResult<_> { body(py) }),
    );
    trap.disarm();
    out
}

/// Converts the output of std::panic::catch_unwind into a Python function output, either by raising a Python
/// exception or by unwrapping the contained success output.
#[inline]
fn panic_result_into_callback_output<R>(
    py: Python<'_>,
    panic_result: Result<PyResult<R>, Box<dyn Any + Send + 'static>>,
) -> R
where
    R: PyCallbackOutput,
{
    let py_err = match panic_result {
        Ok(Ok(value)) => return value,
        Ok(Err(py_err)) => py_err,
        Err(payload) => PanicException::from_panic_payload(payload),
    };
    py_err.restore(py);
    R::ERR_VALUE
}

/// Implementation of trampoline for functions which can't return an error.
///
/// Panics during execution are trapped so that they don't propagate through any
/// outer FFI boundary.
///
/// Exceptions produced are sent to `sys.unraisablehook`.
///
/// # Safety
///
/// - ctx must be either a valid ffi::PyObject or NULL
/// - The thread must be attached to the interpreter when this is called.
#[inline]
unsafe fn trampoline_unraisable<F>(body: F, ctx: *mut ffi::PyObject)
where
    F: for<'py> FnOnce(Python<'py>) -> PyResult<()> + UnwindSafe,
{
    let trap = PanicTrap::new("uncaught panic at ffi boundary");

    // SAFETY: Thread is known to be attached.
    let guard = unsafe { AttachGuard::assume() };
    let py = guard.python();

    if let Err(py_err) = panic::catch_unwind(move || body(py))
        .unwrap_or_else(|payload| Err(PanicException::from_panic_payload(payload)))
    {
        py_err.write_unraisable(py, unsafe { ctx.assume_borrowed_or_opt(py) }.as_deref());
    }
    trap.disarm();
}
