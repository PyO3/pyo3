use crate::object::PyObject;

extern "C" {
    pub fn PyOS_FSPath(path: *mut PyObject) -> *mut PyObject;
}
