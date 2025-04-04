use pyo3::{IntoPyObject, IntoPyObjectRef};

#[derive(IntoPyObject, IntoPyObjectRef)]
struct InvalidIntoPyWithFn {
    #[pyo3(into_py_with = into)]
    inner: String,
}

fn into(_a: String, _py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Bound<'_, pyo3::PyAny>> {
    todo!()
}

fn main() {}
