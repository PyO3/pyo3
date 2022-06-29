#![cfg(feature = "macros")]

use pyo3::prelude::*;

#[pyclass(subclass)]
struct BaseClass {
    val1: usize,
}

#[pymethods]
impl BaseClass {
    #[new]
    fn new() -> Self {
        BaseClass { val1: 10 }
    }

    pub fn method(&self) -> usize {
        self.val1
    }
}

#[pyclass(extends=BaseClass)]
struct SubClass {}

#[pymethods]
impl SubClass {
    #[new]
    fn new() -> (Self, BaseClass) {
        (SubClass {}, BaseClass::new())
    }

    fn method<'a>(self_: PyRef<'_, Self>, py: Python<'a>) -> PyResult<&'a PyAny> {
        let any: Py<PyAny> = self_.into_py(py);
        let super_ = any.into_ref(py).py_super()?;
        super_.call_method("method", (), None)
    }
}

#[test]
fn test_call_super_method() {
    Python::with_gil(|py| {
        let cls = py.get_type::<SubClass>();
        pyo3::py_run!(
            py,
            cls,
            r#"
        obj = cls()
        assert obj.method() == 10
    "#
        )
    });
}
