use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::rustpython_runtime;
#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
use libc::FILE;
#[cfg(any(Py_LIMITED_API, not(Py_3_10), PyPy, GraalPy))]
use std::ffi::c_char;
use std::ffi::c_int;
use rustpython_vm::compiler::Mode;
use rustpython_vm::convert::ToPyException;

#[inline]
pub unsafe fn PyErr_Print() {}

#[inline]
pub unsafe fn PyErr_PrintEx(_set_sys_last_vars: c_int) {}

#[inline]
pub unsafe fn PyErr_Display(
    _exc: *mut PyObject,
    _value: *mut PyObject,
    _tb: *mut PyObject,
) {
}

#[cfg(Py_3_12)]
#[inline]
pub unsafe fn PyErr_DisplayException(_exc: *mut PyObject) {}

#[inline]
pub unsafe fn Py_CompileString(
    string: *const c_char,
    p: *const c_char,
    s: c_int,
) -> *mut PyObject {
    if string.is_null() || p.is_null() {
        return std::ptr::null_mut();
    }
    let Ok(source) = std::ffi::CStr::from_ptr(string).to_str() else {
        return std::ptr::null_mut();
    };
    let Ok(filename) = std::ffi::CStr::from_ptr(p).to_str() else {
        return std::ptr::null_mut();
    };
    let mode = match s {
        crate::compile::Py_eval_input => Mode::Eval,
        crate::compile::Py_single_input => Mode::Single,
        _ => Mode::Exec,
    };
    rustpython_runtime::with_vm(|vm| match vm.compile(source, mode, filename.to_owned()) {
        Ok(code) => pyobject_ref_to_ptr(code.into()),
        Err(exc) => {
            set_vm_exception((exc, Some(source)).to_pyexception(vm));
            std::ptr::null_mut()
        }
    })
}

pub const PYOS_STACK_MARGIN: c_int = 2048;

#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
opaque_struct!(pub _mod);

#[cfg(not(any(PyPy, Py_3_10)))]
opaque_struct!(pub symtable);
#[cfg(not(any(PyPy, Py_3_10)))]
opaque_struct!(pub _node);

#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
#[inline]
pub unsafe fn PyParser_SimpleParseStringFlags(
    _s: *const c_char,
    _b: c_int,
    _flags: c_int,
) -> *mut _node {
    std::ptr::null_mut()
}

#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
#[inline]
pub unsafe fn PyParser_SimpleParseFileFlags(
    _fp: *mut FILE,
    _s: *const c_char,
    _b: c_int,
    _flags: c_int,
) -> *mut _node {
    std::ptr::null_mut()
}

#[cfg(not(any(PyPy, Py_3_10)))]
#[inline]
pub unsafe fn Py_SymtableString(
    _str: *const c_char,
    _filename: *const c_char,
    _start: c_int,
) -> *mut symtable {
    std::ptr::null_mut()
}

#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
#[inline]
pub unsafe fn Py_SymtableStringObject(
    _str: *const c_char,
    _filename: *mut PyObject,
    _start: c_int,
) -> *mut symtable {
    std::ptr::null_mut()
}
