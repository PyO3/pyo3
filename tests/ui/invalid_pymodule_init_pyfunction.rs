use pyo3::prelude::*;

#[pymodule]
mod module {
    use pyo3::prelude::*;

    #[pymodule_init]
    #[pyfunction]
//~^ ERROR: `#[pyfunction]` cannot be used alongside `#[pymodule_init]`
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}

fn main() {}
