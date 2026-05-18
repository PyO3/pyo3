use pyo3::native_enum::NativeEnum;
use pyo3::prelude::*;
use pyo3::py_native_enum;

#[py_native_enum]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[py_native_enum(base = "IntEnum")]
pub enum Status {
    Active = 1,
    Inactive = 2,
    Pending = 3,
}

#[py_native_enum(base = "Flag")]
pub enum Permission {
    Read = 1,
    Write = 2,
    Exec = 4,
}

#[py_native_enum(base = "IntFlag")]
pub enum Bits {
    A = 1,
    B = 2,
    C = 4,
}

#[cfg(Py_3_11)]
#[py_native_enum(base = "StrEnum")]
pub enum Size {
    Small,
    Medium,
    Large,
}

#[pyfunction]
fn identity_bits(b: Bits) -> Bits {
    b
}

#[pyfunction]
fn identity_color(c: Color) -> Color {
    c
}

#[pyfunction]
fn identity_status(s: Status) -> Status {
    s
}

#[pyfunction]
fn identity_permission(p: Permission) -> Permission {
    p
}

#[pymodule]
pub mod native_enums {
    use super::*;

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let py = m.py();
        m.add("Color", Color::py_enum_class(py)?)?;
        m.add("Status", Status::py_enum_class(py)?)?;
        m.add("Permission", Permission::py_enum_class(py)?)?;
        m.add("Bits", Bits::py_enum_class(py)?)?;
        #[cfg(Py_3_11)]
        m.add("Size", Size::py_enum_class(py)?)?;
        Ok(())
    }

    #[pymodule_export]
    use super::{identity_bits, identity_color, identity_permission, identity_status};
}
