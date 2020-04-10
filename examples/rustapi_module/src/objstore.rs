use pyo3::prelude::*;

#[pyclass]
#[derive(Default)]
pub struct ObjStore {
    obj: Vec<Py<PyAny>>,
}

#[pymethods]
impl ObjStore {
    #[new]
    fn new() -> Self {
        ObjStore::default()
    }

    fn push(&mut self, py: Python, obj: &PyAny) {
        self.obj.push(obj.to_object(py).into());
    }
}

#[pymodule]
fn objstore(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<ObjStore>()
}
