use crate::object::*;
use crate::pyport::Py_ssize_t;
#[cfg(PyRustPython)]
use crate::rustpython_runtime;
use libc::size_t;
#[cfg(PyRustPython)]
use rustpython_vm::TryFromBorrowedObject;
use std::ffi::{c_char, c_double, c_int, c_long, c_longlong, c_ulong, c_ulonglong, c_void};

opaque_struct!(pub PyLongObject);

#[inline]
pub unsafe fn PyLong_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_LONG_SUBCLASS)
}

#[inline]
pub unsafe fn PyLong_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &raw mut PyLong_Type) as c_int
}

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromLong")]
    pub fn PyLong_FromLong(arg1: c_long) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromUnsignedLong")]
    pub fn PyLong_FromUnsignedLong(arg1: c_ulong) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromSize_t")]
    pub fn PyLong_FromSize_t(arg1: size_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromSsize_t")]
    pub fn PyLong_FromSsize_t(arg1: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromDouble")]
    pub fn PyLong_FromDouble(arg1: c_double) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLong")]
    pub fn PyLong_AsLong(arg1: *mut PyObject) -> c_long;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLongAndOverflow")]
    pub fn PyLong_AsLongAndOverflow(arg1: *mut PyObject, arg2: *mut c_int) -> c_long;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsSsize_t")]
    pub fn PyLong_AsSsize_t(arg1: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsSize_t")]
    pub fn PyLong_AsSize_t(arg1: *mut PyObject) -> size_t;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLong")]
    pub fn PyLong_AsUnsignedLong(arg1: *mut PyObject) -> c_ulong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLongMask")]
    pub fn PyLong_AsUnsignedLongMask(arg1: *mut PyObject) -> c_ulong;
    // skipped non-limited _PyLong_AsInt
    pub fn PyLong_GetInfo() -> *mut PyObject;
    // skipped PyLong_AS_LONG

    // skipped PyLong_FromPid
    // skipped PyLong_AsPid
    // skipped _Py_PARSE_INTPTR
    // skipped _Py_PARSE_UINTPTR

    // skipped non-limited _PyLong_UnsignedShort_Converter
    // skipped non-limited _PyLong_UnsignedInt_Converter
    // skipped non-limited _PyLong_UnsignedLong_Converter
    // skipped non-limited _PyLong_UnsignedLongLong_Converter
    // skipped non-limited _PyLong_Size_t_Converter

    // skipped non-limited _PyLong_DigitValue
    // skipped non-limited _PyLong_Frexp

    #[cfg_attr(PyPy, link_name = "PyPyLong_AsDouble")]
    pub fn PyLong_AsDouble(arg1: *mut PyObject) -> c_double;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromVoidPtr")]
    pub fn PyLong_FromVoidPtr(arg1: *mut c_void) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsVoidPtr")]
    pub fn PyLong_AsVoidPtr(arg1: *mut PyObject) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromLongLong")]
    pub fn PyLong_FromLongLong(arg1: c_longlong) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromUnsignedLongLong")]
    pub fn PyLong_FromUnsignedLongLong(arg1: c_ulonglong) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLongLong")]
    pub fn PyLong_AsLongLong(arg1: *mut PyObject) -> c_longlong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLongLong")]
    pub fn PyLong_AsUnsignedLongLong(arg1: *mut PyObject) -> c_ulonglong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLongLongMask")]
    pub fn PyLong_AsUnsignedLongLongMask(arg1: *mut PyObject) -> c_ulonglong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLongLongAndOverflow")]
    pub fn PyLong_AsLongLongAndOverflow(arg1: *mut PyObject, arg2: *mut c_int) -> c_longlong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromString")]
    pub fn PyLong_FromString(
        arg1: *const c_char,
        arg2: *mut *mut c_char,
        arg3: c_int,
    ) -> *mut PyObject;
}

#[cfg(not(Py_LIMITED_API))]
extern_libpython! {
    #[cfg_attr(PyPy, link_name = "_PyPyLong_NumBits")]
    #[cfg(not(Py_3_13))]
    #[doc(hidden)]
    pub fn _PyLong_NumBits(obj: *mut PyObject) -> size_t;
}

#[cfg(all(not(Py_LIMITED_API), PyRustPython))]
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
        let value = rustpython_runtime::with_vm(|vm| {
            i128::try_from_borrowed_object(vm, &obj).map_err(|_| ())
        });
        let Ok(value) = value else {
            return -1;
        };
        let full = if little_endian != 0 {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        if n > full.len() {
            let fill = if value < 0 { 0xff } else { 0x00 };
            out.fill(fill);
        }
        let count = n.min(full.len());
        if little_endian != 0 {
            out[..count].copy_from_slice(&full[..count]);
        } else {
            out[n - count..].copy_from_slice(&full[full.len() - count..]);
        }
        0
    } else {
        let value = rustpython_runtime::with_vm(|vm| {
            u128::try_from_borrowed_object(vm, &obj).map_err(|_| ())
        });
        let Ok(value) = value else {
            return -1;
        };
        let full = if little_endian != 0 {
            value.to_le_bytes()
        } else {
            value.to_be_bytes()
        };
        let count = n.min(full.len());
        if little_endian != 0 {
            out[..count].copy_from_slice(&full[..count]);
        } else {
            out[n - count..].copy_from_slice(&full[full.len() - count..]);
        }
        0
    }
}

#[cfg(all(not(Py_LIMITED_API), PyRustPython))]
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
    let mut buf = [if is_signed != 0 && little_endian != 0 && src.last().copied().unwrap_or(0) & 0x80 != 0 {
        0xff
    } else {
        0x00
    }; 16];
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

    if is_signed != 0 && src.first().copied().unwrap_or(0) & 0x80 != 0 {
        buf.fill(0xff);
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

// skipped non-limited _PyLong_Format
// skipped non-limited _PyLong_FormatWriter
// skipped non-limited _PyLong_FormatBytesWriter
// skipped non-limited _PyLong_FormatAdvancedWriter

extern_libpython! {
    pub fn PyOS_strtoul(arg1: *const c_char, arg2: *mut *mut c_char, arg3: c_int) -> c_ulong;
    pub fn PyOS_strtol(arg1: *const c_char, arg2: *mut *mut c_char, arg3: c_int) -> c_long;
}

// skipped non-limited _PyLong_Rshift
// skipped non-limited _PyLong_Lshift
