use core::{mem, ptr};
use pyo3_ffi::*;
use std::ffi::{c_int, c_void};

#[cfg(not(Py_3_15))]
pub static mut MODULE_DEF: PyModuleDef = PyModuleDef {
    m_base: PyModuleDef_HEAD_INIT,
    m_name: c"sequential".as_ptr(),
    m_doc: c"A library for generating sequential ids, written in Rust.".as_ptr(),
    m_size: mem::size_of::<sequential_state>() as Py_ssize_t,
    m_methods: std::ptr::null_mut(),
    m_slots: (&raw mut SEQUENTIAL_SLOTS).cast(),
    m_traverse: Some(sequential_traverse),
    m_clear: Some(sequential_clear),
    m_free: Some(sequential_free),
};

#[cfg(Py_3_15)]
PyABIInfo_VAR!(ABI_INFO);

const SEQUENTIAL_SLOTS_LEN: usize =
    2 + cfg!(Py_3_12) as usize + cfg!(Py_GIL_DISABLED) as usize + 7 * (cfg!(Py_3_15) as usize);
#[cfg(Py_3_15)]
pub static mut SEQUENTIAL_SLOTS: [PySlot; SEQUENTIAL_SLOTS_LEN] = [
    PySlot_STATIC_DATA(Py_mod_abi, (&raw mut ABI_INFO).cast()),
    PySlot_STATIC_DATA(Py_mod_name, c"sequential".as_ptr() as *mut c_void),
    PySlot_STATIC_DATA(
        Py_mod_doc,
        c"A library for generating sequential ids, written in Rust.".as_ptr() as *mut c_void,
    ),
    PySlot_SIZE(
        Py_mod_state_size,
        mem::size_of::<sequential_state>() as Py_ssize_t,
    ),
    PySlot_FUNC(Py_mod_state_traverse, unsafe {
        std::mem::transmute::<traverseproc, _Py_funcptr_t>(sequential_traverse)
    }),
    PySlot_FUNC(Py_mod_state_clear, unsafe {
        std::mem::transmute::<inquiry, _Py_funcptr_t>(sequential_clear)
    }),
    PySlot_FUNC(Py_mod_state_free, unsafe {
        std::mem::transmute::<freefunc, _Py_funcptr_t>(sequential_free)
    }),
    PySlot_FUNC(Py_mod_exec, unsafe {
        std::mem::transmute::<unsafe extern "C" fn(*mut PyObject) -> c_int, _Py_funcptr_t>(
            sequential_exec,
        )
    }),
    PySlot_DATA(
        Py_mod_multiple_interpreters,
        Py_MOD_PER_INTERPRETER_GIL_SUPPORTED,
    ),
    #[cfg(Py_GIL_DISABLED)]
    PySlot_DATA(Py_mod_gil, Py_MOD_GIL_NOT_USED),
    PySlot_END(),
];
#[cfg(not(Py_3_15))]
pub static mut SEQUENTIAL_SLOTS: [PyModuleDef_Slot; SEQUENTIAL_SLOTS_LEN] = [
    PyModuleDef_Slot {
        slot: Py_mod_exec,
        value: sequential_exec as *mut c_void,
    },
    #[cfg(Py_3_12)]
    PyModuleDef_Slot {
        slot: Py_mod_multiple_interpreters,
        value: Py_MOD_PER_INTERPRETER_GIL_SUPPORTED,
    },
    #[cfg(Py_GIL_DISABLED)]
    PyModuleDef_Slot {
        slot: Py_mod_gil,
        vale: Py_MOD_GIL_NOT_USED,
    },
];

unsafe extern "C" fn sequential_exec(module: *mut PyObject) -> c_int {
    let state: *mut sequential_state = PyModule_GetState(module).cast();

    let id_type = PyType_FromModuleAndSpec(module, &raw mut crate::id::ID_SPEC, ptr::null_mut());
    if id_type.is_null() {
        PyErr_SetString(PyExc_SystemError, c"cannot locate type object".as_ptr());
        return -1;
    }
    (*state).id_type = id_type.cast::<PyTypeObject>();

    PyModule_AddObjectRef(module, c"Id".as_ptr(), id_type)
}

unsafe extern "C" fn sequential_traverse(
    module: *mut PyObject,
    visit: visitproc,
    arg: *mut c_void,
) -> c_int {
    let state: *mut sequential_state = PyModule_GetState(module.cast()).cast();
    let id_type: *mut PyObject = (*state).id_type.cast();

    if id_type.is_null() {
        0
    } else {
        (visit)(id_type, arg)
    }
}

unsafe extern "C" fn sequential_clear(module: *mut PyObject) -> c_int {
    let state: *mut sequential_state = PyModule_GetState(module.cast()).cast();
    Py_CLEAR((&raw mut (*state).id_type).cast());
    0
}

unsafe extern "C" fn sequential_free(module: *mut c_void) {
    sequential_clear(module.cast());
}

#[repr(C)]
struct sequential_state {
    id_type: *mut PyTypeObject,
}
