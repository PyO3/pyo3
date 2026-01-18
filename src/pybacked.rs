//! Contains types for working with Python objects that own the underlying data.

#[cfg(feature = "experimental-inspect")]
use crate::inspect::PyStaticExpr;
use crate::{
    types::{
        bytearray::PyByteArrayMethods, bytes::PyBytesMethods, string::PyStringMethods, PyByteArray,
        PyBytes, PyString, PyTuple,
    },
    Borrowed, Bound, CastError, FromPyObject, IntoPyObject, Py, PyAny, PyErr, PyTypeInfo, Python,
};
use std::{borrow::Borrow, convert::Infallible, ops::Deref, ptr::NonNull, sync::Arc};

/// An equivalent to `String` where the storage is owned by a Python `bytes` or `str` object.
///
/// On Python 3.10+ or when not using the stable API, this type is guaranteed to contain a Python `str`
/// for the underlying data.
///
/// This type gives access to the underlying data via a `Deref` implementation.
#[cfg_attr(feature = "py-clone", derive(Clone))]
pub struct PyBackedStr {
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    storage: Py<PyString>,
    #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
    storage: Py<PyBytes>,
    data: NonNull<str>,
}

impl PyBackedStr {
    /// Clones this by incrementing the reference count of the underlying Python object.
    ///
    /// Similar to [`Py::clone_ref`], this method is always available, even when the `py-clone` feature is disabled.
    #[inline]
    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        Self {
            storage: self.storage.clone_ref(py),
            data: self.data,
        }
    }

    /// Returns the underlying data as a `&str` slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        // Safety: `data` is known to be immutable and owned by self
        unsafe { self.data.as_ref() }
    }

    /// Returns the underlying data as a Python `str`.
    ///
    /// Older versions of the Python stable API do not support this zero-cost conversion.
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    #[inline]
    pub fn as_py_str(&self) -> &Py<PyString> {
        &self.storage
    }
}

impl Deref for PyBackedStr {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for PyBackedStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsRef<[u8]> for PyBackedStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Borrow<str> for PyBackedStr {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

// Safety: the underlying Python str (or bytes) is immutable and
// safe to share between threads
unsafe impl Send for PyBackedStr {}
unsafe impl Sync for PyBackedStr {}

impl std::fmt::Display for PyBackedStr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl_traits!(PyBackedStr, str);

impl TryFrom<Bound<'_, PyString>> for PyBackedStr {
    type Error = PyErr;
    fn try_from(py_string: Bound<'_, PyString>) -> Result<Self, Self::Error> {
        #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
        {
            let s = py_string.to_str()?;
            let data = NonNull::from(s);
            Ok(Self {
                storage: py_string.unbind(),
                data,
            })
        }
        #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
        {
            let bytes = py_string.encode_utf8()?;
            let s = unsafe { std::str::from_utf8_unchecked(bytes.as_bytes()) };
            let data = NonNull::from(s);
            Ok(Self {
                storage: bytes.unbind(),
                data,
            })
        }
    }
}

impl FromPyObject<'_, '_> for PyBackedStr {
    type Error = PyErr;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = PyString::TYPE_HINT;

    #[inline]
    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        let py_string = obj.cast::<PyString>()?.to_owned();
        Self::try_from(py_string)
    }
}

impl<'py> IntoPyObject<'py> for PyBackedStr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = PyString::TYPE_HINT;

    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.storage.into_bound(py))
    }

    #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, &self))
    }
}

impl<'py> IntoPyObject<'py> for &PyBackedStr {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = PyString::TYPE_HINT;

    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.storage.bind(py).to_owned())
    }

    #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self))
    }
}

/// A wrapper around `[u8]` where the storage is either owned by a Python `bytes` object, or a Rust `Box<[u8]>`.
///
/// This type gives access to the underlying data via a `Deref` implementation.
#[cfg_attr(feature = "py-clone", derive(Clone))]
pub struct PyBackedBytes {
    storage: PyBackedBytesStorage,
    data: NonNull<[u8]>,
}

