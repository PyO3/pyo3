//! Support for exposing Rust enums to Python as [`enum.Enum`] subclasses.
//!
//! The `enum` base class (e.g. `enum.Enum`, `enum.IntEnum`) is cached per interpreter session
//! using [`PyOnceLock`], while the generated Python class itself is **not** cached here — it is
//! cached per enum type by the `#[native_enum]` / `#[derive(NativeEnum)]` macros.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use pyo3::native_enum::{NativeEnum, NativeEnumBase, NativeEnumSpec, VariantValue};
//! use pyo3::prelude::*;
//! use pyo3::types::PyAny;
//!
//! #[derive(Copy, Clone)]
//! enum Color { Red, Green, Blue }
//!
//! impl NativeEnum for Color {
//!     const SPEC: NativeEnumSpec = NativeEnumSpec {
//!         name: "Color",
//!         base: NativeEnumBase::Enum,
//!         variants: &[
//!             ("Red",   VariantValue::Auto),
//!             ("Green", VariantValue::Auto),
//!             ("Blue",  VariantValue::Auto),
//!         ],
//!         module: None,
//!         qualname: None,
//!     };
//!
//!     fn to_py_member<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
//!         let cls = Self::py_enum_class(py)?;
//!         let name = match self { Self::Red => "Red", Self::Green => "Green", Self::Blue => "Blue" };
//!         cls.getattr(name).map_err(Into::into)
//!     }
//!
//!     fn from_py_member(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
//!         use pyo3::exceptions::{PyTypeError, PyValueError};
//!         let cls = Self::py_enum_class(obj.py())?;
//!         if !obj.is_instance(cls.as_any())? {
//!             return Err(PyTypeError::new_err("expected a Color member"));
//!         }
//!         let name: String = obj.getattr("name")?.extract()?;
//!         match name.as_str() {
//!             "Red"   => Ok(Self::Red),
//!             "Green" => Ok(Self::Green),
//!             "Blue"  => Ok(Self::Blue),
//!             other   => Err(PyValueError::new_err(format!("unknown Color variant: {other}"))),
//!         }
//!     }
//! }
//! ```
//!
//! [`enum.Enum`]: https://docs.python.org/3/library/enum.html
//! [`PyOnceLock`]: crate::sync::PyOnceLock

mod base_cache;
mod construct;
mod spec;
mod trait_def;

pub use self::construct::build_native_enum;
pub use self::spec::{NativeEnumBase, NativeEnumSpec, VariantValue};
pub use self::trait_def::NativeEnum;
