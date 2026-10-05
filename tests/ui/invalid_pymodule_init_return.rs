use pyo3::prelude::*;

#[pymodule]
mod module {
    #[pymodule_init]
    fn init() -> usize {
    //~^ ERROR: `usize` is not a suitable return value for `#[pymodule_init]` functions
        0
    }
}

#[pymodule]
mod module_result {
    use pyo3::prelude::*;

    #[pymodule_init]
    fn init() -> PyResult<usize> {
    //~^ ERROR: `Result<usize, PyErr>` is not a suitable return value for `#[pymodule_init]` functions
        Ok(0)
    }
}

fn main() {}
