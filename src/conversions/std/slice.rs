#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{types::PyBytes, FromPyObject, IntoPy, PyAny, PyObject, PyResult, Python};

impl<'a> IntoPy<PyObject> for &'a [u8] {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyBytes::new_bound(py, self).unbind().into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("bytes")
    }
}

impl<'py> FromPyObject<'py> for &'py [u8] {
    fn extract(obj: &'py PyAny) -> PyResult<Self> {
        Ok(obj.downcast::<PyBytes>()?.as_bytes())
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

#[cfg(test)]
mod tests {
    use crate::Python;

    #[test]
    fn test_extract_bytes() {
        Python::with_gil(|py| {
            let py_bytes = py.eval("b'Hello Python'", None, None).unwrap();
            let bytes: &[u8] = py_bytes.extract().unwrap();
            assert_eq!(bytes, b"Hello Python");
        });
    }
}
