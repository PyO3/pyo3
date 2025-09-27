use crate::moduleobject::PyModuleDef;
use crate::object::PyObject;
use std::ffi::c_int;

#[cfg(all(Py_3_10, not(PyPy), not(Py_LIMITED_API)))]
use crate::PyFrameObject;

#[cfg(not(PyPy))]
use std::ffi::c_long;

pub const MAX_CO_EXTRA_USERS: c_int = 255;

opaque_struct!(pub PyThreadState);
opaque_struct!(pub PyInterpreterState);

extern "C" {
    #[cfg(not(PyPy))]
    pub fn PyInterpreterState_New() -> *mut PyInterpreterState;
    #[cfg(not(PyPy))]
    pub fn PyInterpreterState_Clear(arg1: *mut PyInterpreterState);
    #[cfg(not(PyPy))]
    pub fn PyInterpreterState_Delete(arg1: *mut PyInterpreterState);

    #[cfg(all(Py_3_9, not(PyPy)))]
    pub fn PyInterpreterState_Get() -> *mut PyInterpreterState;

    #[cfg(all(Py_3_8, not(PyPy)))]
    pub fn PyInterpreterState_GetDict(arg1: *mut PyInterpreterState) -> *mut PyObject;

    #[cfg(not(PyPy))]
    pub fn PyInterpreterState_GetID(arg1: *mut PyInterpreterState) -> i64;

    #[cfg_attr(PyPy, link_name = "PyPyState_AddModule")]
    pub fn PyState_AddModule(arg1: *mut PyObject, arg2: *mut PyModuleDef) -> c_int;

    #[cfg_attr(PyPy, link_name = "PyPyState_RemoveModule")]
    pub fn PyState_RemoveModule(arg1: *mut PyModuleDef) -> c_int;

    // only has PyPy prefix since 3.10
    #[cfg_attr(all(PyPy, Py_3_10), link_name = "PyPyState_FindModule")]
    pub fn PyState_FindModule(arg1: *mut PyModuleDef) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyThreadState_New")]
    pub fn PyThreadState_New(arg1: *mut PyInterpreterState) -> *mut PyThreadState;
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_Clear")]
    pub fn PyThreadState_Clear(arg1: *mut PyThreadState);
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_Delete")]
    pub fn PyThreadState_Delete(arg1: *mut PyThreadState);

    #[cfg_attr(PyPy, link_name = "PyPyThreadState_Get")]
    pub fn PyThreadState_Get() -> *mut PyThreadState;
}

#[inline]
pub unsafe fn PyThreadState_GET() -> *mut PyThreadState {
    PyThreadState_Get()
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_Swap")]
    pub fn PyThreadState_Swap(arg1: *mut PyThreadState) -> *mut PyThreadState;
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_GetDict")]
    pub fn PyThreadState_GetDict() -> *mut PyObject;
    #[cfg(not(PyPy))]
    pub fn PyThreadState_SetAsyncExc(arg1: c_long, arg2: *mut PyObject) -> c_int;
}

// skipped non-limited / 3.9 PyThreadState_GetInterpreter
// skipped non-limited / 3.9 PyThreadState_GetID

extern "C" {
    // PyThreadState_GetFrame
    #[cfg(all(Py_3_10, not(PyPy), not(Py_LIMITED_API)))]
    pub fn PyThreadState_GetFrame(arg1: *mut PyThreadState) -> *mut PyFrameObject;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PyGILState_STATE {
    PyGILState_LOCKED,
    PyGILState_UNLOCKED,
}

#[cfg(not(any(Py_3_14, target_arch = "wasm32")))]
struct HangThread;

#[cfg(not(any(Py_3_14, target_arch = "wasm32")))]
impl Drop for HangThread {
    fn drop(&mut self) {
        loop {
            std::thread::park(); // Block forever.
        }
    }
}

// The PyGILState_Ensure function will call pthread_exit during interpreter shutdown,
// which causes undefined behavior. Redirect to the "safe" version that hangs instead,
// as Python 3.14 does.
//
// See https://github.com/rust-lang/rust/issues/135929

// C-unwind only supported (and necessary) since 1.71. Python 3.14+ does not do
// pthread_exit from PyGILState_Ensure (https://github.com/python/cpython/issues/87135).
mod raw {
    #[cfg(not(any(Py_3_14, target_arch = "wasm32")))]
    extern "C-unwind" {
        #[cfg_attr(PyPy, link_name = "PyPyGILState_Ensure")]
        pub fn PyGILState_Ensure() -> super::PyGILState_STATE;
    }

    #[cfg(any(Py_3_14, target_arch = "wasm32"))]
    extern "C" {
        #[cfg_attr(PyPy, link_name = "PyPyGILState_Ensure")]
        pub fn PyGILState_Ensure() -> super::PyGILState_STATE;
    }
}

#[cfg(not(any(Py_3_14, target_arch = "wasm32")))]
pub unsafe extern "C" fn PyGILState_Ensure() -> PyGILState_STATE {
    let guard = HangThread;
    // If `PyGILState_Ensure` calls `pthread_exit`, which it does on Python < 3.14
    // when the interpreter is shutting down, this will cause a forced unwind.
    // doing a forced unwind through a function with a Rust destructor is unspecified
    // behavior.
    //
    // However, currently it runs the destructor, which will cause the thread to
    // hang as it should.
    //
    // And if we don't catch the unwinding here, then one of our callers probably has a destructor,
    // so it's unspecified behavior anyway, and on many configurations causes the process to abort.
    //
    // The alternative is for pyo3 to contain custom C or C++ code that catches the `pthread_exit`,
    // but that's also annoying from a portability point of view.
    //
    // On Windows, `PyGILState_Ensure` calls `_endthreadex` instead, which AFAICT can't be caught
    // and therefore will cause unsafety if there are pinned objects on the stack. AFAICT there's
    // nothing we can do it other than waiting for Python 3.14 or not using Windows. At least,
    // if there is nothing pinned on the stack, it won't cause the process to crash.
    let ret: PyGILState_STATE = raw::PyGILState_Ensure();
    std::mem::forget(guard);
    ret
}

#[cfg(any(Py_3_14, target_arch = "wasm32"))]
pub use self::raw::PyGILState_Ensure;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyGILState_Release")]
    pub fn PyGILState_Release(arg1: PyGILState_STATE);
    #[cfg(not(PyPy))]
    pub fn PyGILState_GetThisThreadState() -> *mut PyThreadState;
}
