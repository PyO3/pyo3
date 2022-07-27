use crate::{exceptions::PyTypeError, PyErr, Python};

#[cold]
pub fn failed_to_extract_enum(
    py: Python<'_>,
    type_name: &str,
    variant_names: &[&str],
    error_names: &[&str],
    errors: &[PyErr],
) -> PyErr {
    let mut err_msg = format!(
        "failed to extract enum {} ('{}')",
        type_name,
        error_names.join(" | ")
    );
    for ((variant_name, error_name), error) in variant_names.iter().zip(error_names).zip(errors) {
        use std::fmt::Write;
        write!(
            &mut err_msg,
            "\n- variant {variant_name} ({error_name}): {error_msg}",
            variant_name = variant_name,
            error_name = error_name,
            error_msg = error.value(py).str().unwrap().to_str().unwrap(),
        )
        .unwrap();
    }
    PyTypeError::new_err(err_msg)
}
