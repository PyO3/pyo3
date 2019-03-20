use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyTuple;
use pyo3::wrap_pyfunction;
use std::isize;

#[macro_use]
mod common;

#[pyclass]
struct MutRefArg {
    n: i32,
}

#[pymethods]
impl MutRefArg {
    fn get(&self) -> PyResult<i32> {
        Ok(self.n)
    }
    fn set_other(&self, other: &mut MutRefArg) -> PyResult<()> {
        other.n = 100;
        Ok(())
    }
}

#[test]
fn mut_ref_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst1 = Py::new(py, MutRefArg { n: 0 }).unwrap();
    let inst2 = Py::new(py, MutRefArg { n: 0 }).unwrap();

    let d = [("inst1", &inst1), ("inst2", &inst2)].into_py_dict(py);

    py.run("inst1.set_other(inst2)", None, Some(d)).unwrap();
    assert_eq!(inst2.as_ref(py).n, 100);
}

#[pyclass]
struct PyUsize {
    #[pyo3(get)]
    pub value: usize,
}

#[pyfunction]
fn get_zero() -> PyResult<PyUsize> {
    Ok(PyUsize { value: 0 })
}

#[test]
/// Checks that we can use return a custom class in arbitrary function and use those functions
/// both in rust and python
fn return_custom_class() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    // Using from rust
    assert_eq!(get_zero().unwrap().value, 0);

    // Using from python
    let get_zero = wrap_pyfunction!(get_zero)(py);
    py_assert!(py, get_zero, "get_zero().value == 0");
}

#[test]
fn intopytuple_primitive() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let tup = (1, 2, "foo");
    py_assert!(py, tup, "tup == (1, 2, 'foo')");
    py_assert!(py, tup, "tup[0] == 1");
    py_assert!(py, tup, "tup[1] == 2");
    py_assert!(py, tup, "tup[2] == 'foo'");
}

#[pyclass]
struct SimplePyClass {}

#[test]
fn intopytuple_pyclass() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let tup = (
        PyRef::new(py, SimplePyClass {}).unwrap(),
        PyRef::new(py, SimplePyClass {}).unwrap(),
    );
    py_assert!(py, tup, "type(tup[0]).__name__ == 'SimplePyClass'");
    py_assert!(py, tup, "type(tup[0]).__name__ == type(tup[1]).__name__");
    py_assert!(py, tup, "tup[0] != tup[1]");
}

#[test]
fn pytuple_primitive_iter() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let tup = PyTuple::new(py, [1u32, 2, 3].iter());
    py_assert!(py, tup, "tup == (1, 2, 3)");
}

#[test]
fn pytuple_pyclass_iter() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let tup = PyTuple::new(
        py,
        [
            PyRef::new(py, SimplePyClass {}).unwrap(),
            PyRef::new(py, SimplePyClass {}).unwrap(),
        ]
        .into_iter(),
    );
    py_assert!(py, tup, "type(tup[0]).__name__ == 'SimplePyClass'");
    py_assert!(py, tup, "type(tup[0]).__name__ == type(tup[0]).__name__");
    py_assert!(py, tup, "tup[0] != tup[1]");
}
