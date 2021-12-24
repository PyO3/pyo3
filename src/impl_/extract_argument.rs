use crate::{
    exceptions::PyTypeError, type_object::PyTypeObject, FromPyObject, PyAny, PyErr, PyResult,
    Python,
};

#[doc(hidden)]
#[inline]
pub fn extract_argument<'py, T>(obj: &'py PyAny, arg_name: &str) -> PyResult<T>
where
    T: FromPyObject<'py>,
{
    match obj.extract() {
        Ok(e) => Ok(e),
        Err(e) => Err(argument_extraction_error(obj.py(), arg_name, e)),
    }
}

/// Adds the argument name to the error message of an error which occurred during argument extraction.
///
/// Only modifies TypeError. (Cannot guarantee all exceptions have constructors from
/// single string.)
#[doc(hidden)]
#[cold]
pub fn argument_extraction_error(py: Python, arg_name: &str, error: PyErr) -> PyErr {
    if error.get_type(py) == PyTypeError::type_object(py) {
        PyTypeError::new_err(format!("argument '{}': {}", arg_name, error.value(py)))
    } else {
        error
    }
}
