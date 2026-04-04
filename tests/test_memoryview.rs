#![cfg(feature = "macros")]
#![cfg(any(Py_3_11, not(Py_LIMITED_API)))]

use pyo3::prelude::*;
use pyo3::types::PyMemoryView;

#[pyclass(frozen)]
struct ByteOwner {
    data: Vec<u8>,
}

#[test]
fn test_from_owned_buffer_basic() {
    Python::attach(|py| {
        let owner = Py::new(
            py,
            ByteOwner {
                data: vec![1, 2, 3, 4, 5],
            },
        )
        .unwrap();
        let view = PyMemoryView::from_owned_buffer(py, owner, |o| &o.data).unwrap();
        assert_eq!(view.len().unwrap(), 5);
        assert!(view.is_truthy().unwrap());
    });
}

#[test]
fn test_from_owned_buffer_readonly() {
    Python::attach(|py| {
        let owner = Py::new(py, ByteOwner { data: vec![42] }).unwrap();
        let view = PyMemoryView::from_owned_buffer(py, owner, |o| &o.data).unwrap();
        // Verify the memoryview is readonly via Python
        let readonly: bool = view.getattr("readonly").unwrap().extract().unwrap();
        assert!(readonly);
    });
}

#[test]
fn test_from_owned_buffer_content() {
    Python::attach(|py| {
        let owner = Py::new(
            py,
            ByteOwner {
                data: b"hello".to_vec(),
            },
        )
        .unwrap();
        let view = PyMemoryView::from_owned_buffer(py, owner, |o| &o.data).unwrap();
        let bytes: Vec<u8> = view.call_method0("tobytes").unwrap().extract().unwrap();
        assert_eq!(bytes, b"hello");
    });
}

#[test]
fn test_from_owned_buffer_empty() {
    Python::attach(|py| {
        let owner = Py::new(py, ByteOwner { data: vec![] }).unwrap();
        let view = PyMemoryView::from_owned_buffer(py, owner, |o| &o.data).unwrap();
        assert_eq!(view.len().unwrap(), 0);
    });
}

#[test]
fn test_from_owned_buffer_static_data() {
    // The closure can also return a &'static [u8]
    Python::attach(|py| {
        let owner = Py::new(py, ByteOwner { data: vec![] }).unwrap();
        let view =
            PyMemoryView::from_owned_buffer(py, owner, |_o| b"static data" as &[u8]).unwrap();
        let bytes: Vec<u8> = view.call_method0("tobytes").unwrap().extract().unwrap();
        assert_eq!(bytes, b"static data");
    });
}

#[test]
fn test_from_owned_buffer_keeps_owner_alive() {
    Python::attach(|py| {
        let owner = Py::new(
            py,
            ByteOwner {
                data: b"kept alive".to_vec(),
            },
        )
        .unwrap();
        let view = PyMemoryView::from_owned_buffer(py, owner, |o| &o.data).unwrap();
        // Force GC to ensure the owner is kept alive by the memoryview
        py.run(c"import gc; gc.collect()", None, None).unwrap();
        let bytes: Vec<u8> = view.call_method0("tobytes").unwrap().extract().unwrap();
        assert_eq!(bytes, b"kept alive");
    });
}
