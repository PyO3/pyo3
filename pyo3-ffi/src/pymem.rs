use core::ffi::c_void;
use libc::size_t;

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyMem_Malloc")]
    pub fn PyMem_Malloc(size: size_t) -> *mut c_void;

    #[cfg_attr(PyPy, link_name = "PyPyMem_Calloc")]
    pub fn PyMem_Calloc(nelem: size_t, elsize: size_t) -> *mut c_void;

    #[cfg_attr(PyPy, link_name = "PyPyMem_Realloc")]
    pub fn PyMem_Realloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void;

    #[cfg_attr(PyPy, link_name = "PyPyMem_Free")]
    pub fn PyMem_Free(ptr: *mut c_void);
}

#[inline]
pub unsafe fn PyMem_New<T>(n: size_t) -> *mut T {
    if n > isize::MAX as size_t / size_of::<T>() as size_t {
        core::ptr::null_mut()
    } else {
        PyMem_Malloc(n * size_of::<T>() as size_t).cast()
    }
}

#[inline]
pub unsafe fn PyMem_Resize<T>(p: &mut *mut T, n: size_t) {
    *p = if n > isize::MAX as size_t / size_of::<T>() as size_t {
        core::ptr::null_mut()
    } else {
        PyMem_Realloc(p.cast(), n * size_of::<T>() as size_t).cast()
    };
}

extern_libpython! {
    // Deprecated aliases:
    //
    // skipped PyMem_MALLOC
    // skipped PyMem_NEW
    // skipped PyMem_REALLOC
    // skipped PyMem_RESIZE
    // skipped PyMem_FREE
    // skipped PyMem_Del
    // skipped PyMem_DEL

    #[cfg_attr(PyPy, link_name = "PyPyMem_RawMalloc")]
    #[cfg(any(Py_3_13, not(Py_LIMITED_API)))]
    pub fn PyMem_RawMalloc(size: size_t) -> *mut c_void;

    #[cfg_attr(PyPy, link_name = "PyPyMem_RawCalloc")]
    #[cfg(any(Py_3_13, not(Py_LIMITED_API)))]
    pub fn PyMem_RawCalloc(nelem: size_t, elsize: size_t) -> *mut c_void;

    #[cfg_attr(PyPy, link_name = "PyPyMem_RawRealloc")]
    #[cfg(any(Py_3_13, not(Py_LIMITED_API)))]
    pub fn PyMem_RawRealloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void;

    #[cfg_attr(PyPy, link_name = "PyPyMem_RawFree")]
    #[cfg(any(Py_3_13, not(Py_LIMITED_API)))]
    pub fn PyMem_RawFree(ptr: *mut c_void);
}
