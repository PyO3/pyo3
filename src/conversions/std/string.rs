use std::{borrow::Cow, convert::Infallible};

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    conversion::IntoPyObject, exceptions::PyValueError, ffi, instance::Bound, types::PyString,
    Borrowed, FromPyObject, PyAny, PyErr, Python,
};

impl<'py> IntoPyObject<'py> for &str {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = String::OUTPUT_TYPE;

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = String::OUTPUT_TYPE;

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = String::OUTPUT_TYPE;

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = String::OUTPUT_TYPE;

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = String::OUTPUT_TYPE;

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = String::OUTPUT_TYPE;

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = "str";

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = String::OUTPUT_TYPE;

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
impl<'a> crate::conversion::FromPyObject<'a, '_> for &'a str {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "str";

    fn extract(ob: crate::Borrowed<'a, '_, PyAny>) -> Result<Self, Self::Error> {
        ob.cast::<PyString>()?.to_str()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        <String as crate::FromPyObject>::type_input()
    }
}

impl<'a> crate::conversion::FromPyObject<'a, '_> for Cow<'a, str> {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "str";

    fn extract(ob: crate::Borrowed<'a, '_, PyAny>) -> Result<Self, Self::Error> {
        ob.cast::<PyString>()?.to_cow()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        <String as crate::FromPyObject>::type_input()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl FromPyObject<'_, '_> for String {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "str";

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        obj.cast::<PyString>()?.to_cow().map(Cow::into_owned)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

impl FromPyObject<'_, '_> for char {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "str";

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        let s = obj.cast::<PyString>()?.to_cow()?;
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

// FFI conversions for CStr and CString

impl<'py> IntoPyObject<'py> for &std::ffi::CStr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = "str";

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        // Convert cstr to &str, it's is safe because cstr is guaranteed to be valid UTF-8
        Ok(PyString::new(py, self.to_str().unwrap()))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'py> IntoPyObject<'py> for std::ffi::CString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = "str";

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        // Convert cstring to &str, it's safe because cstring is guaranteed to be valid UTF-8
        Ok(PyString::new(py, self.to_str().unwrap()))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

impl<'py> IntoPyObject<'py> for &std::ffi::CString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = "str";

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self.to_str().unwrap()))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <String>::type_output()
    }
}

/// Allows extracting cstr from Python objects.
/// Accepts Python `str` objects and converts them to cstr.
/// Fails if the string contains interior nul bytes.
impl<'a> FromPyObject<'a, '_> for &'a std::ffi::CStr {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "str";

    fn extract(obj: Borrowed<'a, '_, PyAny>) -> Result<Self, Self::Error> {
        let py_string = obj.cast::<PyString>()?;

        // Use PyUnicode_AsUTF8AndSize to get the raw bytes with guaranteed NUL termination
        #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
        {
            let mut size: ffi::Py_ssize_t = 0;
            let data: *const u8 =
                unsafe { ffi::PyUnicode_AsUTF8AndSize(py_string.as_ptr(), &mut size).cast() };
            if data.is_null() {
                return Err(crate::PyErr::fetch(obj.py()));
            }

            // Create a slice that includes the NUL terminator
            let bytes_with_nul = unsafe { std::slice::from_raw_parts(data, (size + 1) as usize) };

            // Check for interior NUL bytes (excluding the final NUL)
            for &byte in &bytes_with_nul[..size as usize] {
                if byte == 0 {
                    return Err(PyValueError::new_err("string contains interior NUL bytes"));
                }
            }

            // Now create the CStr from the bytes with NUL
            Ok(unsafe { std::ffi::CStr::from_ptr(data as *const i8) })
        }

        #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
        {
            // Fallback for older Python versions
            use std::ffi::FromBytesWithNulError;
            let cow = py_string.to_cow()?;
            let bytes = cow.as_bytes();

            // Try to create a CStr from the bytes
            // This will fail if there are interior NUL bytes
            std::ffi::CStr::from_bytes_with_nul(bytes).map_err(|e| match e {
                FromBytesWithNulError::InteriorNul { .. } => {
                    PyValueError::new_err("string contains interior NUL bytes")
                }
                FromBytesWithNulError::NotNulTerminated => {
                    PyValueError::new_err("string is not NUL-terminated")
                }
            })
        }
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        <String as FromPyObject>::type_input()
    }
}

/// Allows extracting CString from Python objects.
/// Accepts Python `str` objects and converts them to CString.
/// Fails if the string contains NUL bytes.
impl FromPyObject<'_, '_> for std::ffi::CString {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "str";

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        let py_string = obj.cast::<PyString>()?;
        let cow = py_string.to_cow()?;
        let bytes = cow.as_bytes();
        std::ffi::CString::new(bytes)
            .map_err(|_| PyValueError::new_err("string contains NUL bytes"))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        <String as FromPyObject>::type_input()
    }
}

