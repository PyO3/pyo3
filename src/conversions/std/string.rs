use std::borrow::Cow;

use crate::{
    inspect::types::TypeInfo, types::PyString, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult,
    PyTryFrom, Python, ToPyObject,
};

/// Converts a Rust `str` to a Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for str {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyString::new(py, self).into()
    }
}

impl<'a> IntoPy<PyObject> for &'a str {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyString::new(py, self).into()
    }

    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'a> IntoPy<Py<PyString>> for &'a str {
    #[inline]
    fn into_py(self, py: Python<'_>) -> Py<PyString> {
        PyString::new(py, self).into()
    }

    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

/// Converts a Rust `Cow<'_, str>` to a Python object.
/// See `PyString::new` for details on the conversion.
impl<'a> ToPyObject for Cow<'a, str> {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyString::new(py, self).into()
    }
}

impl IntoPy<PyObject> for Cow<'_, str> {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }

    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

/// Converts a Rust `String` to a Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyString::new(py, self).into()
    }
}

impl ToPyObject for char {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.into_py(py)
    }
}

impl IntoPy<PyObject> for char {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let mut bytes = [0u8; 4];
        PyString::new(py, self.encode_utf8(&mut bytes)).into()
    }

    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl IntoPy<PyObject> for String {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyString::new(py, &self).into()
    }

    fn type_output() -> TypeInfo {
        TypeInfo::builtin("str")
    }
}

impl<'a> IntoPy<PyObject> for &'a String {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyString::new(py, self).into()
    }

    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'source> FromPyObject<'source> for &'source str {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        <PyString as PyTryFrom>::try_from(ob)?.to_str()
    }

    fn type_input() -> TypeInfo {
        <String>::type_input()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl FromPyObject<'_> for String {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        <PyString as PyTryFrom>::try_from(obj)?
            .to_str()
            .map(ToOwned::to_owned)
    }

    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

impl FromPyObject<'_> for char {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        let s = <PyString as PyTryFrom<'_>>::try_from(obj)?.to_str()?;
        let mut iter = s.chars();
        if let (Some(ch), None) = (iter.next(), iter.next()) {
            Ok(ch)
        } else {
            Err(crate::exceptions::PyValueError::new_err(
                "expected a string of length 1",
            ))
        }
    }

    fn type_input() -> TypeInfo {
        <String>::type_input()
    }
}

#[cfg(test)]
mod tests {
    use crate::Python;
    use crate::{FromPyObject, ToPyObject};

    #[test]
    fn test_non_bmp() {
        Python::with_gil(|py| {
            let s = "\u{1F30F}";
            let py_string = s.to_object(py);
            assert_eq!(s, py_string.extract::<String>(py).unwrap());
        })
    }

    #[test]
    fn test_extract_str() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = s.to_object(py);

            let s2: &str = FromPyObject::extract(py_string.as_ref(py)).unwrap();
            assert_eq!(s, s2);
        })
    }

    #[test]
    fn test_extract_char() {
        Python::with_gil(|py| {
            let ch = '😃';
            let py_string = ch.to_object(py);
            let ch2: char = FromPyObject::extract(py_string.as_ref(py)).unwrap();
            assert_eq!(ch, ch2);
        })
    }

    #[test]
    fn test_extract_char_err() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = s.to_object(py);
            let err: crate::PyResult<char> = FromPyObject::extract(py_string.as_ref(py));
            assert!(err
                .unwrap_err()
                .to_string()
                .contains("expected a string of length 1"));
        })
    }
}
