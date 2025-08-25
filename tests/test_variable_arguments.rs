#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

mod test_utils;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[staticmethod]
    #[pyo3(signature = (*args))]
    fn test_args(args: Bound<'_, PyTuple>) -> Bound<'_, PyTuple> {
        args
    }

    #[staticmethod]
    #[pyo3(signature = (**kwargs))]
    fn test_kwargs(kwargs: Option<Bound<'_, PyDict>>) -> Option<Bound<'_, PyDict>> {
        kwargs
    }
}

#[test]
fn variable_args() {
    Python::attach(|py| {
        let my_obj = py.get_type::<MyClass>();
        py_assert!(py, my_obj, "my_obj.test_args() == ()");
        py_assert!(py, my_obj, "my_obj.test_args(1) == (1,)");
        py_assert!(py, my_obj, "my_obj.test_args(1, 2) == (1, 2)");
    });
}

#[test]
fn variable_kwargs() {
    Python::attach(|py| {
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
