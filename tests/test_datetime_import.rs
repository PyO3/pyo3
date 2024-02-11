#![cfg(not(Py_LIMITED_API))]

use pyo3::{prelude::PyAnyMethods, types::PyDate, Python};

#[test]
#[should_panic(expected = "module 'datetime' has no attribute 'datetime_CAPI'")]
fn test_bad_datetime_module_panic() {
    // Create an empty temporary directory
    // with an empty "datetime" module which we'll put on the sys.path
    let tmpdir = std::env::temp_dir();
    let tmpdir = tmpdir.join("pyo3_test_date_check");
    let _ = std::fs::remove_dir_all(&tmpdir);
    std::fs::create_dir(&tmpdir).unwrap();
    std::fs::File::create(tmpdir.join("datetime.py")).unwrap();

    Python::with_gil(|py: Python<'_>| {
        let sys = py.import_bound("sys").unwrap();
        sys.getattr("path")
            .unwrap()
            .call_method1("insert", (0, tmpdir))
            .unwrap();

        // This should panic because the "datetime" module is empty
        PyDate::new_bound(py, 2018, 1, 1).unwrap();
    });
}
