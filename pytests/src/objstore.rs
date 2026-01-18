use pyo3::prelude::*;

#[pymodule]
pub mod objstore {
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

        fn push(&mut self, obj: &Bound<'_, PyAny>) {
            self.obj.push(obj.clone().unbind());
        }
    }
}
