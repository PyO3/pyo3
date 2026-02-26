use pyo3::prelude::*;

#[pyclass]
struct ClassWithBadGetterSignature {
    foo: usize,
    bar: usize,
}
#[pymethods]
impl ClassWithBadGetterSignature {
    #[getter]
    #[pyo3(signature = (extra_arg:"int"))]
    fn get_foo(&self) -> usize {
        self.foo
    }
}

#[pyclass]
struct ClassWithMismatchedSetterSignature {
    foo: usize,
    bar: usize,
}
#[pymethods]
impl ClassWithMismatchedSetterSignature {
    #[getter]
    #[pyo3(signature = (extra_arg:"int"))]
    fn set_foo(&mut self, value: usize) {
        self.foo = value;
    }
}

#[pyclass]
struct ClassWithMissingSetterSignature {
    foo: usize,
    bar: usize,
}
#[pymethods]
impl ClassWithMissingSetterSignature {
    #[getter]
    #[pyo3(signature = ())]
    fn set_foo(&mut self, value: usize) {
        self.foo = value;
    }
}

#[pyclass]
struct ClassWithExtraSetterSignature {
    foo: usize,
    bar: usize,
}
#[pymethods]
impl ClassWithExtraSetterSignature {
    #[getter]
    #[pyo3(signature = (value:"int", extra_arg:"int"))]
    fn set_foo(&mut self, value: usize) {
        self.foo = value;
    }
}

fn main() {}
