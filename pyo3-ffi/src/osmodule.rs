use crate::object::PyObject;

extern_libpython! {
    #[cfg_attr(all(PyPy, not(Py_3_12)), link_name = "PyPyOS_FSPath")]
    pub fn PyOS_FSPath(path: *mut PyObject) -> *mut PyObject;
}
