#![feature(specialization)]
use std::collections::HashMap;

use pyo3::exceptions::KeyError;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyList;
use pyo3::PyMappingProtocol;

#[pyclass]
struct Mapping {
    index: HashMap<String, usize>,
}

#[pymethods]
impl Mapping {
    #[new]
    fn new(elements: Option<&PyList>) -> PyResult<Self> {
        if let Some(pylist) = elements {
            let mut elems = HashMap::with_capacity(pylist.len());
            for (i, pyelem) in pylist.into_iter().enumerate() {
                let elem = String::extract(pyelem)?;
                elems.insert(elem, i);
            }
            Ok(Self { index: elems })
        } else {
            Ok(Self {
                index: HashMap::new(),
            })
        }
    }
}

#[pyproto]
impl PyMappingProtocol for Mapping {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.index.len())
    }

    fn __getitem__(&self, query: String) -> PyResult<usize> {
        self.index
            .get(&query)
            .copied()
            .ok_or_else(|| KeyError::py_err("unknown key"))
    }

    fn __setitem__(&mut self, key: String, value: usize) -> PyResult<()> {
        self.index.insert(key, value);
        Ok(())
    }

    fn __delitem__(&mut self, key: String) -> PyResult<()> {
        if self.index.remove(&key).is_none() {
            KeyError::py_err("unknown key").into()
        } else {
            Ok(())
        }
    }

    /// not an actual reversed implementation, just to demonstrate that the method is callable.
    fn __reversed__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        Ok(self
            .index
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .into_py(gil.python()))
    }
}

#[test]
fn test_getitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let d = [("Mapping", py.get_type::<Mapping>())].into_py_dict(py);

    let run = |code| py.run(code, None, Some(d)).unwrap();
    let err = |code| py.run(code, None, Some(d)).unwrap_err();

    run("m = Mapping(['1', '2', '3']); assert m['1'] == 0");
    run("m = Mapping(['1', '2', '3']); assert m['2'] == 1");
    run("m = Mapping(['1', '2', '3']); assert m['3'] == 2");
    err("m = Mapping(['1', '2', '3']); print(m['4'])");
}

#[test]
fn test_setitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let d = [("Mapping", py.get_type::<Mapping>())].into_py_dict(py);

    let run = |code| py.run(code, None, Some(d)).unwrap();
    let err = |code| py.run(code, None, Some(d)).unwrap_err();

    run("m = Mapping(['1', '2', '3']); m['1'] = 4; assert m['1'] == 4");
    run("m = Mapping(['1', '2', '3']); m['0'] = 0; assert m['0'] == 0");
    run("m = Mapping(['1', '2', '3']); len(m) == 4");
    err("m = Mapping(['1', '2', '3']); m[0] = 'hello'");
    err("m = Mapping(['1', '2', '3']); m[0] = -1");
}

#[test]
fn test_delitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = [("Mapping", py.get_type::<Mapping>())].into_py_dict(py);
    let run = |code| py.run(code, None, Some(d)).unwrap();
    let err = |code| py.run(code, None, Some(d)).unwrap_err();

    run(
        "m = Mapping(['1', '2', '3']); del m['1']; assert len(m) == 2; \
         assert m['2'] == 1; assert m['3'] == 2",
    );
    err("m = Mapping(['1', '2', '3']); del m[-1]");
    err("m = Mapping(['1', '2', '3']); del m['4']");
}

#[test]
fn test_reversed() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = [("Mapping", py.get_type::<Mapping>())].into_py_dict(py);
    let run = |code| py.run(code, None, Some(d)).unwrap();

    run("m = Mapping(['1', '2']); assert set(reversed(m)) == {'1', '2'}");
}
