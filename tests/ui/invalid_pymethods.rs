use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[classattr]
    fn class_attr_with_args(_foo: i32) {}
    //~^ ERROR: #[classattr] can only have one argument (of type pyo3::Python)

    #[classattr(foobar)]
    //~^ ERROR: `#[classattr]` does not take any arguments
    const CLASS_ATTR_WITH_ATTRIBUTE_ARG: i32 = 3;

    fn staticmethod_without_attribute() {}
    //~^ ERROR: static method needs #[staticmethod] attribute

    #[staticmethod]
    fn staticmethod_with_receiver(&self) {}
    //~^ ERROR: unexpected receiver

    #[classmethod]
    fn classmethod_with_receiver(&self) {}
    //~^ ERROR: Expected `&Bound<PyType>` or `Py<PyType>` as the first argument to `#[classmethod]`

    #[classmethod]
    fn classmethod_missing_argument() -> Self {
        //~^ ERROR: Expected `&Bound<PyType>` or `Py<PyType>` as the first argument to `#[classmethod]`
        Self {}
    }
}

struct NotATypeObject;

#[pymethods]
impl MyClass {
    #[classmethod]
    fn classmethod_wrong_first_argument(_t: NotATypeObject) -> Self {
        //~^ ERROR: the trait bound `NotATypeObject: From<&pyo3::Bound<'_, PyType>>` is not satisfied
        Self {}
    }
}

#[pymethods]
impl MyClass {
    #[getter(x)]
    fn getter_without_receiver() {}
    //~^ ERROR: expected receiver for `#[getter]`
}

#[pymethods]
impl MyClass {
    #[setter(x)]
    fn setter_without_receiver() {}
    //~^ ERROR: expected receiver for `#[setter]`
}

#[pymethods]
impl MyClass {
    #[pyo3(name = "__call__", text_signature = "()")]
    fn text_signature_on_call() {}
    //~^ ERROR: static method needs #[staticmethod] attribute

    #[getter(x)]
    #[pyo3(text_signature = "()")]
    //~^ ERROR: `text_signature` not allowed with `getter`
    fn text_signature_on_getter(&self) {}

    #[setter(x)]
    #[pyo3(text_signature = "()")]
    //~^ ERROR: `text_signature` not allowed with `setter`
    fn text_signature_on_setter(&self) {}

    #[classattr]
    #[pyo3(text_signature = "()")]
    //~^ ERROR: `text_signature` not allowed with `classattr`
    fn text_signature_on_classattr() {}

    #[pyo3(text_signature = 1)]
    //~^ ERROR: expected a string literal or `None`
    fn invalid_text_signature() {}

    #[pyo3(text_signature = "()")]
    #[pyo3(text_signature = None)]
    //~^ ERROR: `text_signature` may only be specified once
    fn duplicate_text_signature() {}
}

#[pymethods]
impl MyClass {
    #[getter(x)]
    #[pyo3(signature = ())]
    //~^ ERROR: `signature` not allowed with `getter`
    fn signature_on_getter(&self) {}

    #[setter(x)]
    #[pyo3(signature = ())]
    //~^ ERROR: `signature` not allowed with `setter`
    fn signature_on_setter(&self) {}

    #[classattr]
    #[pyo3(signature = ())]
    //~^ ERROR: `signature` not allowed with `classattr`
    fn signature_on_classattr() {}
}

#[pymethods]
impl MyClass {
    #[new]
    //~^ ERROR: `#[new]` may not be combined with `#[classmethod]` `#[staticmethod]`, `#[classattr]`, `#[getter]`, and `#[setter]`
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
    //~^ ERROR: `#[new]` does not take any arguments
    fn new_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    #[new = ()] // in this form there's no suggestion to move arguments to `#[pyo3()]` attribute
                //~^ ERROR: `#[new]` does not take any arguments
    fn new_takes_no_arguments_nv(&self) {}
}

#[pymethods]
impl MyClass {
    #[classmethod(signature = ())]
    //~^ ERROR: `#[classmethod]` does not take any arguments
    fn classmethod_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    #[staticmethod(signature = ())]
    //~^ ERROR: `#[staticmethod]` does not take any arguments
    fn staticmethod_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    #[classattr(signature = ())]
    //~^ ERROR: `#[classattr]` does not take any arguments
    fn classattr_takes_no_arguments(&self) {}
}

#[pymethods]
impl MyClass {
    fn generic_method<T>(_value: T) {}
    //~^ ERROR: Python functions cannot have generic type parameters
}

#[pymethods]
impl MyClass {
    fn impl_trait_method_first_arg(_impl_trait: impl AsRef<PyAny>) {}
    //~^ ERROR: Python functions cannot have `impl Trait` arguments

    fn impl_trait_method_second_arg(&self, _impl_trait: impl AsRef<PyAny>) {}
    //~^ ERROR: Python functions cannot have `impl Trait` arguments
}

#[pymethods]
impl MyClass {
    #[pyo3(pass_module)]
    //~^ ERROR: `pass_module` cannot be used on Python methods
    fn method_cannot_pass_module(&self, _m: &PyModule) {}
}

#[pymethods]
impl MyClass {
    fn method_self_by_value(self) {}
    //~^ ERROR: Python objects are shared, so 'self' cannot be moved out of the Python interpreter.
}

macro_rules! macro_invocation {
    () => {};
}

#[pymethods]
impl MyClass {
    macro_invocation!();
    //~^ ERROR: macros cannot be used as items in `#[pymethods]` impl blocks
}

#[pymethods]
impl MyClass {
    #[staticmethod]
    //~^ ERROR: `#[staticmethod]` may not be combined with `#[classmethod]`
    #[classmethod]
    fn multiple_errors_static_and_class_method() {}

    #[staticmethod]
    fn multiple_errors_staticmethod_with_receiver(&self) {}
    //~^ ERROR: unexpected receiver

    #[classmethod]
    fn multiple_errors_classmethod_with_receiver(&self) {}
    //~^ ERROR: Expected `&Bound<PyType>` or `Py<PyType>` as the first argument to `#[classmethod]`
}

fn main() {}
