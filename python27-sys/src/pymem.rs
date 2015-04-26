use libc::{c_void, size_t};

#[link(name = "python2.7")]
extern "C" {
    pub fn PyMem_Malloc(n: size_t) -> *mut c_void;
    pub fn PyMem_Realloc(p: *mut c_void, n: size_t)
     -> *mut c_void;
    pub fn PyMem_Free(p: *mut c_void);
}

