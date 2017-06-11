use std::os::raw::c_void;
use libc::size_t;

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyMem_RawMalloc(size: size_t) -> *mut c_void;
    pub fn PyMem_RawCalloc(nelem: size_t, elsize: size_t)
     -> *mut c_void;
    pub fn PyMem_RawRealloc(ptr: *mut c_void, new_size: size_t)
     -> *mut c_void;
    pub fn PyMem_RawFree(ptr: *mut c_void) -> ();
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyMem_Malloc(size: size_t) -> *mut c_void;
    pub fn PyMem_Calloc(nelem: size_t, elsize: size_t) -> *mut c_void;
    pub fn PyMem_Realloc(ptr: *mut c_void, new_size: size_t)
     -> *mut c_void;
    pub fn PyMem_Free(ptr: *mut c_void) -> ();
}

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
#[derive(Copy, Clone)]
pub enum PyMemAllocatorDomain {
    PYMEM_DOMAIN_RAW,
    PYMEM_DOMAIN_MEM,
    PYMEM_DOMAIN_OBJ
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(not(Py_LIMITED_API))]
pub struct PyMemAllocatorEx {
    pub ctx: *mut c_void,
    pub malloc: Option<extern "C" fn(ctx: *mut c_void,
                                                    size: size_t)
                                          -> *mut c_void>,
    pub calloc: Option<extern "C" fn(ctx: *mut c_void,
                                                    nelem: size_t,
                                                    elsize: size_t)
                                          -> *mut c_void>,
    pub realloc: Option<extern "C" fn(ctx: *mut c_void,
                                                     ptr: *mut c_void,
                                                     new_size: size_t)
                                           -> *mut c_void>,
    pub free: Option<extern "C" fn(ctx: *mut c_void,
                                                  ptr: *mut c_void)
                                        -> ()>,
}

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyMem_GetAllocator(domain: PyMemAllocatorDomain,
                              allocator: *mut PyMemAllocatorEx) -> ();
    pub fn PyMem_SetAllocator(domain: PyMemAllocatorDomain,
                              allocator: *mut PyMemAllocatorEx) -> ();
    pub fn PyMem_SetupDebugHooks() -> ();
}

