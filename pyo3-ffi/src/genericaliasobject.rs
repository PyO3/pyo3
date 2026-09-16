#[cfg(Py_3_9)]
use crate::PyObject;
#[cfg(all(Py_3_9, not(RustPython)))]
use crate::PyTypeObject;

extern_libpython! {
    #[cfg(Py_3_9)]
    #[cfg_attr(all(PyPy, not(Py_3_12)), link_name = "PyPy_GenericAlias")]
    pub fn Py_GenericAlias(origin: *mut PyObject, args: *mut PyObject) -> *mut PyObject;

    #[cfg(all(Py_3_9, not(RustPython)))]
    #[cfg_attr(all(PyPy, not(Py_3_12)), link_name = "PyPy_GenericAliasType")]
    pub static mut Py_GenericAliasType: PyTypeObject;
}