#[cfg_attr(feature = "py-clone", derive(Clone))]
enum PyBackedBytesStorage {
    Python(Py<PyBytes>),
    Rust(Arc<[u8]>),
}

impl PyBackedBytes {
    /// Clones this by incrementing the reference count of the underlying data.
    ///
    /// Similar to [`Py::clone_ref`], this method is always available, even when the `py-clone` feature is disabled.
    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        Self {
            storage: match &self.storage {
                PyBackedBytesStorage::Python(bytes) => {
                    PyBackedBytesStorage::Python(bytes.clone_ref(py))
                }
                PyBackedBytesStorage::Rust(bytes) => PyBackedBytesStorage::Rust(bytes.clone()),
            },
            data: self.data,
        }
    }
}

impl Deref for PyBackedBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        // Safety: `data` is known to be immutable and owned by self
        unsafe { self.data.as_ref() }
    }
}

impl AsRef<[u8]> for PyBackedBytes {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

// Safety: the underlying Python bytes or Rust bytes is immutable and
// safe to share between threads
unsafe impl Send for PyBackedBytes {}
unsafe impl Sync for PyBackedBytes {}

impl<const N: usize> PartialEq<[u8; N]> for PyBackedBytes {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.deref() == other
    }
}

impl<const N: usize> PartialEq<PyBackedBytes> for [u8; N] {
    fn eq(&self, other: &PyBackedBytes) -> bool {
        self == other.deref()
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for PyBackedBytes {
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.deref() == *other
    }
}

impl<const N: usize> PartialEq<PyBackedBytes> for &[u8; N] {
    fn eq(&self, other: &PyBackedBytes) -> bool {
        self == &other.deref()
    }
}

impl_traits!(PyBackedBytes, [u8]);

impl From<Bound<'_, PyBytes>> for PyBackedBytes {
    fn from(py_bytes: Bound<'_, PyBytes>) -> Self {
        let b = py_bytes.as_bytes();
        let data = NonNull::from(b);
        Self {
            storage: PyBackedBytesStorage::Python(py_bytes.to_owned().unbind()),
            data,
        }
    }
}

impl From<Bound<'_, PyByteArray>> for PyBackedBytes {
    fn from(py_bytearray: Bound<'_, PyByteArray>) -> Self {
        let s = Arc::<[u8]>::from(py_bytearray.to_vec());
        let data = NonNull::from(s.as_ref());
        Self {
            storage: PyBackedBytesStorage::Rust(s),
            data,
        }
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for PyBackedBytes {
    type Error = CastError<'a, 'py>;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = PyBytes::TYPE_HINT;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(bytes) = obj.cast::<PyBytes>() {
            Ok(Self::from(bytes.to_owned()))
        } else if let Ok(bytearray) = obj.cast::<PyByteArray>() {
            Ok(Self::from(bytearray.to_owned()))
        } else {
            Err(CastError::new(
                obj,
                PyTuple::new(
                    obj.py(),
                    [
                        PyBytes::type_object(obj.py()),
                        PyByteArray::type_object(obj.py()),
                    ],
                )
                .unwrap()
                .into_any(),
            ))
        }
    }
}

impl<'py> IntoPyObject<'py> for PyBackedBytes {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = PyBytes::TYPE_HINT;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self.storage {
            PyBackedBytesStorage::Python(bytes) => Ok(bytes.into_bound(py)),
            PyBackedBytesStorage::Rust(bytes) => Ok(PyBytes::new(py, &bytes)),
        }
    }
}

impl<'py> IntoPyObject<'py> for &PyBackedBytes {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = PyBytes::TYPE_HINT;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match &self.storage {
            PyBackedBytesStorage::Python(bytes) => Ok(bytes.bind(py).clone()),
            PyBackedBytesStorage::Rust(bytes) => Ok(PyBytes::new(py, bytes)),
        }
    }
}

