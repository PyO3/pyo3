

use pyo3::types::{PyDict, IntoPyDict};
use pyo3_macros::IntoPyDict;

#[derive(IntoPyDict)]
pub struct Test1 {
    x: u8,
    y: u8,
}

#[derive(IntoPyDict)]
pub struct Test {
    j: Test1,
    h: u8,
    i: u8
}