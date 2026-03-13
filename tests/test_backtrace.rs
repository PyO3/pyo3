#![cfg(all(feature = "macros", not(Py_LIMITED_API)))]
use insta::assert_snapshot;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[test]
fn test_rust_frames_in_backtrace() {
    use pyo3::prelude::PyDictMethods;
    use pyo3::{pyfunction, types::PyDict, Python};

    #[pyfunction]
    fn produce_err_result() -> PyResult<()> {
        Err(PyValueError::new_err("Error result"))
    }

    Python::attach(|py| {
        let func = wrap_pyfunction!(produce_err_result)(py).unwrap();
        let globals = PyDict::new(py);
        globals.set_item("func", func).unwrap();

        let root_dir = format!("{:?}", std::env::current_dir().unwrap());

        let err = py
            .run(
                c"def python_func():\n  func()\n\npython_func()",
                Some(&globals),
                None,
            )
            .unwrap_err();

        let traceback = err.traceback(py).unwrap().format().unwrap();

        insta::with_settings!({
            snapshot_suffix => std::env::consts::FAMILY,
            filters => [
                (root_dir.trim_matches('"'), "."),
                #[cfg(unix)]
                ("(?:/[\\w\\-\\.]*)+/library/core/src", "[RUST_CORE]"),
                #[cfg(windows)]
                ("(?:(?:/rustc/\\w{40}/)|(?:[\\w\\-.:]*\\\\)+)library\\\\core\\\\src", "[RUST_CORE]"),
            ],
        }, {
            assert_snapshot!(traceback);
        });
    });
}
