use crate::conversion::IntoPyObject;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Bound;
use crate::types::PyString;
use crate::{ffi, Borrowed, FromPyObject, PyAny, PyErr, Python};
use std::borrow::Cow;
use std::convert::Infallible;
use std::ffi::{OsStr, OsString};

impl<'py> IntoPyObject<'py> for &OsStr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        // If the string is UTF-8, take the quick and easy shortcut
        if let Some(valid_utf8_path) = self.to_str() {
            return valid_utf8_path.into_pyobject(py);
        }

        // All targets besides windows support the std::os::unix::ffi::OsStrExt API:
        // https://doc.rust-lang.org/src/std/sys_common/mod.rs.html#59
        #[cfg(not(windows))]
        {
            #[cfg(target_os = "wasi")]
            let bytes = self.to_str().expect("wasi strings are UTF8").as_bytes();
            #[cfg(not(target_os = "wasi"))]
            let bytes = std::os::unix::ffi::OsStrExt::as_bytes(self);

            let ptr = bytes.as_ptr().cast();
            let len = bytes.len() as ffi::Py_ssize_t;
            unsafe {
                // DecodeFSDefault automatically chooses an appropriate decoding mechanism to
                // parse os strings losslessly (i.e. surrogateescape most of the time)
                Ok(ffi::PyUnicode_DecodeFSDefaultAndSize(ptr, len)
                    .assume_owned(py)
                    .cast_into_unchecked::<PyString>())
            }
        }

        #[cfg(windows)]
        {
            let wstr: Vec<u16> = std::os::windows::ffi::OsStrExt::encode_wide(self).collect();

            unsafe {
                // This will not panic because the data from encode_wide is well-formed Windows
                // string data

                Ok(
                    ffi::PyUnicode_FromWideChar(wstr.as_ptr(), wstr.len() as ffi::Py_ssize_t)
                        .assume_owned(py)
                        .cast_into_unchecked::<PyString>(),
                )
            }
        }
    }
}

impl<'py> IntoPyObject<'py> for &&OsStr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

// There's no FromPyObject implementation for &OsStr because albeit possible on Unix, this would
// be impossible to implement on Windows. Hence it's omitted entirely

impl FromPyObject<'_, '_> for OsString {
    type Error = PyErr;

    fn extract(ob: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        let pystring = ob.cast::<PyString>()?;

        #[cfg(not(windows))]
        {
            // Decode from Python's lossless bytes string representation back into raw bytes
            let fs_encoded_bytes = unsafe {
                crate::Py::<crate::types::PyBytes>::from_owned_ptr(
                    ob.py(),
                    ffi::PyUnicode_EncodeFSDefault(pystring.as_ptr()),
                )
            };

            // Create an OsStr view into the raw bytes from Python
            //
            // For WASI: OS strings are UTF-8 by definition.
            #[cfg(target_os = "wasi")]
            let os_str: &OsStr =
                OsStr::new(std::str::from_utf8(fs_encoded_bytes.as_bytes(ob.py()))?);
            #[cfg(not(target_os = "wasi"))]
            let os_str: &OsStr =
                std::os::unix::ffi::OsStrExt::from_bytes(fs_encoded_bytes.as_bytes(ob.py()));

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

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Cow<'_, OsStr> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for OsString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_os_str().into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &OsString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_os_str().into_pyobject(py)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyAnyMethods, PyString, PyStringMethods};
    use crate::{BoundObject, IntoPyObject, Python};
    use std::fmt::Debug;
    use std::{
        borrow::Cow,
        ffi::{OsStr, OsString},
    };

    #[test]
    #[cfg(not(windows))]
    fn test_non_utf8_conversion() {
        Python::attach(|py| {
            #[cfg(not(target_os = "wasi"))]
            use std::os::unix::ffi::OsStrExt;
            #[cfg(target_os = "wasi")]
            use std::os::wasi::ffi::OsStrExt;

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
}
