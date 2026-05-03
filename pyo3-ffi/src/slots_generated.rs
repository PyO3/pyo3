use std::ffi::c_int;

#[allow(unused_variables)]
const fn _Py_SLOT_COMPAT_VALUE(OLD: u16, NEW: u16) -> u16 {
    #[cfg(Py_3_15)]
    {
        NEW
    }

    #[cfg(not(Py_3_15))]
    {
        OLD
    }
}

#[allow(unused_variables)]
const fn _Py_SLOT_COMPAT_VALUE_int(OLD: c_int, NEW: c_int) -> c_int {
    #[cfg(Py_3_15)]
    {
        NEW
    }

    #[cfg(not(Py_3_15))]
    {
        OLD
    }
}

pub const Py_slot_end: u16 = 0;
pub const Py_mod_create: u16 = _Py_SLOT_COMPAT_VALUE(1, 84);
pub const Py_mod_exec: u16 = _Py_SLOT_COMPAT_VALUE(2, 85);
pub const Py_mod_multiple_interpreters: u16 = _Py_SLOT_COMPAT_VALUE(3, 86);
pub const Py_mod_gil: u16 = _Py_SLOT_COMPAT_VALUE(4, 87);
pub const Py_bf_getbuffer: c_int = _Py_SLOT_COMPAT_VALUE_int(1, 88);
pub const Py_bf_releasebuffer: c_int = _Py_SLOT_COMPAT_VALUE_int(2, 89);
pub const Py_mp_ass_subscript: c_int = _Py_SLOT_COMPAT_VALUE_int(3, 90);
pub const Py_mp_length: c_int = _Py_SLOT_COMPAT_VALUE_int(4, 91);
pub const Py_slot_subslots: u16 = 92;
pub const Py_tp_slots: u16 = 93;
pub const Py_mod_slots: u16 = 94;
pub const Py_tp_name: u16 = 95;
pub const Py_tp_basicsize: u16 = 96;
pub const Py_tp_extra_basicsize: u16 = 97;
pub const Py_tp_itemsize: u16 = 98;
pub const Py_tp_flags: u16 = 99;
pub const Py_mod_name: u16 = 100;
pub const Py_mod_doc: u16 = 101;
pub const Py_mod_state_size: u16 = 102;
pub const Py_mod_methods: u16 = 103;
pub const Py_mod_state_traverse: u16 = 104;
pub const Py_mod_state_clear: u16 = 105;
pub const Py_mod_state_free: u16 = 106;
pub const Py_tp_metaclass: u16 = 107;
pub const Py_tp_module: u16 = 108;
pub const Py_mod_abi: u16 = 109;
pub const Py_mod_token: u16 = 110;
