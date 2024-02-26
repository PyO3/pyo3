use pyo3::prelude::*;

#[pymodule]
mod module {
    #[pymodule_init]
    fn init(m: &PyModule) -> PyResult<()> {
        Ok(())
    }

    #[pymodule_init]
    fn init2(m: &PyModule) -> PyResult<()> {
        Ok(())
    }
}

fn main() {}
