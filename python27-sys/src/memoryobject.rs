use libc::{c_int, c_char};
use pyport::Py_ssize_t;
use object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyMemoryView_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyMemoryView_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyMemoryView_Type;
    (Py_TYPE(op) == u) as c_int
}

#[inline(always)]
pub unsafe fn PyMemoryView_GET_BUFFER(op : *mut PyObject) -> *mut Py_buffer {
    &mut (*(op as *mut PyMemoryViewObject)).view
}

#[inline(always)]
pub unsafe fn PyMemoryView_GET_BASE(op : *mut PyObject) -> *mut PyObject {
    (*(op as *mut PyMemoryViewObject)).view.obj
}


#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyMemoryView_GetContiguous(base: *mut PyObject,
                                      buffertype: c_int,
                                      fort: c_char) -> *mut PyObject;
    pub fn PyMemoryView_FromObject(base: *mut PyObject) -> *mut PyObject;
    pub fn PyMemoryView_FromBuffer(info: *mut Py_buffer) -> *mut PyObject;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyMemoryViewObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub base: *mut PyObject,
    pub view: Py_buffer,
}

