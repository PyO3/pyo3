#![cfg(feature = "either")]

//! Conversion to/from
//! [either](https://docs.rs/either/ "A library for easy idiomatic error handling and reporting in Rust applications")’s
//! [`Either`] type to a union of two Python types.
//!
//! Use of a generic sum type like [either] is common when you want to either accept one of two possible
//! types as an argument or return one of two possible types from a function, without having to define
//! a helper type manually yourself.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! ## change * to the version you want to use, ideally the latest.
//! either = "*"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"either\"] }")]
//! ```
//!
//! Note that you must use compatible versions of either and PyO3.
//! The required either version may vary based on the version of PyO3.
//!
//! # Example: Convert a `int | str` to `Either<i32, String>`.
//!
//! ```rust
//! use either::Either;
//! use pyo3::{Python, PyResult, IntoPyObject, types::PyAnyMethods};
//!
//! fn main() -> PyResult<()> {
//!     Python::initialize();
//!     Python::attach(|py| {
//!         // Create a string and an int in Python.
//!         let py_str = "crab".into_pyobject(py)?;
//!         let py_int = 42i32.into_pyobject(py)?;
//!         // Now convert it to an Either<i32, String>.
//!         let either_str: Either<i32, String> = py_str.extract()?;
//!         let either_int: Either<i32, String> = py_int.extract()?;
//!         Ok(())
//!     })
//! }
//! ```
//!
//! [either](https://docs.rs/either/ "A library for easy idiomatic error handling and reporting in Rust applications")’s

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    exceptions::PyTypeError, types::any::PyAnyMethods, Bound, FromPyObject, IntoPyObject,
    IntoPyObjectExt, PyAny, PyErr, PyResult, Python,
};
use either::Either;

#[cfg_attr(docsrs, doc(cfg(feature = "either")))]
impl<'py, L, R> IntoPyObject<'py> for Either<L, R>
where
    L: IntoPyObject<'py>,
    R: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Either::Left(l) => l.into_bound_py_any(py),
            Either::Right(r) => r.into_bound_py_any(py),
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "either")))]
impl<'a, 'py, L, R> IntoPyObject<'py> for &'a Either<L, R>
where
    &'a L: IntoPyObject<'py>,
    &'a R: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Either::Left(l) => l.into_bound_py_any(py),
            Either::Right(r) => r.into_bound_py_any(py),
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "either")))]
impl<'py, L, R> FromPyObject<'py> for Either<L, R>
where
    L: FromPyObject<'py>,
    R: FromPyObject<'py>,
{
    #[inline]
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(l) = obj.extract::<L>() {
            Ok(Either::Left(l))
        } else if let Ok(r) = obj.extract::<R>() {
            Ok(Either::Right(r))
        } else {
            // TODO: it might be nice to use the `type_input()` name here once `type_input`
            // is not experimental, rather than the Rust type names.
            let err_msg = format!(
                "failed to convert the value to 'Union[{}, {}]'",
                std::any::type_name::<L>(),
                std::any::type_name::<R>()
            );
            Err(PyTypeError::new_err(err_msg))
        }
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::union_of(&[L::type_input(), R::type_input()])
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::exceptions::PyTypeError;
    use crate::{IntoPyObject, Python};

    use crate::types::PyAnyMethods;
    use either::Either;

    #[test]
    fn test_either_conversion() {
        type E = Either<i32, String>;
        type E1 = Either<i32, f32>;
        type E2 = Either<f32, i32>;

        Python::attach(|py| {
            let l = E::Left(42);
            let obj_l = (&l).into_pyobject(py).unwrap();
            assert_eq!(obj_l.extract::<i32>().unwrap(), 42);
            assert_eq!(obj_l.extract::<E>().unwrap(), l);

            let r = E::Right("foo".to_owned());
            let obj_r = (&r).into_pyobject(py).unwrap();
            assert_eq!(obj_r.extract::<Cow<'_, str>>().unwrap(), "foo");
            assert_eq!(obj_r.extract::<E>().unwrap(), r);

            let obj_s = "foo".into_pyobject(py).unwrap();
            let err = obj_s.extract::<E1>().unwrap_err();
            assert!(err.is_instance_of::<PyTypeError>(py));
            assert_eq!(
                err.to_string(),
                "TypeError: failed to convert the value to 'Union[i32, f32]'"
            );

            let obj_i = 42i32.into_pyobject(py).unwrap();
            assert_eq!(obj_i.extract::<E1>().unwrap(), E1::Left(42));
            assert_eq!(obj_i.extract::<E2>().unwrap(), E2::Left(42.0));

            let obj_f = 42.0f64.into_pyobject(py).unwrap();
            assert_eq!(obj_f.extract::<E1>().unwrap(), E1::Right(42.0));
            assert_eq!(obj_f.extract::<E2>().unwrap(), E2::Left(42.0));
        });
    }
}
