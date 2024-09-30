#[cfg(Py_GIL_DISABLED)]
use crate::PyMutex;
use crate::PyObject;

#[repr(C)]
#[cfg(Py_GIL_DISABLED)]
pub struct PyCriticalSection {
    _cs_prev: usize,
    _cs_mutex: *mut PyMutex,
}

#[cfg(Py_GIL_DISABLED)]
impl PyCriticalSection {
    pub const fn new() -> PyCriticalSection {
        PyCriticalSection {
            _cs_prev: 0,
            _cs_mutex: std::ptr::null_mut(),
        }
    }
}

#[cfg(Py_GIL_DISABLED)]
impl Default for PyCriticalSection {
    fn default() -> Self {
        PyCriticalSection::new()
    }
}

#[repr(C)]
#[cfg(Py_GIL_DISABLED)]
pub struct PyCriticalSection2 {
    _cs_base: PyCriticalSection,
    _cs_mutex2: *mut PyMutex,
}

#[cfg(Py_GIL_DISABLED)]
impl PyCriticalSection2 {
    pub const fn new() -> PyCriticalSection2 {
        PyCriticalSection2 {
            _cs_base: PyCriticalSection::new(),
            _cs_mutex2: std::ptr::null_mut(),
        }
    }
}

#[cfg(Py_GIL_DISABLED)]
impl Default for PyCriticalSection2 {
    fn default() -> Self {
        PyCriticalSection2::new()
    }
}

#[cfg(not(Py_GIL_DISABLED))]
opaque_struct!(PyCriticalSection);

#[cfg(not(Py_GIL_DISABLED))]
opaque_struct!(PyCriticalSection2);

extern "C" {
    pub fn PyCriticalSection_Begin(c: *mut PyCriticalSection, op: *mut PyObject);
    pub fn PyCriticalSection_End(c: *mut PyCriticalSection);
    pub fn PyCriticalSection2_Begin(c: *mut PyCriticalSection2, a: *mut PyObject, b: *mut PyObject);
    pub fn PyCriticalSection2_End(c: *mut PyCriticalSection2);
}
