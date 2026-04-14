use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use libc::size_t;
use rustpython_vm::TryFromBorrowedObject;
use std::ffi::{c_char, c_double, c_int, c_long, c_longlong, c_ulong, c_ulonglong, c_void};

opaque_struct!(pub PyLongObject);

#[inline]
pub unsafe fn PyLong_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_LONG_SUBCLASS)
}

#[inline]
pub unsafe fn PyLong_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    ptr_to_pyobject_ref_borrowed(op).downcast_ref::<rustpython_vm::builtins::PyInt>().is_some().into()
}

#[inline]
pub unsafe fn PyLong_FromLong(arg1: c_long) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(arg1).into()))
}

#[inline]
pub unsafe fn PyLong_FromUnsignedLong(arg1: c_ulong) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(arg1).into()))
}

#[inline]
pub unsafe fn PyLong_FromSize_t(arg1: size_t) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(arg1).into()))
}

#[inline]
pub unsafe fn PyLong_FromSsize_t(arg1: Py_ssize_t) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(arg1).into()))
}

#[inline]
pub unsafe fn PyLong_FromDouble(arg1: c_double) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(arg1 as i64).into()))
}

#[inline]
pub unsafe fn PyLong_AsLong(arg1: *mut PyObject) -> c_long {
    if arg1.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(arg1);
    rustpython_runtime::with_vm(|vm| match c_long::try_from_borrowed_object(vm, &obj) {
        Ok(value) => value,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[inline]
pub unsafe fn PyLong_AsLongAndOverflow(arg1: *mut PyObject, arg2: *mut c_int) -> c_long {
    if !arg2.is_null() {
        *arg2 = 0;
    }
    PyLong_AsLong(arg1)
}

#[inline]
pub unsafe fn PyLong_AsSsize_t(arg1: *mut PyObject) -> Py_ssize_t {
    PyLong_AsLongLong(arg1) as Py_ssize_t
}

#[inline]
pub unsafe fn PyLong_AsSize_t(arg1: *mut PyObject) -> size_t {
    PyLong_AsUnsignedLongLong(arg1) as size_t
}

#[inline]
pub unsafe fn PyLong_AsUnsignedLong(arg1: *mut PyObject) -> c_ulong {
    PyLong_AsUnsignedLongLong(arg1) as c_ulong
}

#[inline]
pub unsafe fn PyLong_AsUnsignedLongMask(arg1: *mut PyObject) -> c_ulong {
    PyLong_AsUnsignedLong(arg1)
}

#[inline]
pub unsafe fn PyLong_GetInfo() -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyLong_AsDouble(arg1: *mut PyObject) -> c_double {
    PyLong_AsLongLong(arg1) as c_double
}

#[inline]
pub unsafe fn PyLong_FromVoidPtr(arg1: *mut c_void) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(arg1 as usize).into()))
}

#[inline]
pub unsafe fn PyLong_AsVoidPtr(arg1: *mut PyObject) -> *mut c_void {
    PyLong_AsUnsignedLongLong(arg1) as usize as *mut c_void
}

#[inline]
pub unsafe fn PyLong_FromLongLong(arg1: c_longlong) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(arg1).into()))
}

#[inline]
pub unsafe fn PyLong_FromUnsignedLongLong(arg1: c_ulonglong) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(arg1).into()))
}

#[inline]
pub unsafe fn PyLong_AsLongLong(arg1: *mut PyObject) -> c_longlong {
    if arg1.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(arg1);
    rustpython_runtime::with_vm(|vm| match c_longlong::try_from_borrowed_object(vm, &obj) {
        Ok(value) => value,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    })
}

#[inline]
pub unsafe fn PyLong_AsUnsignedLongLong(arg1: *mut PyObject) -> c_ulonglong {
    if arg1.is_null() {
        return u64::MAX;
    }
    let obj = ptr_to_pyobject_ref_borrowed(arg1);
    rustpython_runtime::with_vm(|vm| {
        if let Some(int) = obj.downcast_ref::<rustpython_vm::builtins::PyInt>() {
            if int.as_bigint().to_string().starts_with('-') {
                set_vm_exception(vm.new_overflow_error("can't convert negative int to unsigned"));
                return u64::MAX;
            }
        }

        match c_ulonglong::try_from_borrowed_object(vm, &obj) {
            Ok(value) => value,
            Err(exc) => {
                set_vm_exception(exc);
                u64::MAX
            }
        }
    })
}

