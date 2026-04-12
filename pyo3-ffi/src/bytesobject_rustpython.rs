use crate::object::*;
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use rustpython_vm::builtins::PyBytes;
use rustpython_vm::AsObject;
use std::ffi::{c_char, c_int, CStr};

pub static mut PyBytes_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyBytesIter_Type: PyTypeObject = PyTypeObject { _opaque: [] };

#[inline]
pub unsafe fn PyBytes_Check(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    ptr_to_pyobject_ref_borrowed(op)
        .downcast_ref::<PyBytes>()
        .is_some()
        .into()
}

#[inline]
pub unsafe fn PyBytes_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| {
        obj.downcast_ref::<PyBytes>()
            .is_some_and(|_| obj.class().is(vm.ctx.types.bytes_type))
            .into()
    })
}

#[inline]
pub unsafe fn PyBytes_FromStringAndSize(arg1: *const c_char, arg2: Py_ssize_t) -> *mut PyObject {
    let len = arg2.max(0) as usize;
    rustpython_runtime::with_vm(|vm| {
        let data = if arg1.is_null() {
            vec![0; len]
        } else {
            std::slice::from_raw_parts(arg1.cast::<u8>(), len).to_vec()
        };
        pyobject_ref_to_ptr(vm.ctx.new_bytes(data).into())
    })
}

#[inline]
pub unsafe fn PyBytes_FromString(arg1: *const c_char) -> *mut PyObject {
    if arg1.is_null() {
        return PyBytes_FromStringAndSize(std::ptr::null(), 0);
    }
    let s = CStr::from_ptr(arg1);
    PyBytes_FromStringAndSize(s.as_ptr(), s.to_bytes().len() as Py_ssize_t)
}

#[inline]
pub unsafe fn PyBytes_FromObject(arg1: *mut PyObject) -> *mut PyObject {
    if arg1.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let obj = ptr_to_pyobject_ref_borrowed(arg1);
        match vm.invoke(vm.ctx.types.bytes_type.as_object(), (obj,)) {
            Ok(bytes) => pyobject_ref_to_ptr(bytes),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

#[inline]
pub unsafe fn PyBytes_Size(arg1: *mut PyObject) -> Py_ssize_t {
    if arg1.is_null() {
        return -1;
    }
    ptr_to_pyobject_ref_borrowed(arg1)
        .downcast_ref::<PyBytes>()
        .map_or(-1, |bytes| bytes.as_bytes().len() as Py_ssize_t)
}

#[inline]
pub unsafe fn PyBytes_AsString(arg1: *mut PyObject) -> *mut c_char {
    if arg1.is_null() {
        return std::ptr::null_mut();
    }
    ptr_to_pyobject_ref_borrowed(arg1)
        .downcast_ref::<PyBytes>()
        .map_or(std::ptr::null_mut(), |bytes| {
            bytes.as_bytes().as_ptr().cast_mut().cast()
        })
}

#[inline]
pub unsafe fn PyBytes_AsStringAndSize(
    obj: *mut PyObject,
    s: *mut *mut c_char,
    len: *mut Py_ssize_t,
) -> c_int {
    if obj.is_null() {
        return -1;
    }
    let objref = ptr_to_pyobject_ref_borrowed(obj);
    let Some(bytes) = objref.downcast_ref::<PyBytes>() else {
        return -1;
    };
    if !s.is_null() {
        *s = bytes.as_bytes().as_ptr().cast_mut().cast();
    }
    if !len.is_null() {
        *len = bytes.as_bytes().len() as Py_ssize_t;
    }
    0
}
