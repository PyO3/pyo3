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

#[pymethods]
impl MyClass {
    #[classmethod]
    fn classmethod_with_receiver(&self) {}
}

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
    #[new]
    #[classmethod]
    #[staticmethod]
    #[classattr]
    #[getter(x)]
    #[setter(x)]
    fn multiple_method_types() {}
}

#[pymethods]
impl MyClass {
    #[new(signature = ())]
    fn new_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    #[new = ()] // in this form there's no suggestion to move arguments to `#[pyo3()]` attribute
    fn new_takes_no_arguments_nv(&self) {}
}

#[pymethods]
impl MyClass {
    #[classmethod(signature = ())]
    fn classmethod_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    #[staticmethod(signature = ())]
    fn staticmethod_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    #[classattr(signature = ())]
    fn classattr_takes_no_arguments(&self) {}
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

macro_rules! macro_invocation {
    () => {};
}

#[pymethods]
impl MyClass {
    macro_invocation!();
}

fn main() {}
