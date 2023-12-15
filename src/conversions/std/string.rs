use std::borrow::Cow;

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    types::PyString, FromPyObject, IntoPy, PyAny, PyDetached, PyObject, PyResult, Python,
    ToPyObject,
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

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'a> IntoPy<PyDetached<PyString>> for &'a str {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyDetached<PyString> {
        PyString::new(py, self).into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

/// Converts a Rust `Cow<'_, str>` to a Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for Cow<'_, str> {
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

    #[cfg(feature = "experimental-inspect")]
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

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl IntoPy<PyObject> for String {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyString::new(py, &self).into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("str")
    }
}

impl<'a> IntoPy<PyObject> for &'a String {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyString::new(py, self).into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'source> FromPyObject<'source> for &'source str {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        ob.downcast::<PyString>()?.to_str()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        <String>::type_input()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl FromPyObject<'_> for String {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        obj.downcast::<PyString>()?.to_str().map(ToOwned::to_owned)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

impl FromPyObject<'_> for char {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        let s = obj.downcast::<PyString>()?.to_str()?;
        let mut iter = s.chars();
        if let (Some(ch), None) = (iter.next(), iter.next()) {
            Ok(ch)
        } else {
            Err(crate::exceptions::PyValueError::new_err(
                "expected a string of length 1",
            ))
        }
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        <String>::type_input()
    }
}

#[cfg(test)]
mod tests {
    use crate::Python;
    use crate::{FromPyObject, IntoPy, PyObject, ToPyObject};
    use std::borrow::Cow;

    #[test]
    fn test_cow_into_py() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string: PyObject = Cow::Borrowed(s).into_py(py);
            assert_eq!(s, py_string.extract::<&str>(py).unwrap());
            let py_string: PyObject = Cow::<str>::Owned(s.into()).into_py(py);
            assert_eq!(s, py_string.extract::<&str>(py).unwrap());
        })
    }

    #[test]
    fn test_cow_to_object() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = Cow::Borrowed(s).to_object(py);
            assert_eq!(s, py_string.extract::<&str>(py).unwrap());
            let py_string = Cow::<str>::Owned(s.into()).to_object(py);
            assert_eq!(s, py_string.extract::<&str>(py).unwrap());
        })
    }

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

    #[test]
    fn test_string_into_py() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let s2 = s.to_owned();
            let s3 = &s2;
            assert_eq!(
                s,
                IntoPy::<PyObject>::into_py(s3, py)
                    .extract::<&str>(py)
                    .unwrap()
            );
            assert_eq!(
                s,
                IntoPy::<PyObject>::into_py(s2, py)
                    .extract::<&str>(py)
                    .unwrap()
            );
            assert_eq!(
                s,
                IntoPy::<PyObject>::into_py(s, py)
                    .extract::<&str>(py)
                    .unwrap()
            );
        })
    }
}
