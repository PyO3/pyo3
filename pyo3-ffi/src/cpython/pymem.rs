use libc::size_t;
use std::ffi::c_void;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyMem_RawMalloc")]
    pub fn PyMem_RawMalloc(size: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyMem_RawCalloc")]
    pub fn PyMem_RawCalloc(nelem: size_t, elsize: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyMem_RawRealloc")]
    pub fn PyMem_RawRealloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyMem_RawFree")]
    pub fn PyMem_RawFree(ptr: *mut c_void);

    // skipped _PyMem_GetCurrentAllocatorName
    // skipped _PyMem_RawStrdup
    // skipped _PyMem_Strdup
    // skipped _PyMem_RawWcsdup
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum PyMemAllocatorDomain {
    PYMEM_DOMAIN_RAW,
    PYMEM_DOMAIN_MEM,
    PYMEM_DOMAIN_OBJ,
}

// skipped PyMemAllocatorName
#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyMemAllocatorEx {
    pub ctx: *mut c_void,
    pub malloc: Option<extern "C" fn(ctx: *mut c_void, size: size_t) -> *mut c_void>,
    pub calloc:
        Option<extern "C" fn(ctx: *mut c_void, nelem: size_t, elsize: size_t) -> *mut c_void>,
    pub realloc:
        Option<extern "C" fn(ctx: *mut c_void, ptr: *mut c_void, new_size: size_t) -> *mut c_void>,
    pub free: Option<extern "C" fn(ctx: *mut c_void, ptr: *mut c_void)>,
}

extern "C" {
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyMem_GetAllocator(domain: PyMemAllocatorDomain, allocator: *mut PyMemAllocatorEx);
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyMem_SetAllocator(domain: PyMemAllocatorDomain, allocator: *mut PyMemAllocatorEx);
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyMem_SetupDebugHooks();
}
