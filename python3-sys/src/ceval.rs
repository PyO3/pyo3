use libc::{c_void, c_char, c_int};
use object::PyObject;
use pystate::PyThreadState;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyEval_CallObjectWithKeywords(func: *mut PyObject,
                                         obj: *mut PyObject,
                                         kwargs: *mut PyObject)
     -> *mut PyObject;
}

#[inline]
pub unsafe fn PyEval_CallObject(func: *mut PyObject, arg: *mut PyObject) -> *mut PyObject {
    PyEval_CallObjectWithKeywords(func, arg, ::core::ptr::null_mut())
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyEval_CallFunction(obj: *mut PyObject,
                               format: *const c_char, ...)
     -> *mut PyObject;
    pub fn PyEval_CallMethod(obj: *mut PyObject,
                             methodname: *const c_char,
                             format: *const c_char, ...)
     -> *mut PyObject;
    pub fn PyEval_GetBuiltins() -> *mut PyObject;
    pub fn PyEval_GetGlobals() -> *mut PyObject;
    pub fn PyEval_GetLocals() -> *mut PyObject;
    pub fn PyEval_GetFrame() -> *mut ::PyFrameObject;
    pub fn Py_AddPendingCall(func: Option<extern "C" fn(arg1: *mut c_void) -> c_int>,
                             arg: *mut c_void) -> c_int;
    pub fn Py_MakePendingCalls() -> c_int;
    pub fn Py_SetRecursionLimit(arg1: c_int) -> ();
    pub fn Py_GetRecursionLimit() -> c_int;
    
    fn _Py_CheckRecursiveCall(_where: *mut c_char)
     -> c_int;
    static mut _Py_CheckRecursionLimit: c_int;
}

// TODO: Py_EnterRecursiveCall etc.

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyEval_GetFuncName(arg1: *mut PyObject) -> *const c_char;
    pub fn PyEval_GetFuncDesc(arg1: *mut PyObject) -> *const c_char;
    pub fn PyEval_GetCallStats(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyEval_EvalFrame(arg1: *mut ::PyFrameObject) -> *mut PyObject;
    pub fn PyEval_EvalFrameEx(f: *mut ::PyFrameObject, exc: c_int)
     -> *mut PyObject;
    pub fn PyEval_SaveThread() -> *mut PyThreadState;
    pub fn PyEval_RestoreThread(arg1: *mut PyThreadState) -> ();
}

#[cfg(py_sys_config = "WITH_THREAD")]
#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyEval_ThreadsInitialized() -> c_int;
    pub fn PyEval_InitThreads() -> ();
    pub fn PyEval_AcquireLock() -> ();
    pub fn PyEval_ReleaseLock() -> ();
    pub fn PyEval_AcquireThread(tstate: *mut PyThreadState) -> ();
    pub fn PyEval_ReleaseThread(tstate: *mut PyThreadState) -> ();
    pub fn PyEval_ReInitThreads() -> ();
}

