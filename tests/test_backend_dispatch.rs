#![cfg(feature = "runtime-cpython")]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple, PyTypeMethods};

#[test]
fn active_backend_defaults_to_cpython_family() {
    assert_eq!(
        pyo3::active_backend_kind(),
        pyo3::backend::BackendKind::Cpython
    );
}

#[test]
fn builtin_type_dispatch_matches_cpython_family_types() {
    Python::attach(|py| {
        assert_eq!(PyDict::new(py).get_type().name().unwrap(), "dict");
        assert_eq!(PyList::empty(py).get_type().name().unwrap(), "list");
        assert_eq!(PyTuple::empty(py).get_type().name().unwrap(), "tuple");
    });
}
