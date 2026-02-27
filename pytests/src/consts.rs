use pyo3::pymodule;

#[pymodule]
pub mod consts {
    use pyo3::{pyclass, pymethods};

    /// Exports PI constant as part of the module
    #[pymodule_export]
    pub const PI: f64 = std::f64::consts::PI;

    /// We experiment with "escaping"
    #[pymodule_export]
    pub const ESCAPING: &str = "S\0\x01\t\n\r\"'\\";

    #[pyclass]
    struct ClassWithConst {}

    #[pymethods]
    impl ClassWithConst {
        /// A constant
        #[classattr]
        const INSTANCE: Self = ClassWithConst {};
    }
}
