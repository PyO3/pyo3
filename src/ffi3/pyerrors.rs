use std::os::raw::{c_char, c_int};
use ffi3::pyport::Py_ssize_t;
use ffi3::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_SetNone")]
    pub fn PyErr_SetNone(arg1: *mut PyObject) -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_SetObject")]
    pub fn PyErr_SetObject(arg1: *mut PyObject, arg2: *mut PyObject) -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_SetString")]
    pub fn PyErr_SetString(exception: *mut PyObject,
                           string: *const c_char) -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_Occurred")]
    pub fn PyErr_Occurred() -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_Clear")]
    pub fn PyErr_Clear() -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_Fetch")]
    pub fn PyErr_Fetch(arg1: *mut *mut PyObject, arg2: *mut *mut PyObject,
                       arg3: *mut *mut PyObject) -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_Restore")]
    pub fn PyErr_Restore(arg1: *mut PyObject, arg2: *mut PyObject,
                         arg3: *mut PyObject) -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_GetExcInfo")]
    pub fn PyErr_GetExcInfo(arg1: *mut *mut PyObject,
                            arg2: *mut *mut PyObject,
                            arg3: *mut *mut PyObject) -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_SetExcInfo")]
    pub fn PyErr_SetExcInfo(arg1: *mut PyObject, arg2: *mut PyObject,
                            arg3: *mut PyObject) -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_FatalError")]
    pub fn Py_FatalError(message: *const c_char) -> !;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_GivenExceptionMatches")]
    pub fn PyErr_GivenExceptionMatches(arg1: *mut PyObject,
                                       arg2: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_ExceptionMatches")]
    pub fn PyErr_ExceptionMatches(arg1: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_NormalizeException")]
    pub fn PyErr_NormalizeException(arg1: *mut *mut PyObject,
                                    arg2: *mut *mut PyObject,
                                    arg3: *mut *mut PyObject) -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyException_SetTraceback")]
    pub fn PyException_SetTraceback(arg1: *mut PyObject, arg2: *mut PyObject)
     -> c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyException_GetTraceback")]
    pub fn PyException_GetTraceback(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyException_GetCause")]
    pub fn PyException_GetCause(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyException_SetCause")]
    pub fn PyException_SetCause(arg1: *mut PyObject, arg2: *mut PyObject)
     -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyException_GetContext")]
    pub fn PyException_GetContext(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyException_SetContext")]
    pub fn PyException_SetContext(arg1: *mut PyObject, arg2: *mut PyObject)
     -> ();
}

#[inline]
pub unsafe fn PyExceptionClass_Check(x: *mut PyObject) -> c_int {
    (PyType_Check(x) != 0 &&
     PyType_FastSubclass(x as *mut PyTypeObject, Py_TPFLAGS_BASE_EXC_SUBCLASS) != 0) as c_int
}

#[inline]
pub unsafe fn PyExceptionInstance_Check(x: *mut PyObject) -> c_int {
    PyType_FastSubclass((*x).ob_type, Py_TPFLAGS_BASE_EXC_SUBCLASS)
}

#[inline]
#[cfg_attr(PyPy, link_name="\u{1}_PyPyExceptionInstance_Class")]
pub unsafe fn PyExceptionInstance_Class(x: *mut PyObject) -> *mut PyObject {
    (*x).ob_type as *mut PyObject
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_BaseException")]
    pub static mut PyExc_BaseException: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_Exception")]
    pub static mut PyExc_Exception: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_StopAsyncIteration")]
    pub static mut PyExc_StopAsyncIteration: *mut PyObject;

    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_StopIteration")]
    pub static mut PyExc_StopIteration: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_GeneratorExit")]
    pub static mut PyExc_GeneratorExit: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ArithmeticError")]
    pub static mut PyExc_ArithmeticError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_LookupError")]
    pub static mut PyExc_LookupError: *mut PyObject;

    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_AssertionError")]
    pub static mut PyExc_AssertionError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_AttributeError")]
    pub static mut PyExc_AttributeError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_BufferError")]
    pub static mut PyExc_BufferError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_EOFError")]
    pub static mut PyExc_EOFError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_FloatingPointError")]
    pub static mut PyExc_FloatingPointError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_OSError")]
    pub static mut PyExc_OSError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ImportError")]
    pub static mut PyExc_ImportError: *mut PyObject;
    #[cfg(Py_3_6)]
    pub static mut PyExc_ModuleNotFoundError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_IndexError")]
    pub static mut PyExc_IndexError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_KeyError")]
    pub static mut PyExc_KeyError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_KeyboardInterrupt")]
    pub static mut PyExc_KeyboardInterrupt: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_MemoryError")]
    pub static mut PyExc_MemoryError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_NameError")]
    pub static mut PyExc_NameError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_OverflowError")]
    pub static mut PyExc_OverflowError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_RuntimeError")]
    pub static mut PyExc_RuntimeError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_RecursionError")]
    pub static mut PyExc_RecursionError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_NotImplementedError")]
    pub static mut PyExc_NotImplementedError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_SyntaxError")]
    pub static mut PyExc_SyntaxError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_IndentationError")]
    pub static mut PyExc_IndentationError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_TabError")]
    pub static mut PyExc_TabError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ReferenceError")]
    pub static mut PyExc_ReferenceError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_SystemError")]
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_SystemError")]
    pub static mut PyExc_SystemError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_SystemExit")]
    pub static mut PyExc_SystemExit: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_TypeError")]
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_TypeError")]
    pub static mut PyExc_TypeError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_UnboundLocalError")]
    pub static mut PyExc_UnboundLocalError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_UnicodeError")]
    pub static mut PyExc_UnicodeError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_UnicodeEncodeError")]
    pub static mut PyExc_UnicodeEncodeError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_UnicodeDecodeError")]
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_UnicodeDecodeError")]
    pub static mut PyExc_UnicodeDecodeError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_UnicodeTranslateError")]
    pub static mut PyExc_UnicodeTranslateError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ValueError")]
    pub static mut PyExc_ValueError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ZeroDivisionError")]
    pub static mut PyExc_ZeroDivisionError: *mut PyObject;

    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_BlockingIOError")]
    pub static mut PyExc_BlockingIOError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_BrokenPipeError")]
    pub static mut PyExc_BrokenPipeError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ChildProcessError")]
    pub static mut PyExc_ChildProcessError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ConnectionError")]
    pub static mut PyExc_ConnectionError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ConnectionAbortedError")]
    pub static mut PyExc_ConnectionAbortedError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ConnectionRefusedError")]
    pub static mut PyExc_ConnectionRefusedError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ConnectionResetError")]
    pub static mut PyExc_ConnectionResetError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_FileExistsError")]
    pub static mut PyExc_FileExistsError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_FileNotFoundError")]
    pub static mut PyExc_FileNotFoundError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_InterruptedError")]
    pub static mut PyExc_InterruptedError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_IsADirectoryError")]
    pub static mut PyExc_IsADirectoryError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_NotADirectoryError")]
    pub static mut PyExc_NotADirectoryError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_PermissionError")]
    pub static mut PyExc_PermissionError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ProcessLookupError")]
    pub static mut PyExc_ProcessLookupError: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_TimeoutError")]
    pub static mut PyExc_TimeoutError: *mut PyObject;

    pub static mut PyExc_EnvironmentError: *mut PyObject;
    pub static mut PyExc_IOError: *mut PyObject;
    #[cfg(windows)] pub static mut PyExc_WindowsError: *mut PyObject;

    pub static mut PyExc_RecursionErrorInst: *mut PyObject;

    /* Predefined warning categories */
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_Warning")]
    pub static mut PyExc_Warning: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_UserWarning")]
    pub static mut PyExc_UserWarning: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_DeprecationWarning")]
    pub static mut PyExc_DeprecationWarning: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_PendingDeprecationWarning")]
    pub static mut PyExc_PendingDeprecationWarning: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_SyntaxWarning")]
    pub static mut PyExc_SyntaxWarning: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_RuntimeWarning")]
    pub static mut PyExc_RuntimeWarning: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_FutureWarning")]
    pub static mut PyExc_FutureWarning: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ImportWarning")]
    pub static mut PyExc_ImportWarning: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_UnicodeWarning")]
    pub static mut PyExc_UnicodeWarning: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_BytesWarning")]
    pub static mut PyExc_BytesWarning: *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyExc_ResourceWarning")]
    pub static mut PyExc_ResourceWarning: *mut PyObject;

    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_BadArgument")]
    pub fn PyErr_BadArgument() -> c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_NoMemory")]
    pub fn PyErr_NoMemory() -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_SetFromErrno")]
    pub fn PyErr_SetFromErrno(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_SetFromErrnoWithFilenameObject")]
    pub fn PyErr_SetFromErrnoWithFilenameObject(
        arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
    pub fn PyErr_SetFromErrnoWithFilenameObjects(
        arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject) -> *mut PyObject;
    pub fn PyErr_SetFromErrnoWithFilename(
        exc: *mut PyObject, filename: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_Format")]
    pub fn PyErr_Format(
        exception: *mut PyObject, format: *const c_char, ...) -> *mut PyObject;
    #[cfg(Py_3_6)]
    pub fn PyErr_SetImportErrorSubclass(
        arg1: *mut PyObject, arg2: *mut PyObject,
        arg3: *mut PyObject, arg4: *mut PyObject) -> *mut PyObject;
    pub fn PyErr_SetImportError(arg1: *mut PyObject, arg2: *mut PyObject,
                                arg3: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_BadInternalCall")]
    pub fn PyErr_BadInternalCall() -> ();
    pub fn _PyErr_BadInternalCall(filename: *const c_char,
                                  lineno: c_int) -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_NewException")]
    pub fn PyErr_NewException(name: *const c_char,
                              base: *mut PyObject, dict: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_NewExceptionWithDoc")]
    pub fn PyErr_NewExceptionWithDoc(name: *const c_char,
                                     doc: *const c_char,
                                     base: *mut PyObject, dict: *mut PyObject)
     -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_WriteUnraisable")]
    pub fn PyErr_WriteUnraisable(arg1: *mut PyObject) -> ();
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_CheckSignals")]
    pub fn PyErr_CheckSignals() -> c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyErr_SetInterrupt")]
    pub fn PyErr_SetInterrupt() -> ();
    pub fn PyErr_SyntaxLocation(filename: *const c_char, lineno: c_int) -> ();
    pub fn PyErr_SyntaxLocationEx(filename: *const c_char, lineno: c_int,
                                  col_offset: c_int) -> ();
    pub fn PyErr_ProgramText(filename: *const c_char, lineno: c_int) -> *mut PyObject;
    pub fn PyUnicodeDecodeError_Create(encoding: *const c_char,
                                       object: *const c_char,
                                       length: Py_ssize_t, start: Py_ssize_t,
                                       end: Py_ssize_t,
                                       reason: *const c_char) -> *mut PyObject;
    pub fn PyUnicodeEncodeError_GetEncoding(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicodeDecodeError_GetEncoding(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicodeEncodeError_GetObject(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicodeDecodeError_GetObject(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicodeTranslateError_GetObject(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicodeEncodeError_GetStart(arg1: *mut PyObject,
                                         arg2: *mut Py_ssize_t) -> c_int;
    pub fn PyUnicodeDecodeError_GetStart(arg1: *mut PyObject,
                                         arg2: *mut Py_ssize_t) -> c_int;
    pub fn PyUnicodeTranslateError_GetStart(arg1: *mut PyObject,
                                            arg2: *mut Py_ssize_t) -> c_int;
    pub fn PyUnicodeEncodeError_SetStart(arg1: *mut PyObject,
                                         arg2: Py_ssize_t) -> c_int;
    pub fn PyUnicodeDecodeError_SetStart(arg1: *mut PyObject,
                                         arg2: Py_ssize_t) -> c_int;
    pub fn PyUnicodeTranslateError_SetStart(arg1: *mut PyObject,
                                            arg2: Py_ssize_t) -> c_int;
    pub fn PyUnicodeEncodeError_GetEnd(arg1: *mut PyObject,
                                       arg2: *mut Py_ssize_t) -> c_int;
    pub fn PyUnicodeDecodeError_GetEnd(arg1: *mut PyObject,
                                       arg2: *mut Py_ssize_t) -> c_int;
    pub fn PyUnicodeTranslateError_GetEnd(arg1: *mut PyObject,
                                          arg2: *mut Py_ssize_t) -> c_int;
    pub fn PyUnicodeEncodeError_SetEnd(arg1: *mut PyObject, arg2: Py_ssize_t) -> c_int;
    pub fn PyUnicodeDecodeError_SetEnd(arg1: *mut PyObject, arg2: Py_ssize_t) -> c_int;
    pub fn PyUnicodeTranslateError_SetEnd(arg1: *mut PyObject,
                                          arg2: Py_ssize_t) -> c_int;
    pub fn PyUnicodeEncodeError_GetReason(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicodeDecodeError_GetReason(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicodeTranslateError_GetReason(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicodeEncodeError_SetReason(exc: *mut PyObject,
                                          reason: *const c_char) -> c_int;
    pub fn PyUnicodeDecodeError_SetReason(exc: *mut PyObject,
                                          reason: *const c_char) -> c_int;
    pub fn PyUnicodeTranslateError_SetReason(exc: *mut PyObject,
                                             reason: *const c_char) -> c_int;
}