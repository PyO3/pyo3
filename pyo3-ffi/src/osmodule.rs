use crate::object::PyObject;

extern_python_dll! {
    #[cfg_attr(PyPy, link_name = "PyPyOS_FSPath")]
    pub fn PyOS_FSPath(path: *mut PyObject) -> *mut PyObject;
}
