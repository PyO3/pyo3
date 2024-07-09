use crate::conversion::IntoPyObject;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Bound;
use crate::types::any::PyAnyMethods;
use crate::types::PyString;
use crate::{ffi, FromPyObject, IntoPy, PyAny, PyObject, PyResult, Python, ToPyObject};
use std::borrow::Cow;
use std::convert::Infallible;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

impl ToPyObject for Path {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_os_str().to_object(py)
    }
}

// See osstr.rs for why there's no FromPyObject impl for &Path

impl FromPyObject<'_> for PathBuf {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        // We use os.fspath to get the underlying path as bytes or str
        let path = unsafe { ffi::PyOS_FSPath(ob.as_ptr()).assume_owned_or_err(ob.py())? };
        Ok(path.extract::<OsString>()?.into())
    }
}

impl<'a> IntoPy<PyObject> for &'a Path {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.as_os_str().to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for &Path {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_os_str().into_pyobject(py)
    }
}

impl<'a> ToPyObject for Cow<'a, Path> {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_os_str().to_object(py)
    }
}

impl<'a> IntoPy<PyObject> for Cow<'a, Path> {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for Cow<'_, Path> {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_os_str().into_pyobject(py)
    }
}

impl ToPyObject for PathBuf {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_os_str().to_object(py)
    }
}

impl IntoPy<PyObject> for PathBuf {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_os_string().to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for PathBuf {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_os_str().into_pyobject(py)
    }
}

impl<'a> IntoPy<PyObject> for &'a PathBuf {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.as_os_str().to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for &PathBuf {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_os_str().into_pyobject(py)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyAnyMethods, PyStringMethods};
    use crate::{types::PyString, IntoPy, PyObject, Python, ToPyObject};
    use std::borrow::Cow;
    use std::fmt::Debug;
    use std::path::{Path, PathBuf};

    #[test]
    #[cfg(not(windows))]
    fn test_non_utf8_conversion() {
        Python::with_gil(|py| {
            use std::ffi::OsStr;
            #[cfg(not(target_os = "wasi"))]
            use std::os::unix::ffi::OsStrExt;
            #[cfg(target_os = "wasi")]
            use std::os::wasi::ffi::OsStrExt;

            // this is not valid UTF-8
            let payload = &[250, 251, 252, 253, 254, 255, 0, 255];
            let path = Path::new(OsStr::from_bytes(payload));

            // do a roundtrip into Pythonland and back and compare
            let py_str: PyObject = path.into_py(py);
            let path_2: PathBuf = py_str.extract(py).unwrap();
            assert_eq!(path, path_2);
        });
    }

    #[test]
    fn test_topyobject_roundtrip() {
        Python::with_gil(|py| {
            fn test_roundtrip<T: ToPyObject + AsRef<Path> + Debug>(py: Python<'_>, obj: T) {
                let pyobject = obj.to_object(py);
                let pystring = pyobject.downcast_bound::<PyString>(py).unwrap();
                assert_eq!(pystring.to_string_lossy(), obj.as_ref().to_string_lossy());
                let roundtripped_obj: PathBuf = pystring.extract().unwrap();
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
    fn test_intopy_roundtrip() {
        Python::with_gil(|py| {
            fn test_roundtrip<T: IntoPy<PyObject> + AsRef<Path> + Debug + Clone>(
                py: Python<'_>,
                obj: T,
            ) {
                let pyobject = obj.clone().into_py(py);
                let pystring = pyobject.downcast_bound::<PyString>(py).unwrap();
                assert_eq!(pystring.to_string_lossy(), obj.as_ref().to_string_lossy());
                let roundtripped_obj: PathBuf = pystring.extract().unwrap();
                assert_eq!(obj.as_ref(), roundtripped_obj.as_path());
            }
            let path = Path::new("Hello\0\n🐍");
            test_roundtrip::<&Path>(py, path);
            test_roundtrip::<PathBuf>(py, path.to_path_buf());
            test_roundtrip::<&PathBuf>(py, &path.to_path_buf());
        })
    }
}
