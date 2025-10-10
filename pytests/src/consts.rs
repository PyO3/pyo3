use pyo3::pymodule;

#[pymodule]
pub mod consts {
    use pyo3::{pyclass, pymethods};

    #[pymodule_export]
    pub const PI: f64 = std::f64::consts::PI; // Exports PI constant as part of the module

    #[pymodule_export]
    pub const SIMPLE: &str = "SIMPLE";

    #[pyclass]
    struct ClassWithConst {}

    #[pymethods]
    impl ClassWithConst {
        #[classattr]
        const INSTANCE: Self = ClassWithConst {};
    }
}