macro_rules! impl_traits {
    ($slf:ty, $equiv:ty) => {
        impl std::fmt::Debug for $slf {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.deref().fmt(f)
            }
        }

        impl PartialEq for $slf {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.deref() == other.deref()
            }
        }

        impl PartialEq<$equiv> for $slf {
            #[inline]
            fn eq(&self, other: &$equiv) -> bool {
                self.deref() == other
            }
        }

        impl PartialEq<&$equiv> for $slf {
            #[inline]
            fn eq(&self, other: &&$equiv) -> bool {
                self.deref() == *other
            }
        }

        impl PartialEq<$slf> for $equiv {
            #[inline]
            fn eq(&self, other: &$slf) -> bool {
                self == other.deref()
            }
        }

        impl PartialEq<$slf> for &$equiv {
            #[inline]
            fn eq(&self, other: &$slf) -> bool {
                self == &other.deref()
            }
        }

        impl Eq for $slf {}

        impl PartialOrd for $slf {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialOrd<$equiv> for $slf {
            #[inline]
            fn partial_cmp(&self, other: &$equiv) -> Option<std::cmp::Ordering> {
                self.deref().partial_cmp(other)
            }
        }

        impl PartialOrd<$slf> for $equiv {
            #[inline]
            fn partial_cmp(&self, other: &$slf) -> Option<std::cmp::Ordering> {
                self.partial_cmp(other.deref())
            }
        }

        impl Ord for $slf {
            #[inline]
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.deref().cmp(other.deref())
            }
        }

        impl std::hash::Hash for $slf {
            #[inline]
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.deref().hash(state)
            }
        }
    };
}
use impl_traits;

#[cfg(test)]
mod test {
    use super::*;
    use crate::impl_::pyclass::{value_of, IsSend, IsSync};
    use crate::types::PyAnyMethods as _;
    use crate::{IntoPyObject, Python};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn py_backed_str_empty() {
        Python::attach(|py| {
            let s = PyString::new(py, "");
            let py_backed_str = s.extract::<PyBackedStr>().unwrap();
            assert_eq!(&*py_backed_str, "");
        });
    }

    #[test]
    fn py_backed_str() {
        Python::attach(|py| {
            let s = PyString::new(py, "hello");
            let py_backed_str = s.extract::<PyBackedStr>().unwrap();
            assert_eq!(&*py_backed_str, "hello");
        });
    }

    #[test]
    fn py_backed_str_try_from() {
        Python::attach(|py| {
            let s = PyString::new(py, "hello");
            let py_backed_str = PyBackedStr::try_from(s).unwrap();
            assert_eq!(&*py_backed_str, "hello");
        });
    }

    #[test]
    fn py_backed_str_into_pyobject() {
        Python::attach(|py| {
            let orig_str = PyString::new(py, "hello");
            let py_backed_str = orig_str.extract::<PyBackedStr>().unwrap();
            let new_str = py_backed_str.into_pyobject(py).unwrap();
            assert_eq!(new_str.extract::<PyBackedStr>().unwrap(), "hello");
            #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
            assert!(new_str.is(&orig_str));
        });
    }

    #[test]
    fn py_backed_bytes_empty() {
        Python::attach(|py| {
            let b = PyBytes::new(py, b"");
            let py_backed_bytes = b.extract::<PyBackedBytes>().unwrap();
            assert_eq!(&*py_backed_bytes, b"");
        });
    }

    #[test]
    fn py_backed_bytes() {
        Python::attach(|py| {
            let b = PyBytes::new(py, b"abcde");
            let py_backed_bytes = b.extract::<PyBackedBytes>().unwrap();
            assert_eq!(&*py_backed_bytes, b"abcde");
        });
    }

    #[test]
    fn py_backed_bytes_from_bytes() {
        Python::attach(|py| {
            let b = PyBytes::new(py, b"abcde");
            let py_backed_bytes = PyBackedBytes::from(b);
            assert_eq!(&*py_backed_bytes, b"abcde");
        });
    }

    #[test]
    fn py_backed_bytes_from_bytearray() {
        Python::attach(|py| {
            let b = PyByteArray::new(py, b"abcde");
            let py_backed_bytes = PyBackedBytes::from(b);
            assert_eq!(&*py_backed_bytes, b"abcde");
        });
    }

