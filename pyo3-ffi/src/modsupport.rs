use crate::methodobject::PyMethodDef;
use crate::moduleobject::PyModuleDef;
use crate::object::PyObject;
use crate::pyport::Py_ssize_t;
use std::ffi::{c_char, c_int, c_long};

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyArg_Parse")]
    pub fn PyArg_Parse(arg1: *mut PyObject, arg2: *const c_char, ...) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyArg_ParseTuple")]
    pub fn PyArg_ParseTuple(arg1: *mut PyObject, arg2: *const c_char, ...) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyArg_ParseTupleAndKeywords")]
    pub fn PyArg_ParseTupleAndKeywords(
        arg1: *mut PyObject,
        arg2: *mut PyObject,
        arg3: *const c_char,
        #[cfg(not(Py_3_13))] arg4: *mut *mut c_char,
        #[cfg(Py_3_13)] arg4: *const *const c_char,
        ...
    ) -> c_int;

    // skipped PyArg_VaParse
    // skipped PyArg_VaParseTupleAndKeywords

    pub fn PyArg_ValidateKeywordArguments(arg1: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyArg_UnpackTuple")]
    pub fn PyArg_UnpackTuple(
        arg1: *mut PyObject,
        arg2: *const c_char,
        arg3: Py_ssize_t,
        arg4: Py_ssize_t,
        ...
    ) -> c_int;

    #[cfg_attr(PyPy, link_name = "PyPy_BuildValue")]
    pub fn Py_BuildValue(arg1: *const c_char, ...) -> *mut PyObject;
    // skipped Py_VaBuildValue

    #[cfg(Py_3_13)]
    pub fn PyModule_Add(module: *mut PyObject, name: *const c_char, value: *mut PyObject) -> c_int;
    #[cfg(Py_3_10)]
    #[cfg_attr(PyPy, link_name = "PyPyModule_AddObjectRef")]
    pub fn PyModule_AddObjectRef(
        module: *mut PyObject,
        name: *const c_char,
        value: *mut PyObject,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyModule_AddObject")]
    pub fn PyModule_AddObject(
        module: *mut PyObject,
        name: *const c_char,
        value: *mut PyObject,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyModule_AddIntConstant")]
    pub fn PyModule_AddIntConstant(
        module: *mut PyObject,
        name: *const c_char,
        value: c_long,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyModule_AddStringConstant")]
    pub fn PyModule_AddStringConstant(
        module: *mut PyObject,
        name: *const c_char,
        value: *const c_char,
    ) -> c_int;
    #[cfg(any(Py_3_10, all(Py_3_9, not(Py_LIMITED_API))))]
    #[cfg_attr(PyPy, link_name = "PyPyModule_AddType")]
    pub fn PyModule_AddType(
        module: *mut PyObject,
        type_: *mut crate::object::PyTypeObject,
    ) -> c_int;
    // skipped PyModule_AddIntMacro
    // skipped PyModule_AddStringMacro
    pub fn PyModule_SetDocString(arg1: *mut PyObject, arg2: *const c_char) -> c_int;
    pub fn PyModule_AddFunctions(arg1: *mut PyObject, arg2: *mut PyMethodDef) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyModule_ExecDef")]
    pub fn PyModule_ExecDef(module: *mut PyObject, def: *mut PyModuleDef) -> c_int;
}

pub const Py_CLEANUP_SUPPORTED: i32 = 0x2_0000;

pub const PYTHON_API_VERSION: i32 = 1013;
pub const PYTHON_ABI_VERSION: i32 = 3;

extern "C" {
    #[cfg(not(py_sys_config = "Py_TRACE_REFS"))]
    #[cfg_attr(PyPy, link_name = "PyPyModule_Create2")]
    pub fn PyModule_Create2(module: *mut PyModuleDef, apiver: c_int) -> *mut PyObject;
}

#[inline]
pub unsafe fn PyModule_Create(module: *mut PyModuleDef) -> *mut PyObject {
    PyModule_Create2(
        module,
        if cfg!(Py_LIMITED_API) {
            PYTHON_ABI_VERSION
        } else {
            PYTHON_API_VERSION
        },
    )
}

extern "C" {
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    fn PyModule_Create2TraceRefs(module: *mut PyModuleDef, apiver: c_int) -> *mut PyObject;

    #[cfg(not(py_sys_config = "Py_TRACE_REFS"))]
    #[cfg_attr(PyPy, link_name = "PyPyModule_FromDefAndSpec2")]
    pub fn PyModule_FromDefAndSpec2(
        def: *mut PyModuleDef,
        spec: *mut PyObject,
        module_api_version: c_int,
    ) -> *mut PyObject;

    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    fn PyModule_FromDefAndSpec2TraceRefs(
        def: *mut PyModuleDef,
        spec: *mut PyObject,
        module_api_version: c_int,
    ) -> *mut PyObject;
}

#[cfg(py_sys_config = "Py_TRACE_REFS")]
#[inline]
pub unsafe fn PyModule_Create2(module: *mut PyModuleDef, apiver: c_int) -> *mut PyObject {
    PyModule_Create2TraceRefs(module, apiver)
}

#[cfg(py_sys_config = "Py_TRACE_REFS")]
#[inline]
pub unsafe fn PyModule_FromDefAndSpec2(
    def: *mut PyModuleDef,
    spec: *mut PyObject,
    module_api_version: c_int,
) -> *mut PyObject {
    PyModule_FromDefAndSpec2TraceRefs(def, spec, module_api_version)
}

#[inline]
pub unsafe fn PyModule_FromDefAndSpec(def: *mut PyModuleDef, spec: *mut PyObject) -> *mut PyObject {
    PyModule_FromDefAndSpec2(
        def,
        spec,
        if cfg!(Py_LIMITED_API) {
            PYTHON_ABI_VERSION
        } else {
            PYTHON_API_VERSION
        },
    )
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
extern "C" {
    pub fn PyABIInfo_Check(info: *mut PyABIInfo, module_name: *const c_char) -> c_int;
}

#[cfg(all(Py_LIMITED_API, Py_3_15))]
const _PyABIInfo_DEFAULT_FLAG_STABLE: u16 = PyABIInfo_STABLE;
#[cfg(all(Py_3_15, not(Py_LIMITED_API)))]
const _PyABIInfo_DEFAULT_FLAG_STABLE: u16 = 0;

// skipped PyABIInfo_DEFAULT_ABI_VERSION: depends on Py_VERSION_HEX

#[cfg(all(Py_3_15, Py_GIL_DISABLED))]
const _PyABIInfo_DEFAULT_FLAG_FT: u16 = PyABIInfo_FREETHREADED;
#[cfg(all(Py_3_15, not(Py_GIL_DISABLED)))]
const _PyABIInfo_DEFAULT_FLAG_FT: u16 = PyABIInfo_GIL;

#[cfg(Py_3_15)]
// has an alternate definition if Py_BUILD_CORE is set, ignore that
const _PyABIInfo_DEFAULT_FLAG_INTERNAL: u16 = 0;

#[cfg(Py_3_15)]
pub const PyABIInfo_DEFAULT_FLAGS: u16 =
    _PyABIInfo_DEFAULT_FLAG_STABLE | _PyABIInfo_DEFAULT_FLAG_FT | _PyABIInfo_DEFAULT_FLAG_INTERNAL;

#[cfg(Py_3_15)]
// must be pub because it is used by PyABIInfo_VAR
pub const _PyABIInfo_DEFAULT: PyABIInfo = PyABIInfo {
    abiinfo_major_version: 1,
    abiinfo_minor_version: 0,
    flags: PyABIInfo_DEFAULT_FLAGS,
    build_version: 0,
    abi_version: 0,
};

#[cfg(Py_3_15)]
#[macro_export]
macro_rules! PyABIInfo_VAR {
    ($name:ident) => {
        static mut $name: pyo3_ffi::PyABIInfo = pyo3_ffi::_PyABIInfo_DEFAULT;
    };
}
