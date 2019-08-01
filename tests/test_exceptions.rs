use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::wrap_pyfunction;
use std::fs::File;

mod common;

#[pyfunction]
fn fail_to_open_file() -> PyResult<()> {
    File::open("not_there.txt")?;
    Ok(())
}

#[test]
fn test_filenotfounderror() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let fail_to_open_file = wrap_pyfunction!(fail_to_open_file)(py);
    let d = [("fail_to_open_file", fail_to_open_file)].into_py_dict(py);
    match py.run("fail_to_open_file()", None, Some(d)) {
        Ok(()) => panic!("Call should raise a FileNotFoundError"),
        Err(e) => {
            py_assert!(py, e, "isinstance(e, FileNotFoundError)");
            py_assert!(py, e, "'No such file or directory (os error 2)' == str(e)");
        }
    };
}
