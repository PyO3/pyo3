use crate::object::{PyObject, PyTypeObject};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub fn Py_GenericAlias(origin: *mut PyObject, args: *mut PyObject) -> *mut PyObject;

    pub static mut Py_GenericAliasType: PyTypeObject;
}
