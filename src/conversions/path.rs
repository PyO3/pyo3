use crate::types::PyType;
use crate::{FromPyObject, IntoPy, PyAny, PyObject, PyResult, Python, ToPyObject};
use std::borrow::Cow;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

impl ToPyObject for Path {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_os_str().to_object(py)
    }
}

// See osstr.rs for why there's no FromPyObject impl for &Path

impl FromPyObject<'_> for PathBuf {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let os_str = match OsString::extract(ob) {
            Ok(s) => s,
            Err(err) => {
                let py = ob.py();
                let pathlib = py.import("pathlib")?;
                let pathlib_path: &PyType = pathlib.getattr("Path")?.downcast()?;
                if ob.is_instance(pathlib_path)? {
                    let path_str = ob.call_method0("__str__")?;
                    OsString::extract(path_str)?
                } else {
                    return Err(err);
                }
            }
        };
        Ok(PathBuf::from(os_str))
    }
}

impl<'a> IntoPy<PyObject> for &'a Path {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.as_os_str().to_object(py)
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

impl<'a> IntoPy<PyObject> for &'a PathBuf {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.as_os_str().to_object(py)
    }
}

#[cfg(test)]
mod tests {
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
                let pystring: &PyString = pyobject.extract(py).unwrap();
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
                let pystring: &PyString = pyobject.extract(py).unwrap();
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
