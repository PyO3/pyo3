use std::{borrow::Cow, convert::Infallible};

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    conversion::IntoPyObject,
    instance::Bound,
    types::{any::PyAnyMethods, string::PyStringMethods, PyString},
    FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject,
};

/// Converts a Rust `str` to a Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for str {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }
}

impl<'a> IntoPy<PyObject> for &'a str {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'a> IntoPy<Py<PyString>> for &'a str {
    #[inline]
    fn into_py(self, py: Python<'_>) -> Py<PyString> {
        self.into_pyobject(py).unwrap().unbind()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'py> IntoPyObject<'py> for &str {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self))
    }
}

impl<'py> IntoPyObject<'py> for &&str {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

/// Converts a Rust `Cow<'_, str>` to a Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for Cow<'_, str> {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }
}

impl IntoPy<PyObject> for Cow<'_, str> {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'py> IntoPyObject<'py> for Cow<'_, str> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Cow<'_, str> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

/// Converts a Rust `String` to a Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }
}

impl ToPyObject for char {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }
}

impl IntoPy<PyObject> for char {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'py> IntoPyObject<'py> for char {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let mut bytes = [0u8; 4];
        Ok(PyString::new(py, self.encode_utf8(&mut bytes)))
    }
}

impl<'py> IntoPyObject<'py> for &char {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl IntoPy<PyObject> for String {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("str")
    }
}

impl<'py> IntoPyObject<'py> for String {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, &self))
    }
}

impl<'a> IntoPy<PyObject> for &'a String {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().into_any().unbind()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'py> IntoPyObject<'py> for &String {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self))
    }
}

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
impl<'a> crate::conversion::FromPyObjectBound<'a, '_> for &'a str {
    fn from_py_object_bound(ob: crate::Borrowed<'a, '_, PyAny>) -> PyResult<Self> {
        ob.downcast::<PyString>()?.to_str()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        <String as crate::FromPyObject>::type_input()
    }
}

impl<'a> crate::conversion::FromPyObjectBound<'a, '_> for Cow<'a, str> {
    fn from_py_object_bound(ob: crate::Borrowed<'a, '_, PyAny>) -> PyResult<Self> {
        ob.downcast::<PyString>()?.to_cow()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        <String as crate::FromPyObject>::type_input()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl FromPyObject<'_> for String {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        obj.downcast::<PyString>()?.to_cow().map(Cow::into_owned)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

impl FromPyObject<'_> for char {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let s = obj.downcast::<PyString>()?.to_cow()?;
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
    use crate::types::any::PyAnyMethods;
    use crate::Python;
    use crate::{IntoPy, PyObject, ToPyObject};
    use std::borrow::Cow;

    #[test]
    fn test_cow_into_py() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string: PyObject = Cow::Borrowed(s).into_py(py);
            assert_eq!(s, py_string.extract::<Cow<'_, str>>(py).unwrap());
            let py_string: PyObject = Cow::<str>::Owned(s.into()).into_py(py);
            assert_eq!(s, py_string.extract::<Cow<'_, str>>(py).unwrap());
        })
    }

    #[test]
    fn test_cow_to_object() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = Cow::Borrowed(s).to_object(py);
            assert_eq!(s, py_string.extract::<Cow<'_, str>>(py).unwrap());
            let py_string = Cow::<str>::Owned(s.into()).to_object(py);
            assert_eq!(s, py_string.extract::<Cow<'_, str>>(py).unwrap());
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

            let s2: Cow<'_, str> = py_string.bind(py).extract().unwrap();
            assert_eq!(s, s2);
        })
    }

    #[test]
    fn test_extract_char() {
        Python::with_gil(|py| {
            let ch = '😃';
            let py_string = ch.to_object(py);
            let ch2: char = py_string.bind(py).extract().unwrap();
            assert_eq!(ch, ch2);
        })
    }

    #[test]
    fn test_extract_char_err() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = s.to_object(py);
            let err: crate::PyResult<char> = py_string.bind(py).extract();
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
                    .extract::<Cow<'_, str>>(py)
                    .unwrap()
            );
            assert_eq!(
                s,
                IntoPy::<PyObject>::into_py(s2, py)
                    .extract::<Cow<'_, str>>(py)
                    .unwrap()
            );
            assert_eq!(
                s,
                IntoPy::<PyObject>::into_py(s, py)
                    .extract::<Cow<'_, str>>(py)
                    .unwrap()
            );
        })
    }
}
