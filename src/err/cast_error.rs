use std::borrow::Cow;

use crate::{
    exceptions,
    types::{PyAnyMethods, PyStringMethods, PyTuple, PyTupleMethods, PyType, PyTypeMethods},
    Borrowed, Bound, IntoPyObjectExt, Py, PyAny, PyErr, PyErrArguments, Python,
};

/// Error that indicates a failure to convert a PyAny to a more specific Python type.
#[derive(Debug)]
pub struct CastError<'a, 'py> {
    /// The original object that failed to convert.
    from: Borrowed<'a, 'py, PyAny>,
    /// The type we tried (and failed) to convert to.
    /// (see `PyTypeCheck::classinfo_object`)
    to: Bound<'py, PyAny>,
}

impl<'a, 'py> CastError<'a, 'py> {
    /// Create a new `CastError` representing a failure to convert the object
    /// `from` into the type `to`.
    ///
    /// As with [`PyTypeCheck::classinfo_object`][crate::PyTypeCheck::classinfo_object],
    /// valid `to` values are those which can be used with `isinstance`, such as `type`
    /// objects, tuples of `type` objects, or `typing.Union` instances.
    #[inline]
    pub fn new(from: Borrowed<'a, 'py, PyAny>, to: Bound<'py, PyAny>) -> Self {
        Self { from, to }
    }
}

/// Error that indicates a failure to convert a PyAny to a more specific Python type.
#[derive(Debug)]
pub struct CastIntoError<'py> {
    from: Bound<'py, PyAny>,
    to: Bound<'py, PyAny>,
}

impl<'py> CastIntoError<'py> {
    /// Create a new `CastIntoError` representing a failure to convert the object
    /// `from` into the type `to`.
    ///
    /// Equivalent to [`CastError::new`] for owned objects.
    #[inline]
    pub fn new(from: Bound<'py, PyAny>, to: Bound<'py, PyAny>) -> Self {
        Self { from, to }
    }

    /// Consumes this `CastIntoError` and returns the original object, allowing continued
    /// use of it after a failed conversion.
    ///
    /// See [`cast_into`][Bound::cast_into] for an example.
    pub fn into_inner(self) -> Bound<'py, PyAny> {
        self.from
    }
}

struct CastErrorArguments {
    from: Py<PyType>,
    to: Py<PyAny>,
}

impl PyErrArguments for CastErrorArguments {
    fn arguments(self, py: Python<'_>) -> Py<PyAny> {
        format!(
            "{}",
            DisplayDowncastError {
                from: &self.from.into_bound(py),
                to: &self.to.into_bound(py),
            }
        )
        .into_py_any(py)
        .expect("failed to create Python string")
    }
}

/// Convert `CastError` to Python `TypeError`.
impl std::convert::From<CastError<'_, '_>> for PyErr {
    fn from(err: CastError<'_, '_>) -> PyErr {
        let args = CastErrorArguments {
            from: err.from.get_type().unbind(),
            to: err.to.unbind(),
        };

        exceptions::PyTypeError::new_err(args)
    }
}

impl std::error::Error for CastError<'_, '_> {}

impl std::fmt::Display for CastError<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        DisplayDowncastError {
            from: &self.from.get_type(),
            to: &self.to,
        }
        .fmt(f)
    }
}

/// Convert `CastIntoError` to Python `TypeError`.
impl std::convert::From<CastIntoError<'_>> for PyErr {
    fn from(err: CastIntoError<'_>) -> PyErr {
        let args = CastErrorArguments {
            from: err.from.get_type().unbind(),
            to: err.to.unbind(),
        };

        exceptions::PyTypeError::new_err(args)
    }
}

impl std::error::Error for CastIntoError<'_> {}

impl std::fmt::Display for CastIntoError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        DisplayDowncastError {
            from: &self.from.get_type(),
            to: &self.to,
        }
        .fmt(f)
    }
}

struct DisplayDowncastError<'a, 'py> {
    from: &'a Bound<'py, PyType>,
    to: &'a Bound<'py, PyAny>,
}

impl std::fmt::Display for DisplayDowncastError<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from.qualname();
        let from = from
            .as_ref()
            .map(|name| name.to_string_lossy())
            .unwrap_or(Cow::Borrowed("<failed to extract type name>"));
        let to = DisplayClassInfo(self.to);
        write!(f, "'{from}' object cannot be cast as '{to}'")
    }
}

struct DisplayClassInfo<'a, 'py>(&'a Bound<'py, PyAny>);

impl std::fmt::Display for DisplayClassInfo<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(t) = self.0.cast::<PyType>() {
            t.qualname()
                .map_err(|_| std::fmt::Error)?
                .to_string_lossy()
                .fmt(f)
        } else if let Ok(t) = self.0.cast::<PyTuple>() {
            for (i, t) in t.iter().enumerate() {
                if i > 0 {
                    f.write_str(" | ")?;
                }
                write!(f, "{}", DisplayClassInfo(&t))?;
            }
            Ok(())
        } else {
            self.0.fmt(f)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::PyTypeInfo;

    use super::*;

    #[test]
    fn test_display_cast_error() {
        Python::attach(|py| {
            let obj = py.None().into_bound(py);
            let to_type = py.get_type::<crate::types::PyInt>().into_any();
            let err = CastError::new(obj.as_borrowed(), to_type);
            assert_eq!(err.to_string(), "'NoneType' object cannot be cast as 'int'");
        })
    }

    #[test]
    fn test_display_cast_error_with_tuple() {
        Python::attach(|py| {
            let obj = py.None().into_bound(py);
            let to_type = PyTuple::new(
                py,
                &[
                    py.get_type::<crate::types::PyInt>().into_any(),
                    crate::types::PyNone::type_object(py).into_any(),
                ],
            )
            .unwrap()
            .into_any();
            let err = CastError::new(obj.as_borrowed(), to_type);
            assert_eq!(
                err.to_string(),
                "'NoneType' object cannot be cast as 'int | NoneType'"
            );
        })
    }

    #[test]
    fn test_display_cast_into_error() {
        Python::attach(|py| {
            let obj = py.None().into_bound(py);
            let to_type = py.get_type::<crate::types::PyInt>().into_any();
            let err = CastIntoError::new(obj, to_type);
            assert_eq!(err.to_string(), "'NoneType' object cannot be cast as 'int'");
        })
    }

    #[test]
    fn test_pyerr_from_cast_error() {
        Python::attach(|py| {
            let obj = py.None().into_bound(py);
            let to_type = py.get_type::<crate::types::PyInt>().into_any();
            let err = CastError::new(obj.as_borrowed(), to_type);
            let py_err: PyErr = err.into();
            assert_eq!(
                py_err.to_string(),
                "TypeError: 'NoneType' object cannot be cast as 'int'"
            );
        })
    }

    #[test]
    fn test_pyerr_from_cast_into_error() {
        Python::attach(|py| {
            let obj = py.None().into_bound(py);
            let to_type = py.get_type::<crate::types::PyInt>().into_any();
            let err = CastIntoError::new(obj, to_type);
            let py_err: PyErr = err.into();
            assert_eq!(
                py_err.to_string(),
                "TypeError: 'NoneType' object cannot be cast as 'int'"
            );
        })
    }
}
