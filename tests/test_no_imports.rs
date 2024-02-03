//! Tests that various macros work correctly without any PyO3 imports.

#![cfg(feature = "macros")]

use pyo3::IntoPy;

#[pyo3::pyfunction]
#[pyo3(name = "identity", signature = (x = None))]
fn basic_function(py: pyo3::Python<'_>, x: Option<pyo3::PyObject>) -> pyo3::PyObject {
    x.unwrap_or_else(|| py.None().into_py(py))
}

#[pyo3::pymodule]
fn basic_module(_py: pyo3::Python<'_>, m: &pyo3::types::PyModule) -> pyo3::PyResult<()> {
    #[pyfn(m)]
    fn answer() -> usize {
        42
    }

    m.add_function(pyo3::wrap_pyfunction!(basic_function, m)?)?;

    Ok(())
}

#[pyo3::pyclass]
struct BasicClass {
    #[pyo3(get)]
    v: usize,
    #[pyo3(get, set)]
    s: String,
}

#[pyo3::pymethods]
impl BasicClass {
    #[classattr]
    const OKAY: bool = true;

    #[new]
    fn new(arg: &pyo3::PyAny) -> pyo3::PyResult<Self> {
        if let Ok(v) = arg.extract::<usize>() {
            Ok(Self {
                v,
                s: "".to_string(),
            })
        } else {
            Ok(Self {
                v: 0,
                s: arg.extract()?,
            })
        }
    }

    #[getter]
    fn get_property(&self) -> usize {
        self.v * 100
    }

    #[setter]
    fn set_property(&mut self, value: usize) {
        self.v = value / 100
    }

    /// Some documentation here
    #[classmethod]
    fn classmethod(cls: &pyo3::types::PyType) -> &pyo3::types::PyType {
        cls
    }

    #[staticmethod]
    fn staticmethod(py: pyo3::Python<'_>, v: usize) -> pyo3::Py<pyo3::PyAny> {
        use pyo3::IntoPy;
        v.to_string().into_py(py)
    }

    fn __add__(&self, other: usize) -> usize {
        self.v + other
    }

    fn __iadd__(&mut self, other: pyo3::PyRef<'_, Self>) {
        self.v += other.v;
        self.s.push_str(&other.s);
    }

    fn mutate(mut slf: pyo3::PyRefMut<'_, Self>) {
        slf.v += slf.v;
        slf.s.push('!');
    }
}

#[test]
fn test_basic() {
    pyo3::Python::with_gil(|py| {
        let module = pyo3::wrap_pymodule!(basic_module)(py);
        let cls = py.get_type::<BasicClass>();
        let d = pyo3::types::IntoPyDict::into_py_dict(
            [
                ("mod", module.as_ref(py).as_ref()),
                ("cls", cls.as_ref()),
                ("a", cls.call1((8,)).unwrap()),
                ("b", cls.call1(("foo",)).unwrap()),
            ],
            py,
        );

        pyo3::py_run!(py, *d, "assert mod.answer() == 42");
        pyo3::py_run!(py, *d, "assert mod.identity() is None");
        pyo3::py_run!(py, *d, "v = object(); assert mod.identity(v) is v");
        pyo3::py_run!(py, *d, "assert cls.OKAY");
        pyo3::py_run!(py, *d, "assert (a.v, a.s) == (8, '')");
        pyo3::py_run!(py, *d, "assert (b.v, b.s) == (0, 'foo')");
        pyo3::py_run!(py, *d, "b.property = 314");
        pyo3::py_run!(py, *d, "assert b.property == 300");
        pyo3::py_run!(
            py,
            *d,
            "assert cls.classmethod.__doc__ == 'Some documentation here'"
        );
        pyo3::py_run!(py, *d, "assert cls.classmethod() is cls");
        pyo3::py_run!(py, *d, "assert cls.staticmethod(5) == '5'");
        pyo3::py_run!(py, *d, "a.s = 'bar'; assert a.s == 'bar'");
        pyo3::py_run!(py, *d, "a.mutate(); assert (a.v, a.s) == (16, 'bar!')");
        pyo3::py_run!(py, *d, "assert a + 9 == 25");
        pyo3::py_run!(py, *d, "b += a; assert (b.v, b.s) == (19, 'foobar!')");
    });
}

#[pyo3::pyclass]
struct NewClassMethod {
    #[pyo3(get)]
    cls: pyo3::PyObject,
}

#[pyo3::pymethods]
impl NewClassMethod {
    #[new]
    #[classmethod]
    fn new(cls: &pyo3::types::PyType) -> Self {
        Self { cls: cls.into() }
    }
}

#[test]
fn test_new_class_method() {
    pyo3::Python::with_gil(|py| {
        let cls = py.get_type::<NewClassMethod>();
        pyo3::py_run!(py, cls, "assert cls().cls is cls");
    });
}
