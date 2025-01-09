use pyo3::{IntoPyObject, IntoPyObjectRef};

#[derive(IntoPyObject)]
struct InvalidIntoPyWithFn {
    #[pyo3(into_py_with = into)]
    inner: String,
}

#[derive(IntoPyObjectRef)]
struct InvalidIntoPyWithRefFn {
    #[pyo3(into_py_with_ref = into_ref)]
    inner: String,
}

fn into(_a: String) -> pyo3::Py<pyo3::PyAny> {
    todo!()
}

fn into_ref(_a: String, _py: pyo3::Python<'_>) -> pyo3::Bound<'_, pyo3::PyAny> {
    todo!()
}

fn main() {}
