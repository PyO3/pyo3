use libc::{c_char, c_int};
use pyport::Py_ssize_t;
use object::*;
use classobject::*;
use stringobject::PyString_AS_STRING;
#[cfg(py_sys_config="Py_USING_UNICODE")]
use unicodeobject::Py_UNICODE;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyErr_SetNone(arg1: *mut PyObject);
    pub fn PyErr_SetObject(arg1: *mut PyObject, arg2: *mut PyObject);
    pub fn PyErr_SetString(arg1: *mut PyObject, arg2: *const c_char);
    pub fn PyErr_Occurred() -> *mut PyObject;
    pub fn PyErr_Clear();
    pub fn PyErr_Fetch(arg1: *mut *mut PyObject, arg2: *mut *mut PyObject,
                       arg3: *mut *mut PyObject);
    pub fn PyErr_Restore(arg1: *mut PyObject, arg2: *mut PyObject,
                         arg3: *mut PyObject);
    pub fn PyErr_GivenExceptionMatches(arg1: *mut PyObject,
                                       arg2: *mut PyObject) -> c_int;
    pub fn PyErr_ExceptionMatches(arg1: *mut PyObject) -> c_int;
    pub fn PyErr_NormalizeException(arg1: *mut *mut PyObject,
                                    arg2: *mut *mut PyObject,
                                    arg3: *mut *mut PyObject);
}

#[inline]
pub unsafe fn PyExceptionClass_Check(x: *mut PyObject) -> c_int {
    (PyClass_Check((x)) != 0 || (PyType_Check((x)) != 0 &&
      PyType_FastSubclass(x as *mut PyTypeObject, Py_TPFLAGS_BASE_EXC_SUBCLASS) != 0)) as c_int
}

#[inline]
pub unsafe fn PyExceptionInstance_Check(x: *mut PyObject) -> c_int {
    (PyInstance_Check((x)) != 0 ||
     PyType_FastSubclass((*x).ob_type, Py_TPFLAGS_BASE_EXC_SUBCLASS) != 0) as c_int
}

#[inline]
pub unsafe fn PyExceptionClass_Name(x: *mut PyObject) -> *const c_char {
    if PyClass_Check(x) != 0 {
        PyString_AS_STRING((*(x as *mut PyClassObject)).cl_name)
    } else {
        (*(x as *mut PyTypeObject)).tp_name
    }
}

