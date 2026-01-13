use crate::conversion::IntoPyObject;
#[cfg(not(target_os = "wasi"))]
use crate::ffi;
#[cfg(not(target_os = "wasi"))]
use crate::ffi_ptr_ext::FfiPtrExt;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::PyStaticExpr;
use crate::instance::Bound;
#[cfg(feature = "experimental-inspect")]
use crate::type_object::PyTypeInfo;
use crate::types::PyString;
#[cfg(any(unix, target_os = "emscripten"))]
use crate::types::{PyBytes, PyBytesMethods};
use crate::{Borrowed, FromPyObject, PyAny, PyErr, Python};
use std::borrow::Cow;
use std::convert::Infallible;
use std::ffi::{OsStr, OsString};
#[cfg(any(unix, target_os = "emscripten"))]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

impl<'py> IntoPyObject<'py> for &OsStr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = PyString::TYPE_HINT;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        // If the string is UTF-8, take the quick and easy shortcut
        #[cfg(not(target_os = "wasi"))]
        if let Some(valid_utf8_path) = self.to_str() {
            return valid_utf8_path.into_pyobject(py);
        }

        #[cfg(target_os = "wasi")]
        {
            self.to_str()
                .expect("wasi strings are UTF8")
                .into_pyobject(py)
        }

        #[cfg(any(unix, target_os = "emscripten"))]
        {
            let bytes = self.as_bytes();
            let ptr = bytes.as_ptr().cast();
            let len = bytes.len() as ffi::Py_ssize_t;
            unsafe {
                // DecodeFSDefault automatically chooses an appropriate decoding mechanism to
                // parse os strings losslessly (i.e. surrogateescape most of the time)
                Ok(ffi::PyUnicode_DecodeFSDefaultAndSize(ptr, len)
                    .assume_owned(py)
                    .cast_into_unchecked())
            }
        }

        #[cfg(windows)]
        {
            let wstr: Vec<u16> = self.encode_wide().collect();
            unsafe {
                // This will not panic because the data from encode_wide is well-formed Windows
                // string data

                Ok(
                    ffi::PyUnicode_FromWideChar(wstr.as_ptr(), wstr.len() as ffi::Py_ssize_t)
                        .assume_owned(py)
                        .cast_into_unchecked(),
                )
            }
        }
    }
}

impl<'py> IntoPyObject<'py> for &&OsStr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&OsStr>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl FromPyObject<'_, '_> for OsString {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = PyString::TYPE_HINT;

    fn extract(ob: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        let pystring = ob.cast::<PyString>()?;

        #[cfg(target_os = "wasi")]
        {
            Ok(pystring.to_cow()?.into_owned().into())
        }

        #[cfg(any(unix, target_os = "emscripten"))]
        {
            // Decode from Python's lossless bytes string representation back into raw bytes
            // SAFETY: PyUnicode_EncodeFSDefault returns a new reference or null on error, known to
            // be a `bytes` object, thread is attached to the interpreter
            let fs_encoded_bytes = unsafe {
                ffi::PyUnicode_EncodeFSDefault(pystring.as_ptr())
                    .assume_owned_or_err(ob.py())?
                    .cast_into_unchecked::<PyBytes>()
            };

            // Create an OsStr view into the raw bytes from Python
            let os_str: &OsStr = OsStrExt::from_bytes(fs_encoded_bytes.as_bytes());

            Ok(os_str.to_os_string())
        }

        #[cfg(windows)]
        {
            // Take the quick and easy shortcut if UTF-8
            if let Ok(utf8_string) = pystring.to_cow() {
                return Ok(utf8_string.into_owned().into());
            }

            // Get an owned allocated wide char buffer from PyString, which we have to deallocate
            // ourselves
            let size =
                unsafe { ffi::PyUnicode_AsWideChar(pystring.as_ptr(), std::ptr::null_mut(), 0) };
            crate::err::error_on_minusone(ob.py(), size)?;

            debug_assert!(
                size > 0,
                "PyUnicode_AsWideChar should return at least 1 for null terminator"
            );
            let size = size - 1; // exclude null terminator

            let mut buffer = vec![0; size as usize];
            let bytes_read =
                unsafe { ffi::PyUnicode_AsWideChar(pystring.as_ptr(), buffer.as_mut_ptr(), size) };
            assert_eq!(bytes_read, size);

            // Copy wide char buffer into OsString
            let os_string = std::os::windows::ffi::OsStringExt::from_wide(&buffer);

            Ok(os_string)
        }
    }
}

impl<'py> IntoPyObject<'py> for Cow<'_, OsStr> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&OsStr>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Cow<'_, OsStr> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&OsStr>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

impl<'a> FromPyObject<'a, '_> for Cow<'a, OsStr> {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = OsString::INPUT_TYPE;

    fn extract(obj: Borrowed<'a, '_, PyAny>) -> Result<Self, Self::Error> {
        #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
        if let Ok(s) = obj.extract::<&str>() {
            return Ok(Cow::Borrowed(s.as_ref()));
        }

        obj.extract::<OsString>().map(Cow::Owned)
    }
}

