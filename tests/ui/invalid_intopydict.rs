use pyo3::prelude::IntoPyDict;

#[derive(IntoPyDict)]
pub struct TestPyTupleInvalid(u8);

#[derive(IntoPyDict)]
pub enum TestEnumInvalid {
    Variant1
}

fn main() {}