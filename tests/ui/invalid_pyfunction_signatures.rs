use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

#[pyfunction]
#[pyo3(signature = ())]
//~^ ERROR: missing signature entry for argument `_x`
fn function_with_one_argument_empty_signature(_x: i32) {}

#[pyfunction]
#[pyo3(signature = (x))]
//~^ ERROR: signature entry does not have a corresponding function argument
fn function_with_one_entry_signature_no_args() {}

#[pyfunction]
#[pyo3(signature = (x))]
//~^ ERROR: expected argument from function definition `y` but got argument `x`
fn function_with_incorrect_argument_names(y: i32) {
    let _ = y;
}

#[pyfunction(x)]
//~^ ERROR: expected one of: `name`, `pass_module`, `signature`, `text_signature`, `crate`, `warn`
#[pyo3(signature = (x))]
fn function_with_both_args_and_signature(x: i32) {
    let _ = x;
}

#[pyfunction]
#[pyo3(signature = (*, *args))]
//~^ ERROR: `*args` not allowed after `*`
fn function_with_args_after_args_sep(args: &PyTuple) {
    let _ = args;
}

#[pyfunction]
#[pyo3(signature = (*, *))]
//~^ ERROR: `*` not allowed after `*`
fn function_with_args_sep_after_args_sep() {}

#[pyfunction]
#[pyo3(signature = (**kwargs, *args))]
//~^ ERROR: `*args` not allowed after `**kwargs`
fn function_with_args_after_kwargs(kwargs: Option<&PyDict>, args: &PyTuple) {
    let _ = args;
    let _ = kwargs;
}

#[pyfunction]
#[pyo3(signature = (**kwargs_a, **kwargs_b))]
//~^ ERROR: `**kwargs_b` not allowed after `**kwargs_a`
fn function_with_kwargs_after_kwargs(kwargs_a: Option<&PyDict>, kwargs_b: Option<&PyDict>) {
    let _ = kwargs_a;
    let _ = kwargs_b;
}

#[pyfunction(signature = (py))]
//~^ ERROR: arguments of type `Python` must not be part of the signature
fn signature_contains_py(py: Python<'_>) {
    let _ = py;
}

#[pyfunction(signature = (a: list[]))]
//~^ ERROR: subscripts must not be empty, use `[()]` for an empty tuple
fn annotation_empty_subscript(a: Bound<'_, PyAny>) {
    let _ = a;
}

#[pyfunction(signature = (a: typing.Literal[true]))]
//~^ ERROR: use the Python spelling `True` for booleans in type annotations
fn annotation_rust_bool(a: Bound<'_, PyAny>) {
    let _ = a;
}

#[pyfunction(signature = (a: typing.Literal[1.5]))]
//~^ ERROR: float literals are not valid in type annotations
fn annotation_float(a: Bound<'_, PyAny>) {
    let _ = a;
}

#[pyfunction(signature = (a: typing.Literal[1u8]))]
//~^ ERROR: literal suffixes are not supported in type annotations
fn annotation_int_suffix(a: Bound<'_, PyAny>) {
    let _ = a;
}

#[pyfunction(signature = (a: typing.Literal["foo"bar]))]
//~^ ERROR: literal suffixes are not supported in type annotations
fn annotation_str_suffix(a: Bound<'_, PyAny>) {
    let _ = a;
}

#[pyfunction(signature = (a: typing.Literal[-"foo"]))]
//~^ ERROR: only integers can be negated in type annotations
fn annotation_negated_str(a: Bound<'_, PyAny>) {
    let _ = a;
}

#[pyfunction(signature = (a: typing.Literal['c']))]
//~^ ERROR: unsupported literal in type annotation, only strings and integers are supported
fn annotation_char(a: Bound<'_, PyAny>) {
    let _ = a;
}

#[pyfunction(signature = (a: int +))]
//~^ ERROR: expected `,`
fn annotation_unexpected_token(a: Bound<'_, PyAny>) {
    let _ = a;
}

#[pyclass]
struct MyClass;

#[pymethods]
impl MyClass {
    #[pyo3(signature = (**kwargs, *, *args, x))]
//~^ ERROR: expected argument from function definition `args` but got argument `kwargs`
    fn multiple_errors_same_order(kwargs: Option<&PyDict>, args: &PyTuple, x: i32) {
        let _ = kwargs;
        let _ = args;
        let _ = x;
    }

    #[pyo3(signature = (**kwargs, *, *args, x))]
//~^ ERROR: expected argument from function definition `x` but got argument `kwargs`
    fn multiple_errors_different_order(args: &PyTuple, x: i32, kwargs: Option<&PyDict>) {
        let _ = kwargs;
        let _ = args;
        let _ = x;
    }
}

fn main() {}
