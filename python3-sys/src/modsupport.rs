use libc::{c_char, c_int, c_long};
use pyport::Py_ssize_t;
use object::PyObject;
use moduleobject::PyModuleDef;
#[cfg(Py_3_5)]
use methodobject::PyMethodDef;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyArg_Parse(arg1: *mut PyObject, arg2: *const c_char, ...)
     -> c_int;
    pub fn PyArg_ParseTuple(arg1: *mut PyObject,
                            arg2: *const c_char, ...)
     -> c_int;
    pub fn PyArg_ParseTupleAndKeywords(arg1: *mut PyObject,
                                       arg2: *mut PyObject,
                                       arg3: *const c_char,
                                       arg4: *mut *mut c_char, ...)
     -> c_int;
    pub fn PyArg_ValidateKeywordArguments(arg1: *mut PyObject)
     -> c_int;
    pub fn PyArg_UnpackTuple(arg1: *mut PyObject, arg2: *const c_char,
                             arg3: Py_ssize_t, arg4: Py_ssize_t, ...)
     -> c_int;
    pub fn Py_BuildValue(arg1: *const c_char, ...) -> *mut PyObject;
    //pub fn _Py_BuildValue_SizeT(arg1: *const c_char, ...)
    // -> *mut PyObject;
    //pub fn Py_VaBuildValue(arg1: *const c_char, arg2: va_list)
    // -> *mut PyObject;
    pub fn PyModule_AddObject(arg1: *mut PyObject,
                              arg2: *const c_char,
                              arg3: *mut PyObject) -> c_int;
    pub fn PyModule_AddIntConstant(arg1: *mut PyObject,
                                   arg2: *const c_char,
                                   arg3: c_long) -> c_int;
    pub fn PyModule_AddStringConstant(arg1: *mut PyObject,
                                      arg2: *const c_char,
                                      arg3: *const c_char)
     -> c_int;
    #[cfg(Py_3_5)]
    pub fn PyModule_SetDocString(arg1: *mut PyObject,
                                 arg2: *const c_char)
     -> c_int;
    #[cfg(Py_3_5)]
    pub fn PyModule_AddFunctions(arg1: *mut PyObject, arg2: *mut PyMethodDef)
     -> c_int;
    #[cfg(Py_3_5)]
    pub fn PyModule_ExecDef(module: *mut PyObject, def: *mut PyModuleDef)
     -> c_int;
}

pub const Py_CLEANUP_SUPPORTED: i32 = 0x20000;

pub const PYTHON_API_VERSION: i32 = 1013;
pub const PYTHON_ABI_VERSION: i32 = 3;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg(not(py_sys_config="Py_TRACE_REFS"))]
    pub fn PyModule_Create2(module: *mut PyModuleDef,
                        apiver: c_int) -> *mut PyObject;

    #[cfg(py_sys_config="Py_TRACE_REFS")]
    fn PyModule_Create2TraceRefs(module: *mut PyModuleDef,
                        apiver: c_int) -> *mut PyObject;

    #[cfg(not(py_sys_config="Py_TRACE_REFS"))]
    #[cfg(Py_3_5)]
    pub fn PyModule_FromDefAndSpec2(def: *mut PyModuleDef,
                                    spec: *mut PyObject,
                                    module_api_version: c_int)
     -> *mut PyObject;

    #[cfg(py_sys_config="Py_TRACE_REFS")]
    #[cfg(Py_3_5)]
    fn PyModule_FromDefAndSpec2TraceRefs(def: *mut PyModuleDef,
                                    spec: *mut PyObject,
                                    module_api_version: c_int)
     -> *mut PyObject;
}

#[cfg(py_sys_config="Py_TRACE_REFS")]
#[inline]
pub unsafe fn PyModule_Create2(module: *mut PyModuleDef,
                        apiver: c_int) -> *mut PyObject {
    PyModule_Create2TraceRefs(arg1, apiver)
}

#[cfg(py_sys_config="Py_TRACE_REFS")]
#[cfg(Py_3_5)]
#[inline]
pub unsafe fn PyModule_FromDefAndSpec2(def: *mut PyModuleDef,
                                spec: *mut PyObject,
                                module_api_version: c_int)
 -> *mut PyObject {
    PyModule_FromDefAndSpec2TraceRefs(def, spec, module_api_version)
}

#[inline]
pub unsafe fn PyModule_Create(module: *mut PyModuleDef) -> *mut PyObject {
    PyModule_Create2(module, if cfg!(Py_LIMITED_API) { PYTHON_ABI_VERSION } else { PYTHON_API_VERSION })
}

#[inline]
#[cfg(Py_3_5)]
pub unsafe fn PyModule_FromDefAndSpec(def: *mut PyModuleDef, spec: *mut PyObject) -> *mut PyObject {
    PyModule_FromDefAndSpec2(def, spec, if cfg!(Py_LIMITED_API) { PYTHON_ABI_VERSION } else { PYTHON_API_VERSION })
}

