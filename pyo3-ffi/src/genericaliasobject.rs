#[cfg(Py_3_9)]
use crate::object::{PyObject, PyTypeObject};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg(Py_3_9)]
    #[cfg_attr(PyPy, link_name = "PyPy_GenericAlias")]
    pub fn Py_GenericAlias(origin: *mut PyObject, args: *mut PyObject) -> *mut PyObject;

    #[cfg(Py_3_9)]
    pub static mut Py_GenericAliasType: PyTypeObject;
}
