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
//! use pyo3::{Python, ToPyObject};
//!
//! fn main() {
//!     pyo3::prepare_freethreaded_python();
//!     Python::with_gil(|py| {
//!         // Create a string and an int in Python.
//!         let py_str = "crab".to_object(py);
//!         let py_int = 42.to_object(py);
//!         // Now convert it to an Either<i32, String>.
//!         let either_str: Either<i32, String> = py_str.extract(py).unwrap();
//!         let either_int: Either<i32, String> = py_int.extract(py).unwrap();
//!     });
//! }
//! ```
//!
//! [either](https://docs.rs/either/ "A library for easy idiomatic error handling and reporting in Rust applications")’s

use crate::conversion::AnyBound;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    conversion::IntoPyObject, exceptions::PyTypeError, types::any::PyAnyMethods, Bound,
    FromPyObject, IntoPy, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject,
};
use either::Either;

#[cfg_attr(docsrs, doc(cfg(feature = "either")))]
impl<L, R> IntoPy<PyObject> for Either<L, R>
where
    L: IntoPy<PyObject>,
    R: IntoPy<PyObject>,
{
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            Either::Left(l) => l.into_py(py),
            Either::Right(r) => r.into_py(py),
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "either")))]
impl<'py, L, R, E1, E2> IntoPyObject<'py> for Either<L, R>
where
    L: IntoPyObject<'py, Error = E1>,
    R: IntoPyObject<'py, Error = E2>,
    E1: Into<PyErr>,
    E2: Into<PyErr>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Either::Left(l) => l
                .into_pyobject(py)
                .map(AnyBound::into_any)
                .map(AnyBound::into_bound)
                .map_err(Into::into),
            Either::Right(r) => r
                .into_pyobject(py)
                .map(AnyBound::into_any)
                .map(AnyBound::into_bound)
                .map_err(Into::into),
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "either")))]
impl<L, R> ToPyObject for Either<L, R>
where
    L: ToPyObject,
    R: ToPyObject,
{
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            Either::Left(l) => l.to_object(py),
            Either::Right(r) => r.to_object(py),
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
    use crate::{Python, ToPyObject};

    use either::Either;

    #[test]
    fn test_either_conversion() {
        type E = Either<i32, String>;
        type E1 = Either<i32, f32>;
        type E2 = Either<f32, i32>;

        Python::with_gil(|py| {
            let l = E::Left(42);
            let obj_l = l.to_object(py);
            assert_eq!(obj_l.extract::<i32>(py).unwrap(), 42);
            assert_eq!(obj_l.extract::<E>(py).unwrap(), l);

            let r = E::Right("foo".to_owned());
            let obj_r = r.to_object(py);
            assert_eq!(obj_r.extract::<Cow<'_, str>>(py).unwrap(), "foo");
            assert_eq!(obj_r.extract::<E>(py).unwrap(), r);

            let obj_s = "foo".to_object(py);
            let err = obj_s.extract::<E1>(py).unwrap_err();
            assert!(err.is_instance_of::<PyTypeError>(py));
            assert_eq!(
                err.to_string(),
                "TypeError: failed to convert the value to 'Union[i32, f32]'"
            );

            let obj_i = 42.to_object(py);
            assert_eq!(obj_i.extract::<E1>(py).unwrap(), E1::Left(42));
            assert_eq!(obj_i.extract::<E2>(py).unwrap(), E2::Left(42.0));

            let obj_f = 42.0.to_object(py);
            assert_eq!(obj_f.extract::<E1>(py).unwrap(), E1::Right(42.0));
            assert_eq!(obj_f.extract::<E2>(py).unwrap(), E2::Left(42.0));
        });
    }
}
