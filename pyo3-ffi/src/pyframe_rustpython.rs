use crate::PyFrameObject;
use std::ffi::c_int;

#[inline]
pub unsafe fn PyFrame_GetLineNumber(frame: *mut PyFrameObject) -> c_int {
    if frame.is_null() {
        return 0;
    }
    let frame = crate::object::ptr_to_pyobject_ref_borrowed(frame.cast());
    frame
        .downcast_ref::<rustpython_vm::frame::Frame>()
        .map(|f| f.f_lineno() as c_int)
        .unwrap_or(0)
}

#[cfg(not(GraalPy))]
#[cfg(any(Py_3_10, all(Py_3_9, not(Py_LIMITED_API))))]
#[inline]
pub unsafe fn PyFrame_GetCode(_frame: *mut PyFrameObject) -> *mut crate::PyCodeObject {
    std::ptr::null_mut()
}
