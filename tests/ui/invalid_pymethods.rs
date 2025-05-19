use pyo3::prelude::*;

macro_rules! macro_invocation {
    () => {};
}

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[classattr]
    fn class_attr_with_args(_foo: i32) {}

    #[classattr(foobar)]
    const CLASS_ATTR_WITH_ATTRIBUTE_ARG: i32 = 3;

    fn staticmethod_without_attribute() {}

    #[staticmethod]
    fn staticmethod_with_receiver(&self) {}

    #[classmethod]
    fn classmethod_with_receiver(&self) {}

    #[classmethod]
    fn classmethod_missing_argument() -> Self {
        Self {}
    }

    #[classmethod]
    fn classmethod_wrong_first_argument(_x: i32) -> Self {
        Self {}
    }

    #[getter(x)]
    fn getter_without_receiver() {}

    #[setter(x)]
    fn setter_without_receiver() {}

    #[pyo3(name = "__call__", text_signature = "()")]
    fn text_signature_on_call() {}

    #[getter(x)]
    #[pyo3(text_signature = "()")]
    fn text_signature_on_getter(&self) {}

    #[setter(x)]
    #[pyo3(text_signature = "()")]
    fn text_signature_on_setter(&self) {}

    #[classattr]
    #[pyo3(text_signature = "()")]
    fn text_signature_on_classattr() {}

    #[pyo3(text_signature = 1)]
    fn invalid_text_signature() {}

    #[pyo3(text_signature = "()")]
    #[pyo3(text_signature = None)]
    fn duplicate_text_signature() {}

    #[getter(x)]
    #[pyo3(signature = ())]
    fn signature_on_getter(&self) {}

    #[setter(x)]
    #[pyo3(signature = ())]
    fn signature_on_setter(&self) {}

    #[classattr]
    #[pyo3(signature = ())]
    fn signature_on_classattr() {}

    #[new]
    #[classmethod]
    #[staticmethod]
    #[classattr]
    #[getter(x)]
    #[setter(x)]
    fn multiple_method_types() {}

    #[new(signature = ())]
    fn new_takes_no_arguments(&self) {}

    #[new = ()] // in this form there's no suggestion to move arguments to `#[pyo3()]` attribute
    fn new_takes_no_arguments_nv(&self) {}

    #[classmethod(signature = ())]
    fn classmethod_takes_no_arguments(&self) {}

    #[staticmethod(signature = ())]
    fn staticmethod_takes_no_arguments(&self) {}

    #[classattr(signature = ())]
    fn classattr_takes_no_arguments(&self) {}

    fn generic_method<T>(_value: T) {}

    fn impl_trait_method_first_arg(_impl_trait: impl AsRef<PyAny>) {}

    fn impl_trait_method_second_arg(&self, _impl_trait: impl AsRef<PyAny>) {}

    #[pyo3(pass_module)]
    fn method_cannot_pass_module(&self, _m: &PyModule) {}

    fn method_self_by_value(self) {}

    macro_invocation!();
}

fn main() {}
