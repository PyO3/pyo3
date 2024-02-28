use pyo3::prelude::*;

#[pyclass]
#[derive(Default)]
pub struct ObjStore {
    obj: Vec<PyObject>,
}

#[pymethods]
impl ObjStore {
    #[new]
    fn new() -> Self {
        ObjStore::default()
    }

    fn push(&mut self, py: Python<'_>, obj: &PyAny) {
        self.obj.push(obj.to_object(py));
    }
}

#[pymodule]
pub fn objstore(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ObjStore>()
}
