use libc::{size_t, FILE};
use std::os::raw::{c_char, c_int};
use ffi2::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyFile_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyFile_Check(op : *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyFile_Type)
}

#[inline(always)]
pub unsafe fn PyFile_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyFile_Type) as c_int
}


pub const PY_STDIOTEXTMODE : &'static str = "b";

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyFile_FromString(arg1: *mut c_char,
                             arg2: *mut c_char) -> *mut PyObject;
    pub fn PyFile_SetBufSize(arg1: *mut PyObject, arg2: c_int);
    pub fn PyFile_SetEncoding(arg1: *mut PyObject,
                              arg2: *const c_char) -> c_int;
    pub fn PyFile_SetEncodingAndErrors(arg1: *mut PyObject,
                                       arg2: *const c_char,
                                       errors: *mut c_char)
                                       -> c_int;
    pub fn PyFile_FromFile(arg1: *mut FILE, arg2: *mut c_char,
                           arg3: *mut c_char,
                           arg4: Option<unsafe extern "C" fn (arg1: *mut FILE) -> c_int>)
                           -> *mut PyObject;
    pub fn PyFile_AsFile(arg1: *mut PyObject) -> *mut FILE;
    //pub fn PyFile_IncUseCount(arg1: *mut PyFileObject);
    //pub fn PyFile_DecUseCount(arg1: *mut PyFileObject);
    pub fn PyFile_Name(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyFile_GetLine(arg1: *mut PyObject, arg2: c_int)
                          -> *mut PyObject;
    pub fn PyFile_WriteObject(arg1: *mut PyObject, arg2: *mut PyObject,
                              arg3: c_int) -> c_int;
    pub fn PyFile_SoftSpace(arg1: *mut PyObject, arg2: c_int)
     -> c_int;
    pub fn PyFile_WriteString(arg1: *const c_char,
                              arg2: *mut PyObject) -> c_int;
    pub fn PyObject_AsFileDescriptor(arg1: *mut PyObject) -> c_int;
    pub fn Py_UniversalNewlineFgets(arg1: *mut c_char,
                                    arg2: c_int, arg3: *mut FILE,
                                    arg4: *mut PyObject)
     -> *mut c_char;
    pub fn Py_UniversalNewlineFread(arg1: *mut c_char, arg2: size_t,
                                    arg3: *mut FILE, arg4: *mut PyObject)
     -> size_t;

    pub static mut Py_FileSystemDefaultEncoding: *const c_char;
}

