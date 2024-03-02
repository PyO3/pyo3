use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

mod submodule;

#[pyclass]
struct ExampleClass {
    #[pyo3(get, set)]
    value: i32,
}

#[pymethods]
impl ExampleClass {
    #[new]
    pub fn new(value: i32) -> Self {
        ExampleClass { value }
    }
}

/// An example module implemented in Rust using PyO3.
#[pymodule]
fn _setuptools_rust_starter(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<ExampleClass>()?;
    m.add_wrapped(wrap_pymodule!(submodule::submodule))?;

    // Inserting to sys.modules allows importing submodules nicely from Python
    // e.g. from setuptools_rust_starter.submodule import SubmoduleClass

    let sys = PyModule::import_bound(py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.downcast_into()?;
    sys_modules.set_item("setuptools_rust_starter.submodule", m.getattr("submodule")?)?;

    Ok(())
}
