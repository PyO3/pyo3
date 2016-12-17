use libc::{c_char, c_int};
use object::PyObject;

pub const PY_STDIOTEXTMODE : &'static str = "b";

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyFile_FromFd(arg1: c_int, arg2: *const c_char,
                         arg3: *const c_char, arg4: c_int,
                         arg5: *const c_char,
                         arg6: *const c_char,
                         arg7: *const c_char, arg8: c_int)
     -> *mut PyObject;
    pub fn PyFile_GetLine(arg1: *mut PyObject, arg2: c_int)
     -> *mut PyObject;
    pub fn PyFile_WriteObject(arg1: *mut PyObject, arg2: *mut PyObject,
                              arg3: c_int) -> c_int;
    pub fn PyFile_WriteString(arg1: *const c_char,
                              arg2: *mut PyObject) -> c_int;
                              
    pub static mut Py_FileSystemDefaultEncoding: *const c_char;
    #[cfg(Py_3_6)]
    pub static mut Py_FileSystemDefaultEncodeErrors: *const c_char;
    pub static mut Py_HasFileSystemDefaultEncoding: c_int;
}

