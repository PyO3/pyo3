use pyo3::prelude::*;

#[pyclass]
#[derive(Default)]
pub struct ObjStore {
    obj: Vec<Py<PyObject>>,
}

#[pymethods]
impl ObjStore {
    #[new]
    fn new() -> Self {
        ObjStore::default()
    }

    fn push(&mut self, obj: &PyObject) {
        self.obj.push(obj.into());
    }
}

#[pymodule]
fn objstore(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<ObjStore>()
}
