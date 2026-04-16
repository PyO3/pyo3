use libc::size_t;
use std::ffi::c_void;

#[inline]
pub unsafe fn PyMem_Malloc(size: size_t) -> *mut c_void {
    libc::malloc(size)
}

#[inline]
pub unsafe fn PyMem_Calloc(nelem: size_t, elsize: size_t) -> *mut c_void {
    libc::calloc(nelem, elsize)
}

#[inline]
pub unsafe fn PyMem_Realloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void {
    libc::realloc(ptr, new_size)
}

#[inline]
pub unsafe fn PyMem_Free(ptr: *mut c_void) {
    libc::free(ptr)
}
