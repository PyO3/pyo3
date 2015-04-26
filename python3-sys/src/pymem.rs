use libc::{c_void, size_t};

extern "C" {
    pub fn PyMem_Malloc(size: size_t) -> *mut c_void;
    pub fn PyMem_Realloc(ptr: *mut c_void, new_size: size_t)
     -> *mut c_void;
    pub fn PyMem_Free(ptr: *mut c_void);
}

