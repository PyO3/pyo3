use crate::{exceptions::PyTypeError, FromPyObject, PyAny, PyErr, PyResult, Python};

#[cold]
pub fn failed_to_extract_enum(
    py: Python<'_>,
    type_name: &str,
    variant_names: &[&str],
    error_names: &[&str],
    errors: &[PyErr],
) -> PyErr {
    // TODO maybe use ExceptionGroup on Python 3.11+ ?
    let mut err_msg = format!(
        "failed to extract enum {} ('{}')",
        type_name,
        error_names.join(" | ")
    );
    for ((variant_name, error_name), error) in variant_names.iter().zip(error_names).zip(errors) {
        err_msg.push('\n');
        err_msg.push_str(&format!(
            "- variant {variant_name} ({error_name}): {error_msg}",
            variant_name = variant_name,
            error_name = error_name,
            error_msg = extract_traceback(py, error.clone_ref(py)),
        ));
    }
    PyTypeError::new_err(err_msg)
}

/// Flattens a chain of errors into a single string.
fn extract_traceback(py: Python<'_>, mut error: PyErr) -> String {
    let mut error_msg = error.to_string();
    while let Some(cause) = error.cause(py) {
        error_msg.push_str(", caused by ");
        error_msg.push_str(&cause.to_string());
        error = cause
    }
    error_msg
}

pub fn extract_struct_field<'py, T>(
    obj: &'py PyAny,
    struct_name: &str,
    field_name: &str,
) -> PyResult<T>
where
    T: FromPyObject<'py>,
{
    match obj.extract() {
        Ok(value) => Ok(value),
        Err(err) => Err(failed_to_extract_struct_field(
            obj.py(),
            err,
            struct_name,
            field_name,
        )),
    }
}

pub fn extract_struct_field_with<'py, T>(
    extractor: impl FnOnce(&'py PyAny) -> PyResult<T>,
    obj: &'py PyAny,
    struct_name: &str,
    field_name: &str,
) -> PyResult<T> {
    match extractor(obj) {
        Ok(value) => Ok(value),
        Err(err) => Err(failed_to_extract_struct_field(
            obj.py(),
            err,
            struct_name,
            field_name,
        )),
    }
}

#[cold]
fn failed_to_extract_struct_field(
    py: Python<'_>,
    inner_err: PyErr,
    struct_name: &str,
    field_name: &str,
) -> PyErr {
    let new_err = PyTypeError::new_err(format!(
        "failed to extract field {}.{}",
        struct_name, field_name
    ));
    new_err.set_cause(py, ::std::option::Option::Some(inner_err));
    new_err
}

pub fn extract_tuple_struct_field<'py, T>(
    obj: &'py PyAny,
    struct_name: &str,
    index: usize,
) -> PyResult<T>
where
    T: FromPyObject<'py>,
{
    match obj.extract() {
        Ok(value) => Ok(value),
        Err(err) => Err(failed_to_extract_tuple_struct_field(
            obj.py(),
            err,
            struct_name,
            index,
        )),
    }
}

pub fn extract_tuple_struct_field_with<'py, T>(
    extractor: impl FnOnce(&'py PyAny) -> PyResult<T>,
    obj: &'py PyAny,
    struct_name: &str,
    index: usize,
) -> PyResult<T> {
    match extractor(obj) {
        Ok(value) => Ok(value),
        Err(err) => Err(failed_to_extract_tuple_struct_field(
            obj.py(),
            err,
            struct_name,
            index,
        )),
    }
}

#[cold]
fn failed_to_extract_tuple_struct_field(
    py: Python<'_>,
    inner_err: PyErr,
    struct_name: &str,
    index: usize,
) -> PyErr {
    let new_err =
        PyTypeError::new_err(format!("failed to extract field {}.{}", struct_name, index));
    new_err.set_cause(py, ::std::option::Option::Some(inner_err));
    new_err
}
