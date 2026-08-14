use pyo3::prelude::*;

#[pymodule]
//~^ ERROR: the trait bound `usize: pyo3::impl_::pymodule::PyModuleInitResult` is not satisfied
mod module {
    #[pymodule_init]
    fn init() -> usize {
        0
    }
}

fn main() {}
