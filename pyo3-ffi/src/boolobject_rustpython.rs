use crate::object::*;
use crate::rustpython_runtime;
use std::ffi::{c_int, c_long};

#[inline]
pub unsafe fn PyBool_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &raw mut PyBool_Type) as c_int
}

#[inline]
pub unsafe fn Py_False() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.false_value.clone().into()))
}

#[inline]
pub unsafe fn Py_True() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.true_value.clone().into()))
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
pub unsafe fn PyBool_FromLong(arg1: c_long) -> *mut PyObject {
    if arg1 == 0 { Py_False() } else { Py_True() }
}
