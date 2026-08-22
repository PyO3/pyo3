use pyo3::prelude::*;

#[pymodule]
mod module {
    use pyo3::prelude::*;

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>, _extra: usize) -> PyResult<()> {
//~^ ERROR: `#[pymodule_init]` takes either no argument or the module
        Ok(())
    }
}

fn main() {}
