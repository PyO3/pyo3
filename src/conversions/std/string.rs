use std::{borrow::Cow, convert::Infallible};

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    conversion::IntoPyObject,
    instance::Bound,
    types::{any::PyAnyMethods, string::PyStringMethods, PyString},
    FromPyObject, PyAny, PyResult, Python,
};

impl<'py> IntoPyObject<'py> for &str {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
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

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
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

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
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

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'py> IntoPyObject<'py> for String {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, &self))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("str")
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

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
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
    use crate::{IntoPyObject, Python};
    use std::borrow::Cow;

    #[test]
    fn test_cow_into_pyobject() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = Cow::Borrowed(s).into_pyobject(py).unwrap();
            assert_eq!(s, py_string.extract::<Cow<'_, str>>().unwrap());
            let py_string = Cow::<str>::Owned(s.into()).into_pyobject(py).unwrap();
            assert_eq!(s, py_string.extract::<Cow<'_, str>>().unwrap());
        })
    }

    #[test]
    fn test_non_bmp() {
        Python::with_gil(|py| {
            let s = "\u{1F30F}";
            let py_string = s.into_pyobject(py).unwrap();
            assert_eq!(s, py_string.extract::<String>().unwrap());
        })
    }

    #[test]
    fn test_extract_str() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = s.into_pyobject(py).unwrap();

            let s2: Cow<'_, str> = py_string.extract().unwrap();
            assert_eq!(s, s2);
        })
    }

    #[test]
    fn test_extract_char() {
        Python::with_gil(|py| {
            let ch = '😃';
            let py_string = ch.into_pyobject(py).unwrap();
            let ch2: char = py_string.extract().unwrap();
            assert_eq!(ch, ch2);
        })
    }

    #[test]
    fn test_extract_char_err() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let py_string = s.into_pyobject(py).unwrap();
            let err: crate::PyResult<char> = py_string.extract();
            assert!(err
                .unwrap_err()
                .to_string()
                .contains("expected a string of length 1"));
        })
    }

    #[test]
    fn test_string_into_pyobject() {
        Python::with_gil(|py| {
            let s = "Hello Python";
            let s2 = s.to_owned();
            let s3 = &s2;
            assert_eq!(
                s,
                s3.into_pyobject(py)
                    .unwrap()
                    .extract::<Cow<'_, str>>()
                    .unwrap()
            );
            assert_eq!(
                s,
                s2.into_pyobject(py)
                    .unwrap()
                    .extract::<Cow<'_, str>>()
                    .unwrap()
            );
            assert_eq!(
                s,
                s.into_pyobject(py)
                    .unwrap()
                    .extract::<Cow<'_, str>>()
                    .unwrap()
            );
        })
    }
}
