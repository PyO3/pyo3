use crate::object::*;
use crate::rustpython_runtime;
use rustpython_vm::builtins::PyBool;
use rustpython_vm::AsObject;
use std::ffi::{c_int, c_long};

#[inline]
pub unsafe fn PyBool_Check(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    ptr_to_pyobject_ref_borrowed(op)
        .downcast_ref::<PyBool>()
        .is_some()
        .into()
}

#[inline]
pub unsafe fn Py_False() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let value = vm.ctx.false_value.clone().into();
        pyobject_ref_as_ptr(&value)
    })
}

#[inline]
pub unsafe fn Py_True() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let value = vm.ctx.true_value.clone().into();
        pyobject_ref_as_ptr(&value)
    })
}

#[inline]
pub unsafe fn Py_IsTrue(x: *mut PyObject) -> c_int {
    if x.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(x);
    rustpython_runtime::with_vm(|vm| obj.is(vm.ctx.true_value.as_object()).into())
}

#[inline]
pub unsafe fn Py_IsFalse(x: *mut PyObject) -> c_int {
    if x.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(x);
    rustpython_runtime::with_vm(|vm| obj.is(vm.ctx.false_value.as_object()).into())
}

#[inline]
pub unsafe fn PyBool_FromLong(arg1: c_long) -> *mut PyObject {
    if arg1 == 0 {
        Py_False()
    } else {
        Py_True()
    }
}
