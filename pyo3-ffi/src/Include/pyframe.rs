extern_libpython! {
    pub fn PyFrame_GetLineNumber(frame: *mut PyFrameObject) -> c_int;

    #[cfg(not(GraalPy))]
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    pub fn PyFrame_GetCode(frame: *mut PyFrameObject) -> *mut PyCodeObject;
}

#[cfg(not(Py_LIMITED_API))]
include!("cpython/pyframe.rs");