    #[test]
    fn py_backed_bytes_into_pyobject() {
        Python::attach(|py| {
            let orig_bytes = PyBytes::new(py, b"abcde");
            let py_backed_bytes = PyBackedBytes::from(orig_bytes.clone());
            assert!((&py_backed_bytes)
                .into_pyobject(py)
                .unwrap()
                .is(&orig_bytes));
        });
    }

    #[test]
    fn rust_backed_bytes_into_pyobject() {
        Python::attach(|py| {
            let orig_bytes = PyByteArray::new(py, b"abcde");
            let rust_backed_bytes = PyBackedBytes::from(orig_bytes);
            assert!(matches!(
                rust_backed_bytes.storage,
                PyBackedBytesStorage::Rust(_)
            ));
            let to_object = (&rust_backed_bytes).into_pyobject(py).unwrap();
            assert!(&to_object.is_exact_instance_of::<PyBytes>());
            assert_eq!(&to_object.extract::<PyBackedBytes>().unwrap(), b"abcde");
        });
    }

    #[test]
    fn test_backed_types_send_sync() {
        assert!(value_of!(IsSend, PyBackedStr));
        assert!(value_of!(IsSync, PyBackedStr));

        assert!(value_of!(IsSend, PyBackedBytes));
        assert!(value_of!(IsSync, PyBackedBytes));
    }

    #[cfg(feature = "py-clone")]
    #[test]
    fn test_backed_str_clone() {
        Python::attach(|py| {
            let s1: PyBackedStr = PyString::new(py, "hello").try_into().unwrap();
            let s2 = s1.clone();
            assert_eq!(s1, s2);

            drop(s1);
            assert_eq!(s2, "hello");
        });
    }

    #[test]
    fn test_backed_str_clone_ref() {
        Python::attach(|py| {
            let s1: PyBackedStr = PyString::new(py, "hello").try_into().unwrap();
            let s2 = s1.clone_ref(py);
            assert_eq!(s1, s2);
            assert!(s1.storage.is(&s2.storage));

            drop(s1);
            assert_eq!(s2, "hello");
        });
    }

    #[test]
    fn test_backed_str_eq() {
        Python::attach(|py| {
            let s1: PyBackedStr = PyString::new(py, "hello").try_into().unwrap();
            let s2: PyBackedStr = PyString::new(py, "hello").try_into().unwrap();
            assert_eq!(s1, "hello");
            assert_eq!(s1, s2);

            let s3: PyBackedStr = PyString::new(py, "abcde").try_into().unwrap();
            assert_eq!("abcde", s3);
            assert_ne!(s1, s3);
        });
    }

    #[test]
    fn test_backed_str_hash() {
        Python::attach(|py| {
            let h = {
                let mut hasher = DefaultHasher::new();
                "abcde".hash(&mut hasher);
                hasher.finish()
            };

            let s1: PyBackedStr = PyString::new(py, "abcde").try_into().unwrap();
            let h1 = {
                let mut hasher = DefaultHasher::new();
                s1.hash(&mut hasher);
                hasher.finish()
            };

            assert_eq!(h, h1);
        });
    }

    #[test]
    fn test_backed_str_ord() {
        Python::attach(|py| {
            let mut a = vec!["a", "c", "d", "b", "f", "g", "e"];
            let mut b = a
                .iter()
                .map(|s| PyString::new(py, s).try_into().unwrap())
                .collect::<Vec<PyBackedStr>>();

            a.sort();
            b.sort();

            assert_eq!(a, b);
        })
    }

    #[test]
    fn test_backed_str_map_key() {
        Python::attach(|py| {
            use std::collections::HashMap;

            let mut map: HashMap<PyBackedStr, usize> = HashMap::new();
            let s: PyBackedStr = PyString::new(py, "key1").try_into().unwrap();

            map.insert(s, 1);

            assert_eq!(map.get("key1"), Some(&1));
        });
    }

    #[test]
    fn test_backed_str_as_str() {
        Python::attach(|py| {
            let s: PyBackedStr = PyString::new(py, "hello").try_into().unwrap();
            assert_eq!(s.as_str(), "hello");
        });
    }

