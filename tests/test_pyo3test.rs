#![cfg(not(any(PyPy, GraalPy, Py_3_7, Py_3_8, all(windows, Py_LIMITED_API, Py_3_9))))]
use pyo3::{prelude::*, types::PyDict};

// The example from the Guide ...
fn o3_addone(num: isize) -> isize {
    num + 1
}

#[pyfunction]
#[pyo3(name = "addone")]
fn py_addone(num: isize) -> isize {
    o3_addone(num)
}

#[pymodule]
#[pyo3(name = "adders")]
fn py_adders(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(py_addone, module)?)?;
    Ok(())
}

// This is how the test would be written WITHOUT using the pyo3test macro. This validates that
// adders.addone is correctly constructed.
#[test]
fn test_pyo3test_without_macro() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let sys = PyModule::import_bound(py, "sys").unwrap();
        let py_modules: Bound<'_, PyDict> =
            sys.getattr("modules").unwrap().downcast_into().unwrap();
        let py_adders_pymodule = unsafe { Bound::from_owned_ptr(py, py_adders::__pyo3_init()) };
        py_modules
            .set_item("adders", py_adders_pymodule)
            .expect("Failed to import adders");
        let adders = py_modules.get_item("adders").unwrap().unwrap();
        let addone = adders
            .getattr("addone")
            .expect("Failed to get addone function");
        let result: PyResult<isize> = match addone.call1((1_isize,)) {
            Ok(r) => r.extract(),
            Err(e) => Err(e),
        };
        let result = result.unwrap();
        let expected_result = 2_isize;
        assert_eq!(result, expected_result);
    });
}

// ... and this is how the test can be written using the pyo3test macro and pyo3import attribute
#[pyo3test]
#[pyo3import(py_adders: from adders import addone)]
fn test_pyo3test_simple_case() {
    let result: PyResult<isize> = match addone.call1((1_isize,)) {
        Ok(r) => r.extract(),
        Err(e) => Err(e),
    };
    let result = result.unwrap();
    let expected_result = 2_isize;
    assert_eq!(result, expected_result);
}

#[pyo3test]
#[pyo3import(py_adders: import adders)]
fn test_pyo3test_import_module_only() {
    let result: isize = adders
        .getattr("addone")
        .unwrap()
        .call1((1_isize,))
        .unwrap()
        .extract()
        .unwrap();
    let expected_result = 2_isize;
    assert_eq!(result, expected_result);
}
