#![deny(deprecated)]

use pyo3::prelude::*;

#[pyclass]
#[pyo3(text_signature = "()")]
struct MyClass;

fn main() {}
