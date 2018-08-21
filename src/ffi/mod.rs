#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

#[cfg(not(Py_3))]
pub use ffi2::*;

#[cfg(Py_3)]
pub use ffi3::*;

pub use self::datetime::*;

pub(crate) mod datetime;