mod tests {
    use crate::types::any::PyAnyMethods;
    use crate::{IntoPyObject, Python};
    use std::borrow::Cow;

    #[test]
    fn test_cow_into_pyobject() {
        Python::attach(|py| {
            let s = "Hello Python";
            let py_string = Cow::Borrowed(s).into_pyobject(py).unwrap();
            assert_eq!(s, py_string.extract::<Cow<'_, str>>().unwrap());
            let py_string = Cow::<str>::Owned(s.into()).into_pyobject(py).unwrap();
            assert_eq!(s, py_string.extract::<Cow<'_, str>>().unwrap());
        })
    }

    #[test]
    fn test_non_bmp() {
        Python::attach(|py| {
            let s = "\u{1F30F}";
            let py_string = s.into_pyobject(py).unwrap();
            assert_eq!(s, py_string.extract::<String>().unwrap());
        })
    }

    #[test]
    fn test_extract_str() {
        Python::attach(|py| {
            let s = "Hello Python";
            let py_string = s.into_pyobject(py).unwrap();

            let s2: Cow<'_, str> = py_string.extract().unwrap();
            assert_eq!(s, s2);

            let cstr: &std::ffi::CStr = py_string.extract().unwrap();
            assert_eq!(s, cstr.to_str().unwrap());

            let cstring: std::ffi::CString = py_string.extract().unwrap();
            assert_eq!(s, cstring.to_str().unwrap());
        })
    }

    #[test]
    fn test_extract_str_unicode() {
        Python::attach(|py| {
            let s = "Hello 🐍 Python";
            let py_string = s.into_pyobject(py).unwrap();
            let s2: Cow<'_, str> = py_string.extract().unwrap();
            assert_eq!(s, s2);

            let cstr: &std::ffi::CStr = py_string.extract().unwrap();
            assert_eq!(s, cstr.to_str().unwrap());

            let cstring: std::ffi::CString = py_string.extract().unwrap();
            assert_eq!(s, cstring.to_str().unwrap());
        })
    }

    #[test]
    fn test_extract_char() {
        Python::attach(|py| {
            let ch = '😃';
            let py_string = ch.into_pyobject(py).unwrap();
            let ch2: char = py_string.extract().unwrap();
            assert_eq!(ch, ch2);
        })
    }

    #[test]
    fn test_extract_char_err() {
        Python::attach(|py| {
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
        Python::attach(|py| {
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

            let cstr = std::ffi::CStr::from_bytes_with_nul(b"Hello Python\0").unwrap();
            let py_string = cstr.into_pyobject(py).unwrap();
            assert_eq!(s, py_string.extract::<String>().unwrap());

            let cstring = std::ffi::CString::new("Hello Python").unwrap();
            let py_string = cstring.clone().into_pyobject(py).unwrap();
            assert_eq!(s, py_string.extract::<String>().unwrap());

            let py_string = (&cstring).into_pyobject(py).unwrap();
            assert_eq!(s, py_string.extract::<String>().unwrap());
        })
    }

    #[test]
    fn test_extract_with_nul_error() {
        Python::attach(|py| {
            let s = "Hello\0Python";
            let py_string = s.into_pyobject(py).unwrap();

            let err: crate::PyResult<&std::ffi::CStr> = py_string.extract();
            assert!(err
                .unwrap_err()
                .to_string()
                .contains("string contains interior NUL bytes"));

            let err: crate::PyResult<std::ffi::CString> = py_string.extract();
            assert!(err
                .unwrap_err()
                .to_string()
                .contains("string contains NUL bytes"));
        })
    }
}
