use std::ffi::c_int;

pub const Py_single_input: c_int = 256;
pub const Py_file_input: c_int = 257;
pub const Py_eval_input: c_int = 258;
#[cfg(Py_3_8)]
pub const Py_func_type_input: c_int = 345;

#[cfg(Py_3_9)]
pub const Py_fstring_input: c_int = 800;
