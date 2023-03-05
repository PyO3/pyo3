// Copyright (c) 2017-present PyO3 Project and Contributors

#[cfg(not(Py_LIMITED_API))]
use crate::exceptions::PyUnicodeDecodeError;
use crate::types::PyBytes;
use crate::{ffi, AsPyPointer, PyAny, PyResult, Python};
use std::borrow::Cow;
use std::os::raw::c_char;
use std::str;

/// Represents raw data backing a Python `str`.
///
/// Python internally stores strings in various representations. This enumeration
/// represents those variations.
#[cfg(not(Py_LIMITED_API))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PyStringData<'a> {
    /// UCS1 representation.
    Ucs1(&'a [u8]),

    /// UCS2 representation.
    Ucs2(&'a [u16]),

    /// UCS4 representation.
    Ucs4(&'a [u32]),
}

#[cfg(not(Py_LIMITED_API))]
impl<'a> PyStringData<'a> {
    /// Obtain the raw bytes backing this instance as a [u8] slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ucs1(s) => s,
            Self::Ucs2(s) => unsafe {
                std::slice::from_raw_parts(
                    s.as_ptr() as *const u8,
                    s.len() * self.value_width_bytes(),
                )
            },
            Self::Ucs4(s) => unsafe {
                std::slice::from_raw_parts(
                    s.as_ptr() as *const u8,
                    s.len() * self.value_width_bytes(),
                )
            },
        }
    }

    /// Size in bytes of each value/item in the underlying slice.
    #[inline]
    pub fn value_width_bytes(&self) -> usize {
        match self {
            Self::Ucs1(_) => 1,
            Self::Ucs2(_) => 2,
            Self::Ucs4(_) => 4,
        }
    }

    /// Convert the raw data to a Rust string.
    ///
    /// For UCS-1 / UTF-8, returns a borrow into the original slice. For UCS-2 and UCS-4,
    /// returns an owned string.
    ///
    /// Returns [PyUnicodeDecodeError] if the string data isn't valid in its purported
    /// storage format. This should only occur for strings that were created via Python
    /// C APIs that skip input validation (like `PyUnicode_FromKindAndData`) and should
    /// never occur for strings that were created from Python code.
    pub fn to_string(self, py: Python<'_>) -> PyResult<Cow<'a, str>> {
        use std::ffi::CStr;
        match self {
            Self::Ucs1(data) => match str::from_utf8(data) {
                Ok(s) => Ok(Cow::Borrowed(s)),
                Err(e) => Err(crate::PyErr::from_value(PyUnicodeDecodeError::new_utf8(
                    py, data, e,
                )?)),
            },
            Self::Ucs2(data) => match String::from_utf16(data) {
                Ok(s) => Ok(Cow::Owned(s)),
                Err(e) => {
                    let mut message = e.to_string().as_bytes().to_vec();
                    message.push(0);

                    Err(crate::PyErr::from_value(PyUnicodeDecodeError::new(
                        py,
                        CStr::from_bytes_with_nul(b"utf-16\0").unwrap(),
                        self.as_bytes(),
                        0..self.as_bytes().len(),
                        CStr::from_bytes_with_nul(&message).unwrap(),
                    )?))
                }
            },
            Self::Ucs4(data) => match data.iter().map(|&c| std::char::from_u32(c)).collect() {
                Some(s) => Ok(Cow::Owned(s)),
                None => Err(crate::PyErr::from_value(PyUnicodeDecodeError::new(
                    py,
                    CStr::from_bytes_with_nul(b"utf-32\0").unwrap(),
                    self.as_bytes(),
                    0..self.as_bytes().len(),
                    CStr::from_bytes_with_nul(b"error converting utf-32\0").unwrap(),
                )?)),
            },
        }
    }

    /// Convert the raw data to a Rust string, possibly with data loss.
    ///
    /// Invalid code points will be replaced with `U+FFFD REPLACEMENT CHARACTER`.
    ///
    /// Returns a borrow into original data, when possible, or owned data otherwise.
    ///
    /// The return value of this function should only disagree with [Self::to_string]
    /// when that method would error.
    pub fn to_string_lossy(self) -> Cow<'a, str> {
        match self {
            Self::Ucs1(data) => String::from_utf8_lossy(data),
            Self::Ucs2(data) => Cow::Owned(String::from_utf16_lossy(data)),
            Self::Ucs4(data) => Cow::Owned(
                data.iter()
                    .map(|&c| std::char::from_u32(c).unwrap_or('\u{FFFD}'))
                    .collect(),
            ),
        }
    }
}

