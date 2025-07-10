use crate::{sync::PyOnceCell, types::PyType, Bound, Py, PyErr, Python};

pub struct ImportedExceptionTypeObject {
    imported_value: PyOnceCell<Py<PyType>>,
    module: &'static str,
    name: &'static str,
}

impl ImportedExceptionTypeObject {
    pub const fn new(module: &'static str, name: &'static str) -> Self {
        Self {
            imported_value: PyOnceCell::new(),
            module,
            name,
        }
    }

    pub fn get<'py>(&self, py: Python<'py>) -> &Bound<'py, PyType> {
        self.imported_value
            .import(py, self.module, self.name)
            .unwrap_or_else(|e: PyErr| {
                panic!(
                    "failed to import exception {}.{}: {}",
                    self.module, self.name, e
                )
            })
    }
}
