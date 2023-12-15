#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyo3::{py_run, PyCell};

use std::fmt;

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass]
struct MutRefArg {
    n: i32,
}

#[pymethods]
impl MutRefArg {
    fn get(&self) -> i32 {
        self.n
    }
    fn set_other(&self, mut other: PyRefMut<'_, MutRefArg>) {
        other.n = 100;
    }
}

#[test]
fn mut_ref_arg() {
    Python::with_gil(|py| {
        let inst1 = PyDetached::new(py, MutRefArg { n: 0 }).unwrap();
        let inst2 = PyDetached::new(py, MutRefArg { n: 0 }).unwrap();

        py_run!(py, inst1 inst2, "inst1.set_other(inst2)");
        let inst2 = inst2.as_ref(py).borrow();
        assert_eq!(inst2.n, 100);
    });
}

#[pyclass]
struct PyUsize {
    #[pyo3(get)]
    pub value: usize,
}

#[pyfunction]
fn get_zero() -> PyUsize {
    PyUsize { value: 0 }
}

#[test]
/// Checks that we can use return a custom class in arbitrary function and use those functions
/// both in rust and python
fn return_custom_class() {
    Python::with_gil(|py| {
        // Using from rust
        assert_eq!(get_zero().value, 0);

        // Using from python
        let get_zero = wrap_pyfunction!(get_zero)(py).unwrap();
        py_assert!(py, get_zero, "get_zero().value == 0");
    });
}

#[test]
fn intopytuple_primitive() {
    Python::with_gil(|py| {
        let tup = (1, 2, "foo");
        py_assert!(py, tup, "tup == (1, 2, 'foo')");
        py_assert!(py, tup, "tup[0] == 1");
        py_assert!(py, tup, "tup[1] == 2");
        py_assert!(py, tup, "tup[2] == 'foo'");
    });
}

#[pyclass]
struct SimplePyClass {}

#[test]
fn intopytuple_pyclass() {
    Python::with_gil(|py| {
        let tup = (
            PyCell::new(py, SimplePyClass {}).unwrap(),
            PyCell::new(py, SimplePyClass {}).unwrap(),
        );
        py_assert!(py, tup, "type(tup[0]).__name__ == 'SimplePyClass'");
        py_assert!(py, tup, "type(tup[0]).__name__ == type(tup[1]).__name__");
        py_assert!(py, tup, "tup[0] != tup[1]");
    });
}

#[test]
fn pytuple_primitive_iter() {
    Python::with_gil(|py| {
        let tup = PyTuple::new(py, [1u32, 2, 3].iter());
        py_assert!(py, tup, "tup == (1, 2, 3)");
    });
}

#[test]
fn pytuple_pyclass_iter() {
    Python::with_gil(|py| {
        let tup = PyTuple::new(
            py,
            [
                PyCell::new(py, SimplePyClass {}).unwrap(),
                PyCell::new(py, SimplePyClass {}).unwrap(),
            ]
            .iter(),
        );
        py_assert!(py, tup, "type(tup[0]).__name__ == 'SimplePyClass'");
        py_assert!(py, tup, "type(tup[0]).__name__ == type(tup[0]).__name__");
        py_assert!(py, tup, "tup[0] != tup[1]");
    });
}

#[pyclass(dict, module = "test_module")]
struct PickleSupport {}

#[pymethods]
impl PickleSupport {
    #[new]
    fn new() -> PickleSupport {
        PickleSupport {}
    }

    pub fn __reduce__<'py>(
        slf: &'py PyCell<Self>,
        py: Python<'py>,
    ) -> PyResult<(PyObject, &'py PyTuple, PyObject)> {
        let cls = slf.to_object(py).getattr(py, "__class__")?;
        let dict = slf.to_object(py).getattr(py, "__dict__")?;
        Ok((cls, PyTuple::empty(py), dict))
    }
}

fn add_module(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    py.import("sys")?
        .dict()
        .get_item("modules")
        .unwrap()
        .unwrap()
        .downcast::<PyDict>()?
        .set_item(module.name()?, module)
}

#[test]
#[cfg_attr(all(Py_LIMITED_API, not(Py_3_10)), ignore)]
fn test_pickle() {
    Python::with_gil(|py| {
        let module = PyModule::new(py, "test_module").unwrap();
        module.add_class::<PickleSupport>().unwrap();
        add_module(py, module).unwrap();
        let inst = PyCell::new(py, PickleSupport {}).unwrap();
        py_run!(
            py,
            inst,
            r#"
        inst.a = 1
        assert inst.__dict__ == {'a': 1}

        import pickle
        inst2 = pickle.loads(pickle.dumps(inst))

        assert inst2.__dict__ == {'a': 1}
    "#
        );
    });
}

/// Testing https://github.com/PyO3/pyo3/issues/1106. A result type that
/// implements `From<MyError> for PyErr` should be automatically converted
/// when using `#[pyfunction]`.
///
/// This only makes sure that valid `Result` types do work. For an invalid
/// enum type, see `ui/invalid_result_conversion.py`.
#[derive(Debug)]
struct MyError {
    pub descr: &'static str,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "My error message: {}", self.descr)
    }
}

/// Important for the automatic conversion to `PyErr`.
impl From<MyError> for PyErr {
    fn from(err: MyError) -> pyo3::PyErr {
        pyo3::exceptions::PyOSError::new_err(err.to_string())
    }
}

#[pyfunction]
fn result_conversion_function() -> Result<(), MyError> {
    Err(MyError {
        descr: "something went wrong",
    })
}

#[test]
fn test_result_conversion() {
    Python::with_gil(|py| {
        wrap_pyfunction!(result_conversion_function)(py).unwrap();
    });
}
