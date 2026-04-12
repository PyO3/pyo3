#![cfg(not(Py_LIMITED_API))]

//! Support for the Python `marshal` format.

#[cfg(not(PyRustPython))]
use crate::ffi_ptr_ext::FfiPtrExt;
#[cfg(not(PyRustPython))]
use crate::py_result_ext::PyResultExt;
use crate::types::{PyAny, PyBytes};
#[cfg(PyRustPython)]
use crate::types::PyAnyMethods;
#[cfg(not(PyRustPython))]
use crate::{ffi, Bound};
#[cfg(PyRustPython)]
use crate::Bound;
use crate::{PyResult, Python};
#[cfg(not(PyRustPython))]
use std::ffi::c_int;

/// The current version of the marshal binary format.
pub const VERSION: i32 = 4;

/// Serialize an object to bytes using the Python built-in marshal module.
///
/// The built-in marshalling only supports a limited range of objects.
/// The exact types supported depend on the version argument.
/// The [`VERSION`] constant holds the highest version currently supported.
///
/// See the [Python documentation](https://docs.python.org/3/library/marshal.html) for more details.
///
/// # Examples
/// ```
/// # use pyo3::{marshal, types::PyDict, prelude::PyDictMethods};
/// # pyo3::Python::attach(|py| {
/// let dict = PyDict::new(py);
/// dict.set_item("aap", "noot").unwrap();
/// dict.set_item("mies", "wim").unwrap();
/// dict.set_item("zus", "jet").unwrap();
///
/// let bytes = marshal::dumps(&dict, marshal::VERSION);
/// # });
/// ```
pub fn dumps<'py>(object: &Bound<'py, PyAny>, version: i32) -> PyResult<Bound<'py, PyBytes>> {
    #[cfg(PyRustPython)]
    {
        let marshal = object.py().import("marshal")?;
        return Ok(unsafe {
            marshal
                .call_method1("dumps", (object, version))?
                .cast_into_unchecked()
        });
    }

    #[cfg(not(PyRustPython))]
    unsafe {
        ffi::PyMarshal_WriteObjectToString(object.as_ptr(), version as c_int)
            .assume_owned_or_err(object.py())
            .cast_into_unchecked()
    }
}

/// Deserialize an object from bytes using the Python built-in marshal module.
pub fn loads<'py, B>(py: Python<'py>, data: &B) -> PyResult<Bound<'py, PyAny>>
where
    B: AsRef<[u8]> + ?Sized,
{
    let data = data.as_ref();

    #[cfg(PyRustPython)]
    {
        let marshal = py.import("marshal")?;
        return Ok(marshal.call_method1("loads", (data,))?);
    }

    #[cfg(not(PyRustPython))]
    unsafe {
        ffi::PyMarshal_ReadObjectFromString(data.as_ptr().cast(), data.len() as isize)
            .assume_owned_or_err(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{bytes::PyBytesMethods, dict::PyDictMethods, PyAnyMethods, PyDict};

    #[test]
    fn marshal_roundtrip() {
        Python::attach(|py| {
            let dict = PyDict::new(py);
            dict.set_item("aap", "noot").unwrap();
            dict.set_item("mies", "wim").unwrap();
            dict.set_item("zus", "jet").unwrap();

            let pybytes = dumps(&dict, VERSION).expect("marshalling failed");
            let deserialized = loads(py, pybytes.as_bytes()).expect("unmarshalling failed");

            assert!(dict.eq(&deserialized).unwrap());
        });
    }
}
