#[cfg(not(any(PyPy, GraalPy)))]
use core::ffi::c_void;
#[cfg(not(any(PyPy, GraalPy)))]
use libc::size_t;

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

extern_libpython! {
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyMem_GetAllocator(domain: PyMemAllocatorDomain, allocator: *mut PyMemAllocatorEx);
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyMem_SetAllocator(domain: PyMemAllocatorDomain, allocator: *mut PyMemAllocatorEx);
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyMem_SetupDebugHooks();
}
