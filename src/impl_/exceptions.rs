use crate::{
    sync::GILOnceCell,
    types::{PyAnyMethods, PyTracebackMethods, PyType},
    Bound, Py, Python,
};

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
            .get_or_init(py, || {
                let imp = py.import_bound(self.module).unwrap_or_else(|err| {
                    let traceback = err
                        .traceback_bound(py)
                        .map(|tb| tb.format().expect("raised exception will have a traceback"))
                        .unwrap_or_default();
                    panic!(
                        "Can not import module {}: {}\n{}",
                        self.module, err, traceback
                    );
                });
                let cls = imp.getattr(self.name).unwrap_or_else(|_| {
                    panic!(
                        "Can not load exception class: {}.{}",
                        self.module, self.name
                    )
                });

                cls.extract()
                    .expect("Imported exception should be a type object")
            })
            .bind(py)
    }
}
