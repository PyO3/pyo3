use pyo3::prelude::*;

#[pymodule]
mod module {
    use pyo3::prelude::*;

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }

    #[pymodule_init]
    fn init2(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}

fn main() {}
