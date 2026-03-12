#![allow(deprecated)]

use std::borrow::Cow;

use crate::{
    exceptions,
    types::{PyAnyMethods, PyStringMethods, PyType, PyTypeMethods},
    Borrowed, Bound, IntoPyObject, Py, PyAny, PyErr, PyErrArguments, Python,
};

/// Error that indicates a failure to convert a PyAny to a more specific Python type.
#[derive(Debug)]
#[deprecated(since = "0.27.0", note = "replaced with `CastError`")]
pub struct DowncastError<'a, 'py> {
    from: Borrowed<'a, 'py, PyAny>,
    to: Cow<'static, str>,
}

impl<'a, 'py> DowncastError<'a, 'py> {
    /// Create a new `DowncastError` representing a failure to convert the object
    /// `from` into the type named in `to`.
    pub fn new(from: &'a Bound<'py, PyAny>, to: impl Into<Cow<'static, str>>) -> Self {
        Self {
            from: from.as_borrowed(),
            to: to.into(),
        }
    }
}

/// Error that indicates a failure to convert a PyAny to a more specific Python type.
#[derive(Debug)]
#[deprecated(since = "0.27.0", note = "replaced with `CastIntoError`")]
pub struct DowncastIntoError<'py> {
    from: Bound<'py, PyAny>,
    to: Cow<'static, str>,
}

impl<'py> DowncastIntoError<'py> {
    /// Create a new `DowncastIntoError` representing a failure to convert the object
    /// `from` into the type named in `to`.
    pub fn new(from: Bound<'py, PyAny>, to: impl Into<Cow<'static, str>>) -> Self {
        Self {
            from,
            to: to.into(),
        }
    }

    /// Consumes this `DowncastIntoError` and returns the original object, allowing continued
    /// use of it after a failed conversion.
    ///
    /// See [`cast_into`][Bound::cast_into] for an example.
    pub fn into_inner(self) -> Bound<'py, PyAny> {
        self.from
    }
}

struct DowncastErrorArguments {
    from: Py<PyType>,
    to: Cow<'static, str>,
}

impl PyErrArguments for DowncastErrorArguments {
    fn arguments(self, py: Python<'_>) -> Py<PyAny> {
        let from = self.from.bind(py).qualname();
        let from = from
            .as_ref()
            .map(|name| name.to_string_lossy())
            .unwrap_or(Cow::Borrowed("<failed to extract type name>"));
        format!("'{}' object cannot be converted to '{}'", from, self.to)
            .into_pyobject(py)
            .unwrap()
            .into_any()
            .unbind()
    }
}

/// Convert `CastError` to Python `TypeError`.
impl std::convert::From<DowncastError<'_, '_>> for PyErr {
    fn from(err: DowncastError<'_, '_>) -> PyErr {
        let args = DowncastErrorArguments {
            from: err.from.get_type().into(),
            to: err.to,
        };

        exceptions::PyTypeError::new_err(args)
    }
}

impl std::error::Error for DowncastError<'_, '_> {}

impl std::fmt::Display for DowncastError<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        display_downcast_error(f, &self.from, &self.to)
    }
}

/// Convert `DowncastIntoError` to Python `TypeError`.
impl std::convert::From<DowncastIntoError<'_>> for PyErr {
    fn from(err: DowncastIntoError<'_>) -> PyErr {
        let args = DowncastErrorArguments {
            from: err.from.get_type().into(),
            to: err.to,
        };

        exceptions::PyTypeError::new_err(args)
    }
}

impl std::error::Error for DowncastIntoError<'_> {}

impl std::fmt::Display for DowncastIntoError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        display_downcast_error(f, &self.from, &self.to)
    }
}

fn display_downcast_error(
    f: &mut std::fmt::Formatter<'_>,
    from: &Bound<'_, PyAny>,
    to: &str,
) -> std::fmt::Result {
    write!(
        f,
        "'{}' object cannot be converted to '{}'",
        from.get_type().qualname().map_err(|_| std::fmt::Error)?,
        to
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_downcast_error() {
        Python::attach(|py| {
            let obj = py.None().into_bound(py);
            let err = DowncastError::new(&obj, "int");
            assert_eq!(
                err.to_string(),
                "'NoneType' object cannot be converted to 'int'"
            );
        })
    }

    #[test]
    fn test_display_downcast_into_error() {
        Python::attach(|py| {
            let obj = py.None().into_bound(py);
            let err = DowncastIntoError::new(obj, "int");
            assert_eq!(
                err.to_string(),
                "'NoneType' object cannot be converted to 'int'"
            );
        })
    }

    #[test]
    fn test_pyerr_from_downcast_error() {
        Python::attach(|py| {
            let obj = py.None().into_bound(py);
            let err = DowncastError::new(&obj, "int");
            let py_err: PyErr = err.into();
            assert_eq!(
                py_err.to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'int'"
            );
        })
    }

    #[test]
    fn test_pyerr_from_downcast_into_error() {
        Python::attach(|py| {
            let obj = py.None().into_bound(py);
            let err = DowncastIntoError::new(obj, "int");
            let py_err: PyErr = err.into();
            assert_eq!(
                py_err.to_string(),
                "TypeError: 'NoneType' object cannot be converted to 'int'"
            );
        })
    }
}
