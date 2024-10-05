use crate::moduleobject::PyModuleDef;
use crate::object::PyObject;
use std::os::raw::c_int;

#[cfg(not(PyPy))]
use std::os::raw::c_long;

pub const MAX_CO_EXTRA_USERS: c_int = 255;

opaque_struct!(PyThreadState);
opaque_struct!(PyInterpreterState);

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
// skipped non-limited / 3.9 PyThreadState_GetFrame
// skipped non-limited / 3.9 PyThreadState_GetID

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
    #[cfg(not(PyPy))]
    pub fn PyGILState_GetThisThreadState() -> *mut PyThreadState;
}
