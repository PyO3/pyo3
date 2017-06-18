#![feature(proc_macro)]
extern crate pyo3;

use pyo3::{py, PyResult, Python, PyModule};

#[py::class]
//~^ ERROR: custom attribute panicked
//~^^ HELP: #[class] can only be used with normal structs
enum MyClass {
    A,
    B,
}
