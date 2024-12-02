#![forbid(unsafe_code)]
#![forbid(unsafe_op_in_unsafe_fn)]

use pyo3::*;

#[allow(unexpected_cfgs)]
#[path = "../../src/tests/hygiene/mod.rs"]
mod hygiene;

mod gh_4394 {
    use pyo3::prelude::*;

    #[derive(Eq, Ord, PartialEq, PartialOrd, Clone)]
    #[pyclass(get_all)]
    pub struct VersionSpecifier {
        pub(crate) operator: Operator,
        pub(crate) version: Version,
    }

    #[derive(Eq, Ord, PartialEq, PartialOrd, Debug, Hash, Clone, Copy)]
    #[pyo3::pyclass(eq, eq_int)]
    pub enum Operator {
        Equal,
    }

    #[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
    #[pyclass]
    pub struct Version;
}

mod from_py_with {
    use pyo3::prelude::*;
    use pyo3::types::PyBytes;

    fn bytes_from_py(bytes: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
        Ok(bytes.downcast::<PyBytes>()?.as_bytes().to_vec())
    }

    #[pyfunction]
    fn f(#[pyo3(from_py_with = "bytes_from_py")] _bytes: Vec<u8>) {}
}

fn main() {}
