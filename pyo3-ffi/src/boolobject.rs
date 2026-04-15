#[cfg(all(not(GraalPy), not(all(Py_3_13, Py_LIMITED_API))))]
use crate::longobject::PyLongObject;
use crate::object::*;
use std::ffi::{c_int, c_long};

#[inline]
pub unsafe fn PyBool_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &raw mut PyBool_Type) as c_int
}

extern_libpython! {
    #[cfg(all(not(GraalPy), not(all(Py_3_13, Py_LIMITED_API))))]
    #[cfg_attr(PyPy, link_name = "_PyPy_FalseStruct")]
    static mut _Py_FalseStruct: PyLongObject;
    #[cfg(all(not(GraalPy), not(all(Py_3_13, Py_LIMITED_API))))]
    #[cfg_attr(PyPy, link_name = "_PyPy_TrueStruct")]
    static mut _Py_TrueStruct: PyLongObject;

    #[cfg(GraalPy)]
    static mut _Py_FalseStructReference: *mut PyObject;
    #[cfg(GraalPy)]
    static mut _Py_TrueStructReference: *mut PyObject;
}

#[inline]
pub unsafe fn Py_False() -> *mut PyObject {
    #[cfg(all(not(GraalPy), all(Py_3_13, Py_LIMITED_API)))]
    return Py_GetConstantBorrowed(Py_CONSTANT_FALSE);

    #[cfg(all(not(GraalPy), not(all(Py_3_13, Py_LIMITED_API))))]
    return (&raw mut _Py_FalseStruct).cast();

    #[cfg(GraalPy)]
    return _Py_FalseStructReference;
}

#[inline]
pub unsafe fn Py_True() -> *mut PyObject {
    #[cfg(all(not(GraalPy), all(Py_3_13, Py_LIMITED_API)))]
    return Py_GetConstantBorrowed(Py_CONSTANT_TRUE);

    #[cfg(all(not(GraalPy), not(all(Py_3_13, Py_LIMITED_API))))]
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

#[inline]
#[cfg(all(Py_LIMITED_API, not(Py_3_12)))]
pub unsafe fn Py_RETURN_TRUE() -> *mut PyObject {
    Py_NewRef(Py_True())
}

#[inline]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn Py_RETURN_TRUE() -> *mut PyObject {
    Py_True()
}

#[inline]
#[cfg(all(Py_LIMITED_API, not(Py_3_12)))]
pub unsafe fn Py_RETURN_FALSE() -> *mut PyObject {
    Py_NewRef(Py_False())
}

#[inline]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn Py_RETURN_FALSE() -> *mut PyObject {
    Py_False()
}

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyBool_FromLong")]
    pub fn PyBool_FromLong(arg1: c_long) -> *mut PyObject;
}
