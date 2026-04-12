use crate::object::*;
use crate::pyport::Py_ssize_t;
use crate::pyerrors::{PyErr_SetString, PyExc_TypeError};
use crate::rustpython_runtime;
use rustpython_vm::builtins::PyTuple;
use rustpython_vm::AsObject;
use std::ffi::c_int;

#[inline]
pub unsafe fn PyTuple_Check(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| obj.class().fast_issubclass(vm.ctx.types.tuple_type.as_object()) as c_int)
}

#[inline]
pub unsafe fn PyTuple_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| obj.class().is(vm.ctx.types.tuple_type) as c_int)
}

#[inline]
pub unsafe fn PyTuple_New(size: Py_ssize_t) -> *mut PyObject {
    if size < 0 {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let items = vec![vm.ctx.none(); size as usize];
        pyobject_ref_to_ptr(vm.ctx.new_tuple(items).into())
    })
}

#[inline]
pub unsafe fn PyTuple_Size(arg1: *mut PyObject) -> Py_ssize_t {
    if arg1.is_null() {
        return -1;
    }
    let tuple = ptr_to_pyobject_ref_borrowed(arg1);
    match tuple.downcast_ref::<PyTuple>() {
        Some(t) => t.len() as Py_ssize_t,
        None => -1,
    }
}

#[inline]
pub unsafe fn PyTuple_GetItem(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject {
    if arg1.is_null() || arg2 < 0 {
        return std::ptr::null_mut();
    }
    let tuple = ptr_to_pyobject_ref_borrowed(arg1);
    let Some(inner) = tuple.downcast_ref::<PyTuple>() else {
        return std::ptr::null_mut();
    };
    inner
        .as_slice()
        .get(arg2 as usize)
        .map(pyobject_ref_as_ptr)
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyTuple_SetItem(arg1: *mut PyObject, _arg2: Py_ssize_t, _arg3: *mut PyObject) -> c_int {
    PyErr_SetString(
        PyExc_TypeError,
        c"PyTuple_SetItem is not supported on RustPython tuple objects; callers must use backend-safe tuple construction".as_ptr(),
    );
    let _ = arg1;
    -1
}

#[inline]
pub unsafe fn PyTuple_GetSlice(
    arg1: *mut PyObject,
    arg2: Py_ssize_t,
    arg3: Py_ssize_t,
) -> *mut PyObject {
    if arg1.is_null() {
        return std::ptr::null_mut();
    }
    let tuple = ptr_to_pyobject_ref_borrowed(arg1);
    let Some(inner) = tuple.downcast_ref::<PyTuple>() else {
        return std::ptr::null_mut();
    };
    let len = inner.len() as Py_ssize_t;
    let low = arg2.clamp(0, len) as usize;
    let high = arg3.clamp(arg2.max(0), len) as usize;
    rustpython_runtime::with_vm(|vm| {
        pyobject_ref_to_ptr(vm.ctx.new_tuple(inner.as_slice()[low..high].to_vec()).into())
    })
}

#[cfg(not(Py_3_9))]
#[inline]
pub unsafe fn PyTuple_ClearFreeList() -> c_int {
    0
}
