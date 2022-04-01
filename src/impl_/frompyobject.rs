use crate::{exceptions::PyTypeError, PyErr, Python};

#[cold]
pub fn failed_to_extract_enum(
    py: Python,
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
        err_msg.push('\n');
        err_msg.push_str(&format!(
            "- variant {variant_name} ({error_name}): {error_msg}",
            variant_name = variant_name,
            error_name = error_name,
            error_msg = error.value(py).str().unwrap().to_str().unwrap(),
        ));
    }
    PyTypeError::new_err(err_msg)
}
