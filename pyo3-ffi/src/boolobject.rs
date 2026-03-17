#[cfg(not(GraalPy))]
use crate::longobject::PyLongObject;
use crate::object::*;
use std::ffi::{c_int, c_long};

#[inline]
pub unsafe fn PyBool_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &raw mut PyBool_Type) as c_int
}

extern_libpython! {
    #[cfg(not(GraalPy))]
    #[cfg_attr(PyPy, link_name = "_PyPy_FalseStruct")]
    static mut _Py_FalseStruct: PyLongObject;
    #[cfg(not(GraalPy))]
    #[cfg_attr(PyPy, link_name = "_PyPy_TrueStruct")]
    static mut _Py_TrueStruct: PyLongObject;

    #[cfg(GraalPy)]
    static mut _Py_FalseStructReference: *mut PyObject;
    #[cfg(GraalPy)]
    static mut _Py_TrueStructReference: *mut PyObject;
}

#[inline]
pub unsafe fn Py_False() -> *mut PyObject {
    #[cfg(not(GraalPy))]
    return (&raw mut _Py_FalseStruct).cast();
    #[cfg(GraalPy)]
    return _Py_FalseStructReference;
}

#[inline]
pub unsafe fn Py_True() -> *mut PyObject {
    #[cfg(not(GraalPy))]
    return (&raw mut _Py_TrueStruct).cast();
    #[cfg(GraalPy)]
    return _Py_TrueStructReference;
}

#[inline]
pub unsafe fn Py_IsTrue(x: *mut PyObject) -> c_int {
    Py_Is(x, Py_True())
}

#[inline]
pub unsafe fn Py_IsFalse(x: *mut PyObject) -> c_int {
    Py_Is(x, Py_False())
}

// skipped Py_RETURN_TRUE
// skipped Py_RETURN_FALSE

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyBool_FromLong")]
    pub fn PyBool_FromLong(arg1: c_long) -> *mut PyObject;
}
