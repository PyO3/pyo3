use crate::object::*;
use crate::pyport::Py_ssize_t;
use crate::pyerrors::{PyErr_Clear, PyErr_SetRaisedException};
use crate::rustpython_runtime;
use libc::wchar_t;
use rustpython_vm::builtins::PyStr;
use rustpython_vm::{AsObject, PyObjectRef};
use std::ffi::{c_char, c_int, CStr};

#[cfg_attr(
    Py_3_13,
    deprecated(note = "Deprecated since Python 3.13. Use `libc::wchar_t` instead.")
)]
pub type Py_UNICODE = wchar_t;

pub type Py_UCS4 = u32;
pub type Py_UCS2 = u16;
pub type Py_UCS1 = u8;

pub const Py_UNICODE_REPLACEMENT_CHARACTER: Py_UCS4 = 0xFFFD;

fn cstr_opt(ptr: *const c_char) -> Option<&'static CStr> {
    (!ptr.is_null()).then(|| unsafe { CStr::from_ptr(ptr) })
}

fn cstr_to_str(ptr: *const c_char) -> Option<&'static str> {
    cstr_opt(ptr)?.to_str().ok()
}

unsafe fn object_to_str(obj: *mut PyObject) -> Option<PyObjectRef> {
    (!obj.is_null()).then(|| ptr_to_pyobject_ref_borrowed(obj))
}

#[inline]
pub unsafe fn PyUnicode_Check(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| obj.class().fast_issubclass(vm.ctx.types.str_type.as_object()) as c_int)
}

#[inline]
pub unsafe fn PyUnicode_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| obj.class().is(vm.ctx.types.str_type) as c_int)
}

#[inline]
pub unsafe fn PyUnicode_FromStringAndSize(u: *const c_char, size: Py_ssize_t) -> *mut PyObject {
    if u.is_null() || size < 0 {
        return std::ptr::null_mut();
    }
    let bytes = std::slice::from_raw_parts(u.cast::<u8>(), size as usize);
    let vm = rustpython_runtime::current_vm()
        .expect("RustPython unicode API used outside an attached interpreter context");
    match std::str::from_utf8(bytes) {
        Ok(s) => pyobject_ref_to_ptr(vm.ctx.new_str(s).into()),
        Err(e) => {
            let start = e.valid_up_to();
            let end = start + e.error_len().unwrap_or(1);
            let exc = vm.new_unicode_decode_error_real(
                vm.ctx.new_str("utf-8"),
                vm.ctx.new_bytes(bytes.to_vec()),
                start,
                end,
                vm.ctx.new_str("invalid utf-8"),
            );
            PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
            std::ptr::null_mut()
        }
    }
}

#[inline]
pub unsafe fn PyUnicode_FromString(u: *const c_char) -> *mut PyObject {
    let Some(cstr) = cstr_opt(u) else {
        return std::ptr::null_mut();
    };
    match cstr.to_str() {
        Ok(s) => rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.new_str(s).into())),
        Err(_) => std::ptr::null_mut(),
    }
}

#[inline]
pub unsafe fn PyUnicode_FromEncodedObject(
    obj: *mut PyObject,
    encoding: *const c_char,
    errors: *const c_char,
) -> *mut PyObject {
    let Some(obj) = object_to_str(obj) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| {
        let result = match (cstr_to_str(encoding), cstr_to_str(errors)) {
            (None, None) => vm.call_method(&obj, "decode", ()),
            (Some(enc), None) => vm.call_method(&obj, "decode", (vm.ctx.new_str(enc),)),
            (None, Some(errs)) => vm.call_method(&obj, "decode", (vm.ctx.none(), vm.ctx.new_str(errs))),
            (Some(enc), Some(errs)) => {
                vm.call_method(&obj, "decode", (vm.ctx.new_str(enc), vm.ctx.new_str(errs)))
            }
        };
        match result {
            Ok(value) => pyobject_ref_to_ptr(value),
            Err(exc) => {
                PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
                std::ptr::null_mut()
            }
        }
    })
}

#[inline]
pub unsafe fn PyUnicode_InternInPlace(arg1: *mut *mut PyObject) {
    if arg1.is_null() || (*arg1).is_null() {
        return;
    }
    let obj = ptr_to_pyobject_ref_borrowed(*arg1);
    rustpython_runtime::with_vm(|vm| {
        let Ok(s) = obj.clone().downcast::<PyStr>() else {
            return;
        };
        let interned: PyObjectRef = vm
            .ctx
            .intern_str(AsRef::<str>::as_ref(&s))
            .to_owned()
            .into();
        let new_ptr = pyobject_ref_to_ptr(interned);
        Py_DECREF(*arg1);
        *arg1 = new_ptr;
    });
}

#[inline]
pub unsafe fn PyUnicode_InternFromString(u: *const c_char) -> *mut PyObject {
    let Some(s) = cstr_to_str(u) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| pyobject_ref_to_ptr(vm.ctx.intern_str(s).to_owned().into()))
}

