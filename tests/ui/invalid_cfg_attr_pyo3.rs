#![cfg(feature = "macros")]

use pyo3::prelude::*;

#[cfg_attr(feature = "pyo3", pyclass)]
struct Foo {
    #[cfg_attr(feature = "pyo3", pyo3(get), fail)]
    x: i32,
}

fn main() {
    Foo { x: 1 };
}
