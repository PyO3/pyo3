#![feature(proc_macro)]
extern crate pyo3;

use pyo3::{py, PyResult, Python, PyModule};

#[py::modinit(rust2py)] 
//~^ ERROR: custom attribute panicked
//~^^ HELP: #[modinit] can only be used with fn block
struct Rust2Py;