#[inline]
pub unsafe fn PyUnicode_AsUTF8String(unicode: *mut PyObject) -> *mut PyObject {
    if unicode.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(unicode);
    rustpython_runtime::with_vm(|vm| match obj.str(vm) {
        Ok(s) => pyobject_ref_to_ptr(
            vm.ctx
                .new_bytes(AsRef::<str>::as_ref(&s).as_bytes().to_vec())
                .into(),
        ),
        Err(exc) => {
            PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
            std::ptr::null_mut()
        }
    })
}

#[inline]
pub unsafe fn PyUnicode_AsUTF8AndSize(unicode: *mut PyObject, size: *mut Py_ssize_t) -> *const c_char {
    if unicode.is_null() {
        return std::ptr::null();
    }
    let obj = ptr_to_pyobject_ref_borrowed(unicode);
    let vm = rustpython_runtime::current_vm()
        .expect("RustPython unicode API used outside an attached interpreter context");
    let Some(s) = obj.downcast_ref::<PyStr>() else {
        return std::ptr::null();
    };
    match s.try_as_utf8(vm) {
        Ok(utf8) => {
            if !size.is_null() {
                *size = utf8.as_str().len() as Py_ssize_t;
            }
            utf8.as_str().as_ptr().cast()
        }
        Err(exc) => {
            PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
            std::ptr::null()
        }
    }
}

#[inline]
pub unsafe fn PyUnicode_AsEncodedString(
    unicode: *mut PyObject,
    encoding: *const c_char,
    errors: *const c_char,
) -> *mut PyObject {
    if unicode.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(unicode);
    rustpython_runtime::with_vm(|vm| {
        let result = match (cstr_to_str(encoding), cstr_to_str(errors)) {
            (None, None) => vm.call_method(&obj, "encode", ()),
            (Some(enc), None) => vm.call_method(&obj, "encode", (vm.ctx.new_str(enc),)),
            (None, Some(errs)) => vm.call_method(&obj, "encode", (vm.ctx.none(), vm.ctx.new_str(errs))),
            (Some(enc), Some(errs)) => {
                vm.call_method(&obj, "encode", (vm.ctx.new_str(enc), vm.ctx.new_str(errs)))
            }
        };
        match result {
            Ok(value) => pyobject_ref_to_ptr(value),
            Err(exc) => {
                PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
                std::ptr::null_mut()
            }
        }
    })
}

#[inline]
pub unsafe fn PyUnicode_DecodeFSDefaultAndSize(
    s: *const c_char,
    size: Py_ssize_t,
) -> *mut PyObject {
    if s.is_null() || size < 0 {
        return std::ptr::null_mut();
    }
    let bytes = std::slice::from_raw_parts(s.cast::<u8>(), size as usize);
    let vm = rustpython_runtime::current_vm()
        .expect("RustPython unicode API used outside an attached interpreter context");
    match std::str::from_utf8(bytes) {
        Ok(text) => pyobject_ref_to_ptr(vm.ctx.new_str(text).into()),
        Err(e) => {
            let start = e.valid_up_to();
            let end = start + e.error_len().unwrap_or(1);
            let exc = vm.new_unicode_decode_error_real(
                vm.ctx.new_str("utf-8"),
                vm.ctx.new_bytes(bytes.to_vec()),
                start,
                end,
                vm.ctx.new_str("invalid utf-8"),
            );
            PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
            std::ptr::null_mut()
        }
    }
}

#[inline]
pub unsafe fn PyUnicode_EncodeFSDefault(unicode: *mut PyObject) -> *mut PyObject {
    if unicode.is_null() {
        return std::ptr::null_mut();
    }
    let obj = ptr_to_pyobject_ref_borrowed(unicode);
    rustpython_runtime::with_vm(|vm| match obj.str(vm) {
        Ok(s) => pyobject_ref_to_ptr(
            vm.ctx
                .new_bytes(AsRef::<str>::as_ref(&s).as_bytes().to_vec())
                .into(),
        ),
        Err(exc) => {
            PyErr_SetRaisedException(pyobject_ref_to_ptr(exc.into()));
            std::ptr::null_mut()
        }
    })
}

#[inline]
pub unsafe fn PyUnicode_ClearFreeList() -> c_int {
    0
}

#[inline]
pub unsafe fn PyUnicode_GetLength(unicode: *mut PyObject) -> Py_ssize_t {
    if unicode.is_null() {
        return -1;
    }
    let obj = ptr_to_pyobject_ref_borrowed(unicode);
    rustpython_runtime::with_vm(|vm| obj.str(vm).map(|s| s.char_len() as Py_ssize_t).unwrap_or(-1))
}

#[inline]
pub unsafe fn PyUnicode_DecodeFSDefault(s: *const c_char) -> *mut PyObject {
    let Some(cstr) = cstr_opt(s) else {
        return std::ptr::null_mut();
    };
    PyUnicode_DecodeFSDefaultAndSize(cstr.as_ptr(), cstr.to_bytes().len() as Py_ssize_t)
}

#[inline]
pub unsafe fn PyUnicode_READY(_unicode: *mut PyObject) -> c_int {
    PyErr_Clear();
    0
}
