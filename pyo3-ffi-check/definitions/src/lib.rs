#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    improper_ctypes,
    clippy::all,
    // clippy fails with lots of errors if this is not set specifically
    clippy::used_underscore_binding
)]
pub mod bindgen {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod pyo3_ffi {
    #[doc(inline)]
    pub use pyo3_ffi::*;
}
