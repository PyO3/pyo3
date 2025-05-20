use crate::PyObject;
#[cfg(not(any(PyPy, GraalPy)))]
use crate::Py_ssize_t;

#[repr(C)]
#[derive(Debug)]
pub struct PyBaseExceptionObject {
    pub ob_base: PyObject,
    #[cfg(not(PyPy))]
    pub dict: *mut PyObject,
    #[cfg(not(PyPy))]
    pub args: *mut PyObject,
    #[cfg(all(Py_3_11, not(PyPy)))]
    pub notes: *mut PyObject,
    #[cfg(not(PyPy))]
    pub traceback: *mut PyObject,
    #[cfg(not(PyPy))]
    pub context: *mut PyObject,
    #[cfg(not(PyPy))]
    pub cause: *mut PyObject,
    #[cfg(not(PyPy))]
    pub suppress_context: char,
}

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
pub struct PySyntaxErrorObject {
    pub ob_base: PyObject,
    pub dict: *mut PyObject,
    pub args: *mut PyObject,
    #[cfg(Py_3_11)]
    pub notes: *mut PyObject,
    pub traceback: *mut PyObject,
    pub context: *mut PyObject,
    pub cause: *mut PyObject,
    pub suppress_context: char,

    pub msg: *mut PyObject,
    pub filename: *mut PyObject,
    pub lineno: *mut PyObject,
    pub offset: *mut PyObject,
    #[cfg(Py_3_10)]
    pub end_lineno: *mut PyObject,
    #[cfg(Py_3_10)]
    pub end_offset: *mut PyObject,
    pub text: *mut PyObject,
    pub print_file_and_line: *mut PyObject,
    #[cfg(Py_3_14)]
    pub metadata: *mut PyObject,
}

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
pub struct PyImportErrorObject {
    pub ob_base: PyObject,
    pub dict: *mut PyObject,
    pub args: *mut PyObject,
    #[cfg(Py_3_11)]
    pub notes: *mut PyObject,
    pub traceback: *mut PyObject,
    pub context: *mut PyObject,
    pub cause: *mut PyObject,
    pub suppress_context: char,

    pub msg: *mut PyObject,
    pub name: *mut PyObject,
    pub path: *mut PyObject,
    #[cfg(Py_3_12)]
    pub name_from: *mut PyObject,
}

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
pub struct PyUnicodeErrorObject {
    pub ob_base: PyObject,
    pub dict: *mut PyObject,
    pub args: *mut PyObject,
    #[cfg(Py_3_11)]
    pub notes: *mut PyObject,
    pub traceback: *mut PyObject,
    pub context: *mut PyObject,
    pub cause: *mut PyObject,
    pub suppress_context: char,

    pub encoding: *mut PyObject,
    pub object: *mut PyObject,
    pub start: Py_ssize_t,
    pub end: Py_ssize_t,
    pub reason: *mut PyObject,
}

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
pub struct PySystemExitObject {
    pub ob_base: PyObject,
    pub dict: *mut PyObject,
    pub args: *mut PyObject,
    #[cfg(Py_3_11)]
    pub notes: *mut PyObject,
    pub traceback: *mut PyObject,
    pub context: *mut PyObject,
    pub cause: *mut PyObject,
    pub suppress_context: char,

    pub code: *mut PyObject,
}

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Debug)]
pub struct PyOSErrorObject {
    pub ob_base: PyObject,
    pub dict: *mut PyObject,
    pub args: *mut PyObject,
    #[cfg(Py_3_11)]
    pub notes: *mut PyObject,
    pub traceback: *mut PyObject,
    pub context: *mut PyObject,
    pub cause: *mut PyObject,
    pub suppress_context: char,

    pub myerrno: *mut PyObject,
    pub strerror: *mut PyObject,
    pub filename: *mut PyObject,
    pub filename2: *mut PyObject,
    #[cfg(windows)]
    pub winerror: *mut PyObject,
    pub written: Py_ssize_t,
}

#[repr(C)]
#[derive(Debug)]
pub struct PyStopIterationObject {
    pub ob_base: PyObject,
    #[cfg(not(PyPy))]
    pub dict: *mut PyObject,
    #[cfg(not(PyPy))]
    pub args: *mut PyObject,
    #[cfg(all(Py_3_11, not(PyPy)))]
    pub notes: *mut PyObject,
    #[cfg(not(PyPy))]
    pub traceback: *mut PyObject,
    #[cfg(not(PyPy))]
    pub context: *mut PyObject,
    #[cfg(not(PyPy))]
    pub cause: *mut PyObject,
    #[cfg(not(PyPy))]
    pub suppress_context: char,

    pub value: *mut PyObject,
}

// skipped _PyErr_ChainExceptions

// skipped PyNameErrorObject
// skipped PyAttributeErrorObject

// skipped PyEnvironmentErrorObject
// skipped PyWindowsErrorObject

// skipped _PyErr_SetKeyError
// skipped _PyErr_GetTopmostException
// skipped _PyErr_GetExcInfo

// skipped PyErr_SetFromErrnoWithUnicodeFilename

// skipped _PyErr_FormatFromCause

// skipped PyErr_SetFromWindowsErrWithUnicodeFilename
// skipped PyErr_SetExcFromWindowsErrWithUnicodeFilename

// skipped _PyErr_TrySetFromCause

// skipped PySignal_SetWakeupFd
// skipped _PyErr_CheckSignals

// skipped PyErr_SyntaxLocationObject
// skipped PyErr_RangedSyntaxLocationObject
// skipped PyErr_ProgramTextObject

// skipped _PyErr_ProgramDecodedTextObject
// skipped _PyUnicodeTranslateError_Create
// skipped _PyErr_WriteUnraisableMsg
// skipped _Py_FatalErrorFunc
// skipped _Py_FatalErrorFormat
// skipped Py_FatalError
