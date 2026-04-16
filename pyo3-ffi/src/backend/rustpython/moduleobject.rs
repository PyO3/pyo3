use crate::moduleobject::PyModuleDef;
use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::rustpython_runtime;
use rustpython_vm::builtins::PyModule;
use rustpython_vm::AsObject;
use std::ffi::{c_char, c_int, c_void, CStr};

pub static mut PyModule_Type: PyTypeObject = PyTypeObject { _opaque: [] };
pub static mut PyModuleDef_Type: PyTypeObject = PyTypeObject { _opaque: [] };

#[inline]
pub unsafe fn PyModule_Check(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    let obj = ptr_to_pyobject_ref_borrowed(op);
    rustpython_runtime::with_vm(|vm| {
        obj.class()
            .fast_issubclass(vm.ctx.types.module_type.as_object())
            .into()
    })
}

#[inline]
pub unsafe fn PyModule_CheckExact(op: *mut PyObject) -> c_int {
    if op.is_null() {
        return 0;
    }
    ptr_to_pyobject_ref_borrowed(op)
        .downcast_ref::<PyModule>()
        .is_some()
        .into()
}

#[inline]
unsafe fn as_module(obj: *mut PyObject) -> Option<rustpython_vm::PyRef<PyModule>> {
    (!obj.is_null())
        .then(|| ptr_to_pyobject_ref_borrowed(obj))
        .and_then(|o| o.downcast::<PyModule>().ok())
}

#[inline]
fn cstr_name(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
    }
}

#[inline]
pub unsafe fn PyModule_NewObject(name: *mut PyObject) -> *mut PyObject {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let name = ptr_to_pyobject_ref_borrowed(name);
    rustpython_runtime::with_vm(|vm| {
        let Ok(name) = name.str(vm) else {
            return std::ptr::null_mut();
        };
        let dict = vm.ctx.new_dict();
        let module = vm.new_module(AsRef::<str>::as_ref(&name), dict, None);
        pyobject_ref_to_ptr(module.into())
    })
}

#[inline]
pub unsafe fn PyModule_New(name: *const c_char) -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        let dict = vm.ctx.new_dict();
        let module = vm.new_module(&cstr_name(name), dict, None);
        pyobject_ref_to_ptr(module.into())
    })
}

#[inline]
pub unsafe fn PyModule_GetDict(arg1: *mut PyObject) -> *mut PyObject {
    as_module(arg1)
        .map(|m| pyobject_ref_to_ptr(m.dict().into()))
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyModule_GetNameObject(arg1: *mut PyObject) -> *mut PyObject {
    let Some(module) = as_module(arg1) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| match module.get_attr("__name__", vm) {
        Ok(name) => pyobject_ref_to_ptr(name),
        Err(exc) => {
            set_vm_exception(exc);
            std::ptr::null_mut()
        }
    })
}

#[inline]
pub unsafe fn PyModule_GetName(arg1: *mut PyObject) -> *const c_char {
    let name = PyModule_GetNameObject(arg1);
    if name.is_null() {
        return std::ptr::null();
    }
    crate::PyUnicode_AsUTF8AndSize(name, std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyModule_GetFilename(arg1: *mut PyObject) -> *const c_char {
    let name = PyModule_GetFilenameObject(arg1);
    if name.is_null() {
        return std::ptr::null();
    }
    crate::PyUnicode_AsUTF8AndSize(name, std::ptr::null_mut())
}

#[inline]
pub unsafe fn PyModule_GetFilenameObject(arg1: *mut PyObject) -> *mut PyObject {
    let Some(module) = as_module(arg1) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(|vm| match module.get_attr("__file__", vm) {
        Ok(name) => pyobject_ref_to_ptr(name),
        Err(exc) => {
            set_vm_exception(exc);
            std::ptr::null_mut()
        }
    })
}

#[inline]
pub unsafe fn PyModule_GetDef(_arg1: *mut PyObject) -> *mut PyModuleDef {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyModule_GetState(_arg1: *mut PyObject) -> *mut c_void {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyModuleDef_Init(arg1: *mut PyModuleDef) -> *mut PyObject {
    arg1.cast()
}

#[cfg(all(not(Py_LIMITED_API), Py_GIL_DISABLED))]
#[inline]
pub unsafe fn PyUnstable_Module_SetGIL(_module: *mut PyObject, _gil: *mut c_void) -> c_int {
    0
}

#[cfg(Py_3_15)]
#[inline]
pub unsafe fn PyModule_FromSlotsAndSpec(
    _slots: *const crate::moduleobject::PyModuleDef_Slot,
    spec: *mut PyObject,
) -> *mut PyObject {
    PyModule_NewObject(spec)
}

#[cfg(Py_3_15)]
#[inline]
pub unsafe fn PyModule_Exec(_mod: *mut PyObject) -> c_int {
    0
}

#[cfg(Py_3_15)]
#[inline]
pub unsafe fn PyModule_GetStateSize(
    _mod: *mut PyObject,
    result: *mut crate::pyport::Py_ssize_t,
) -> c_int {
    if !result.is_null() {
        *result = 0;
    }
    0
}

#[cfg(Py_3_15)]
#[inline]
pub unsafe fn PyModule_GetToken(_module: *mut PyObject, result: *mut *mut c_void) -> c_int {
    if !result.is_null() {
        *result = std::ptr::null_mut();
    }
    0
}
