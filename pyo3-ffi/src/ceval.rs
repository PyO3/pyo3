use crate::object::PyObject;
use crate::pystate::PyThreadState;
use std::ffi::{c_char, c_int, c_void};

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyEval_EvalCode")]
    pub fn PyEval_EvalCode(
        arg1: *mut PyObject,
        arg2: *mut PyObject,
        arg3: *mut PyObject,
    ) -> *mut PyObject;

    pub fn PyEval_EvalCodeEx(
        co: *mut PyObject,
        globals: *mut PyObject,
        locals: *mut PyObject,
        args: *const *mut PyObject,
        argc: c_int,
        kwds: *const *mut PyObject,
        kwdc: c_int,
        defs: *const *mut PyObject,
        defc: c_int,
        kwdefs: *mut PyObject,
        closure: *mut PyObject,
    ) -> *mut PyObject;

    #[cfg(not(Py_3_13))]
    #[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
    #[cfg_attr(PyPy, link_name = "PyPyEval_CallObjectWithKeywords")]
    pub fn PyEval_CallObjectWithKeywords(
        func: *mut PyObject,
        obj: *mut PyObject,
        kwargs: *mut PyObject,
    ) -> *mut PyObject;
}

#[cfg(not(Py_3_13))]
#[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
#[inline]
pub unsafe fn PyEval_CallObject(func: *mut PyObject, arg: *mut PyObject) -> *mut PyObject {
    #[allow(deprecated)]
    PyEval_CallObjectWithKeywords(func, arg, std::ptr::null_mut())
}

extern "C" {
    #[cfg(not(Py_3_13))]
    #[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
    #[cfg_attr(PyPy, link_name = "PyPyEval_CallFunction")]
    pub fn PyEval_CallFunction(obj: *mut PyObject, format: *const c_char, ...) -> *mut PyObject;
    #[cfg(not(Py_3_13))]
    #[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
    #[cfg_attr(PyPy, link_name = "PyPyEval_CallMethod")]
    pub fn PyEval_CallMethod(
        obj: *mut PyObject,
        methodname: *const c_char,
        format: *const c_char,
        ...
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyEval_GetBuiltins")]
    pub fn PyEval_GetBuiltins() -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyEval_GetGlobals")]
    pub fn PyEval_GetGlobals() -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyEval_GetLocals")]
    pub fn PyEval_GetLocals() -> *mut PyObject;
    pub fn PyEval_GetFrame() -> *mut crate::PyFrameObject;
    #[cfg_attr(PyPy, link_name = "PyPy_AddPendingCall")]
    pub fn Py_AddPendingCall(
        func: Option<extern "C" fn(arg1: *mut c_void) -> c_int>,
        arg: *mut c_void,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_MakePendingCalls")]
    pub fn Py_MakePendingCalls() -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_SetRecursionLimit")]
    pub fn Py_SetRecursionLimit(arg1: c_int);
    #[cfg_attr(PyPy, link_name = "PyPy_GetRecursionLimit")]
    pub fn Py_GetRecursionLimit() -> c_int;
    fn _Py_CheckRecursiveCall(_where: *mut c_char) -> c_int;
}

extern "C" {
    #[cfg(Py_3_9)]
    #[cfg_attr(PyPy, link_name = "PyPy_EnterRecursiveCall")]
    pub fn Py_EnterRecursiveCall(arg1: *const c_char) -> c_int;
    #[cfg(Py_3_9)]
    #[cfg_attr(PyPy, link_name = "PyPy_LeaveRecursiveCall")]
    pub fn Py_LeaveRecursiveCall();
}

extern "C" {
    pub fn PyEval_GetFuncName(arg1: *mut PyObject) -> *const c_char;
    pub fn PyEval_GetFuncDesc(arg1: *mut PyObject) -> *const c_char;
    pub fn PyEval_GetCallStats(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyEval_EvalFrame(arg1: *mut crate::PyFrameObject) -> *mut PyObject;
    pub fn PyEval_EvalFrameEx(f: *mut crate::PyFrameObject, exc: c_int) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyEval_SaveThread")]
    pub fn PyEval_SaveThread() -> *mut PyThreadState;
    #[cfg_attr(PyPy, link_name = "PyPyEval_RestoreThread")]
    pub fn PyEval_RestoreThread(arg1: *mut PyThreadState);
}

extern "C" {
    #[cfg(not(Py_3_13))]
    #[cfg_attr(PyPy, link_name = "PyPyEval_ThreadsInitialized")]
    #[cfg_attr(
        Py_3_9,
        deprecated(
            note = "Deprecated in Python 3.9, this function always returns true in Python 3.7 or newer."
        )
    )]
    pub fn PyEval_ThreadsInitialized() -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyEval_InitThreads")]
    #[cfg_attr(
        Py_3_9,
        deprecated(
            note = "Deprecated in Python 3.9, this function does nothing in Python 3.7 or newer."
        )
    )]
    pub fn PyEval_InitThreads();
    pub fn PyEval_AcquireLock();
    pub fn PyEval_ReleaseLock();
    #[cfg_attr(PyPy, link_name = "PyPyEval_AcquireThread")]
    pub fn PyEval_AcquireThread(tstate: *mut PyThreadState);
    #[cfg_attr(PyPy, link_name = "PyPyEval_ReleaseThread")]
    pub fn PyEval_ReleaseThread(tstate: *mut PyThreadState);
    #[cfg(not(Py_3_8))]
    pub fn PyEval_ReInitThreads();
}

// skipped Py_BEGIN_ALLOW_THREADS
// skipped Py_BLOCK_THREADS
// skipped Py_UNBLOCK_THREADS
// skipped Py_END_ALLOW_THREADS
// skipped FVC_MASK
// skipped FVC_NONE
// skipped FVC_STR
// skipped FVC_REPR
// skipped FVC_ASCII
// skipped FVS_MASK
// skipped FVS_HAVE_SPEC
