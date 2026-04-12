use crate::methodobject::PyMethodDef;
use crate::moduleobject::PyModuleDef;
use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::rustpython_runtime;
use std::ffi::{c_char, c_int, c_long};

pub const Py_CLEANUP_SUPPORTED: i32 = 0x2_0000;
pub const PYTHON_API_VERSION: i32 = 1013;
pub const PYTHON_ABI_VERSION: i32 = 3;

#[inline]
pub unsafe fn PyArg_ValidateKeywordArguments(_arg1: *mut PyObject) -> c_int {
    1
}

#[inline]
pub unsafe fn PyModule_AddObject(
    module: *mut PyObject,
    name: *const c_char,
    value: *mut PyObject,
) -> c_int {
    if module.is_null() || name.is_null() || value.is_null() {
        return -1;
    }
    let module = ptr_to_pyobject_ref_borrowed(module);
    let value = ptr_to_pyobject_ref_owned(value);
    let name = std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
    rustpython_runtime::with_vm(move |vm| {
        let attr = vm.ctx.new_str(name.clone());
        match module.set_attr(&attr, value, vm) {
        Ok(()) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    }})
}

#[cfg(Py_3_10)]
#[inline]
pub unsafe fn PyModule_AddObjectRef(
    module: *mut PyObject,
    name: *const c_char,
    value: *mut PyObject,
) -> c_int {
    if module.is_null() || name.is_null() || value.is_null() {
        return -1;
    }
    let module = ptr_to_pyobject_ref_borrowed(module);
    let value = ptr_to_pyobject_ref_borrowed(value);
    let name = std::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
    rustpython_runtime::with_vm(move |vm| {
        let attr = vm.ctx.new_str(name.clone());
        match module.set_attr(&attr, value, vm) {
        Ok(()) => 0,
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    }})
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyModule_Add(
    module: *mut PyObject,
    name: *const c_char,
    value: *mut PyObject,
) -> c_int {
    PyModule_AddObject(module, name, value)
}

#[inline]
pub unsafe fn PyModule_AddIntConstant(
    module: *mut PyObject,
    name: *const c_char,
    value: c_long,
) -> c_int {
    rustpython_runtime::with_vm(|vm| PyModule_AddObject(module, name, pyobject_ref_to_ptr(vm.ctx.new_int(value).into())))
}

#[inline]
pub unsafe fn PyModule_AddStringConstant(
    module: *mut PyObject,
    name: *const c_char,
    value: *const c_char,
) -> c_int {
    if value.is_null() {
        return -1;
    }
    rustpython_runtime::with_vm(|vm| {
        let value = std::ffi::CStr::from_ptr(value).to_string_lossy().into_owned();
        PyModule_AddObject(module, name, pyobject_ref_to_ptr(vm.ctx.new_str(value).into()))
    })
}

#[cfg(any(Py_3_10, all(Py_3_9, not(Py_LIMITED_API))))]
#[inline]
pub unsafe fn PyModule_AddType(module: *mut PyObject, type_: *mut crate::object::PyTypeObject) -> c_int {
    if type_.is_null() {
        return -1;
    }
    let ty = ptr_to_pyobject_ref_borrowed(type_ as *mut PyObject);
    let name_obj = rustpython_runtime::with_vm(|vm| ty.get_attr("__name__", vm));
    match name_obj {
        Ok(name_obj) => {
        let name_ptr = crate::PyUnicode_AsUTF8AndSize(pyobject_ref_as_ptr(&name_obj), std::ptr::null_mut());
        PyModule_AddObjectRef(module, name_ptr, type_ as *mut PyObject)
        }
        Err(exc) => {
            set_vm_exception(exc);
            -1
        }
    }
}

#[inline]
pub unsafe fn PyModule_SetDocString(arg1: *mut PyObject, arg2: *const c_char) -> c_int {
    if arg2.is_null() {
        return 0;
    }
    rustpython_runtime::with_vm(|vm| {
        let doc = std::ffi::CStr::from_ptr(arg2).to_string_lossy().into_owned();
        match ptr_to_pyobject_ref_borrowed(arg1).set_attr("__doc__", vm.ctx.new_str(doc), vm) {
            Ok(()) => 0,
            Err(exc) => {
                set_vm_exception(exc);
                -1
            }
        }
    })
}

#[inline]
pub unsafe fn PyModule_AddFunctions(_arg1: *mut PyObject, _arg2: *mut PyMethodDef) -> c_int {
    0
}

#[inline]
pub unsafe fn PyModule_ExecDef(_module: *mut PyObject, _def: *mut PyModuleDef) -> c_int {
    0
}

#[inline]
pub unsafe fn PyModule_Create2(module: *mut PyModuleDef, _apiver: c_int) -> *mut PyObject {
    crate::PyModule_FromDefAndSpec2(module, std::ptr::null_mut(), _apiver)
}

#[inline]
pub unsafe fn PyModule_FromDefAndSpec2(
    def: *mut PyModuleDef,
    spec: *mut PyObject,
    _module_api_version: c_int,
) -> *mut PyObject {
    if !spec.is_null() {
        let module = crate::PyModule_NewObject(spec);
        if !module.is_null() {
            return module;
        }
    }
    let name = if def.is_null() { std::ptr::null() } else { (*def).m_name };
    crate::PyModule_New(name)
}

#[inline]
pub unsafe fn PyModule_FromDefAndSpec(def: *mut PyModuleDef, spec: *mut PyObject) -> *mut PyObject {
    PyModule_FromDefAndSpec2(def, spec, PYTHON_API_VERSION)
}

#[cfg(Py_3_15)]
#[repr(C)]
pub struct PyABIInfo {
    pub abiinfo_major_version: u8,
    pub abiinfo_minor_version: u8,
    pub flags: u16,
    pub build_version: u32,
    pub abi_version: u32,
}

#[cfg(Py_3_15)]
pub const PyABIInfo_STABLE: u16 = 0x0001;
#[cfg(Py_3_15)]
pub const PyABIInfo_GIL: u16 = 0x0002;
#[cfg(Py_3_15)]
pub const PyABIInfo_FREETHREADED: u16 = 0x0004;
#[cfg(Py_3_15)]
pub const PyABIInfo_INTERNAL: u16 = 0x0008;
#[cfg(Py_3_15)]
pub const PyABIInfo_FREETHREADING_AGNOSTIC: u16 = PyABIInfo_GIL | PyABIInfo_FREETHREADED;
#[cfg(Py_3_15)]
pub const PyABIInfo_DEFAULT_FLAGS: u16 = PyABIInfo_GIL;
#[cfg(Py_3_15)]
pub const _PyABIInfo_DEFAULT: PyABIInfo = PyABIInfo {
    abiinfo_major_version: 1,
    abiinfo_minor_version: 0,
    flags: PyABIInfo_DEFAULT_FLAGS,
    build_version: 0,
    abi_version: 0,
};

#[cfg(Py_3_15)]
#[inline]
pub unsafe fn PyABIInfo_Check(_info: *mut PyABIInfo, _module_name: *const c_char) -> c_int {
    1
}
