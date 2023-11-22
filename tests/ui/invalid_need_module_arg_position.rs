use pyo3::prelude::*;

#[pymodule]
fn module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, pass_module)]
    fn fail<'py>(string: &str, module: &'py PyModule) -> PyResult<&'py str> {
        module.name()
    }
    Ok(())
}

fn main() {}