    #[test]
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    fn test_backed_str_as_py_str() {
        Python::attach(|py| {
            let s: PyBackedStr = PyString::new(py, "hello").try_into().unwrap();
            let py_str = s.as_py_str().bind(py);
            assert!(py_str.is(&s.storage));
            assert_eq!(py_str.to_str().unwrap(), "hello");
        });
    }

    #[cfg(feature = "py-clone")]
    #[test]
    fn test_backed_bytes_from_bytes_clone() {
        Python::attach(|py| {
            let b1: PyBackedBytes = PyBytes::new(py, b"abcde").into();
            let b2 = b1.clone();
            assert_eq!(b1, b2);

            drop(b1);
            assert_eq!(b2, b"abcde");
        });
    }

    #[test]
    fn test_backed_bytes_from_bytes_clone_ref() {
        Python::attach(|py| {
            let b1: PyBackedBytes = PyBytes::new(py, b"abcde").into();
            let b2 = b1.clone_ref(py);
            assert_eq!(b1, b2);
            let (PyBackedBytesStorage::Python(s1), PyBackedBytesStorage::Python(s2)) =
                (&b1.storage, &b2.storage)
            else {
                panic!("Expected Python-backed bytes");
            };
            assert!(s1.is(s2));

            drop(b1);
            assert_eq!(b2, b"abcde");
        });
    }

    #[cfg(feature = "py-clone")]
    #[test]
    fn test_backed_bytes_from_bytearray_clone() {
        Python::attach(|py| {
            let b1: PyBackedBytes = PyByteArray::new(py, b"abcde").into();
            let b2 = b1.clone();
            assert_eq!(b1, b2);

            drop(b1);
            assert_eq!(b2, b"abcde");
        });
    }

    #[test]
    fn test_backed_bytes_from_bytearray_clone_ref() {
        Python::attach(|py| {
            let b1: PyBackedBytes = PyByteArray::new(py, b"abcde").into();
            let b2 = b1.clone_ref(py);
            assert_eq!(b1, b2);
            let (PyBackedBytesStorage::Rust(s1), PyBackedBytesStorage::Rust(s2)) =
                (&b1.storage, &b2.storage)
            else {
                panic!("Expected Rust-backed bytes");
            };
            assert!(Arc::ptr_eq(s1, s2));

            drop(b1);
            assert_eq!(b2, b"abcde");
        });
    }

    #[test]
    fn test_backed_bytes_eq() {
        Python::attach(|py| {
            let b1: PyBackedBytes = PyBytes::new(py, b"abcde").into();
            let b2: PyBackedBytes = PyByteArray::new(py, b"abcde").into();

            assert_eq!(b1, b"abcde");
            assert_eq!(b1, b2);

            let b3: PyBackedBytes = PyBytes::new(py, b"hello").into();
            assert_eq!(b"hello", b3);
            assert_ne!(b1, b3);
        });
    }

    #[test]
    fn test_backed_bytes_hash() {
        Python::attach(|py| {
            let h = {
                let mut hasher = DefaultHasher::new();
                b"abcde".hash(&mut hasher);
                hasher.finish()
            };

            let b1: PyBackedBytes = PyBytes::new(py, b"abcde").into();
            let h1 = {
                let mut hasher = DefaultHasher::new();
                b1.hash(&mut hasher);
                hasher.finish()
            };

            let b2: PyBackedBytes = PyByteArray::new(py, b"abcde").into();
            let h2 = {
                let mut hasher = DefaultHasher::new();
                b2.hash(&mut hasher);
                hasher.finish()
            };

            assert_eq!(h, h1);
            assert_eq!(h, h2);
        });
    }

    #[test]
    fn test_backed_bytes_ord() {
        Python::attach(|py| {
            let mut a = vec![b"a", b"c", b"d", b"b", b"f", b"g", b"e"];
            let mut b = a
                .iter()
                .map(|&b| PyBytes::new(py, b).into())
                .collect::<Vec<PyBackedBytes>>();

            a.sort();
            b.sort();

            assert_eq!(a, b);
        })
    }
}
