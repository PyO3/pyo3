#![cfg(not(Py_LIMITED_API))]

//! Support for the Python `marshal` format.

use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::types::{PyAny, PyBytes};
use crate::{ffi, Bound};
use crate::{PyResult, Python};
use std::os::raw::c_int;

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
/// # pyo3::Python::with_gil(|py| {
/// let dict = PyDict::new(py);
/// dict.set_item("aap", "noot").unwrap();
/// dict.set_item("mies", "wim").unwrap();
/// dict.set_item("zus", "jet").unwrap();
///
/// let bytes = marshal::dumps(&dict, marshal::VERSION);
/// # });
/// ```
pub fn dumps<'py>(object: &Bound<'py, PyAny>, version: i32) -> PyResult<Bound<'py, PyBytes>> {
    unsafe {
        ffi::PyMarshal_WriteObjectToString(object.as_ptr(), version as c_int)
            .assume_owned_or_err(object.py())
            .downcast_into_unchecked()
    }
}

/// Deprecated form of [`dumps`].
#[deprecated(since = "0.23.0", note = "use `dumps` instead")]
pub fn dumps_bound<'py>(
    py: Python<'py>,
    object: &impl crate::AsPyPointer,
    version: i32,
) -> PyResult<Bound<'py, PyBytes>> {
    dumps(
        unsafe { object.as_ptr().assume_borrowed(py) }.as_any(),
        version,
    )
}

/// Deserialize an object from bytes using the Python built-in marshal module.
pub fn loads<'py, B>(py: Python<'py>, data: &B) -> PyResult<Bound<'py, PyAny>>
where
    B: AsRef<[u8]> + ?Sized,
{
    let data = data.as_ref();
    unsafe {
        ffi::PyMarshal_ReadObjectFromString(data.as_ptr().cast(), data.len() as isize)
            .assume_owned_or_err(py)
    }
}

/// Deprecated form of [`loads`].
#[deprecated(since = "0.23.0", note = "renamed to `loads`")]
pub fn loads_bound<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyAny>> {
    loads(py, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{bytes::PyBytesMethods, dict::PyDictMethods, PyAnyMethods, PyDict};

    #[test]
    fn marshal_roundtrip() {
        Python::with_gil(|py| {
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
