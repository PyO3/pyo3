use crate::conversion::IntoPyObject;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
use crate::{ffi, Borrowed, Bound, FromPyObject, Py, PyAny, PyErr, Python};
use std::borrow::Cow;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

impl FromPyObject<'_, '_> for PathBuf {
    type Error = PyErr;

    fn extract(ob: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        // We use os.fspath to get the underlying path as bytes or str
        let path = unsafe { ffi::PyOS_FSPath(ob.as_ptr()).assume_owned_or_err(ob.py())? };
        Ok(path.extract::<OsString>()?.into())
    }
}

impl<'py> IntoPyObject<'py> for &Path {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static PY_PATH: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
        PY_PATH
            .import(py, "pathlib", "Path")?
            .call((self.as_os_str(),), None)
    }
}

impl<'py> IntoPyObject<'py> for &&Path {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for Cow<'_, Path> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Cow<'_, Path> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

impl<'a> FromPyObject<'a, '_> for Cow<'a, Path> {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, '_, PyAny>) -> Result<Self, Self::Error> {
        #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
        if let Ok(s) = obj.extract::<&str>() {
            return Ok(Cow::Borrowed(s.as_ref()));
        }

        obj.extract::<PathBuf>().map(Cow::Owned)
    }
}

impl<'py> IntoPyObject<'py> for PathBuf {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &PathBuf {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyAnyMethods, PyString};
    use crate::{IntoPyObject, IntoPyObjectExt, Python};
    use std::borrow::Cow;
    use std::fmt::Debug;
    use std::path::{Path, PathBuf};

    #[test]
    #[cfg(not(windows))]
    fn test_non_utf8_conversion() {
        Python::attach(|py| {
            use crate::types::PyAnyMethods;
            use std::ffi::OsStr;
            #[cfg(not(target_os = "wasi"))]
            use std::os::unix::ffi::OsStrExt;
            #[cfg(target_os = "wasi")]
            use std::os::wasi::ffi::OsStrExt;

            // this is not valid UTF-8
            let payload = &[250, 251, 252, 253, 254, 255, 0, 255];
            let path = Path::new(OsStr::from_bytes(payload));

            // do a roundtrip into Pythonland and back and compare
            let py_str = path.into_pyobject(py).unwrap();
            let path_2: PathBuf = py_str.extract().unwrap();
            assert_eq!(path, path_2);
        });
    }

    #[test]
    fn test_intopyobject_roundtrip() {
        Python::attach(|py| {
            fn test_roundtrip<'py, T>(py: Python<'py>, obj: T)
            where
                T: IntoPyObject<'py> + AsRef<Path> + Debug + Clone,
                T::Error: Debug,
            {
                let pyobject = obj.clone().into_bound_py_any(py).unwrap();
                let roundtripped_obj: PathBuf = pyobject.extract().unwrap();
                assert_eq!(obj.as_ref(), roundtripped_obj.as_path());
            }
            let path = Path::new("Hello\0\n🐍");
            test_roundtrip::<&Path>(py, path);
            test_roundtrip::<Cow<'_, Path>>(py, Cow::Borrowed(path));
            test_roundtrip::<Cow<'_, Path>>(py, Cow::Owned(path.to_path_buf()));
            test_roundtrip::<PathBuf>(py, path.to_path_buf());
        });
    }

    #[test]
    fn test_from_pystring() {
        Python::attach(|py| {
            let path = "Hello\0\n🐍";
            let pystring = PyString::new(py, path);
            let roundtrip: PathBuf = pystring.extract().unwrap();
            assert_eq!(roundtrip, Path::new(path));
        });
    }

    #[test]
    fn test_extract_cow() {
        Python::attach(|py| {
            fn test_extract(py: Python<'_>, path: &str) {
                let pystring = path.into_pyobject(py).unwrap();
                let cow: Cow<'_, Path> = pystring.extract().unwrap();
                assert_eq!(cow, AsRef::<Path>::as_ref(path));
            }

            // Test extracting both valid UTF-8 and non-UTF-8 strings
            test_extract(py, "Hello\0\n🐍");
            test_extract(py, "Hello, world!");
        });
    }
}
