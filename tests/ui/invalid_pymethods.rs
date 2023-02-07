use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[classattr]
    fn class_attr_with_args(foo: i32) {}
}

#[pymethods]
impl MyClass {
    #[classattr(foobar)]
    const CLASS_ATTR_WITH_ATTRIBUTE_ARG: i32 = 3;
}

#[pymethods]
impl MyClass {
    fn staticmethod_without_attribute() {}
}

#[pymethods]
impl MyClass {
    #[staticmethod]
    fn staticmethod_with_receiver(&self) {}
}

// FIXME: This currently doesn't fail
// #[pymethods]
// impl MyClass {
//     #[classmethod]
//     fn classmethod_with_receiver(&self) {}
// }

#[pymethods]
impl MyClass {
    #[getter(x)]
    fn getter_without_receiver() {}
}

#[pymethods]
impl MyClass {
    #[setter(x)]
    fn setter_without_receiver() {}
}

#[pymethods]
impl MyClass {
    #[pyo3(name = "__call__", text_signature = "()")]
    fn text_signature_on_call() {}
}

#[pymethods]
impl MyClass {
    #[getter(x)]
    #[pyo3(text_signature = "()")]
    fn text_signature_on_getter(&self) {}
}

#[pymethods]
impl MyClass {
    #[setter(x)]
    #[pyo3(text_signature = "()")]
    fn text_signature_on_setter(&self) {}
}

#[pymethods]
impl MyClass {
    #[classattr]
    #[pyo3(text_signature = "()")]
    fn text_signature_on_classattr() {}
}

#[pymethods]
impl MyClass {
    #[pyo3(text_signature = 1)]
    fn invalid_text_signature() {}
}

#[pymethods]
impl MyClass {
    #[pyo3(text_signature = "()")]
    #[pyo3(text_signature = None)]
    fn duplicate_text_signature() {}
}

#[pymethods]
impl MyClass {
    #[getter(x)]
    #[pyo3(signature = ())]
    fn signature_on_getter(&self) {}
}

#[pymethods]
impl MyClass {
    #[setter(x)]
    #[pyo3(signature = ())]
    fn signature_on_setter(&self) {}
}

#[pymethods]
impl MyClass {
    #[classattr]
    #[pyo3(signature = ())]
    fn signature_on_classattr() {}
}

#[pymethods]
impl MyClass {
    #[classattr]
    #[staticmethod]
    fn multiple_method_types() {}
}

#[pymethods]
impl MyClass {
    fn generic_method<T>(value: T) {}
}

#[pymethods]
impl MyClass {
    fn impl_trait_method_first_arg(impl_trait: impl AsRef<PyAny>) {}
}

#[pymethods]
impl MyClass {
    fn impl_trait_method_second_arg(&self, impl_trait: impl AsRef<PyAny>) {}
}

#[pymethods]
impl MyClass {
    async fn async_method(&self) {}
}

#[pymethods]
impl MyClass {
    #[pyo3(pass_module)]
    fn method_cannot_pass_module(&self, m: &PyModule) {}
}

#[pymethods]
impl MyClass {
    fn method_self_by_value(self) {}
}

struct TwoNew {}

#[pymethods]
impl TwoNew {
    #[new]
    fn new_1() -> Self {
        Self {}
    }

    #[new]
    fn new_2() -> Self {
        Self {}
    }
}

struct DuplicateMethod {}

#[pymethods]
impl DuplicateMethod {
    #[pyo3(name = "func")]
    fn func_a(&self) {}

    #[pyo3(name = "func")]
    fn func_b(&self) {}
}

fn main() {}
