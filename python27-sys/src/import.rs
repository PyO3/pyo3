use libc::{c_char, c_uchar, c_int, c_long};
use object::*;

#[repr(C)]
#[derive(Copy)]
pub struct PyImport_Struct_inittab {
    pub name: *mut c_char,
    pub initfunc: Option<unsafe extern "C" fn()>,
}

impl Clone for PyImport_Struct_inittab {
    #[inline] fn clone(&self) -> PyImport_Struct_inittab { *self }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyImport_Struct_frozen {
    pub name: *mut c_char,
    pub code: *mut c_uchar,
    pub size: c_int,
}

#[inline]
pub unsafe fn PyImport_ImportModuleEx(name: *mut c_char,
                                      globals: *mut PyObject,
                                      locals: *mut PyObject,
                                      fromlist: *mut PyObject) -> *mut PyObject {
    PyImport_ImportModuleLevel(name, globals, locals, fromlist, -1)
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyImport_ImportModule(name: *const c_char)
     -> *mut PyObject;
    pub fn PyImport_ImportModuleNoBlock(name: *const c_char)
     -> *mut PyObject;
    pub fn PyImport_ImportModuleLevel(name: *mut c_char,
                                      globals: *mut PyObject,
                                      locals: *mut PyObject,
                                      fromlist: *mut PyObject,
                                      level: c_int) -> *mut PyObject;

    pub fn PyImport_Import(name: *mut PyObject) -> *mut PyObject;
    pub fn PyImport_ReloadModule(m: *mut PyObject) -> *mut PyObject;
    pub fn PyImport_AddModule(name: *const c_char) -> *mut PyObject;
    pub fn PyImport_ExecCodeModule(name: *mut c_char,
                                   co: *mut PyObject) -> *mut PyObject;
    pub fn PyImport_ExecCodeModuleEx(name: *mut c_char,
                                     co: *mut PyObject,
                                     pathname: *mut c_char)
     -> *mut PyObject;
    pub fn PyImport_GetMagicNumber() -> c_long;
    pub fn PyImport_GetImporter(path: *mut PyObject) -> *mut PyObject;
    pub fn PyImport_GetModuleDict() -> *mut PyObject;
    pub fn PyImport_ImportFrozenModule(name: *mut c_char)
     -> c_int;
    
    pub fn PyImport_AppendInittab(name: *const c_char,
                                  initfunc:
                                      Option<unsafe extern "C" fn()>)
     -> c_int;
    pub fn PyImport_ExtendInittab(newtab: *mut PyImport_Struct_inittab)
     -> c_int;
    
    pub static mut PyImport_Inittab: *mut PyImport_Struct_inittab;
    pub static mut PyImport_FrozenModules: *mut PyImport_Struct_frozen;
    
    /*for internal use only:
    pub fn PyImport_Cleanup();
    pub fn _PyImport_AcquireLock();
    pub fn _PyImport_ReleaseLock() -> c_int;
    pub fn _PyImport_FindModule(arg1: *const c_char,
                                arg2: *mut PyObject,
                                arg3: *mut c_char, arg4: size_t,
                                arg5: *mut *mut FILE,
                                arg6: *mut *mut PyObject)
     -> *mut Struct_filedescr;
    pub fn _PyImport_IsScript(arg1: *mut Struct_filedescr) -> c_int;
    pub fn _PyImport_ReInitLock();
    pub fn _PyImport_FindExtension(arg1: *mut c_char,
                                   arg2: *mut c_char)
     -> *mut PyObject;
    pub fn _PyImport_FixupExtension(arg1: *mut c_char,
                                    arg2: *mut c_char)
     -> *mut PyObject;*/
}

