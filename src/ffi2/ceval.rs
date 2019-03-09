use crate::ffi2::frameobject::PyFrameObject;
use crate::ffi2::object::PyObject;
use crate::ffi2::pyport::Py_ssize_t;
use crate::ffi2::pystate::{PyThreadState, Py_tracefunc};
use crate::ffi2::pythonrun::PyCompilerFlags;
use std::os::raw::{c_char, c_int, c_void};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyEval_CallObjectWithKeywords")]
    pub fn PyEval_CallObjectWithKeywords(
        callable: *mut PyObject,
        args: *mut PyObject,
        kwds: *mut PyObject,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyEval_CallFunction")]
    pub fn PyEval_CallFunction(obj: *mut PyObject, format: *const c_char, ...) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyEval_CallMethod")]
    pub fn PyEval_CallMethod(
        obj: *mut PyObject,
        methodname: *const c_char,
        format: *const c_char,
        ...
    ) -> *mut PyObject;
    pub fn PyEval_SetProfile(func: Option<Py_tracefunc>, obj: *mut PyObject);
    pub fn PyEval_SetTrace(func: Option<Py_tracefunc>, obj: *mut PyObject);
    #[cfg_attr(PyPy, link_name = "PyPyEval_GetBuiltins")]
    pub fn PyEval_GetBuiltins() -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyEval_GetGlobals")]
    pub fn PyEval_GetGlobals() -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyEval_GetLocals")]
    pub fn PyEval_GetLocals() -> *mut PyObject;
    pub fn PyEval_GetFrame() -> *mut PyFrameObject;
    pub fn PyEval_GetRestricted() -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyEval_MergeCompilerFlags")]
    pub fn PyEval_MergeCompilerFlags(cf: *mut PyCompilerFlags) -> c_int;
    pub fn Py_FlushLine() -> c_int;
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

    pub fn PyEval_GetFuncName(arg1: *mut PyObject) -> *const c_char;
    pub fn PyEval_GetFuncDesc(arg1: *mut PyObject) -> *const c_char;
    pub fn PyEval_GetCallStats(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyEval_EvalFrame(arg1: *mut PyFrameObject) -> *mut PyObject;
    pub fn PyEval_EvalFrameEx(f: *mut PyFrameObject, exc: c_int) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyEval_SaveThread")]
    pub fn PyEval_SaveThread() -> *mut PyThreadState;
    #[cfg_attr(PyPy, link_name = "PyPyEval_RestoreThread")]
    pub fn PyEval_RestoreThread(arg1: *mut PyThreadState);

    #[cfg_attr(PyPy, link_name = "_PyPyEval_SliceIndex")]
    fn _PyEval_SliceIndex(arg1: *mut PyObject, arg2: *mut Py_ssize_t) -> c_int;
}

#[cfg(py_sys_config = "WITH_THREAD")]
#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyEval_ThreadsInitialized")]
    pub fn PyEval_ThreadsInitialized() -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyEval_InitThreads")]
    pub fn PyEval_InitThreads();
    pub fn PyEval_AcquireLock();
    pub fn PyEval_ReleaseLock();
    #[cfg_attr(PyPy, link_name = "PyPyEval_AcquireThread")]
    pub fn PyEval_AcquireThread(tstate: *mut PyThreadState);
    #[cfg_attr(PyPy, link_name = "PyPyEval_ReleaseThread")]
    pub fn PyEval_ReleaseThread(tstate: *mut PyThreadState);
    pub fn PyEval_ReInitThreads();
}
