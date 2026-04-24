#[cfg(Py_3_9)]
use crate::PyObject;
#[cfg(not(RustPython))]
use crate::PyTypeObject;

extern_libpython! {
    #[cfg(Py_3_9)]
    #[cfg_attr(PyPy, link_name = "PyPy_GenericAlias")]
    pub fn Py_GenericAlias(origin: *mut PyObject, args: *mut PyObject) -> *mut PyObject;

    #[cfg(all(Py_3_9, not(RustPython)))]
    pub static mut Py_GenericAliasType: PyTypeObject;
}
