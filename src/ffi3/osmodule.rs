// This header is new in Python 3.6
use ffi3::object::PyObject;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyOS_FSPath(path: *mut PyObject) -> *mut PyObject;
}
