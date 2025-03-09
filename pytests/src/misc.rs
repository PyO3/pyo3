use pyo3::{
    prelude::*,
    types::{PyDict, PyString},
};

#[pyfunction]
fn issue_219() {
    // issue 219: acquiring GIL inside #[pyfunction] deadlocks.
    Python::with_gil(|_| {});
}

#[pyclass]
struct LockHolder {
    #[allow(unused)]
    sender: std::sync::mpsc::Sender<()>,
}

// This will hammer the GIL once the retyrbed LockHolder is dropped.
#[pyfunction]
fn hammer_gil_in_thread() -> PyResult<LockHolder> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        receiver.recv().ok();
        loop {
            Python::with_gil(|_py| ());
        }
    });
    Ok(LockHolder { sender })
}

#[pyfunction]
fn get_type_fully_qualified_name<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyString>> {
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

#[pymodule(gil_used = false)]
pub fn misc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(issue_219, m)?)?;
    m.add_function(wrap_pyfunction!(hammer_gil_in_thread, m)?)?;
    m.add_function(wrap_pyfunction!(get_type_fully_qualified_name, m)?)?;
    m.add_function(wrap_pyfunction!(accepts_bool, m)?)?;
    m.add_function(wrap_pyfunction!(get_item_and_run_callback, m)?)?;
    Ok(())
}
