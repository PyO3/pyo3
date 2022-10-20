use crate::{
    inspect::types::TypeInfo, types::PyBytes, FromPyObject, IntoPy, PyAny, PyObject, PyResult,
    PyTryFrom, Python, ToPyObject,
};

impl<'a> IntoPy<PyObject> for &'a [u8] {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyBytes::new(py, self).to_object(py)
    }

    fn type_output() -> TypeInfo {
        TypeInfo::builtin("bytes")
    }
}

impl<'a> FromPyObject<'a> for &'a [u8] {
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        Ok(<PyBytes as PyTryFrom>::try_from(obj)?.as_bytes())
    }

    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

#[cfg(test)]
mod tests {
    use crate::FromPyObject;
    use crate::Python;

    #[test]
    fn test_extract_bytes() {
        Python::with_gil(|py| {
            let py_bytes = py.eval("b'Hello Python'", None, None).unwrap();
            let bytes: &[u8] = FromPyObject::extract(py_bytes).unwrap();
            assert_eq!(bytes, b"Hello Python");
        });
    }
}
