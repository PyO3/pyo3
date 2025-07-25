#![cfg(not(Py_LIMITED_API))]

use pyo3::{prelude::*, types::PyDate};
use tempfile::Builder;

#[test]
#[should_panic(expected = "module 'datetime' has no attribute 'datetime_CAPI'")]
fn test_bad_datetime_module_panic() {
    // Create an empty temporary directory
    // with an empty "datetime" module which we'll put on the sys.path
    let tmpdir = Builder::new()
        .prefix("pyo3_test_data_check")
        .tempdir()
        .unwrap();
    std::fs::File::create(tmpdir.path().join("datetime.py")).unwrap();

    Python::attach(|py: Python<'_>| {
        let sys = py.import("sys").unwrap();
        sys.getattr("path")
            .unwrap()
            .call_method1("insert", (0, tmpdir.path().as_os_str()))
            .unwrap();

        // This should panic because the "datetime" module is empty
        PyDate::new(py, 2018, 1, 1).unwrap();
    });
    tmpdir.close().unwrap();
}
