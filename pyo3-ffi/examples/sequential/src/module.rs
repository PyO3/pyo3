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
    m_slots: std::ptr::addr_of_mut!(SEQUENTIAL_SLOTS).cast(),
    m_traverse: Some(sequential_traverse),
    m_clear: Some(sequential_clear),
    m_free: Some(sequential_free),
};

#[cfg(Py_3_15)]
PyABIInfo_VAR!(ABI_INFO);

const SEQUENTIAL_SLOTS_LEN: usize =
    2 + cfg!(Py_3_12) as usize + cfg!(Py_GIL_DISABLED) as usize + 7 * (cfg!(Py_3_15) as usize);
pub static mut SEQUENTIAL_SLOTS: [PyModuleDef_Slot; SEQUENTIAL_SLOTS_LEN] = [
    #[cfg(Py_3_15)]
    PyModuleDef_Slot {
        slot: Py_mod_abi,
        value: std::ptr::addr_of_mut!(ABI_INFO).cast(),
    },
    #[cfg(Py_3_15)]
    PyModuleDef_Slot {
        slot: Py_mod_name,
        // safety: Python does not write to this field
        value: c"sequential".as_ptr() as *mut c_void,
    },
    #[cfg(Py_3_15)]
    PyModuleDef_Slot {
        slot: Py_mod_doc,
        // safety: Python does not write to this field
        value: c"A library for generating sequential ids, written in Rust.".as_ptr() as *mut c_void,
    },
    #[cfg(Py_3_15)]
    PyModuleDef_Slot {
        slot: Py_mod_state_size,
        value: mem::size_of::<sequential_state>() as *mut c_void,
    },
    #[cfg(Py_3_15)]
    PyModuleDef_Slot {
        slot: Py_mod_state_traverse,
        value: sequential_traverse as *mut c_void,
    },
    #[cfg(Py_3_15)]
    PyModuleDef_Slot {
        slot: Py_mod_state_clear,
        value: sequential_clear as *mut c_void,
    },
    #[cfg(Py_3_15)]
    PyModuleDef_Slot {
        slot: Py_mod_state_free,
        value: sequential_free as *mut c_void,
    },
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
        value: Py_MOD_GIL_NOT_USED,
    },
    PyModuleDef_Slot {
        slot: 0,
        value: ptr::null_mut(),
    },
];

unsafe extern "C" fn sequential_exec(module: *mut PyObject) -> c_int {
    let state: *mut sequential_state = PyModule_GetState(module).cast();

    let id_type = PyType_FromModuleAndSpec(
        module,
        ptr::addr_of_mut!(crate::id::ID_SPEC),
        ptr::null_mut(),
    );
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
    Py_CLEAR(ptr::addr_of_mut!((*state).id_type).cast());
    0
}

unsafe extern "C" fn sequential_free(module: *mut c_void) {
    sequential_clear(module.cast());
}

#[repr(C)]
struct sequential_state {
    id_type: *mut PyTypeObject,
}
