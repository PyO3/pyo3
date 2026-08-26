use pyo3::prelude::*;

#[pymodule]
//~^ ERROR: `usize` is not a suitable return value for `#[pymodule_init]` functions
mod module {
    #[pymodule_init]
    fn init() -> usize {
        0
    }
}

#[pymodule]
//~^ ERROR: `Result<usize, PyErr>` is not a suitable return value for `#[pymodule_init]` functions
mod module_result {
    use pyo3::prelude::*;

    #[pymodule_init]
    fn init() -> PyResult<usize> {
        Ok(0)
    }
}

fn main() {}
