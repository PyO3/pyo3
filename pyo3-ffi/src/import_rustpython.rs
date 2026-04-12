use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::rustpython_runtime;
use std::ffi::{c_char, c_int, c_long, CStr};

#[inline]
fn cstr_to_string(name: *const c_char) -> Option<String> {
    (!name.is_null()).then(|| unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned())
}

fn import_module_by_name(
    vm: &rustpython_vm::VirtualMachine,
    name: &str,
    level: usize,
) -> rustpython_vm::PyResult<rustpython_vm::PyObjectRef> {
    if level != 0 || !name.contains('.') {
        let py_name = vm.ctx.new_str(name.to_owned());
        return vm.import(&py_name, level);
    }

    let mut parts = name.split('.');
    let top = parts.next().unwrap_or(name);
    let top_name = vm.ctx.new_str(top.to_owned());
    let mut module = vm.import(&top_name, 0)?;
    for part in parts {
        let attr_name = vm.ctx.intern_str(part);
        module = module.get_attr(attr_name, vm)?;
    }
    Ok(module)
}

#[inline]
pub unsafe fn PyImport_GetMagicNumber() -> c_long {
    0
}

#[inline]
pub unsafe fn PyImport_GetMagicTag() -> *const c_char {
    std::ptr::null()
}

#[inline]
pub unsafe fn PyImport_ExecCodeModule(name: *const c_char, co: *mut PyObject) -> *mut PyObject {
    PyImport_ExecCodeModuleEx(name, co, std::ptr::null())
}

