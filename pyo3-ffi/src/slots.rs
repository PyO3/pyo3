#[cfg(Py_3_15)]
use crate::Py_ssize_t;
#[cfg(Py_3_15)]
use core::ffi::{c_int, c_void};

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

#[cfg(Py_3_15)]
pub const PySlot_OPTIONAL: u16 = 0x01;
#[cfg(Py_3_15)]
pub const PySlot_STATIC: u16 = 0x02;
#[cfg(Py_3_15)]
pub const PySlot_INTPTR: u16 = 0x04;
#[cfg(Py_3_15)]
pub const Py_slot_invalid: u16 = 0xffff;

// The slot IDs are integer constants in the C headers and don't have
// explicit types. Legacy slot IDs are c integers but PySlot IDs are
// u16s. We use this to convert from c_int to u16 safely, without using
// `as`. The panic should only be possible if there is a programming
// error somewhere leading to a slot ID outside the range of u16. If
// TryInto becomes possible in const contexts we could use that instead.
#[cfg(Py_3_15)]
const fn safe_cast_c_int_to_u16(val: c_int) -> u16 {
    if val >= 0 && val <= u16::MAX as c_int {
        val as u16
    } else {
        panic!("Slot ID out of range for u16");
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_DATA(NAME: c_int, VALUE: *mut c_void) -> PySlot {
    PySlot {
        sl_id: safe_cast_c_int_to_u16(NAME),
        sl_flags: PySlot_INTPTR,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_ptr: VALUE },
    }
}

#[expect(clippy::incompatible_msrv, reason = "guarded by cfg(const_is_null)")]
#[cfg(Py_3_15)]
pub const unsafe fn PySlot_FUNC(NAME: c_int, VALUE: *mut c_void) -> PySlot {
    #[cfg(const_is_null)]
    assert!(!VALUE.is_null(), "value may not be null");
    PySlot {
        sl_id: safe_cast_c_int_to_u16(NAME),
        sl_flags: 0,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b {
            sl_func: unsafe { core::mem::transmute::<*mut c_void, _Py_funcptr_t>(VALUE) },
        },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_SIZE(NAME: c_int, VALUE: Py_ssize_t) -> PySlot {
    PySlot {
        sl_id: safe_cast_c_int_to_u16(NAME),
        sl_flags: 0,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_size: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_INT64(NAME: c_int, VALUE: i64) -> PySlot {
    PySlot {
        sl_id: safe_cast_c_int_to_u16(NAME),
        sl_flags: 0,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_int64: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_UINT64(NAME: c_int, VALUE: u64) -> PySlot {
    PySlot {
        sl_id: safe_cast_c_int_to_u16(NAME),
        sl_flags: 0,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_uint64: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_STATIC_DATA(NAME: c_int, VALUE: *mut c_void) -> PySlot {
    PySlot {
        sl_id: safe_cast_c_int_to_u16(NAME),
        sl_flags: PySlot_STATIC,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_ptr: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_PTR(NAME: c_int, VALUE: *mut c_void) -> PySlot {
    PySlot {
        sl_id: safe_cast_c_int_to_u16(NAME),
        sl_flags: PySlot_INTPTR,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_ptr: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_PTR_STATIC(NAME: c_int, VALUE: *mut c_void) -> PySlot {
    PySlot {
        sl_id: safe_cast_c_int_to_u16(NAME),
        sl_flags: PySlot_INTPTR | PySlot_STATIC,
        anon1: _anon_union_32b { sl_reserved: 0 },
        anon2: _anon_union_64b { sl_ptr: VALUE },
    }
}

#[cfg(Py_3_15)]
pub const fn PySlot_END() -> PySlot {
    unsafe { core::mem::zeroed() }
}