impl<'py> IntoPyObject<'py> for OsString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&OsStr>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_os_str().into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &OsString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&OsStr>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_os_str().into_pyobject(py)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "wasi")]
    use crate::exceptions::PyFileNotFoundError;
    use crate::types::{PyAnyMethods, PyString, PyStringMethods};
    use crate::{Bound, BoundObject, IntoPyObject, Python};
    use std::fmt::Debug;
    #[cfg(any(unix, target_os = "emscripten"))]
    use std::os::unix::ffi::OsStringExt;
    #[cfg(windows)]
    use std::os::windows::ffi::OsStringExt;
    use std::{
        borrow::Cow,
        ffi::{OsStr, OsString},
    };

    #[test]
    #[cfg(any(unix, target_os = "emscripten"))]
    fn test_non_utf8_conversion() {
        Python::attach(|py| {
            use std::os::unix::ffi::OsStrExt;

            // this is not valid UTF-8
            let payload = &[250, 251, 252, 253, 254, 255, 0, 255];
            let os_str = OsStr::from_bytes(payload);

            // do a roundtrip into Pythonland and back and compare
            let py_str = os_str.into_pyobject(py).unwrap();
            let os_str_2: OsString = py_str.extract().unwrap();
            assert_eq!(os_str, os_str_2);
        });
    }

    #[test]
    #[cfg(target_os = "wasi")]
    fn test_extract_non_utf8_wasi_should_error() {
        Python::attach(|py| {
            // Non utf-8 strings are not valid wasi paths
            let open_result = py.run(c"open('\\udcff', 'rb')", None, None).unwrap_err();
            assert!(
                !open_result.is_instance_of::<PyFileNotFoundError>(py),
                "Opening invalid utf8 will error with OSError, not FileNotFoundError"
            );

            // Create a Python string with not valid UTF-8: &[255]
            let py_str = py.eval(c"'\\udcff'", None, None).unwrap();
            assert!(
                py_str.extract::<OsString>().is_err(),
                "Extracting invalid UTF-8 as OsString should error"
            );
        });
    }

    #[test]
    fn test_intopyobject_roundtrip() {
        Python::attach(|py| {
            fn test_roundtrip<'py, T>(py: Python<'py>, obj: T)
            where
                T: IntoPyObject<'py> + AsRef<OsStr> + Debug + Clone,
                T::Error: Debug,
            {
                let pyobject = obj.clone().into_pyobject(py).unwrap().into_any();
                let pystring = pyobject.as_borrowed().cast::<PyString>().unwrap();
                assert_eq!(pystring.to_string_lossy(), obj.as_ref().to_string_lossy());
                let roundtripped_obj: OsString = pystring.extract().unwrap();
                assert_eq!(obj.as_ref(), roundtripped_obj.as_os_str());
            }
            let os_str = OsStr::new("Hello\0\n🐍");
            test_roundtrip::<&OsStr>(py, os_str);
            test_roundtrip::<Cow<'_, OsStr>>(py, Cow::Borrowed(os_str));
            test_roundtrip::<Cow<'_, OsStr>>(py, Cow::Owned(os_str.to_os_string()));
            test_roundtrip::<OsString>(py, os_str.to_os_string());
        });
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_non_utf8_osstring_roundtrip() {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};

        Python::attach(|py| {
            // Example: Unpaired surrogate (0xD800) is not valid UTF-8, but valid in Windows OsString
            let wide: &[u16] = &['A' as u16, 0xD800, 'B' as u16]; // 'A', unpaired surrogate, 'B'
            let os_str = OsString::from_wide(wide);

            assert_eq!(os_str.to_string_lossy(), "A�B");

            // This cannot be represented as UTF-8, so .to_str() would return None
            assert!(os_str.to_str().is_none());

            // Convert to Python and back
            let py_str = os_str.as_os_str().into_pyobject(py).unwrap();
            let os_str_2 = py_str.extract::<OsString>().unwrap();

            // The roundtrip should preserve the original wide data
            assert_eq!(os_str, os_str_2);

            // Show that encode_wide is necessary: direct UTF-8 conversion would lose information
            let encoded: Vec<u16> = os_str.encode_wide().collect();
            assert_eq!(encoded, wide);
        });
    }

    #[test]
    fn test_extract_cow() {
        Python::attach(|py| {
            fn test_extract<'py, T>(py: Python<'py>, input: &T, is_borrowed: bool)
            where
                for<'a> &'a T: IntoPyObject<'py, Output = Bound<'py, PyString>>,
                for<'a> <&'a T as IntoPyObject<'py>>::Error: Debug,
                T: AsRef<OsStr> + ?Sized,
            {
                let pystring = input.into_pyobject(py).unwrap();
                let cow: Cow<'_, OsStr> = pystring.extract().unwrap();
                assert_eq!(cow, input.as_ref());
                assert_eq!(is_borrowed, matches!(cow, Cow::Borrowed(_)));
            }

            // On Python 3.10+ or when not using the limited API, we can borrow strings from python
            let can_borrow_str = cfg!(any(Py_3_10, not(Py_LIMITED_API)));
            // This can be borrowed because it is valid UTF-8
            test_extract::<str>(py, "Hello\0\n🐍", can_borrow_str);
            test_extract::<str>(py, "Hello, world!", can_borrow_str);

            #[cfg(windows)]
            let os_str = {
                // 'A', unpaired surrogate, 'B'
                OsString::from_wide(&['A' as u16, 0xD800, 'B' as u16])
            };

            #[cfg(any(unix, target_os = "emscripten"))]
            let os_str = { OsString::from_vec(vec![250, 251, 252, 253, 254, 255, 0, 255]) };

            // This cannot be borrowed because it is not valid UTF-8
            #[cfg(any(windows, unix, target_os = "emscripten"))]
            test_extract::<OsStr>(py, &os_str, false);
        });
    }
}
