use crate::conversion::AsPyPointer;
use crate::ffi;
use crate::{PyErr, PyObject, PyResult, Python};
use std::os::raw::c_int;

/// The current version of the marshal binary format.
pub const VERSION: i32 = 4;

/// Serialize an object to bytes using the Python built-in marshal module.
///
/// The built-in marshalling only supports a limited range of object.
/// See the [python documentation](https://docs.python.org/3/library/marshal.html) for more details.
pub fn dumps(py: Python, object: &PyObject, version: i32) -> PyResult<Vec<u8>> {
    let bytes = unsafe { ffi::PyMarshal_WriteObjectToString(object.as_ptr(), version as c_int) };
    if bytes.is_null() {
        return Err(PyErr::fetch(py));
    }

    let mut size = 0isize;
    let mut data = std::ptr::null_mut();
    unsafe {
        ffi::PyBytes_AsStringAndSize(bytes, &mut data, &mut size);
        let data = Vec::from(std::slice::from_raw_parts(data as *const u8, size as usize));
        ffi::Py_DecRef(bytes);
        Ok(data)
    }
}

/// Deserialize an object from bytes using the Python built-in marshal module.
pub fn loads(py: Python, data: &impl AsRef<[u8]>) -> PyResult<PyObject> {
    let data = data.as_ref();

    let object = unsafe {
        ffi::PyMarshal_ReadObjectFromString(data.as_ptr() as *const i8, data.len() as isize)
    };
    if object.is_null() {
        return Err(PyErr::fetch(py));
    }

    Ok(unsafe { PyObject::from_owned_ptr(py, object) })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{types::PyDict, ToPyObject};

    #[test]
    fn marhshal_roundtrip() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let dict = PyDict::new(py);
        dict.set_item("aap", "noot").unwrap();
        dict.set_item("mies", "wim").unwrap();
        dict.set_item("zus", "jet").unwrap();
        let dict = dict.to_object(py);

        let bytes = dumps(py, &dict, VERSION).expect("marshalling failed");
        let deserialzed = loads(py, &bytes).expect("unmarshalling failed");

        assert!(equal(py, &dict, &deserialzed));
    }

    fn equal(_py: Python, a: &PyObject, b: &PyObject) -> bool {
        unsafe { ffi::PyObject_RichCompareBool(a.as_ptr(), b.as_ptr(), ffi::Py_EQ) != 0 }
    }
}
