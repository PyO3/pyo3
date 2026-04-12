use crate::moduleobject::PyModuleDef;
use crate::object::PyObject;
use crate::pytypedefs::{PyInterpreterState, PyThreadState};
use crate::rustpython_runtime;
use std::ffi::c_int;

#[cfg(not(PyPy))]
use std::ffi::c_long;

static mut DUMMY_INTERPRETER_STATE: u8 = 0;
static mut DUMMY_THREAD_STATE: u8 = 0;

pub const MAX_CO_EXTRA_USERS: c_int = 255;

#[inline]
pub unsafe fn PyInterpreterState_New() -> *mut PyInterpreterState {
    (&raw mut DUMMY_INTERPRETER_STATE).cast()
}

#[inline]
pub unsafe fn PyInterpreterState_Clear(_interp: *mut PyInterpreterState) {}

#[inline]
pub unsafe fn PyInterpreterState_Delete(_interp: *mut PyInterpreterState) {}

#[cfg(all(Py_3_9, not(PyPy)))]
#[inline]
pub unsafe fn PyInterpreterState_Get() -> *mut PyInterpreterState {
    (&raw mut DUMMY_INTERPRETER_STATE).cast()
}

#[cfg(not(PyPy))]
#[inline]
pub unsafe fn PyInterpreterState_GetDict(_interp: *mut PyInterpreterState) -> *mut PyObject {
    std::ptr::null_mut()
}

#[cfg(not(PyPy))]
#[inline]
pub unsafe fn PyInterpreterState_GetID(_interp: *mut PyInterpreterState) -> i64 {
    1
}

#[inline]
pub unsafe fn PyState_AddModule(_module: *mut PyObject, _def: *mut PyModuleDef) -> c_int {
    0
}

#[inline]
pub unsafe fn PyState_RemoveModule(_def: *mut PyModuleDef) -> c_int {
    0
}

#[inline]
pub unsafe fn PyState_FindModule(_def: *mut PyModuleDef) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyThreadState_New(_interp: *mut PyInterpreterState) -> *mut PyThreadState {
    (&raw mut DUMMY_THREAD_STATE).cast()
}

#[inline]
pub unsafe fn PyThreadState_Clear(_tstate: *mut PyThreadState) {}

#[inline]
pub unsafe fn PyThreadState_Delete(_tstate: *mut PyThreadState) {}

#[inline]
pub unsafe fn PyThreadState_Get() -> *mut PyThreadState {
    (&raw mut DUMMY_THREAD_STATE).cast()
}

#[inline]
pub unsafe fn PyThreadState_GET() -> *mut PyThreadState {
    PyThreadState_Get()
}

#[inline]
pub unsafe fn PyThreadState_Swap(tstate: *mut PyThreadState) -> *mut PyThreadState {
    tstate
}

#[inline]
pub unsafe fn PyThreadState_GetDict() -> *mut PyObject {
    std::ptr::null_mut()
}

#[cfg(not(PyPy))]
#[inline]
pub unsafe fn PyThreadState_SetAsyncExc(_id: c_long, _exc: *mut PyObject) -> c_int {
    0
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PyGILState_STATE {
    PyGILState_LOCKED,
    PyGILState_UNLOCKED,
}

#[inline]
pub unsafe extern "C" fn PyGILState_Ensure() -> PyGILState_STATE {
    match rustpython_runtime::ensure_attached() {
        rustpython_runtime::AttachState::Assumed => PyGILState_STATE::PyGILState_LOCKED,
        rustpython_runtime::AttachState::Ensured => PyGILState_STATE::PyGILState_UNLOCKED,
    }
}

#[inline]
pub unsafe fn PyGILState_Release(_state: PyGILState_STATE) {
    rustpython_runtime::release_attached();
}

#[inline]
pub unsafe fn PyGILState_Check() -> c_int {
    rustpython_runtime::is_attached() as c_int
}

#[cfg(not(PyPy))]
#[inline]
pub unsafe fn PyGILState_GetThisThreadState() -> *mut PyThreadState {
    PyThreadState_Get()
}

#[inline]
pub unsafe fn _PyThreadState_UncheckedGet() -> *mut PyThreadState {
    PyThreadState_Get()
}
