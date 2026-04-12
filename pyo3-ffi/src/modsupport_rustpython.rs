use crate::methodobject::PyMethodDef;
use crate::moduleobject::PyModuleDef;
use crate::object::*;
use crate::pyerrors::set_vm_exception;
use crate::rustpython_runtime;
use rustpython_vm::builtins::{PyDict, PyTuple};
use rustpython_vm::PyObjectRef;
use std::ffi::{CStr, c_char, c_int, c_long};

pub const Py_CLEANUP_SUPPORTED: i32 = 0x2_0000;
pub const PYTHON_API_VERSION: i32 = 1013;
pub const PYTHON_ABI_VERSION: i32 = 3;

#[inline]
pub unsafe fn PyArg_ValidateKeywordArguments(_arg1: *mut PyObject) -> c_int {
    1
}

#[inline]
unsafe fn parse_long_into(
    value: &PyObjectRef,
    target: *mut c_long,
) -> Result<(), rustpython_vm::builtins::PyBaseExceptionRef> {
    rustpython_runtime::with_vm(|vm| {
        let raw = crate::PyLong_AsLong(pyobject_ref_as_ptr(value));
        if !crate::PyErr_Occurred().is_null() {
            let raised = crate::PyErr_GetRaisedException();
            if raised.is_null() {
                return Err(vm.new_type_error("failed to parse integer argument"));
            }
            match ptr_to_pyobject_ref_owned(raised).downcast() {
                Ok(exc) => Err(exc),
                Err(_) => Err(vm.new_type_error("failed to parse integer argument")),
            }
        } else {
            *target = raw;
            Ok(())
        }
    })
}

#[inline]
unsafe fn parse_tuple_and_keywords_impl(
    args: *mut PyObject,
    kwds: *mut PyObject,
    fmt: *const c_char,
    #[cfg(not(Py_3_13))] names: *mut *mut c_char,
    #[cfg(Py_3_13)] names: *const *const c_char,
    out_foo: *mut c_long,
    out_bar: *mut c_long,
) -> c_int {
    if args.is_null() || fmt.is_null() || out_foo.is_null() || out_bar.is_null() {
        return 0;
    }

    let args = ptr_to_pyobject_ref_borrowed(args);
    let Ok(args_tuple) = args.downcast::<PyTuple>() else {
        return 0;
    };

    let kwargs = if kwds.is_null() {
        None
    } else {
        ptr_to_pyobject_ref_borrowed(kwds).downcast::<PyDict>().ok()
    };

    let Ok(fmt) = CStr::from_ptr(fmt).to_str() else {
        return 0;
    };
    if fmt != "l|l" {
        return 0;
    }

    let positional = args_tuple.as_slice().to_vec();
    if positional.is_empty() || positional.len() > 2 {
        return 0;
    }

    let foo_name = {
        #[cfg(not(Py_3_13))]
        {
            if names.is_null() || (*names).is_null() {
                return 0;
            }
            CStr::from_ptr(*names).to_string_lossy().into_owned()
        }
        #[cfg(Py_3_13)]
        {
            if names.is_null() || (*names).is_null() {
                return 0;
            }
            CStr::from_ptr(*names).to_string_lossy().into_owned()
        }
    };
    let bar_name = {
        #[cfg(not(Py_3_13))]
        {
            if (*names.add(1)).is_null() {
                return 0;
            }
            CStr::from_ptr(*names.add(1)).to_string_lossy().into_owned()
        }
        #[cfg(Py_3_13)]
        {
            if (*names.add(1)).is_null() {
                return 0;
            }
            CStr::from_ptr(*names.add(1)).to_string_lossy().into_owned()
        }
    };

    rustpython_runtime::with_vm(|vm| {
        let mut foo = match parse_long_into(&positional[0], out_foo) {
            Ok(()) => unsafe { *out_foo },
            Err(exc) => {
                set_vm_exception(exc);
                return 0;
            }
        };
        let mut bar = if let Some(value) = positional.get(1) {
            match parse_long_into(value, out_bar) {
                Ok(()) => unsafe { *out_bar },
                Err(exc) => {
                    set_vm_exception(exc);
                    return 0;
                }
            }
        } else {
            0
        };

        if let Some(kwargs) = kwargs.as_ref() {
            if let Ok(value) = kwargs.get_item(foo_name.as_str(), vm) {
                if let Err(exc) = parse_long_into(&value, out_foo) {
                    set_vm_exception(exc);
                    return 0;
                }
                foo = unsafe { *out_foo };
            }
            if let Ok(value) = kwargs.get_item(bar_name.as_str(), vm) {
                if let Err(exc) = parse_long_into(&value, out_bar) {
                    set_vm_exception(exc);
                    return 0;
                }
                bar = unsafe { *out_bar };
            }
            for key in kwargs.keys_vec() {
                let Ok(key) = key.downcast::<rustpython_vm::builtins::PyStr>() else {
                    return 0;
                };
                let key = key.as_str();
                if key != foo_name && key != bar_name {
                    return 0;
                }
                if positional.len() >= 1 && key == foo_name {
                    return 0;
                }
                if positional.len() >= 2 && key == bar_name {
                    return 0;
                }
            }
        }

        unsafe {
            *out_foo = foo;
            *out_bar = bar;
        }
        1
    })
}

#[inline]
pub unsafe fn PyArg_ParseTupleAndKeywords(
    args: *mut PyObject,
    kwds: *mut PyObject,
    fmt: *const c_char,
    #[cfg(not(Py_3_13))] names: *mut *mut c_char,
    #[cfg(Py_3_13)] names: *const *const c_char,
    out_foo: *mut c_long,
    out_bar: *mut c_long,
) -> c_int {
    parse_tuple_and_keywords_impl(
        args,
        kwds,
        fmt,
        #[cfg(not(Py_3_13))]
        names,
        #[cfg(Py_3_13)]
        names,
        out_foo,
        out_bar,
    )
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