#[inline]
pub unsafe fn PyExceptionInstance_Class(x: *mut PyObject) -> *mut PyObject {
    if PyInstance_Check(x) != 0 {
        (*(x as *mut PyInstanceObject)).in_class as *mut PyObject
    } else {
        (*x).ob_type as *mut PyObject
    }
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyExc_BaseException: *mut PyObject;
    pub static mut PyExc_Exception: *mut PyObject;
    pub static mut PyExc_StopIteration: *mut PyObject;
    pub static mut PyExc_GeneratorExit: *mut PyObject;
    pub static mut PyExc_StandardError: *mut PyObject;
    pub static mut PyExc_ArithmeticError: *mut PyObject;
    pub static mut PyExc_LookupError: *mut PyObject;
    pub static mut PyExc_AssertionError: *mut PyObject;
    pub static mut PyExc_AttributeError: *mut PyObject;
    pub static mut PyExc_EOFError: *mut PyObject;
    pub static mut PyExc_FloatingPointError: *mut PyObject;
    pub static mut PyExc_EnvironmentError: *mut PyObject;
    pub static mut PyExc_IOError: *mut PyObject;
    pub static mut PyExc_OSError: *mut PyObject;
    pub static mut PyExc_ImportError: *mut PyObject;
    pub static mut PyExc_IndexError: *mut PyObject;
    pub static mut PyExc_KeyError: *mut PyObject;
    pub static mut PyExc_KeyboardInterrupt: *mut PyObject;
    pub static mut PyExc_MemoryError: *mut PyObject;
    pub static mut PyExc_NameError: *mut PyObject;
    pub static mut PyExc_OverflowError: *mut PyObject;
    pub static mut PyExc_RuntimeError: *mut PyObject;
    pub static mut PyExc_NotImplementedError: *mut PyObject;
    pub static mut PyExc_SyntaxError: *mut PyObject;
    pub static mut PyExc_IndentationError: *mut PyObject;
    pub static mut PyExc_TabError: *mut PyObject;
    pub static mut PyExc_ReferenceError: *mut PyObject;
    pub static mut PyExc_SystemError: *mut PyObject;
    pub static mut PyExc_SystemExit: *mut PyObject;
    pub static mut PyExc_TypeError: *mut PyObject;
    pub static mut PyExc_UnboundLocalError: *mut PyObject;
    pub static mut PyExc_UnicodeError: *mut PyObject;
    pub static mut PyExc_UnicodeEncodeError: *mut PyObject;
    pub static mut PyExc_UnicodeDecodeError: *mut PyObject;
    pub static mut PyExc_UnicodeTranslateError: *mut PyObject;
    pub static mut PyExc_ValueError: *mut PyObject;
    pub static mut PyExc_ZeroDivisionError: *mut PyObject;
    #[cfg(windows)] pub static mut PyExc_WindowsError: *mut PyObject;
    pub static mut PyExc_BufferError: *mut PyObject;
    pub static mut PyExc_MemoryErrorInst: *mut PyObject;
    pub static mut PyExc_RecursionErrorInst: *mut PyObject;
    pub static mut PyExc_Warning: *mut PyObject;
    pub static mut PyExc_UserWarning: *mut PyObject;
    pub static mut PyExc_DeprecationWarning: *mut PyObject;
    pub static mut PyExc_PendingDeprecationWarning: *mut PyObject;
    pub static mut PyExc_SyntaxWarning: *mut PyObject;
    pub static mut PyExc_RuntimeWarning: *mut PyObject;
    pub static mut PyExc_FutureWarning: *mut PyObject;
    pub static mut PyExc_ImportWarning: *mut PyObject;
    pub static mut PyExc_UnicodeWarning: *mut PyObject;
    pub static mut PyExc_BytesWarning: *mut PyObject;
    
    pub fn PyErr_BadArgument() -> c_int;
    pub fn PyErr_NoMemory() -> *mut PyObject;
    pub fn PyErr_SetFromErrno(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyErr_SetFromErrnoWithFilenameObject(arg1: *mut PyObject,
                                                arg2: *mut PyObject)
     -> *mut PyObject;
    pub fn PyErr_SetFromErrnoWithFilename(arg1: *mut PyObject,
                                          arg2: *const c_char)
     -> *mut PyObject;
    pub fn PyErr_Format(arg1: *mut PyObject, arg2: *const c_char, ...)
     -> *mut PyObject;
    pub fn PyErr_BadInternalCall();
    pub fn _PyErr_BadInternalCall(filename: *mut c_char,
                                  lineno: c_int);
    pub fn PyErr_NewException(name: *mut c_char, base: *mut PyObject,
                              dict: *mut PyObject) -> *mut PyObject;
    pub fn PyErr_NewExceptionWithDoc(name: *mut c_char,
                                     doc: *mut c_char,
                                     base: *mut PyObject, dict: *mut PyObject)
     -> *mut PyObject;
    pub fn PyErr_WriteUnraisable(arg1: *mut PyObject);
    pub fn PyErr_CheckSignals() -> c_int;
    pub fn PyErr_SetInterrupt();
    pub fn PySignal_SetWakeupFd(fd: c_int) -> c_int;
    pub fn PyErr_SyntaxLocation(arg1: *const c_char,
                                arg2: c_int);
    pub fn PyErr_ProgramText(arg1: *const c_char, arg2: c_int)
     -> *mut PyObject;
}

#[cfg(py_sys_config="Py_USING_UNICODE")]
#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyUnicodeDecodeError_Create(arg1: *const c_char,
                                       arg2: *const c_char,
                                       arg3: Py_ssize_t, arg4: Py_ssize_t,
                                       arg5: Py_ssize_t,
                                       arg6: *const c_char)
     -> *mut PyObject;
    pub fn PyUnicodeEncodeError_Create(arg1: *const c_char,
                                       arg2: *const Py_UNICODE,
                                       arg3: Py_ssize_t, arg4: Py_ssize_t,
                                       arg5: Py_ssize_t,
                                       arg6: *const c_char)
     -> *mut PyObject;
    pub fn PyUnicodeTranslateError_Create(arg1: *const Py_UNICODE,
                                          arg2: Py_ssize_t, arg3: Py_ssize_t,
                                          arg4: Py_ssize_t,
                                          arg5: *const c_char)
     -> *mut PyObject;
    pub fn PyUnicodeEncodeError_GetEncoding(arg1: *mut PyObject)
     -> *mut PyObject;
    pub fn PyUnicodeDecodeError_GetEncoding(arg1: *mut PyObject)
     -> *mut PyObject;
    pub fn PyUnicodeEncodeError_GetObject(arg1: *mut PyObject)
     -> *mut PyObject;
    pub fn PyUnicodeDecodeError_GetObject(arg1: *mut PyObject)
     -> *mut PyObject;
    pub fn PyUnicodeTranslateError_GetObject(arg1: *mut PyObject)
     -> *mut PyObject;
    pub fn PyUnicodeEncodeError_GetStart(arg1: *mut PyObject,
                                         arg2: *mut Py_ssize_t)
     -> c_int;
    pub fn PyUnicodeDecodeError_GetStart(arg1: *mut PyObject,
                                         arg2: *mut Py_ssize_t)
     -> c_int;
    pub fn PyUnicodeTranslateError_GetStart(arg1: *mut PyObject,
                                            arg2: *mut Py_ssize_t)
     -> c_int;
    pub fn PyUnicodeEncodeError_SetStart(arg1: *mut PyObject,
                                         arg2: Py_ssize_t) -> c_int;
    pub fn PyUnicodeDecodeError_SetStart(arg1: *mut PyObject,
                                         arg2: Py_ssize_t) -> c_int;
    pub fn PyUnicodeTranslateError_SetStart(arg1: *mut PyObject,
                                            arg2: Py_ssize_t)
     -> c_int;
    pub fn PyUnicodeEncodeError_GetEnd(arg1: *mut PyObject,
                                       arg2: *mut Py_ssize_t)
     -> c_int;
    pub fn PyUnicodeDecodeError_GetEnd(arg1: *mut PyObject,
                                       arg2: *mut Py_ssize_t)
     -> c_int;
    pub fn PyUnicodeTranslateError_GetEnd(arg1: *mut PyObject,
                                          arg2: *mut Py_ssize_t)
     -> c_int;
    pub fn PyUnicodeEncodeError_SetEnd(arg1: *mut PyObject, arg2: Py_ssize_t)
     -> c_int;
    pub fn PyUnicodeDecodeError_SetEnd(arg1: *mut PyObject, arg2: Py_ssize_t)
     -> c_int;
    pub fn PyUnicodeTranslateError_SetEnd(arg1: *mut PyObject,
                                          arg2: Py_ssize_t) -> c_int;
    pub fn PyUnicodeEncodeError_GetReason(arg1: *mut PyObject)
     -> *mut PyObject;
    pub fn PyUnicodeDecodeError_GetReason(arg1: *mut PyObject)
     -> *mut PyObject;
    pub fn PyUnicodeTranslateError_GetReason(arg1: *mut PyObject)
     -> *mut PyObject;
    pub fn PyUnicodeEncodeError_SetReason(arg1: *mut PyObject,
                                          arg2: *const c_char)
     -> c_int;
    pub fn PyUnicodeDecodeError_SetReason(arg1: *mut PyObject,
                                          arg2: *const c_char)
     -> c_int;
    pub fn PyUnicodeTranslateError_SetReason(arg1: *mut PyObject,
                                             arg2: *const c_char)
     -> c_int;
}

