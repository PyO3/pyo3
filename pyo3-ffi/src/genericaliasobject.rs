use crate::object::{PyObject, PyTypeObject};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg(Py_3_9)]
    pub fn Py_GenericAlias(origin: *mut PyObject, args: *mut PyObject) -> *mut PyObject;

    #[cfg(Py_3_9)]
    pub static mut Py_GenericAliasType: PyTypeObject;
}
