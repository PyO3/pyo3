#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyMutex {
    pub _bits: u8,
}

extern "C" {
    pub fn PyMutex_Lock(m: *mut PyMutex);
    pub fn PyMutex_UnLock(m: *mut PyMutex);
}
