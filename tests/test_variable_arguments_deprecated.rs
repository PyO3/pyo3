#![cfg(feature = "macros")]
#![allow(deprecated)]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

mod common;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[staticmethod]
    #[args(args = "*")]
    fn test_args(args: &PyTuple) -> &PyTuple {
        args
    }

    #[staticmethod]
    #[args(kwargs = "**")]
    fn test_kwargs(kwargs: Option<&PyDict>) -> Option<&PyDict> {
        kwargs
    }
}

#[test]
fn variable_args() {
    Python::with_gil(|py| {
        let my_obj = py.get_type::<MyClass>();
        py_assert!(py, my_obj, "my_obj.test_args() == ()");
        py_assert!(py, my_obj, "my_obj.test_args(1) == (1,)");
        py_assert!(py, my_obj, "my_obj.test_args(1, 2) == (1, 2)");
    });
}

#[test]
fn variable_kwargs() {
    Python::with_gil(|py| {
        let my_obj = py.get_type::<MyClass>();
        py_assert!(py, my_obj, "my_obj.test_kwargs() == None");
        py_assert!(py, my_obj, "my_obj.test_kwargs(test=1) == {'test': 1}");
        py_assert!(
            py,
            my_obj,
            "my_obj.test_kwargs(test1=1, test2=2) == {'test1':1, 'test2':2}"
        );
    });
}
