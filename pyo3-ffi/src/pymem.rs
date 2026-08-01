use core::ffi::c_void;
use libc::size_t;

extern_libpython! {
    // skipped PyMem_Malloc
    // skipped PyMem_Calloc
    // skipped PyMem_Realloc
    // skipped PyMem_Free
    // skipped PyMem_New
    // skipped PyMem_Resize
    // skipped PyMem_MALLOC
    // skipped PyMem_NEW
    // skipped PyMem_REALLOC
    // skipped PyMem_RESIZE
    // skipped PyMem_FREE
    // skipped PyMem_Del
    // skipped PyMem_DEL

    #[cfg_attr(PyPy, link_name = "PyPyMem_RawMalloc")]
    pub fn PyMem_RawMalloc(size: size_t) -> *mut c_void;

    #[cfg_attr(PyPy, link_name = "PyPyMem_RawCalloc")]
    pub fn PyMem_RawCalloc(nelem: size_t, elsize: size_t) -> *mut c_void;

    #[cfg_attr(PyPy, link_name = "PyPyMem_RawRealloc")]
    pub fn PyMem_RawRealloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void;

    #[cfg_attr(PyPy, link_name = "PyPyMem_RawFree")]
    pub fn PyMem_RawFree(ptr: *mut c_void);
}