#[inline]
pub unsafe fn PyLong_AsUnsignedLongLongMask(arg1: *mut PyObject) -> c_ulonglong {
    PyLong_AsUnsignedLongLong(arg1)
}

#[inline]
pub unsafe fn PyLong_AsLongLongAndOverflow(arg1: *mut PyObject, arg2: *mut c_int) -> c_longlong {
    if !arg2.is_null() {
        *arg2 = 0;
    }
    PyLong_AsLongLong(arg1)
}

#[inline]
pub unsafe fn PyLong_FromString(
    arg1: *const c_char,
    arg2: *mut *mut c_char,
    arg3: c_int,
) -> *mut PyObject {
    if arg1.is_null() {
        return std::ptr::null_mut();
    }
    if !arg2.is_null() {
        *arg2 = arg1.cast_mut();
    }
    let s = std::ffi::CStr::from_ptr(arg1).to_string_lossy();
    let radix = if arg3 == 0 { 10 } else { arg3 as u32 };
    match i128::from_str_radix(&s, radix) {
        Ok(v) => rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(v).into())),
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg(not(Py_LIMITED_API))]
#[inline]
pub unsafe fn _PyLong_NumBits(obj: *mut PyObject) -> size_t {
    if obj.is_null() {
        return 0;
    }
    let value = PyLong_AsUnsignedLongLong(obj);
    (u64::BITS - value.leading_zeros()) as size_t
}

#[cfg(not(Py_LIMITED_API))]
#[inline]
pub unsafe fn _PyLong_AsByteArray(
    obj: *mut PyLongObject,
    bytes: *mut u8,
    n: size_t,
    little_endian: c_int,
    is_signed: c_int,
) -> c_int {
    if obj.is_null() || bytes.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(obj.cast());
    let out = std::slice::from_raw_parts_mut(bytes, n);
    out.fill(if is_signed != 0 { 0xff } else { 0x00 });
    if is_signed != 0 {
        let value = match rustpython_runtime::with_vm(|vm| i128::try_from_borrowed_object(vm, &obj)) {
            Ok(value) => value,
            Err(exc) => {
                set_vm_exception(exc);
                return -1;
            }
        };
        let full = if little_endian != 0 { value.to_le_bytes() } else { value.to_be_bytes() };
        let count = n.min(full.len());
        if little_endian != 0 {
            out[..count].copy_from_slice(&full[..count]);
        } else {
            out[n - count..].copy_from_slice(&full[full.len() - count..]);
        }
    } else {
        let value = match rustpython_runtime::with_vm(|vm| u128::try_from_borrowed_object(vm, &obj)) {
            Ok(value) => value,
            Err(exc) => {
                set_vm_exception(exc);
                return -1;
            }
        };
        let full = if little_endian != 0 { value.to_le_bytes() } else { value.to_be_bytes() };
        let count = n.min(full.len());
        if little_endian != 0 {
            out[..count].copy_from_slice(&full[..count]);
        } else {
            out[n - count..].copy_from_slice(&full[full.len() - count..]);
        }
    }
    0
}

#[cfg(not(Py_LIMITED_API))]
#[inline]
pub unsafe fn _PyLong_FromByteArray(
    bytes: *const u8,
    n: size_t,
    little_endian: c_int,
    is_signed: c_int,
) -> *mut PyObject {
    if bytes.is_null() {
        return std::ptr::null_mut();
    }
    let src = std::slice::from_raw_parts(bytes, n);
    let mut buf = [0u8; 16];
    let count = src.len().min(buf.len());
    if little_endian != 0 {
        buf[..count].copy_from_slice(&src[..count]);
        if is_signed != 0 {
            let value = i128::from_le_bytes(buf);
            return rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(value).into()));
        }
        let value = u128::from_le_bytes(buf);
        return rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(value).into()));
    }
    let start = buf.len() - count;
    buf[start..].copy_from_slice(&src[src.len() - count..]);
    if is_signed != 0 {
        let value = i128::from_be_bytes(buf);
        rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(value).into()))
    } else {
        let value = u128::from_be_bytes(buf);
        rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_int(value).into()))
    }
}

unsafe extern "C" {
    pub fn PyOS_strtoul(arg1: *const c_char, arg2: *mut *mut c_char, arg3: c_int) -> c_ulong;
    pub fn PyOS_strtol(arg1: *const c_char, arg2: *mut *mut c_char, arg3: c_int) -> c_long;
}
