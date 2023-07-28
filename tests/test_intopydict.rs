use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString, PyTuple, IntoPyDict};
use pyo3_macros::IntoPyDict;

#[derive(IntoPyDict)]
pub struct Test1 {
    x: u8,
    y: u8
}

#[derive(IntoPyDict)]
pub struct Test {
    j: Test1,
    h: u8,
    i: u8,
}