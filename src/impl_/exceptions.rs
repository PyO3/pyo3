use crate::{sync::GILOnceCell, types::PyType, Bound, Py, Python};

pub struct ImportedExceptionTypeObject {
    imported_value: GILOnceCell<Py<PyType>>,
    module: &'static str,
    name: &'static str,
}

impl ImportedExceptionTypeObject {
    pub const fn new(module: &'static str, name: &'static str) -> Self {
        Self {
            imported_value: GILOnceCell::new(),
            module,
            name,
        }
    }

    pub fn get<'py>(&self, py: Python<'py>) -> &Bound<'py, PyType> {
        self.imported_value
            .get_or_try_init_type_ref(py, self.module, self.name)
            .unwrap_or_else(|e| {
                panic!(
                    "failed to import exception {}.{}: {}",
                    self.module, self.name, e
                )
            })
    }
}
