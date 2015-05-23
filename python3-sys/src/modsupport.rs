use libc::{c_char, c_int, c_long};
use pyport::Py_ssize_t;
use object::PyObject;
use moduleobject::PyModuleDef;

extern "C" {
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
}

pub const Py_CLEANUP_SUPPORTED: i32 = 0x20000;

pub const PYTHON_API_VERSION: i32 = 1013;
pub const PYTHON_ABI_VERSION: i32 = 3;

extern "C" {
    #[cfg(not(py_sys_config="Py_TRACE_REFS"))]
    pub fn PyModule_Create2(module: *mut PyModuleDef,
                        apiver: c_int) -> *mut PyObject;

    #[cfg(py_sys_config="Py_TRACE_REFS")]
    fn PyModule_Create2TraceRefs(module: *mut PyModuleDef,
                        apiver: c_int) -> *mut PyObject;
}

#[cfg(py_sys_config="Py_TRACE_REFS")]
#[inline]
pub unsafe fn PyModule_Create2(module: *mut PyModuleDef,
                        apiver: c_int) -> *mut PyObject {
    PyModule_Create2TraceRefs(arg1, apiver)
}

#[inline]
pub unsafe fn PyModule_Create(module: *mut PyModuleDef) -> *mut PyObject {
    PyModule_Create2(module, PYTHON_ABI_VERSION)
}

