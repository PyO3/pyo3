#![cfg(feature = "arcstr")]

//!  Conversions to and from [arcstr](https://docs.rs/arcstr/)’s
//! `ArcStr` and `Substr`.
//!
//! [`arcstr::ArcStr`] is a reference-counted string type,
//! with zero-cost (allocation-free) support for string literals.
//! And [`arcstr::Substr`] is a reference counted substrings.
//!
//! A custom thin Arc is used to guarantee a better performance compare to `Arc<str>`.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! # change * to the latest versions
//! arcstr = "*"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"arcstr\"] }")]
//! ```
//!
//! Note that you must use compatible versions of arcstr and PyO3.
//! The required arcstr version may vary based on the version of PyO3.
//!
//! # Examples
//!
//! Using [arcstr](https://docs.rs/arcstr) to accept the reference of string, then echo it and
//! return back. Because of the thin Arc inside [`arcstr::ArcStr`] and [`arcstr::Substr`], the clone is very cheap.
//! ```rust
//! use arcstr::{ArcStr, Substr};
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn echo_arcstr(input: &ArcStr) -> ArcStr {
//!     println!("{input}");
//!     input.clone()
//! }
//!
//! #[pyfunction]
//! fn echo_substr(input: &Substr) -> Substr {
//!     println!("{input}");
//!     input.clone()
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(echo_arcstr, m)?)?;
//!     m.add_function(wrap_pyfunction!(echo_substr, m)?)?;
//!     Ok(())
//! }
//! ```
//!
//! Python code:
//! ```python
//! from my_module import echo_arcstr, echo_substr
//!
//! print(echo_arcstr("Hello, World!"))
//! print(echo_substr("Hello, World!"))
//! ```

use crate::conversion::IntoPyObject;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::types::*;
use crate::{Bound, FromPyObject, PyObject, PyResult, Python};
#[allow(deprecated)]
use crate::{IntoPy, ToPyObject};
use std::convert::Infallible;

#[allow(deprecated)]
impl ToPyObject for arcstr::ArcStr {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for arcstr::ArcStr {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for &arcstr::ArcStr {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }
}

impl<'py> IntoPyObject<'py> for arcstr::ArcStr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self.as_str()))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'py> IntoPyObject<'py> for &arcstr::ArcStr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self.as_str()))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl FromPyObject<'_> for arcstr::ArcStr {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        obj.downcast::<PyString>()?
            .to_cow()
            .map(arcstr::ArcStr::from)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

#[allow(deprecated)]
impl ToPyObject for arcstr::Substr {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for arcstr::Substr {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for &arcstr::Substr {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }
}

impl<'py> IntoPyObject<'py> for arcstr::Substr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self.as_str()))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'py> IntoPyObject<'py> for &arcstr::Substr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self.as_str()))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl FromPyObject<'_> for arcstr::Substr {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        obj.downcast::<PyString>()?
            .to_cow()
            .map(arcstr::Substr::from)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

#[cfg(test)]
mod test_arcstr {
    use crate::types::*;
    use crate::{IntoPyObject, Python};

    #[test]
    fn test_arcstr_arcstr_into_pyobject() {
        Python::with_gil(|py| {
            let s = arcstr::ArcStr::from("Hello, World!");
            let py_s = (&s).into_pyobject(py).unwrap();

            assert!(py_s == "Hello, World!");
            assert_eq!(
                arcstr::ArcStr::from("Hello, World!"),
                py_s.extract::<arcstr::ArcStr>().unwrap()
            );
        });
    }

    #[test]
    fn test_arcstr_substr_into_pyobject() {
        Python::with_gil(|py| {
            let s = arcstr::Substr::from("Hello, World!");
            let py_s = (&s).into_pyobject(py).unwrap();

            assert!(py_s == "Hello, World!");
            assert_eq!(
                arcstr::Substr::from("Hello, World!"),
                py_s.extract::<arcstr::Substr>().unwrap()
            );
        });
    }
}
