#![cfg(not(Py_LIMITED_API))]

//! Support for the Python `marshal` format.

use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::types::{PyAny, PyBytes};
use crate::{ffi, Bound};
use crate::{AsPyPointer, PyResult, Python};
use std::os::raw::c_int;

/// The current version of the marshal binary format.
pub const VERSION: i32 = 4;

/// Deprecated form of [`dumps_bound`]
#[cfg(feature = "gil-refs")]
#[deprecated(
    since = "0.21.0",
    note = "`dumps` will be replaced by `dumps_bound` in a future PyO3 version"
)]
pub fn dumps<'py>(
    py: Python<'py>,
    object: &impl AsPyPointer,
    version: i32,
) -> PyResult<&'py PyBytes> {
    dumps_bound(py, object, version).map(Bound::into_gil_ref)
}

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
/// let dict = PyDict::new_bound(py);
/// dict.set_item("aap", "noot").unwrap();
/// dict.set_item("mies", "wim").unwrap();
/// dict.set_item("zus", "jet").unwrap();
///
/// let bytes = marshal::dumps_bound(py, &dict, marshal::VERSION);
/// # });
/// ```
pub fn dumps_bound<'py>(
    py: Python<'py>,
    object: &impl AsPyPointer,
    version: i32,
) -> PyResult<Bound<'py, PyBytes>> {
    unsafe {
        ffi::PyMarshal_WriteObjectToString(object.as_ptr(), version as c_int)
            .assume_owned_or_err(py)
            .downcast_into_unchecked()
    }
}

/// Deprecated form of [`loads_bound`]
#[cfg(feature = "gil-refs")]
#[deprecated(
    since = "0.21.0",
    note = "`loads` will be replaced by `loads_bound` in a future PyO3 version"
)]
pub fn loads<'py, B>(py: Python<'py>, data: &B) -> PyResult<&'py PyAny>
where
    B: AsRef<[u8]> + ?Sized,
{
    loads_bound(py, data).map(Bound::into_gil_ref)
}

/// Deserialize an object from bytes using the Python built-in marshal module.
pub fn loads_bound<'py, B>(py: Python<'py>, data: &B) -> PyResult<Bound<'py, PyAny>>
where
    B: AsRef<[u8]> + ?Sized,
{
    let data = data.as_ref();
    unsafe {
        ffi::PyMarshal_ReadObjectFromString(data.as_ptr().cast(), data.len() as isize)
            .assume_owned_or_err(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{bytes::PyBytesMethods, dict::PyDictMethods, PyDict};

    #[test]
    fn marshal_roundtrip() {
        Python::with_gil(|py| {
            let dict = PyDict::new_bound(py);
            dict.set_item("aap", "noot").unwrap();
            dict.set_item("mies", "wim").unwrap();
            dict.set_item("zus", "jet").unwrap();

            let pybytes = dumps_bound(py, &dict, VERSION).expect("marshalling failed");
            let deserialized = loads_bound(py, pybytes.as_bytes()).expect("unmarshalling failed");

            assert!(equal(py, &dict, &deserialized));
        });
    }

    fn equal(_py: Python<'_>, a: &impl AsPyPointer, b: &impl AsPyPointer) -> bool {
        unsafe { ffi::PyObject_RichCompareBool(a.as_ptr(), b.as_ptr(), ffi::Py_EQ) != 0 }
    }
}
