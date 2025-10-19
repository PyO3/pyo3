use pyo3::prelude::*;

#[pyclass]
struct ClassWithGetter {}

#[pymethods]
impl ClassWithGetter {
    #[getter]
    fn getter_with_arg(&self, _py: Python<'_>, _index: u32) {}
}

#[pyclass]
struct ClassWithSetter {}

#[pymethods]
impl ClassWithSetter {
    #[setter]
    fn setter_with_no_arg(&mut self, _py: Python<'_>) {}
}

#[pymethods]
impl ClassWithSetter {
    #[setter]
    fn setter_with_too_many_args(&mut self, _py: Python<'_>, _foo: u32, _bar: u32) {}
}

#[pyclass]
struct TupleGetterSetterNoName(#[pyo3(get, set)] i32);

#[pyclass]
struct MultipleGet(#[pyo3(get, get)] i32);

#[pyclass]
struct MultipleSet(#[pyo3(set, set)] i32);

#[pyclass]
struct MultipleName(#[pyo3(name = "foo", name = "bar")] i32);

#[pyclass]
struct NameWithoutGetSet(#[pyo3(name = "value")] i32);

#[pyclass]
struct InvalidGetterType {
    #[pyo3(get)]
    value: ::std::marker::PhantomData<i32>,
}

fn main() {}
