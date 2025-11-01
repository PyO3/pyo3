use std::ptr;

use pyo3_ffi::*;

mod id;
mod module;
use crate::module::MODULE_DEF;

#[allow(non_snake_case, reason = "must be named `PyInit_<your_module>`")]
#[no_mangle]
pub unsafe extern "C" fn PyInit_sequential() -> *mut PyObject {
    PyModuleDef_Init(ptr::addr_of_mut!(MODULE_DEF))
}
