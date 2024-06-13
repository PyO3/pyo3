use pyo3::{prelude::*, types::PyDict};

#[pyfunction]
fn issue_219() {
    // issue 219: acquiring GIL inside #[pyfunction] deadlocks.
    Python::with_gil(|_| {});
}

#[pyfunction]
fn get_type_fully_qualified_name(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    obj.get_type().fully_qualified_name()
}

#[pyfunction]
fn accepts_bool(val: bool) -> bool {
    val
}

#[pyfunction]
fn get_item_and_run_callback(dict: Bound<'_, PyDict>, callback: Bound<'_, PyAny>) -> PyResult<()> {
    // This function gives the opportunity to run a pure-Python callback so that
    // gevent can instigate a context switch. This had problematic interactions
    // with PyO3's removed "GIL Pool".
    // For context, see https://github.com/PyO3/pyo3/issues/3668
    let item = dict.get_item("key")?.expect("key not found in dict");
    let string = item.to_string();
    callback.call0()?;
    assert_eq!(item.to_string(), string);
    Ok(())
}

#[pymodule]
pub fn misc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(issue_219, m)?)?;
    m.add_function(wrap_pyfunction!(get_type_fully_qualified_name, m)?)?;
    m.add_function(wrap_pyfunction!(accepts_bool, m)?)?;
    m.add_function(wrap_pyfunction!(get_item_and_run_callback, m)?)?;
    Ok(())
}
