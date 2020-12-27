use crate::ffi::ceval::_PyFrameEvalFunction;
use crate::ffi::moduleobject::PyModuleDef;
use crate::ffi::object::PyObject;
use crate::ffi::PyFrameObject;
use std::os::raw::{c_int, c_long};

pub const MAX_CO_EXTRA_USERS: c_int = 255;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyInterpreterState {
    pub ob_base: PyObject,
    pub eval_frame: _PyFrameEvalFunction,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyThreadState {
    pub ob_base: PyObject,
    pub interp: *mut PyInterpreterState,
}

extern "C" {
    pub fn PyInterpreterState_New() -> *mut PyInterpreterState;
    pub fn PyInterpreterState_Clear(arg1: *mut PyInterpreterState);
    pub fn PyInterpreterState_Delete(arg1: *mut PyInterpreterState);
    //fn _PyState_AddModule(arg1: *mut PyObject,
    //                      arg2: *mut PyModuleDef) -> c_int;
    pub fn PyState_FindModule(arg1: *mut PyModuleDef) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_New")]
    pub fn PyThreadState_New(arg1: *mut PyInterpreterState) -> *mut PyThreadState;
    //fn _PyThreadState_Prealloc(arg1: *mut PyInterpreterState)
    // -> *mut PyThreadState;
    //fn _PyThreadState_Init(arg1: *mut PyThreadState);
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_Clear")]
    pub fn PyThreadState_Clear(arg1: *mut PyThreadState);
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_Delete")]
    pub fn PyThreadState_Delete(arg1: *mut PyThreadState);
    #[cfg(py_sys_config = "WITH_THREAD")]
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_DeleteCurrent")]
    pub fn PyThreadState_DeleteCurrent();
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_Get")]
    pub fn PyThreadState_Get() -> *mut PyThreadState;
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_Swap")]
    pub fn PyThreadState_Swap(arg1: *mut PyThreadState) -> *mut PyThreadState;
    #[cfg_attr(PyPy, link_name = "PyPyThreadState_GetDict")]
    pub fn PyThreadState_GetDict() -> *mut PyObject;
    pub fn PyThreadState_SetAsyncExc(arg1: c_long, arg2: *mut PyObject) -> c_int;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PyGILState_STATE {
    PyGILState_LOCKED,
    PyGILState_UNLOCKED,
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyGILState_Ensure")]
    pub fn PyGILState_Ensure() -> PyGILState_STATE;
    #[cfg_attr(PyPy, link_name = "PyPyGILState_Release")]
    pub fn PyGILState_Release(arg1: PyGILState_STATE);
    pub fn PyGILState_GetThisThreadState() -> *mut PyThreadState;
    pub fn PyGILState_Check() -> c_int;
}

#[inline]
pub unsafe fn PyThreadState_GET() -> *mut PyThreadState {
    PyThreadState_Get()
}

pub type Py_tracefunc = extern "C" fn(
    obj: *mut PyObject,
    frame: *mut PyFrameObject,
    what: c_int,
    arg: *mut PyObject,
) -> c_int;
