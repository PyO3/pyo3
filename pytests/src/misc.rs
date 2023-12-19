use pyo3::{prelude::*, types::PyDict};
use std::borrow::Cow;

#[pyfunction]
fn issue_219() {
    // issue 219: acquiring GIL inside #[pyfunction] deadlocks.
    Python::with_gil(|_| {});
}

#[pyfunction]
fn get_type_full_name(obj: &PyAny) -> PyResult<Cow<'_, str>> {
    obj.get_type().name()
}

#[pyfunction]
fn accepts_bool(val: bool) -> bool {
    val
}

#[pyfunction]
fn get_item_and_run_callback(dict: &PyDict, callback: &PyAny) -> PyResult<()> {
    let item = dict.get_item("key")?.expect("key not found in dict");
    let string = item.to_string();
    callback.call0()?;
    assert_eq!(item.to_string(), string);
    Ok(())
}

#[pymodule]
pub fn misc(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(issue_219, m)?)?;
    m.add_function(wrap_pyfunction!(get_type_full_name, m)?)?;
    m.add_function(wrap_pyfunction!(accepts_bool, m)?)?;
    m.add_function(wrap_pyfunction!(get_item_and_run_callback, m)?)?;
    Ok(())
}
