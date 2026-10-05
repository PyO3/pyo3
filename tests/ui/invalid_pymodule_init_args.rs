use pyo3::prelude::*;

#[pymodule]
mod module {
    use pyo3::prelude::*;

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>, _extra: usize) -> PyResult<()> {
//~^ ERROR: `#[pymodule_init]` takes an optional `Python` argument followed by an optional module argument
        Ok(())
    }
}

#[pymodule]
mod module_with_python_last {
    use pyo3::prelude::*;

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>, _py: Python<'_>) -> PyResult<()> {
//~^ ERROR: `#[pymodule_init]` takes an optional `Python` argument followed by an optional module argument
        Ok(())
    }
}

fn main() {}
