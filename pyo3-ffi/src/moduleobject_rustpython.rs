use crate::methodobject::PyMethodDef;
use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::pyport::Py_ssize_t;
use crate::rustpython_runtime;
use rustpython_vm::AsObject;
use rustpython_vm::builtins::PyModule;
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

#[repr(C)]
pub struct PyModuleDef_Base {
    pub ob_base: PyObject,
    pub m_init: Option<extern "C" fn() -> *mut PyObject>,
    pub m_index: Py_ssize_t,
    pub m_copy: *mut PyObject,
}

pub const PyModuleDef_HEAD_INIT: PyModuleDef_Base = PyModuleDef_Base {
    ob_base: PyObject_HEAD_INIT,
    m_init: None,
    m_index: 0,
    m_copy: std::ptr::null_mut(),
};

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PyModuleDef_Slot {
    pub slot: c_int,
    pub value: *mut c_void,
}

impl Default for PyModuleDef_Slot {
    fn default() -> Self {
        Self {
            slot: 0,
            value: std::ptr::null_mut(),
        }
    }
}

pub const Py_mod_create: c_int = 1;
pub const Py_mod_exec: c_int = 2;
#[cfg(Py_3_12)]
pub const Py_mod_multiple_interpreters: c_int = 3;
#[cfg(Py_3_13)]
pub const Py_mod_gil: c_int = 4;
#[cfg(Py_3_15)]
pub const Py_mod_abi: c_int = 5;
#[cfg(Py_3_15)]
pub const Py_mod_name: c_int = 6;
#[cfg(Py_3_15)]
pub const Py_mod_doc: c_int = 7;
#[cfg(Py_3_15)]
pub const Py_mod_state_size: c_int = 8;
#[cfg(Py_3_15)]
pub const Py_mod_methods: c_int = 9;
#[cfg(Py_3_15)]
pub const Py_mod_state_traverse: c_int = 10;
#[cfg(Py_3_15)]
pub const Py_mod_state_clear: c_int = 11;
#[cfg(Py_3_15)]
pub const Py_mod_state_free: c_int = 12;
#[cfg(Py_3_15)]
pub const Py_mod_token: c_int = 13;

#[cfg(Py_3_12)]
pub const Py_MOD_MULTIPLE_INTERPRETERS_NOT_SUPPORTED: *mut c_void = 0 as *mut c_void;
#[cfg(Py_3_12)]
pub const Py_MOD_MULTIPLE_INTERPRETERS_SUPPORTED: *mut c_void = 1 as *mut c_void;
#[cfg(Py_3_12)]
pub const Py_MOD_PER_INTERPRETER_GIL_SUPPORTED: *mut c_void = 2 as *mut c_void;
#[cfg(Py_3_13)]
pub const Py_MOD_GIL_USED: *mut c_void = 0 as *mut c_void;
#[cfg(Py_3_13)]
pub const Py_MOD_GIL_NOT_USED: *mut c_void = 1 as *mut c_void;

#[repr(C)]
pub struct PyModuleDef {
    pub m_base: PyModuleDef_Base,
    pub m_name: *const c_char,
    pub m_doc: *const c_char,
    pub m_size: Py_ssize_t,
    pub m_methods: *mut PyMethodDef,
    pub m_slots: *mut PyModuleDef_Slot,
    pub m_traverse: Option<traverseproc>,
    pub m_clear: Option<inquiry>,
    pub m_free: Option<freefunc>,
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
        let module = vm.new_module(name.as_str(), dict, None);
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
    _slots: *const PyModuleDef_Slot,
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
pub unsafe fn PyModule_GetStateSize(_mod: *mut PyObject, result: *mut Py_ssize_t) -> c_int {
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
