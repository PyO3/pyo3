#[cfg(Py_3_15)]
use crate::Py_ssize_t;
#[cfg(Py_3_15)]
use std::ffi::c_void;

#[cfg(Py_3_15)]
pub type _Py_funcptr_t = unsafe extern "C" fn();

#[derive(Copy, Clone)]
#[repr(C)]
#[cfg(Py_3_15)]
pub union _anon_union_32b {
    pub sl_reserved: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
#[cfg(Py_3_15)]
pub union _anon_union_64b {
    pub sl_ptr: *mut c_void,
    pub sl_func: _Py_funcptr_t,
    pub sl_size: Py_ssize_t,
    pub sl_int64: i64,
    pub sl_uint64: u64,
}

#[derive(Copy, Clone)]
#[repr(C)]
#[cfg(Py_3_15)]
pub struct PySlot {
    pub sl_id: u16,
    pub sl_flags: u16,
    pub anon1: _anon_union_32b,
    pub anon2: _anon_union_64b,
}

#[cfg(not(Py_3_15))]
opaque_struct!(pub PySlot);

#[cfg(Py_3_15)]
impl PartialEq for PySlot {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            // memcmp returns 0 if the memory blocks are identical
            libc::memcmp(
                self as *const Self as *const c_void,
                other as *const Self as *const c_void,
                std::mem::size_of::<Self>(),
            ) == 0
        }
    }
}

#[cfg(Py_3_15)]
pub const PySlot_OPTIONAL: u16 = 0x01;
#[cfg(Py_3_15)]
pub const PySlot_STATIC: u16 = 0x02;
#[cfg(Py_3_15)]
pub const PySlot_INTPTR: u16 = 0x04;
#[cfg(Py_3_15)]
pub const Py_slot_invalid: u16 = 0xffff;

#[cfg(Py_3_15)]
pub const fn PySlot_DATA(NAME: u16, VALUE: *mut c_void) -> PySlot {
    PySlot {
        sl_id: NAME,
        sl_flags: PySlot_INTPTR,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_ptr: VALUE },
    }
}

/// # Safety
///
/// Like the C macro, this performs no signature check on `$value`. The caller
/// must ensure the function pointer is appropriate for the given slot id.
/// (Constructing the `PySlot` is itself sound — all `extern "C"` fn pointers
/// share size and ABI with `_Py_funcptr_t` — but CPython will eventually call
/// the function with a particular signature, and a mismatch is UB.)
#[macro_export]
#[cfg(Py_3_15)]
macro_rules! PySlot_FUNC {
    ($name:expr, $value:expr) => {
        $crate::PySlot {
            sl_id: $name,
            sl_flags: 0,
            anon1: $crate::_anon_union_32b { sl_reserved: 0 },
            anon2: $crate::_anon_union_64b {
                sl_func: unsafe { ::std::mem::transmute::<_, $crate::_Py_funcptr_t>($value) },
            },
        }
    };
}

#[cfg(Py_3_15)]
pub const fn PySlot_SIZE(NAME: u16, VALUE: Py_ssize_t) -> PySlot {
    PySlot {
        sl_id: NAME,
        sl_flags: 0,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_size: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_INT64(NAME: u16, VALUE: i64) -> PySlot {
    PySlot {
        sl_id: NAME,
        sl_flags: 0,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_int64: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_UINT64(NAME: u16, VALUE: u64) -> PySlot {
    PySlot {
        sl_id: NAME,
        sl_flags: 0,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_uint64: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_STATIC_DATA(NAME: u16, VALUE: *mut c_void) -> PySlot {
    PySlot {
        sl_id: NAME,
        sl_flags: PySlot_STATIC,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_ptr: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_PTR(NAME: u16, VALUE: *mut c_void) -> PySlot {
    PySlot {
        sl_id: NAME,
        sl_flags: PySlot_INTPTR,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_ptr: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_PTR_STATIC(NAME: u16, VALUE: *mut c_void) -> PySlot {
    PySlot {
        sl_id: NAME,
        sl_flags: PySlot_INTPTR | PySlot_STATIC,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_ptr: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_END() -> PySlot {
    unsafe { std::mem::zeroed() }
}
