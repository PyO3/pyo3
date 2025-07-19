#![cfg(all(feature = "macros", not(any(PyPy, GraalPy))))]

use pyo3::{prelude::*, types::PySuper};

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

    fn method<'py>(self_: &Bound<'py, Self>) -> PyResult<Bound<'py, PyAny>> {
        let super_ = self_.py_super()?;
        super_.call_method("method", (), None)
    }

    fn method_super_new<'py>(self_: &Bound<'py, Self>) -> PyResult<Bound<'py, PyAny>> {
        let super_ = PySuper::new(&self_.get_type(), self_)?;
        super_.call_method("method", (), None)
    }
}

#[test]
fn test_call_super_method() {
    Python::attach(|py| {
        let cls = py.get_type::<SubClass>();
        pyo3::py_run!(
            py,
            cls,
            r#"
        obj = cls()
        assert obj.method() == 10
        assert obj.method_super_new() == 10
    "#
        )
    });
}
