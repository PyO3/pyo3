use crate::object::*;
use crate::PyFrameObject;
#[cfg(all(Py_3_11, not(any(PyPy, GraalPy, Py_3_14))))]
use std::ffi::c_char;
use std::ffi::c_int;
use std::ptr::addr_of_mut;

#[cfg(not(any(PyPy, GraalPy, Py_3_14)))]
#[repr(C)]
pub struct PyGenObject {
    pub ob_base: PyObject,
    #[cfg(not(Py_3_11))]
    pub gi_frame: *mut PyFrameObject,
    #[cfg(not(Py_3_10))]
    pub gi_running: c_int,
    #[cfg(not(Py_3_12))]
    pub gi_code: *mut PyObject,
    pub gi_weakreflist: *mut PyObject,
    pub gi_name: *mut PyObject,
    pub gi_qualname: *mut PyObject,
    #[allow(private_interfaces)]
    pub gi_exc_state: crate::cpython::pystate::_PyErr_StackItem,
    #[cfg(Py_3_11)]
    pub gi_origin_or_finalizer: *mut PyObject,
    #[cfg(Py_3_11)]
    pub gi_hooks_inited: c_char,
    #[cfg(Py_3_11)]
    pub gi_closed: c_char,
    #[cfg(Py_3_11)]
    pub gi_running_async: c_char,
    #[cfg(Py_3_11)]
    pub gi_frame_state: i8,
    #[cfg(Py_3_11)]
    pub gi_iframe: [*mut PyObject; 1],
}

#[cfg(all(Py_3_14, not(any(PyPy, GraalPy))))]
opaque_struct!(pub PyGenObject);

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyGen_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyGen_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut!(PyGen_Type))
}

#[inline]
pub unsafe fn PyGen_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyGen_Type)) as c_int
}

extern "C" {
    pub fn PyGen_New(frame: *mut PyFrameObject) -> *mut PyObject;
    // skipped PyGen_NewWithQualName
    // skipped _PyGen_SetStopIterationValue
    // skipped _PyGen_FetchStopIterationValue
    // skipped _PyGen_yf
    // skipped _PyGen_Finalize
    #[cfg(not(any(Py_3_9, PyPy)))]
    #[deprecated(note = "This function was never documented in the Python API.")]
    pub fn PyGen_NeedsFinalizing(op: *mut PyGenObject) -> c_int;
}

// skipped PyCoroObject

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyCoro_Type: PyTypeObject;
}

// skipped _PyCoroWrapper_Type

#[inline]
pub unsafe fn PyCoro_CheckExact(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut!(PyCoro_Type))
}

// skipped _PyCoro_GetAwaitableIter
// skipped PyCoro_New

// skipped PyAsyncGenObject

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyAsyncGen_Type: PyTypeObject;
    // skipped _PyAsyncGenASend_Type
    // skipped _PyAsyncGenWrappedValue_Type
    // skipped _PyAsyncGenAThrow_Type
}

// skipped PyAsyncGen_New

#[inline]
pub unsafe fn PyAsyncGen_CheckExact(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut!(PyAsyncGen_Type))
}

// skipped _PyAsyncGenValueWrapperNew
