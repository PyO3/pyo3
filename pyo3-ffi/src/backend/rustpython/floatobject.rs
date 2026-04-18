use crate::object::*;
use crate::rustpython_runtime;
use rustpython_vm::TryFromObject;
use std::ffi::c_double;
use std::ffi::c_int;

pub static mut PyFloat_Type: PyTypeObject = PyTypeObject { _opaque: [] };

opaque_struct!(pub PyFloatObject);

#[inline]
pub unsafe fn PyFloat_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &raw mut PyFloat_Type)
}

#[inline]
pub unsafe fn PyFloat_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    ptr_to_pyobject_ref_borrowed(op)
        .downcast_ref::<rustpython_vm::builtins::PyFloat>()
        .is_some()
        .into()
}

#[inline]
pub unsafe fn PyFloat_GetMax() -> c_double {
    f64::MAX
}

#[inline]
pub unsafe fn PyFloat_GetMin() -> c_double {
    f64::MIN_POSITIVE
}

#[inline]
pub unsafe fn PyFloat_GetInfo() -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyFloat_FromString(arg1: *mut PyObject) -> *mut PyObject {
    if arg1.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(arg1);
    rustpython_runtime::with_vm(|vm| {
        obj.str(vm)
            .ok()
            .and_then(|s| AsRef::<str>::as_ref(&s).parse::<f64>().ok())
            .map(|v| pyobject_ref_to_ptr(vm.ctx.new_float(v).into()))
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PyFloat_FromDouble(arg1: c_double) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_float(arg1).into()))
}

#[inline]
pub unsafe fn PyFloat_AsDouble(arg1: *mut PyObject) -> c_double {
    if arg1.is_null() {
        return -1.0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(arg1);
    rustpython_runtime::with_vm(|vm| f64::try_from_object(vm, obj).unwrap_or(-1.0))
}

#[inline]
pub unsafe fn PyFloat_AS_DOUBLE(arg1: *mut PyObject) -> c_double {
    PyFloat_AsDouble(arg1)
}
