#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::Unpackable;

mod test_utils;

#[test]
fn test_unpack_3() {
    Python::attach(|py| {
        let tuple = (0, 1, 2);
        let py_tuple = tuple.into_pyobject(py).unwrap();
        let unpacked: (i32, i32, i32) =
            Unpackable::unpack(py_tuple.as_any().as_borrowed()).unwrap();

        assert_eq!(tuple, unpacked);
    });
}

#[test]
fn test_unpack_not_enough() {
    Python::attach(|py| {
        let tuple = (0, 1);
        let py_tuple = tuple.into_pyobject(py).unwrap();
        let try_unpack =
            <(i32, i32, i32) as Unpackable>::unpack(py_tuple.as_any().as_borrowed()).unwrap_err();

        assert_eq!(
            try_unpack.value(py).to_string(),
            "not enough values to unpack (expected 3)"
        );
    });
}

#[test]
fn test_unpack_too_many() {
    Python::attach(|py| {
        let tuple = (0, 1, 2);
        let py_tuple = tuple.into_pyobject(py).unwrap();
        let try_unpack =
            <(i32, i32) as Unpackable>::unpack(py_tuple.as_any().as_borrowed()).unwrap_err();

        assert_eq!(
            try_unpack.value(py).to_string(),
            "too many values to unpack (expected 2)"
        );
    });
}
