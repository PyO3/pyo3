//! Test slf: PyRef/PyMutRef<Self>(especially, slf.into::<Py>) works
use pyo3;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};
use pyo3::PyIterProtocol;
use std::collections::HashMap;

mod common;

/// Assumes it's a file reader or so.
/// Inspired by https://github.com/jothan/cordoba, thanks.
#[pyclass]
#[derive(Clone)]
struct Reader {
    inner: HashMap<u8, String>,
}

#[pymethods]
impl Reader {
    fn get_iter(slf: PyRef<Self>, keys: Py<PyBytes>) -> PyResult<Iter> {
        Ok(Iter {
            reader: slf.into(),
            keys,
            idx: 0,
        })
    }
    fn get_iter_and_reset(
        mut slf: PyRefMut<Self>,
        keys: Py<PyBytes>,
        py: Python,
    ) -> PyResult<Iter> {
        let reader = Py::new(py, slf.clone())?;
        slf.inner.clear();
        Ok(Iter {
            reader,
            keys,
            idx: 0,
        })
    }
}

#[pyclass]
struct Iter {
    reader: Py<Reader>,
    keys: Py<PyBytes>,
    idx: usize,
}

#[pyproto]
impl PyIterProtocol for Iter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyObject> {
        let py = unsafe { Python::assume_gil_acquired() };
        Ok(slf.to_object(py))
    }
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        let py = unsafe { Python::assume_gil_acquired() };
        match slf.keys.as_ref(py).as_bytes().get(slf.idx) {
            Some(&b) => {
                let res = slf
                    .reader
                    .as_ref(py)
                    .inner
                    .get(&b)
                    .map(|s| PyString::new(py, s).into());
                slf.idx += 1;
                Ok(res)
            }
            None => Ok(None),
        }
    }
}

#[test]
fn test_nested_iter() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let reader = [(1, "a"), (2, "b"), (3, "c"), (4, "d"), (5, "e")];
    let reader = Reader {
        inner: reader.iter().map(|(k, v)| (*k, v.to_string())).collect(),
    }
    .into_object(py);
    py_assert!(
        py,
        reader,
        "list(reader.get_iter(bytes([3, 5, 2]))) == ['c', 'e', 'b']"
    );
}

#[test]
fn test_nested_iter_reset() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let reader = [(1, "a"), (2, "b"), (3, "c"), (4, "d"), (5, "e")];
    let reader = PyRef::new(
        py,
        Reader {
            inner: reader.iter().map(|(k, v)| (*k, v.to_string())).collect(),
        },
    )
    .unwrap();
    let obj = reader.into_object(py);
    py_assert!(
        py,
        obj,
        "list(obj.get_iter_and_reset(bytes([3, 5, 2]))) == ['c', 'e', 'b']"
    );
    assert!(reader.inner.is_empty());
}
