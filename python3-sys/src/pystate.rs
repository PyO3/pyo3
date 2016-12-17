use libc::{c_int, c_long};
use object::PyObject;
use moduleobject::PyModuleDef;

#[cfg(Py_3_6)]
pub const MAX_CO_EXTRA_USERS: c_int = 255;

pub enum PyInterpreterState { }
pub enum PyThreadState { }

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyInterpreterState_New() -> *mut PyInterpreterState;
    pub fn PyInterpreterState_Clear(arg1: *mut PyInterpreterState) -> ();
    pub fn PyInterpreterState_Delete(arg1: *mut PyInterpreterState) -> ();
    //fn _PyState_AddModule(arg1: *mut PyObject,
    //                      arg2: *mut PyModuleDef) -> c_int;
    pub fn PyState_FindModule(arg1: *mut PyModuleDef) -> *mut PyObject;
    pub fn PyThreadState_New(arg1: *mut PyInterpreterState)
     -> *mut PyThreadState;
    //fn _PyThreadState_Prealloc(arg1: *mut PyInterpreterState)
    // -> *mut PyThreadState;
    //fn _PyThreadState_Init(arg1: *mut PyThreadState) -> ();
    pub fn PyThreadState_Clear(arg1: *mut PyThreadState) -> ();
    pub fn PyThreadState_Delete(arg1: *mut PyThreadState) -> ();
    #[cfg(py_sys_config="WITH_THREAD")]
    pub fn PyThreadState_DeleteCurrent() -> ();
    pub fn PyThreadState_Get() -> *mut PyThreadState;
    pub fn PyThreadState_Swap(arg1: *mut PyThreadState) -> *mut PyThreadState;
    pub fn PyThreadState_GetDict() -> *mut PyObject;
    pub fn PyThreadState_SetAsyncExc(arg1: c_long,
                                     arg2: *mut PyObject) -> c_int;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum PyGILState_STATE {
    PyGILState_LOCKED,
    PyGILState_UNLOCKED
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyGILState_Ensure() -> PyGILState_STATE;
    pub fn PyGILState_Release(arg1: PyGILState_STATE) -> ();
    pub fn PyGILState_GetThisThreadState() -> *mut PyThreadState;
}

#[inline(always)]
pub unsafe fn PyThreadState_GET() -> *mut PyThreadState {
    PyThreadState_Get()
}

