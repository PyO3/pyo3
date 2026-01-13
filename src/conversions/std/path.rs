use crate::conversion::IntoPyObject;
use crate::ffi_ptr_ext::FfiPtrExt;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::{type_hint_identifier, type_hint_union, PyStaticExpr};
use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
use crate::{ffi, Borrowed, Bound, FromPyObject, Py, PyAny, PyErr, Python};
use std::borrow::Cow;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

impl FromPyObject<'_, '_> for PathBuf {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = type_hint_union!(
        OsString::INPUT_TYPE,
        type_hint_identifier!("os", "PathLike")
    );

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = type_hint_identifier!("pathlib", "Path");

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&Path>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for Cow<'_, Path> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&Path>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &Cow<'_, Path> {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&Path>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

impl<'a> FromPyObject<'a, '_> for Cow<'a, Path> {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = PathBuf::INPUT_TYPE;

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&Path>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for &PathBuf {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <&Path>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        types::{PyAnyMethods, PyString},
        IntoPyObjectExt,
    };
    #[cfg(not(target_os = "wasi"))]
    use std::ffi::OsStr;
    use std::fmt::Debug;
    #[cfg(any(unix, target_os = "emscripten"))]
    use std::os::unix::ffi::OsStringExt;
    #[cfg(windows)]
    use std::os::windows::ffi::OsStringExt;

    #[test]
    #[cfg(any(unix, target_os = "emscripten"))]
    fn test_non_utf8_conversion() {
        Python::attach(|py| {
            use std::os::unix::ffi::OsStrExt;

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
            fn test_extract<'py, T>(py: Python<'py>, path: &T, is_borrowed: bool)
            where
                for<'a> &'a T: IntoPyObject<'py, Output = Bound<'py, PyString>>,
                for<'a> <&'a T as IntoPyObject<'py>>::Error: Debug,
                T: AsRef<Path> + ?Sized,
            {
                let pystring = path.into_pyobject(py).unwrap();
                let cow: Cow<'_, Path> = pystring.extract().unwrap();
                assert_eq!(cow, path.as_ref());
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
            #[cfg(any(unix, windows, target_os = "emscripten"))]
            test_extract::<OsStr>(py, &os_str, false);
        });
    }
}
