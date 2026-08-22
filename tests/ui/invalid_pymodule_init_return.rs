use pyo3::prelude::*;

#[pymodule]
//~^ ERROR: the trait bound `usize: pyo3::impl_::pymodule::PyModuleInitResult` is not satisfied
mod module {
    #[pymodule_init]
    fn init() -> usize {
        0
    }
}

#[pymodule]
//~^ ERROR: the trait bound `Result<usize, PyErr>: pyo3::impl_::pymodule::PyModuleInitResult` is not satisfied
mod module_result {
    use pyo3::prelude::*;

    #[pymodule_init]
    fn init() -> PyResult<usize> {
        Ok(0)
    }
}

fn main() {}
