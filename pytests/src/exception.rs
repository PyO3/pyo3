use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;

create_exception!(pyo3_pytests.exception, CustomValueError, PyValueError);

create_exception!(pyo3_pytests.exception, CustomException, PyException);

#[pymodule(gil_used = false)]
pub mod exception {
    #[pymodule_export]
    use super::{CustomException, CustomValueError};
    use pyo3::prelude::*;

    #[pyfunction]
    fn raise_custom_value_error() -> PyResult<()> {
        Err(CustomValueError::new_err("custom value error"))
    }
}
