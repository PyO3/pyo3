use pyo3::prelude::*;

#[pymodule]
fn module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, pass_module)]
    fn fail(string: &str, module: &PyModule) -> PyResult<&str> {
        module.name()
    }
    Ok(())
}

fn main(){}
