use core::{mem, ptr};
use pyo3_ffi::*;
use std::os::raw::{c_int, c_void};

pub static mut MODULE_DEF: PyModuleDef = PyModuleDef {
    m_base: PyModuleDef_HEAD_INIT(),
    m_name: c_str!("sequential").as_ptr(),
    m_doc: c_str!("A library for generating sequential ids, written in Rust.").as_ptr(),
    m_size: mem::size_of::<sequential_state>() as Py_ssize_t,
    m_methods: std::ptr::null_mut(),
    m_slots: unsafe { SEQUENTIAL_SLOTS as *const [PyModuleDef_Slot] as *mut PyModuleDef_Slot },
    m_traverse: Some(sequential_traverse),
    m_clear: Some(sequential_clear),
    m_free: Some(sequential_free),
};

static mut SEQUENTIAL_SLOTS: &[PyModuleDef_Slot] = &[
    PyModuleDef_Slot {
        slot: Py_mod_exec,
        value: sequential_exec as *mut c_void,
    },
    PyModuleDef_Slot {
        slot: Py_mod_multiple_interpreters,
        value: Py_MOD_PER_INTERPRETER_GIL_SUPPORTED,
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
        PyErr_SetString(
            PyExc_SystemError,
            c_str!("cannot locate type object").as_ptr(),
        );
        return -1;
    }
    (*state).id_type = id_type.cast::<PyTypeObject>();

    PyModule_AddObjectRef(module, c_str!("Id").as_ptr(), id_type)
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