#[inline]
pub unsafe fn PyImport_ExecCodeModuleEx(
    name: *const c_char,
    co: *mut PyObject,
    _pathname: *const c_char,
) -> *mut PyObject {
    let Some(name) = cstr_to_string(name) else {
        return std::ptr::null_mut();
    };
    if co.is_null() {
        return std::ptr::null_mut();
    }
    let code = ptr_to_pyobject_ref_borrowed(co);
    rustpython_runtime::with_vm(move |vm| {
        let Ok(code) = code.downcast::<rustpython_vm::builtins::PyCode>() else {
            return std::ptr::null_mut();
        };
        let globals = vm.ctx.new_dict();
        let scope = rustpython_vm::scope::Scope::with_builtins(None, globals.clone(), vm);
        let module = vm.new_module(&name, globals, None);
        match vm
            .sys_module
            .get_attr("modules", vm)
            .and_then(|mods| mods.set_item(name.as_str(), module.clone().into(), vm))
            .and_then(|_| vm.run_code_obj(code, scope))
        {
            Ok(_) => pyobject_ref_to_ptr(module.into()),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

#[inline]
pub unsafe fn PyImport_ExecCodeModuleWithPathnames(
    name: *const c_char,
    co: *mut PyObject,
    pathname: *const c_char,
    _cpathname: *const c_char,
) -> *mut PyObject {
    PyImport_ExecCodeModuleEx(name, co, pathname)
}

#[inline]
pub unsafe fn PyImport_ExecCodeModuleObject(
    name: *mut PyObject,
    co: *mut PyObject,
    _pathname: *mut PyObject,
    _cpathname: *mut PyObject,
) -> *mut PyObject {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let name_utf8 = crate::PyUnicode_AsUTF8AndSize(name, std::ptr::null_mut());
    if name_utf8.is_null() {
        return std::ptr::null_mut();
    }
    PyImport_ExecCodeModuleEx(name_utf8, co, std::ptr::null())
}

#[inline]
pub unsafe fn PyImport_GetModuleDict() -> *mut PyObject {
    rustpython_runtime::with_vm(|vm| {
        vm.sys_module
            .get_attr("modules", vm)
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PyImport_GetModule(name: *mut PyObject) -> *mut PyObject {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    rustpython_runtime::with_vm(|vm| {
        vm.sys_module
            .get_attr("modules", vm)
            .and_then(|mods| {
                let key = ptr_to_pyobject_ref_borrowed(name);
                mods.get_item(&*key, vm)
            })
            .map(pyobject_ref_to_ptr)
            .unwrap_or(std::ptr::null_mut())
    })
}

#[inline]
pub unsafe fn PyImport_AddModuleObject(name: *mut PyObject) -> *mut PyObject {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let name_utf8 = crate::PyUnicode_AsUTF8AndSize(name, std::ptr::null_mut());
    if name_utf8.is_null() {
        return std::ptr::null_mut();
    }
    PyImport_AddModule(name_utf8)
}

#[inline]
pub unsafe fn PyImport_AddModule(name: *const c_char) -> *mut PyObject {
    let Some(name) = cstr_to_string(name) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(move |vm| {
        let Ok(sys_modules) = vm.sys_module.get_attr("modules", vm) else {
            return std::ptr::null_mut();
        };
        if let Ok(module) = sys_modules.get_item(name.as_str(), vm) {
            return pyobject_ref_to_ptr(module);
        }
        let module = vm.new_module(&name, vm.ctx.new_dict(), None);
        if sys_modules
            .set_item(name.as_str(), module.clone().into(), vm)
            .is_err()
        {
            return std::ptr::null_mut();
        }
        pyobject_ref_to_ptr(module.into())
    })
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyImport_AddModuleRef(name: *const c_char) -> *mut PyObject {
    PyImport_AddModule(name)
}

#[inline]
pub unsafe fn PyImport_ImportModule(name: *const c_char) -> *mut PyObject {
    let Some(name) = cstr_to_string(name) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(move |vm| {
        match import_module_by_name(vm, &name, 0) {
            Ok(module) => pyobject_ref_to_ptr(module),
            Err(exc) => {
                set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

#[inline]
pub unsafe fn PyImport_ImportModuleNoBlock(name: *const c_char) -> *mut PyObject {
    PyImport_ImportModule(name)
}

#[inline]
pub unsafe fn PyImport_ImportModuleLevel(
    name: *const c_char,
    _globals: *mut PyObject,
    _locals: *mut PyObject,
    _fromlist: *mut PyObject,
    level: c_int,
) -> *mut PyObject {
    let Some(name) = cstr_to_string(name) else {
        return std::ptr::null_mut();
    };
    rustpython_runtime::with_vm(move |vm| {
        match import_module_by_name(vm, &name, level.max(0) as usize) {
            Ok(module) => pyobject_ref_to_ptr(module),
            Err(exc) => {
                set_vm_exception(exc);
                std::ptr::null_mut()
            }
        }
    })
}

#[inline]
pub unsafe fn PyImport_ImportModuleLevelObject(
    name: *mut PyObject,
    _globals: *mut PyObject,
    _locals: *mut PyObject,
    _fromlist: *mut PyObject,
    level: c_int,
) -> *mut PyObject {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let name_utf8 = crate::PyUnicode_AsUTF8AndSize(name, std::ptr::null_mut());
    if name_utf8.is_null() {
        return std::ptr::null_mut();
    }
    PyImport_ImportModuleLevel(name_utf8, _globals, _locals, _fromlist, level)
}

#[inline]
pub unsafe fn PyImport_GetImporter(_path: *mut PyObject) -> *mut PyObject {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyImport_Import(name: *mut PyObject) -> *mut PyObject {
    PyImport_ImportModuleLevelObject(name, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), 0)
}

#[inline]
pub unsafe fn PyImport_ReloadModule(m: *mut PyObject) -> *mut PyObject {
    if m.is_null() {
        std::ptr::null_mut()
    } else {
        let obj = ptr_to_pyobject_ref_borrowed(m);
        pyobject_ref_to_ptr(obj)
    }
}

#[cfg(not(Py_3_9))]
#[inline]
pub unsafe fn PyImport_Cleanup() {}

#[inline]
pub unsafe fn PyImport_ImportFrozenModuleObject(_name: *mut PyObject) -> c_int {
    -1
}

#[inline]
pub unsafe fn PyImport_ImportFrozenModule(_name: *const c_char) -> c_int {
    -1
}

#[inline]
pub unsafe fn PyImport_AppendInittab(
    _name: *const c_char,
    _initfunc: Option<unsafe extern "C" fn() -> *mut PyObject>,
) -> c_int {
    0
}
