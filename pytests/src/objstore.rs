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

    fn push(&mut self, obj: &Bound<'_, PyAny>) {
        self.obj.push(obj.clone().unbind());
    }
}

#[pymodule(supports_free_threaded = true)]
pub fn objstore(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ObjStore>()
}
