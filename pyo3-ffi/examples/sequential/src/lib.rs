use pyo3_ffi::*;

mod id;
mod module;
#[cfg(not(Py_3_15))]
use crate::module::MODULE_DEF;
#[cfg(Py_3_15)]
use crate::module::SEQUENTIAL_SLOTS;

#[cfg(not(Py_3_15))]
#[allow(non_snake_case, reason = "must be named `PyInit_<your_module>`")]
#[no_mangle]
pub unsafe extern "C" fn PyInit_sequential() -> *mut PyObject {
    PyModuleDef_Init(&raw mut MODULE_DEF)
}

#[cfg(Py_3_15)]
#[allow(non_snake_case, reason = "must be named `PyModExport_<your_module>`")]
#[no_mangle]
pub unsafe extern "C" fn PyModExport_sequential() -> *mut PyModuleDef_Slot {
    (&raw mut SEQUENTIAL_SLOTS).cast()
}
