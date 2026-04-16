use crate::object::PyObject;
use std::ffi::{c_char, c_int};

extern_libpython! {
  pub fn PyErr_WarnExplicitObject(
    category: *mut PyObject,
    message: *mut PyObject,
    filename: *mut PyObject,
    lineno: c_int,
    module: *mut PyObject,
    registry: *mut PyObject);
  ) -> c_int;
}
