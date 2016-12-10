// This header is new in Python 3.6
use object::PyObject;

extern "C" {
    pub fn PyOS_FSPath(path: *mut PyObject) -> *mut PyObject;
}
