#![deny(unused_imports)]
use pyo3::prelude::*;

#[pymodule]
mod probe_no_fields {
    use pyo3::prelude::*;
    #[pyclass]
    pub struct Probe {}
    
    #[pymethods]
    impl Probe {
        #[new]
        fn new() -> Self {
            Self {}
        }
    }
}

#[pymodule]
mod probe_with_fields {
    use pyo3::prelude::*;
    #[pyclass(get_all)]
    pub struct Probe {
        field: u8,
    }
    
    #[pymethods]
    impl Probe {
        #[new]
        fn new() -> Self {
            Self { field: 0 }
        }
    }
}

#[pyclass]
struct Check5029();

macro_rules! impl_methods {
    ($name:ident) => {
        #[pymethods]
        impl Check5029 {
            fn $name(&self, _value: Option<&str>) -> PyResult<()> {
                Ok(())
            }
        }
    };
}

impl_methods!(some_method);

fn main() {}
