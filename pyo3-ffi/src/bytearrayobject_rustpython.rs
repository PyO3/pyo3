use crate::object::*;
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use rustpython_vm::builtins::PyByteArray;
use rustpython_vm::AsObject;
use std::ffi::{c_char, c_int};

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
pub struct PyByteArrayObject {
    pub ob_base: PyVarObject,
    pub ob_alloc: Py_ssize_t,
    pub ob_bytes: *mut c_char,
    pub ob_start: *mut c_char,
    pub ob_exports: c_int,
}

pub static mut PyByteArray_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyByteArrayIter_Type: PyTypeObject = PyTypeObject { _opaque: [] };

#[inline]
pub unsafe fn PyByteArray_Check(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    ptr_to_pyobject_ref_borrowed(op)
        .downcast_ref::<PyByteArray>()
        .is_some()
        .into()
}

#[inline]
pub unsafe fn PyByteArray_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| {
        obj.downcast_ref::<PyByteArray>()
            .is_some_and(|_| obj.class().is(vm.ctx.types.bytearray_type))
            .into()
    })
}

#[inline]
pub unsafe fn PyByteArray_FromObject(o: *mut PyObject) -> *mut PyObject {
    if o.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let obj = ptr_to_pyobject_ref_borrowed(o);
        match vm.invoke(vm.ctx.types.bytearray_type.as_object(), (obj,)) {
            Ok(bytearray) => pyobject_ref_to_ptr(bytearray),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

#[inline]
pub unsafe fn PyByteArray_Concat(a: *mut PyObject, b: *mut PyObject) -> *mut PyObject {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        let a = ptr_to_pyobject_ref_borrowed(a);
        let b = ptr_to_pyobject_ref_borrowed(b);
        match vm._add(&a, &b) {
            Ok(bytearray) => pyobject_ref_to_ptr(bytearray),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

#[inline]
pub unsafe fn PyByteArray_FromStringAndSize(
    string: *const c_char,
    len: Py_ssize_t,
) -> *mut PyObject {
    let len = len.max(0) as usize;
    rustpython_runtime::with_vm(|vm| {
        let data = if string.is_null() {
            vec![0; len]
        } else {
            std::slice::from_raw_parts(string.cast::<u8>(), len).to_vec()
        };
        pyobject_ref_to_ptr(vm.ctx.new_bytearray(data).into())
    })
}

#[inline]
pub unsafe fn PyByteArray_Size(bytearray: *mut PyObject) -> Py_ssize_t {
    if bytearray.is_null() {
        return -1;
    }
    ptr_to_pyobject_ref_borrowed(bytearray)
        .downcast_ref::<PyByteArray>()
        .map_or(-1, |bytearray| bytearray.borrow_buf().len() as Py_ssize_t)
}

#[inline]
pub unsafe fn PyByteArray_AsString(bytearray: *mut PyObject) -> *mut c_char {
    if bytearray.is_null() {
        return std::ptr::null_mut();
    }
    ptr_to_pyobject_ref_borrowed(bytearray)
        .downcast_ref::<PyByteArray>()
        .map_or(std::ptr::null_mut(), |bytearray| {
            bytearray.borrow_buf().as_ptr().cast_mut().cast()
        })
}

#[inline]
pub unsafe fn PyByteArray_Resize(bytearray: *mut PyObject, len: Py_ssize_t) -> c_int {
    if bytearray.is_null() || len < 0 {
        return -1;
    }
    let objref = ptr_to_pyobject_ref_borrowed(bytearray);
    let Some(bytearray) = objref.downcast_ref::<PyByteArray>() else {
        return -1;
    };
    bytearray.borrow_buf_mut().resize(len as usize, 0);
    0
}