/// Represents a Python `string` (a Unicode string object).
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyString(PyAny);

pyobject_native_type_core!(PyString, ffi::PyUnicode_Type, #checkfunction=ffi::PyUnicode_Check);

impl PyString {
    /// Creates a new Python string object.
    ///
    /// Panics if out of memory.
    pub fn new<'p>(py: Python<'p>, s: &str) -> &'p PyString {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe { py.from_owned_ptr(ffi::PyUnicode_FromStringAndSize(ptr, len)) }
    }

    /// Intern the given string
    ///
    /// This will return a reference to the same Python string object if called repeatedly with the same string.
    ///
    /// Note that while this is more memory efficient than [`PyString::new`], it unconditionally allocates a
    /// temporary Python string object and is thereby slower than [`PyString::new`].
    ///
    /// Panics if out of memory.
    pub fn intern<'p>(py: Python<'p>, s: &str) -> &'p PyString {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            let mut ob = ffi::PyUnicode_FromStringAndSize(ptr, len);
            if !ob.is_null() {
                ffi::PyUnicode_InternInPlace(&mut ob);
            }
            py.from_owned_ptr(ob)
        }
    }

    /// Attempts to create a Python string from a Python [bytes-like object].
    ///
    /// [bytes-like object]: (https://docs.python.org/3/glossary.html#term-bytes-like-object).
    pub fn from_object<'p>(src: &'p PyAny, encoding: &str, errors: &str) -> PyResult<&'p PyString> {
        unsafe {
            src.py()
                .from_owned_ptr_or_err::<PyString>(ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(),
                    encoding.as_ptr() as *const c_char,
                    errors.as_ptr() as *const c_char,
                ))
        }
    }

    /// Gets the Python string as a byte slice.
    ///
    /// Returns a `UnicodeEncodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    #[inline]
    pub fn to_str(&self) -> PyResult<&str> {
        let utf8_slice = {
            cfg_if::cfg_if! {
                if #[cfg(any(Py_3_10, not(Py_LIMITED_API)))] {
                    // PyUnicode_AsUTF8AndSize only available on limited API starting with 3.10.
                    let mut size: ffi::Py_ssize_t = 0;
                    let data = unsafe { ffi::PyUnicode_AsUTF8AndSize(self.as_ptr(), &mut size) };
                    if data.is_null() {
                        return Err(crate::PyErr::fetch(self.py()));
                    } else {
                        unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) }
                    }
                } else {
                    let bytes = unsafe {
                        self.py().from_owned_ptr_or_err::<PyBytes>(ffi::PyUnicode_AsUTF8String(self.as_ptr()))?
                    };
                    bytes.as_bytes()
                }
            }
        };
        Ok(unsafe { std::str::from_utf8_unchecked(utf8_slice) })
    }

    /// Converts the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with `U+FFFD REPLACEMENT CHARACTER`.
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        match self.to_str() {
            Ok(s) => Cow::Borrowed(s),
            Err(_) => {
                let bytes = unsafe {
                    self.py()
                        .from_owned_ptr::<PyBytes>(ffi::PyUnicode_AsEncodedString(
                            self.as_ptr(),
                            b"utf-8\0" as *const _ as _,
                            b"surrogatepass\0" as *const _ as _,
                        ))
                };
                String::from_utf8_lossy(bytes.as_bytes())
            }
        }
    }

    /// Obtains the raw data backing the Python string.
    ///
    /// If the Python string object was created through legacy APIs, its internal storage format
    /// will be canonicalized before data is returned.
    ///
    /// # Safety
    ///
    /// This function implementation relies on manually decoding a C bitfield. In practice, this
    /// works well on common little-endian architectures such as x86_64, where the bitfield has a
    /// common representation (even if it is not part of the C spec). The PyO3 CI tests this API on
    /// x86_64 platforms.
    ///
    /// By using this API, you accept responsibility for testing that PyStringData behaves as
    /// expected on the targets where you plan to distribute your software.
    #[cfg(not(Py_LIMITED_API))]
    pub unsafe fn data(&self) -> PyResult<PyStringData<'_>> {
        let ptr = self.as_ptr();

        #[cfg(not(Py_3_12))]
        #[allow(deprecated)]
        {
            let ready = ffi::PyUnicode_READY(ptr);
            if ready != 0 {
                // Exception was created on failure.
                return Err(crate::PyErr::fetch(self.py()));
            }
        }

        // The string should be in its canonical form after calling `PyUnicode_READY()`.
        // And non-canonical form not possible after Python 3.12. So it should be safe
        // to call these APIs.
        let length = ffi::PyUnicode_GET_LENGTH(ptr) as usize;
        let raw_data = ffi::PyUnicode_DATA(ptr);
        let kind = ffi::PyUnicode_KIND(ptr);

        match kind {
            ffi::PyUnicode_1BYTE_KIND => Ok(PyStringData::Ucs1(std::slice::from_raw_parts(
                raw_data as *const u8,
                length,
            ))),
            ffi::PyUnicode_2BYTE_KIND => Ok(PyStringData::Ucs2(std::slice::from_raw_parts(
                raw_data as *const u16,
                length,
            ))),
            ffi::PyUnicode_4BYTE_KIND => Ok(PyStringData::Ucs4(std::slice::from_raw_parts(
                raw_data as *const u32,
                length,
            ))),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Python;
    use crate::{PyObject, ToPyObject};
    #[cfg(not(Py_LIMITED_API))]
    use std::borrow::Cow;

    #[test]
    fn test_to_str_ascii() {
        Python::with_gil(|py| {
            let s = "ascii 🐈";
            let obj: PyObject = PyString::new(py, s).into();
            let py_string: &PyString = obj.downcast(py).unwrap();
            assert_eq!(s, py_string.to_str().unwrap());
        })
    }

    #[test]
    fn test_to_str_surrogate() {
        Python::with_gil(|py| {
            let obj: PyObject = py.eval(r#"'\ud800'"#, None, None).unwrap().into();
            let py_string: &PyString = obj.downcast(py).unwrap();
            assert!(py_string.to_str().is_err());
        })
    }

    #[test]
    fn test_to_str_unicode() {
        Python::with_gil(|py| {
            let s = "哈哈🐈";
            let obj: PyObject = PyString::new(py, s).into();
            let py_string: &PyString = obj.downcast(py).unwrap();
            assert_eq!(s, py_string.to_str().unwrap());
        })
    }

    #[test]
    fn test_to_string_lossy() {
        Python::with_gil(|py| {
            let obj: PyObject = py
                .eval(r#"'🐈 Hello \ud800World'"#, None, None)
                .unwrap()
                .into();
            let py_string: &PyString = obj.downcast(py).unwrap();
            assert_eq!(py_string.to_string_lossy(), "🐈 Hello ���World");
        })
    }

    #[test]
    fn test_debug_string() {
        Python::with_gil(|py| {
            let v = "Hello\n".to_object(py);
            let s: &PyString = v.downcast(py).unwrap();
            assert_eq!(format!("{:?}", s), "'Hello\\n'");
        })
    }

    #[test]
    fn test_display_string() {
        Python::with_gil(|py| {
            let v = "Hello\n".to_object(py);
            let s: &PyString = v.downcast(py).unwrap();
            assert_eq!(format!("{}", s), "Hello\n");
        })
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_string_data_ucs1() {
        Python::with_gil(|py| {
            let s = PyString::new(py, "hello, world");
            let data = unsafe { s.data().unwrap() };

            assert_eq!(data, PyStringData::Ucs1(b"hello, world"));
            assert_eq!(data.to_string(py).unwrap(), Cow::Borrowed("hello, world"));
            assert_eq!(data.to_string_lossy(), Cow::Borrowed("hello, world"));
        })
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_string_data_ucs1_invalid() {
        Python::with_gil(|py| {
            // 0xfe is not allowed in UTF-8.
            let buffer = b"f\xfe\0";
            let ptr = unsafe {
                crate::ffi::PyUnicode_FromKindAndData(
                    crate::ffi::PyUnicode_1BYTE_KIND as _,
                    buffer.as_ptr() as *const _,
                    2,
                )
            };
            assert!(!ptr.is_null());
            let s: &PyString = unsafe { py.from_owned_ptr(ptr) };
            let data = unsafe { s.data().unwrap() };
            assert_eq!(data, PyStringData::Ucs1(b"f\xfe"));
            let err = data.to_string(py).unwrap_err();
            assert!(err.get_type(py).is(py.get_type::<PyUnicodeDecodeError>()));
            assert!(err
                .to_string()
                .contains("'utf-8' codec can't decode byte 0xfe in position 1"));
            assert_eq!(data.to_string_lossy(), Cow::Borrowed("f�"));
        });
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_string_data_ucs2() {
        Python::with_gil(|py| {
            let s = py.eval("'foo\\ud800'", None, None).unwrap();
            let py_string = s.downcast::<PyString>().unwrap();
            let data = unsafe { py_string.data().unwrap() };

            assert_eq!(data, PyStringData::Ucs2(&[102, 111, 111, 0xd800]));
            assert_eq!(
                data.to_string_lossy(),
                Cow::Owned::<str>("foo�".to_string())
            );
        })
    }

    #[test]
    #[cfg(all(not(Py_LIMITED_API), target_endian = "little"))]
    fn test_string_data_ucs2_invalid() {
        Python::with_gil(|py| {
            // U+FF22 (valid) & U+d800 (never valid)
            let buffer = b"\x22\xff\x00\xd8\x00\x00";
            let ptr = unsafe {
                crate::ffi::PyUnicode_FromKindAndData(
                    crate::ffi::PyUnicode_2BYTE_KIND as _,
                    buffer.as_ptr() as *const _,
                    2,
                )
            };
            assert!(!ptr.is_null());
            let s: &PyString = unsafe { py.from_owned_ptr(ptr) };
            let data = unsafe { s.data().unwrap() };
            assert_eq!(data, PyStringData::Ucs2(&[0xff22, 0xd800]));
            let err = data.to_string(py).unwrap_err();
            assert!(err.get_type(py).is(py.get_type::<PyUnicodeDecodeError>()));
            assert!(err
                .to_string()
                .contains("'utf-16' codec can't decode bytes in position 0-3"));
            assert_eq!(data.to_string_lossy(), Cow::Owned::<str>("Ｂ�".into()));
        });
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_string_data_ucs4() {
        Python::with_gil(|py| {
            let s = "哈哈🐈";
            let py_string = PyString::new(py, s);
            let data = unsafe { py_string.data().unwrap() };

            assert_eq!(data, PyStringData::Ucs4(&[21704, 21704, 128008]));
            assert_eq!(data.to_string_lossy(), Cow::Owned::<str>(s.to_string()));
        })
    }

    #[test]
    #[cfg(all(not(Py_LIMITED_API), target_endian = "little"))]
    fn test_string_data_ucs4_invalid() {
        Python::with_gil(|py| {
            // U+20000 (valid) & U+d800 (never valid)
            let buffer = b"\x00\x00\x02\x00\x00\xd8\x00\x00\x00\x00\x00\x00";
            let ptr = unsafe {
                crate::ffi::PyUnicode_FromKindAndData(
                    crate::ffi::PyUnicode_4BYTE_KIND as _,
                    buffer.as_ptr() as *const _,
                    2,
                )
            };
            assert!(!ptr.is_null());
            let s: &PyString = unsafe { py.from_owned_ptr(ptr) };
            let data = unsafe { s.data().unwrap() };
            assert_eq!(data, PyStringData::Ucs4(&[0x20000, 0xd800]));
            let err = data.to_string(py).unwrap_err();
            assert!(err.get_type(py).is(py.get_type::<PyUnicodeDecodeError>()));
            assert!(err
                .to_string()
                .contains("'utf-32' codec can't decode bytes in position 0-7"));
            assert_eq!(data.to_string_lossy(), Cow::Owned::<str>("𠀀�".into()));
        });
    }

    #[test]
    fn test_intern_string() {
        Python::with_gil(|py| {
            let py_string1 = PyString::intern(py, "foo");
            assert_eq!(py_string1.to_str().unwrap(), "foo");

            let py_string2 = PyString::intern(py, "foo");
            assert_eq!(py_string2.to_str().unwrap(), "foo");

            assert_eq!(py_string1.as_ptr(), py_string2.as_ptr());

            let py_string3 = PyString::intern(py, "bar");
            assert_eq!(py_string3.to_str().unwrap(), "bar");

            assert_ne!(py_string1.as_ptr(), py_string3.as_ptr());
        });
    }
}
