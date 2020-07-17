use pyo3::prelude::*;

#[pyenum]
pub enum MyEnum {
    Variant = 1,
    OtherVariant = 2,
}
