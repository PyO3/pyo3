use crate::exceptions::PyValueError;
use crate::types::PyString;
use crate::{ffi, Borrowed, Bound, FromPyObject, IntoPyObject, PyAny, PyErr, Python};
use std::borrow::Cow;
use std::ffi::{CStr, CString, FromBytesWithNulError};
use std::slice;
use std::str::Utf8Error;

impl<'py> IntoPyObject<'py> for &CStr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Utf8Error;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.to_str()?.into_pyobject(py).map_err(|err| match err {})
    }
}

impl<'py> IntoPyObject<'py> for CString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Utf8Error;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &CString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Utf8Error;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for Cow<'_, CStr> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Utf8Error;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Cow<'_, CStr> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Utf8Error;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
impl<'a> FromPyObject<'a, '_> for &'a CStr {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, '_, PyAny>) -> Result<Self, Self::Error> {
        let mut size = 0;
        let ptr = unsafe { ffi::PyUnicode_AsUTF8AndSize(obj.as_ptr(), &mut size) };

        if ptr.is_null() {
            return Err(PyErr::fetch(obj.py()));
        }

        // SAFETY: PyUnicode_AsUTF8AndSize always returns a NUL-terminated string
        let slice = unsafe { slice::from_raw_parts(ptr.cast(), size as usize + 1) };

        CStr::from_bytes_with_nul(slice).map_err(|err| match err {
            FromBytesWithNulError::InteriorNul { .. } => PyValueError::new_err(err.to_string()),
            FromBytesWithNulError::NotNulTerminated => {
                unreachable!("PyUnicode_AsUTF8AndSize always returns a NUL-terminated string")
            }
        })
    }
}

impl<'a> FromPyObject<'a, '_> for Cow<'a, CStr> {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, '_, PyAny>) -> Result<Self, Self::Error> {
        #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
        {
            Ok(Cow::Borrowed(obj.extract::<&CStr>()?))
        }

        #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
        {
            Ok(Cow::Owned(obj.extract::<CString>()?))
        }
    }
}
impl FromPyObject<'_, '_> for CString {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
        {
            Ok(obj.extract::<&CStr>()?.to_owned())
        }

        #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
        {
            CString::new(&*obj.cast::<PyString>()?.to_cow()?).map_err(Into::into)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::string::PyStringMethods;
    use crate::types::PyAnyMethods;
    use crate::Python;

    #[test]
    fn test_into_pyobject() {
        Python::attach(|py| {
            let s = "Hello, Python!";
            let cstr = CString::new(s).unwrap();

            let py_string = cstr.as_c_str().into_pyobject(py).unwrap();
            assert_eq!(py_string.to_cow().unwrap(), s);

            let py_string = cstr.into_pyobject(py).unwrap();
            assert_eq!(py_string.to_cow().unwrap(), s);
        })
    }

    #[test]
    fn test_extract_with_nul_error() {
        Python::attach(|py| {
            let s = "Hello\0Python";
            let py_string = s.into_pyobject(py).unwrap();

            #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
            {
                let err = py_string.extract::<&CStr>();
                assert_eq!(
                    err.unwrap_err().value(py).repr().unwrap().to_string(),
                    "ValueError('data provided contains an interior nul byte at byte pos 5')"
                );
            }

            let err = py_string.extract::<CString>();
            assert_eq!(
                err.unwrap_err().to_string(),
                "ValueError: data provided contains an interior nul byte at byte pos 5"
            );
        })
    }

    #[test]
    fn test_extract_cstr_and_cstring() {
        Python::attach(|py| {
            let s = "Hello, world!";
            let cstr = CString::new(s).unwrap();
            let py_string = cstr.as_c_str().into_pyobject(py).unwrap();

            #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
            {
                let extracted_cstr: &CStr = py_string.extract().unwrap();
                assert_eq!(extracted_cstr.to_str().unwrap(), s);
            }

            let extracted_cstring: CString = py_string.extract().unwrap();
            assert_eq!(extracted_cstring.to_str().unwrap(), s);
        })
    }

    #[test]
    fn test_cow_roundtrip() {
        Python::attach(|py| {
            let s = "Hello, world!";
            let cstr = CString::new(s).unwrap();
            let cow: Cow<'_, CStr> = Cow::Borrowed(cstr.as_c_str());

            let py_string = cow.into_pyobject(py).unwrap();
            assert_eq!(py_string.to_cow().unwrap(), s);

            let roundtripped: Cow<'_, CStr> = py_string.extract().unwrap();
            assert_eq!(roundtripped, cstr);
        })
    }
}
