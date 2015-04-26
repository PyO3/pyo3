use libc::{c_int, c_long};
use object::PyObject;

#[allow(missing_copy_implementations)]
pub enum PyInterpreterState { }

#[allow(missing_copy_implementations)]
pub enum PyFrameObject { }

pub type Py_tracefunc =
    unsafe extern "C" fn
                              (arg1: *mut PyObject, arg2: *mut PyFrameObject,
                               arg3: c_int, arg4: *mut PyObject)
                              -> c_int;

/* The following values are used for 'what' for tracefunc functions: */
pub const PyTrace_CALL : c_int = 0;
pub const PyTrace_EXCEPTION : c_int = 1;
pub const PyTrace_LINE : c_int = 2;
pub const PyTrace_RETURN : c_int = 3;
pub const PyTrace_C_CALL : c_int = 4;
pub const PyTrace_C_EXCEPTION : c_int = 5;
pub const PyTrace_C_RETURN : c_int = 6;

#[repr(C)]
#[derive(Copy)]
pub struct PyThreadState {
    pub next: *mut PyThreadState,
    pub interp: *mut PyInterpreterState,
    pub frame: *mut PyFrameObject,
    pub recursion_depth: c_int,
    pub tracing: c_int,
    pub use_tracing: c_int,
    pub c_profilefunc: Option<Py_tracefunc>,
    pub c_tracefunc: Option<Py_tracefunc>,
    pub c_profileobj: *mut PyObject,
    pub c_traceobj: *mut PyObject,
    pub curexc_type: *mut PyObject,
    pub curexc_value: *mut PyObject,
    pub curexc_traceback: *mut PyObject,
    pub exc_type: *mut PyObject,
    pub exc_value: *mut PyObject,
    pub exc_traceback: *mut PyObject,
    pub dict: *mut PyObject,
    pub tick_counter: c_int,
    pub gilstate_counter: c_int,
    pub async_exc: *mut PyObject,
    pub thread_id: c_long,
    pub trash_delete_nesting: c_int,
    pub trash_delete_later: *mut PyObject,
}

impl Clone for PyThreadState {
    #[inline] fn clone(&self) -> PyThreadState { *self }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum PyGILState_STATE {
    PyGILState_LOCKED,
    PyGILState_UNLOCKED
}


#[link(name = "python2.7")]
extern "C" {
    static mut _PyThreadState_Current: *mut PyThreadState;
    //static mut _PyThreadState_GetFrame: PyThreadFrameGetter;

    pub fn PyInterpreterState_New() -> *mut PyInterpreterState;
    pub fn PyInterpreterState_Clear(arg1: *mut PyInterpreterState);
    pub fn PyInterpreterState_Delete(arg1: *mut PyInterpreterState);
    pub fn PyThreadState_New(arg1: *mut PyInterpreterState)
     -> *mut PyThreadState;
    pub fn _PyThreadState_Prealloc(arg1: *mut PyInterpreterState)
     -> *mut PyThreadState;
    pub fn _PyThreadState_Init(arg1: *mut PyThreadState);
    pub fn PyThreadState_Clear(arg1: *mut PyThreadState);
    pub fn PyThreadState_Delete(arg1: *mut PyThreadState);
    #[cfg(feature="WITH_THREAD")]
    pub fn PyThreadState_DeleteCurrent();
    pub fn PyThreadState_Get() -> *mut PyThreadState;
    pub fn PyThreadState_Swap(arg1: *mut PyThreadState) -> *mut PyThreadState;
    pub fn PyThreadState_GetDict() -> *mut PyObject;
    pub fn PyThreadState_SetAsyncExc(arg1: c_long,
                                     arg2: *mut PyObject) -> c_int;
    pub fn PyGILState_Ensure() -> PyGILState_STATE;
    pub fn PyGILState_Release(arg1: PyGILState_STATE);
    pub fn PyGILState_GetThisThreadState() -> *mut PyThreadState;
    fn _PyThread_CurrentFrames() -> *mut PyObject;
    pub fn PyInterpreterState_Head() -> *mut PyInterpreterState;
    pub fn PyInterpreterState_Next(arg1: *mut PyInterpreterState)
     -> *mut PyInterpreterState;
    pub fn PyInterpreterState_ThreadHead(arg1: *mut PyInterpreterState)
     -> *mut PyThreadState;
    pub fn PyThreadState_Next(arg1: *mut PyThreadState) -> *mut PyThreadState;
}

#[cfg(feature="Py_DEBUG")]
#[inline(always)]
pub unsafe fn PyThreadState_GET() -> *mut PyThreadState {
    PyThreadState_Get()
}

#[cfg(not(feature="Py_DEBUG"))]
#[inline(always)]
pub unsafe fn PyThreadState_GET() -> *mut PyThreadState {
    _PyThreadState_Current
}


