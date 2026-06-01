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
//~^ ERROR: only one `#[pymodule_init]` may be specified
        Ok(())
    }
}

fn main() {}
