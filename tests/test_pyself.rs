#![cfg(feature = "macros")]

//! Test slf: PyRef/PyMutRef<Self>(especially, slf.into::<Py>) works
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};
use std::collections::HashMap;

mod test_utils;

/// Assumes it's a file reader or so.
/// Inspired by https://github.com/jothan/cordoba, thanks.
#[pyclass]
#[derive(Clone, Debug)]
struct Reader {
    inner: HashMap<u8, String>,
}

#[pymethods]
impl Reader {
    fn clone_ref<'a, 'py>(slf: &'a Bound<'py, Self>) -> &'a Bound<'py, Self> {
        slf
    }
    fn clone_ref_with_py<'a, 'py>(
        slf: &'a Bound<'py, Self>,
        _py: Python<'py>,
    ) -> &'a Bound<'py, Self> {
        slf
    }
    fn get_iter(slf: &Bound<'_, Self>, keys: Py<PyBytes>) -> Iter {
        Iter {
            reader: slf.clone().unbind(),
            keys,
            idx: 0,
        }
    }
    fn get_iter_and_reset(
        mut slf: PyRefMut<'_, Self>,
        keys: Py<PyBytes>,
        py: Python<'_>,
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
#[derive(Debug)]
struct Iter {
    reader: Py<Reader>,
    keys: Py<PyBytes>,
    idx: usize,
}

#[pymethods]
impl Iter {
    #[allow(clippy::self_named_constructors)]
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<Py<PyAny>>> {
        let bytes = slf.keys.bind(slf.py()).as_bytes();
        match bytes.get(slf.idx) {
            Some(&b) => {
                slf.idx += 1;
                let py = slf.py();
                let reader = slf.reader.bind(py);
                let reader_ref = reader.try_borrow()?;
                let res = reader_ref
                    .inner
                    .get(&b)
                    .map(|s| PyString::new(py, s).into());
                Ok(res)
            }
            None => Ok(None),
        }
    }
}

fn reader() -> Reader {
    let reader = [(1, "a"), (2, "b"), (3, "c"), (4, "d"), (5, "e")];
    Reader {
        inner: reader.iter().map(|(k, v)| (*k, (*v).to_string())).collect(),
    }
}

#[test]
fn test_nested_iter() {
    Python::attach(|py| {
        let reader = reader().into_pyobject(py).unwrap();
        py_assert!(
            py,
            reader,
            "list(reader.get_iter(bytes([3, 5, 2]))) == ['c', 'e', 'b']"
        );
    });
}

#[test]
fn test_clone_ref() {
    Python::attach(|py| {
        let reader = reader().into_pyobject(py).unwrap();
        py_assert!(py, reader, "reader == reader.clone_ref()");
        py_assert!(py, reader, "reader == reader.clone_ref_with_py()");
    });
}

#[test]
fn test_nested_iter_reset() {
    Python::attach(|py| {
        let reader = Bound::new(py, reader()).unwrap();
        py_assert!(
            py,
            reader,
            "list(reader.get_iter_and_reset(bytes([3, 5, 2]))) == ['c', 'e', 'b']"
        );
        let reader_ref = reader.borrow();
        assert!(reader_ref.inner.is_empty());
    });
}
