//! Rust value -> Python Iterator
//! Inspired by https://github.com/jothan/cordoba, thanks.
use pyo3;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};
use pyo3::PyIterProtocol;
use std::collections::HashMap;

#[macro_use]
mod common;

/// Assumes it's a file reader or so.
#[pyclass]
struct Reader {
    inner: HashMap<u8, String>,
}

#[pymethods]
impl Reader {
    fn get_optional(&self, test: Option<i32>) -> PyResult<i32> {
        Ok(test.unwrap_or(10))
    }
    fn get_iter(slf: PyRef<Reader>, keys: Py<PyBytes>) -> PyResult<Iter> {
        Ok(Iter {
            reader: slf.into(),
            keys: keys,
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
