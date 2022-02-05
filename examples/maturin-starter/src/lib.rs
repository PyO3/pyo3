use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

mod submodule;
use submodule::*;

#[derive(Clone, Debug, Eq, PartialEq)]
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
fn maturin_starter(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ExampleClass>()?;
    m.add_wrapped(wrap_pymodule!(submodule))?;

    // Inserting to sys.modules allows importing submodules nicely from Python
    // e.g. from maturin_starter.submodule import SubmoduleClass

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("maturin_starter.submodule", m.getattr("submodule")?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_class() {
        Python::with_gil(|py| {
            let class = Py::new(py, ExampleClass::new(5)).unwrap();
            assert_eq!(
                class.extract::<ExampleClass>(py).unwrap(),
                ExampleClass { value: 5 }
            );
        })
    }
}
