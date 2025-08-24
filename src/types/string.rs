#[cfg(not(Py_LIMITED_API))]
use crate::exceptions::PyUnicodeDecodeError;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Borrowed;
use crate::py_result_ext::PyResultExt;
use crate::types::bytes::PyBytesMethods;
use crate::types::PyBytes;
use crate::{ffi, Bound, Py, PyAny, PyResult, Python};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
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
                std::slice::from_raw_parts(s.as_ptr().cast(), s.len() * self.value_width_bytes())
            },
            Self::Ucs4(s) => unsafe {
                std::slice::from_raw_parts(s.as_ptr().cast(), s.len() * self.value_width_bytes())
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
        match self {
            Self::Ucs1(data) => match str::from_utf8(data) {
                Ok(s) => Ok(Cow::Borrowed(s)),
                Err(e) => Err(PyUnicodeDecodeError::new_utf8(py, data, e)?.into()),
            },
            Self::Ucs2(data) => match String::from_utf16(data) {
                Ok(s) => Ok(Cow::Owned(s)),
                Err(e) => {
                    let mut message = e.to_string().as_bytes().to_vec();
                    message.push(0);

                    Err(PyUnicodeDecodeError::new(
                        py,
                        ffi::c_str!("utf-16"),
                        self.as_bytes(),
                        0..self.as_bytes().len(),
                        CStr::from_bytes_with_nul(&message).unwrap(),
                    )?
                    .into())
                }
            },
            Self::Ucs4(data) => match data.iter().map(|&c| std::char::from_u32(c)).collect() {
                Some(s) => Ok(Cow::Owned(s)),
                None => Err(PyUnicodeDecodeError::new(
                    py,
                    ffi::c_str!("utf-32"),
                    self.as_bytes(),
                    0..self.as_bytes().len(),
                    ffi::c_str!("error converting utf-32"),
                )?
                .into()),
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
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyString>`][crate::Py] or [`Bound<'py, PyString>`][Bound].
///
/// For APIs available on `str` objects, see the [`PyStringMethods`] trait which is implemented for
/// [`Bound<'py, PyString>`][Bound].
///
/// # Equality
///
/// For convenience, [`Bound<'py, PyString>`] implements [`PartialEq<str>`] to allow comparing the
/// data in the Python string to a Rust UTF-8 string slice.
///
/// This is not always the most appropriate way to compare Python strings, as Python string
/// subclasses may have different equality semantics. In situations where subclasses overriding
/// equality might be relevant, use [`PyAnyMethods::eq`](crate::types::any::PyAnyMethods::eq), at
/// cost of the additional overhead of a Python method call.
///
/// ```rust
/// # use pyo3::prelude::*;
/// use pyo3::types::PyString;
///
/// # Python::attach(|py| {
/// let py_string = PyString::new(py, "foo");
/// // via PartialEq<str>
/// assert_eq!(py_string, "foo");
///
/// // via Python equality
/// assert!(py_string.as_any().eq("foo").unwrap());
/// # });
/// ```
#[repr(transparent)]
pub struct PyString(PyAny);

pyobject_native_type_core!(PyString, pyobject_native_static_type_object!(ffi::PyUnicode_Type), #checkfunction=ffi::PyUnicode_Check);

impl PyString {
    /// Creates a new Python string object.
    ///
    /// Panics if out of memory.
    pub fn new<'py>(py: Python<'py>, s: &str) -> Bound<'py, PyString> {
        let ptr = s.as_ptr().cast();
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            ffi::PyUnicode_FromStringAndSize(ptr, len)
                .assume_owned(py)
                .cast_into_unchecked()
        }
    }

    /// Intern the given string
    ///
    /// This will return a reference to the same Python string object if called repeatedly with the same string.
    ///
    /// Note that while this is more memory efficient than [`PyString::new`], it unconditionally allocates a
    /// temporary Python string object and is thereby slower than [`PyString::new`].
    ///
    /// Panics if out of memory.
    pub fn intern<'py>(py: Python<'py>, s: &str) -> Bound<'py, PyString> {
        let ptr = s.as_ptr().cast();
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            let mut ob = ffi::PyUnicode_FromStringAndSize(ptr, len);
            if !ob.is_null() {
                ffi::PyUnicode_InternInPlace(&mut ob);
            }
            ob.assume_owned(py).cast_into_unchecked()
        }
    }

    /// Attempts to create a Python string from a Python [bytes-like object].
    ///
    /// The `encoding` and `errors` parameters are optional:
    /// - If `encoding` is `None`, the default encoding is used (UTF-8).
    /// - If `errors` is `None`, the default error handling is used ("strict").
    ///
    /// See the [Python documentation on codecs] for more information.
    ///
    /// [bytes-like object]: (https://docs.python.org/3/glossary.html#term-bytes-like-object).
    /// [Python documentation on codecs]: https://docs.python.org/3/library/codecs.html#standard-encodings
    pub fn from_encoded_object<'py>(
        src: &Bound<'py, PyAny>,
        encoding: Option<&CStr>,
        errors: Option<&CStr>,
    ) -> PyResult<Bound<'py, PyString>> {
        let encoding = encoding.map_or(std::ptr::null(), CStr::as_ptr);
        let errors = errors.map_or(std::ptr::null(), CStr::as_ptr);
        // Safety:
        // - `src` is a valid Python object
        // - `encoding` and `errors` are either null or valid C strings. `encoding` and `errors` are
        //   documented as allowing null.
        // - `ffi::PyUnicode_FromEncodedObject` returns a new `str` object, or sets an error.
        unsafe {
            ffi::PyUnicode_FromEncodedObject(src.as_ptr(), encoding, errors)
                .assume_owned_or_err(src.py())
                .cast_into_unchecked()
        }
    }

    /// Deprecated form of `PyString::from_encoded_object`.
    ///
    /// This version took `&str` arguments for `encoding` and `errors`, which required a runtime
    /// conversion to `CString` internally.
    #[deprecated(
        since = "0.25.0",
        note = "replaced with to `PyString::from_encoded_object`"
    )]
    pub fn from_object<'py>(
        src: &Bound<'py, PyAny>,
        encoding: &str,
        errors: &str,
    ) -> PyResult<Bound<'py, PyString>> {
        let encoding = CString::new(encoding)?;
        let errors = CString::new(errors)?;
        PyString::from_encoded_object(src, Some(&encoding), Some(&errors))
    }
}

/// Implementation of functionality for [`PyString`].
///
/// These methods are defined for the `Bound<'py, PyString>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyString")]
pub trait PyStringMethods<'py>: crate::sealed::Sealed {
    /// Gets the Python string as a Rust UTF-8 string slice.
    ///
    /// Returns a `UnicodeEncodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    fn to_str(&self) -> PyResult<&str>;

    /// Converts the `PyString` into a Rust string, avoiding copying when possible.
    ///
    /// Returns a `UnicodeEncodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    fn to_cow(&self) -> PyResult<Cow<'_, str>>;

    /// Converts the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with `U+FFFD REPLACEMENT CHARACTER`.
    fn to_string_lossy(&self) -> Cow<'_, str>;

    /// Encodes this string as a Python `bytes` object, using UTF-8 encoding.
    fn encode_utf8(&self) -> PyResult<Bound<'py, PyBytes>>;

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
    #[cfg(not(any(Py_LIMITED_API, GraalPy, PyPy)))]
    unsafe fn data(&self) -> PyResult<PyStringData<'_>>;
}

impl<'py> PyStringMethods<'py> for Bound<'py, PyString> {
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    fn to_str(&self) -> PyResult<&str> {
        self.as_borrowed().to_str()
    }

    fn to_cow(&self) -> PyResult<Cow<'_, str>> {
        self.as_borrowed().to_cow()
    }

    fn to_string_lossy(&self) -> Cow<'_, str> {
        self.as_borrowed().to_string_lossy()
    }

    fn encode_utf8(&self) -> PyResult<Bound<'py, PyBytes>> {
        unsafe {
            ffi::PyUnicode_AsUTF8String(self.as_ptr())
                .assume_owned_or_err(self.py())
                .cast_into_unchecked::<PyBytes>()
        }
    }

    #[cfg(not(any(Py_LIMITED_API, GraalPy, PyPy)))]
    unsafe fn data(&self) -> PyResult<PyStringData<'_>> {
        unsafe { self.as_borrowed().data() }
    }
}

impl<'a> Borrowed<'a, '_, PyString> {
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_str(self) -> PyResult<&'a str> {
        // PyUnicode_AsUTF8AndSize only available on limited API starting with 3.10.
        let mut size: ffi::Py_ssize_t = 0;
        let data: *const u8 =
            unsafe { ffi::PyUnicode_AsUTF8AndSize(self.as_ptr(), &mut size).cast() };
        if data.is_null() {
            Err(crate::PyErr::fetch(self.py()))
        } else {
            Ok(unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(data, size as usize))
            })
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_cow(self) -> PyResult<Cow<'a, str>> {
        // TODO: this method can probably be deprecated once Python 3.9 support is dropped,
        // because all versions then support the more efficient `to_str`.
        #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
        {
            self.to_str().map(Cow::Borrowed)
        }

        #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
        {
            let bytes = self.encode_utf8()?;
            Ok(Cow::Owned(
                unsafe { str::from_utf8_unchecked(bytes.as_bytes()) }.to_owned(),
            ))
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_string_lossy(self) -> Cow<'a, str> {
        let ptr = self.as_ptr();
        let py = self.py();

        #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
        if let Ok(s) = self.to_str() {
            return Cow::Borrowed(s);
        }

        let bytes = unsafe {
            ffi::PyUnicode_AsEncodedString(
                ptr,
                ffi::c_str!("utf-8").as_ptr(),
                ffi::c_str!("surrogatepass").as_ptr(),
            )
            .assume_owned(py)
            .cast_into_unchecked::<PyBytes>()
        };
        Cow::Owned(String::from_utf8_lossy(bytes.as_bytes()).into_owned())
    }

    #[cfg(not(any(Py_LIMITED_API, GraalPy, PyPy)))]
    unsafe fn data(self) -> PyResult<PyStringData<'a>> {
        unsafe {
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
}

impl Py<PyString> {
    /// Gets the Python string as a Rust UTF-8 string slice.
    ///
    /// Returns a `UnicodeEncodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    ///
    /// Because `str` objects are immutable, the returned slice is independent of
    /// the GIL lifetime.
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    pub fn to_str<'a>(&'a self, py: Python<'_>) -> PyResult<&'a str> {
        self.bind_borrowed(py).to_str()
    }

    /// Converts the `PyString` into a Rust string, avoiding copying when possible.
    ///
    /// Returns a `UnicodeEncodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    ///
    /// Because `str` objects are immutable, the returned slice is independent of
    /// the GIL lifetime.
    pub fn to_cow<'a>(&'a self, py: Python<'_>) -> PyResult<Cow<'a, str>> {
        self.bind_borrowed(py).to_cow()
    }

    /// Converts the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with `U+FFFD REPLACEMENT CHARACTER`.
    ///
    /// Because `str` objects are immutable, the returned slice is independent of
    /// the GIL lifetime.
    pub fn to_string_lossy<'a>(&'a self, py: Python<'_>) -> Cow<'a, str> {
        self.bind_borrowed(py).to_string_lossy()
    }
}

/// Compares whether the data in the Python string is equal to the given UTF8.
///
/// In some cases Python equality might be more appropriate; see the note on [`PyString`].
impl PartialEq<str> for Bound<'_, PyString> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_borrowed() == *other
    }
}

/// Compares whether the data in the Python string is equal to the given UTF8.
///
/// In some cases Python equality might be more appropriate; see the note on [`PyString`].
impl PartialEq<&'_ str> for Bound<'_, PyString> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_borrowed() == **other
    }
}

/// Compares whether the data in the Python string is equal to the given UTF8.
///
/// In some cases Python equality might be more appropriate; see the note on [`PyString`].
impl PartialEq<Bound<'_, PyString>> for str {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyString>) -> bool {
        *self == other.as_borrowed()
    }
}

/// Compares whether the data in the Python string is equal to the given UTF8.
///
/// In some cases Python equality might be more appropriate; see the note on [`PyString`].
impl PartialEq<&'_ Bound<'_, PyString>> for str {
    #[inline]
    fn eq(&self, other: &&Bound<'_, PyString>) -> bool {
        *self == other.as_borrowed()
    }
}

/// Compares whether the data in the Python string is equal to the given UTF8.
///
/// In some cases Python equality might be more appropriate; see the note on [`PyString`].
impl PartialEq<Bound<'_, PyString>> for &'_ str {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyString>) -> bool {
        **self == other.as_borrowed()
    }
}

/// Compares whether the data in the Python string is equal to the given UTF8.
///
/// In some cases Python equality might be more appropriate; see the note on [`PyString`].
impl PartialEq<str> for &'_ Bound<'_, PyString> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_borrowed() == other
    }
}

/// Compares whether the data in the Python string is equal to the given UTF8.
///
/// In some cases Python equality might be more appropriate; see the note on [`PyString`].
impl PartialEq<str> for Borrowed<'_, '_, PyString> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        #[cfg(not(Py_3_13))]
        {
            self.to_cow().is_ok_and(|s| s == other)
        }

        #[cfg(Py_3_13)]
        unsafe {
            ffi::PyUnicode_EqualToUTF8AndSize(
                self.as_ptr(),
                other.as_ptr().cast(),
                other.len() as _,
            ) == 1
        }
    }
}

/// Compares whether the data in the Python string is equal to the given UTF8.
///
/// In some cases Python equality might be more appropriate; see the note on [`PyString`].
impl PartialEq<&str> for Borrowed<'_, '_, PyString> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        *self == **other
    }
}

/// Compares whether the data in the Python string is equal to the given UTF8.
///
/// In some cases Python equality might be more appropriate; see the note on [`PyString`].
impl PartialEq<Borrowed<'_, '_, PyString>> for str {
    #[inline]
    fn eq(&self, other: &Borrowed<'_, '_, PyString>) -> bool {
        other == self
    }
}

/// Compares whether the data in the Python string is equal to the given UTF8.
///
/// In some cases Python equality might be more appropriate; see the note on [`PyString`].
impl PartialEq<Borrowed<'_, '_, PyString>> for &'_ str {
    #[inline]
    fn eq(&self, other: &Borrowed<'_, '_, PyString>) -> bool {
        other == self
    }
}

#[cfg(test)]
mod tests {
    use pyo3_ffi::c_str;

    use super::*;
    use crate::{exceptions::PyLookupError, types::PyAnyMethods as _, IntoPyObject};

    #[test]
    fn test_to_cow_utf8() {
        Python::attach(|py| {
            let s = "ascii 🐈";
            let py_string = PyString::new(py, s);
            assert_eq!(s, py_string.to_cow().unwrap());
        })
    }

    #[test]
    fn test_to_cow_surrogate() {
        Python::attach(|py| {
            let py_string = py
                .eval(ffi::c_str!(r"'\ud800'"), None, None)
                .unwrap()
                .cast_into::<PyString>()
                .unwrap();
            assert!(py_string.to_cow().is_err());
        })
    }

    #[test]
    fn test_to_cow_unicode() {
        Python::attach(|py| {
            let s = "哈哈🐈";
            let py_string = PyString::new(py, s);
            assert_eq!(s, py_string.to_cow().unwrap());
        })
    }

    #[test]
    fn test_encode_utf8_unicode() {
        Python::attach(|py| {
            let s = "哈哈🐈";
            let obj = PyString::new(py, s);
            assert_eq!(s.as_bytes(), obj.encode_utf8().unwrap().as_bytes());
        })
    }

    #[test]
    fn test_encode_utf8_surrogate() {
        Python::attach(|py| {
            let obj: Py<PyAny> = py
                .eval(ffi::c_str!(r"'\ud800'"), None, None)
                .unwrap()
                .into();
            assert!(obj
                .bind(py)
                .cast::<PyString>()
                .unwrap()
                .encode_utf8()
                .is_err());
        })
    }

    #[test]
    fn test_to_string_lossy() {
        Python::attach(|py| {
            let py_string = py
                .eval(ffi::c_str!(r"'🐈 Hello \ud800World'"), None, None)
                .unwrap()
                .cast_into::<PyString>()
                .unwrap();

            assert_eq!(py_string.to_string_lossy(), "🐈 Hello ���World");
        })
    }

    #[test]
    fn test_debug_string() {
        Python::attach(|py| {
            let s = "Hello\n".into_pyobject(py).unwrap();
            assert_eq!(format!("{s:?}"), "'Hello\\n'");
        })
    }

    #[test]
    fn test_display_string() {
        Python::attach(|py| {
            let s = "Hello\n".into_pyobject(py).unwrap();
            assert_eq!(format!("{s}"), "Hello\n");
        })
    }

    #[test]
    fn test_string_from_encoded_object() {
        Python::attach(|py| {
            let py_bytes = PyBytes::new(py, b"ab\xFFcd");

            // default encoding is utf-8, default error handler is strict
            let py_string = PyString::from_encoded_object(&py_bytes, None, None).unwrap_err();
            assert!(py_string
                .get_type(py)
                .is(py.get_type::<crate::exceptions::PyUnicodeDecodeError>()));

            // with `ignore` error handler, the invalid byte is dropped
            let py_string =
                PyString::from_encoded_object(&py_bytes, None, Some(c_str!("ignore"))).unwrap();

            let result = py_string.to_cow().unwrap();
            assert_eq!(result, "abcd");

            #[allow(deprecated)]
            let py_string = PyString::from_object(&py_bytes, "utf-8", "ignore").unwrap();

            let result = py_string.to_cow().unwrap();
            assert_eq!(result, "abcd");
        });
    }

    #[test]
    fn test_string_from_encoded_object_with_invalid_encoding_errors() {
        Python::attach(|py| {
            let py_bytes = PyBytes::new(py, b"abcd");

            // invalid encoding
            let err =
                PyString::from_encoded_object(&py_bytes, Some(c_str!("wat")), None).unwrap_err();
            assert!(err.is_instance(py, &py.get_type::<PyLookupError>()));
            assert_eq!(err.to_string(), "LookupError: unknown encoding: wat");

            // invalid error handler
            let err = PyString::from_encoded_object(
                &PyBytes::new(py, b"ab\xFFcd"),
                None,
                Some(c_str!("wat")),
            )
            .unwrap_err();
            assert!(err.is_instance(py, &py.get_type::<PyLookupError>()));
            assert_eq!(
                err.to_string(),
                "LookupError: unknown error handler name 'wat'"
            );

            #[allow(deprecated)]
            let result = PyString::from_object(&py_bytes, "utf\0-8", "ignore");
            assert!(result.is_err());

            #[allow(deprecated)]
            let result = PyString::from_object(&py_bytes, "utf-8", "ign\0ore");
            assert!(result.is_err());
        });
    }

    #[test]
    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    fn test_string_data_ucs1() {
        Python::attach(|py| {
            let s = PyString::new(py, "hello, world");
            let data = unsafe { s.data().unwrap() };

            assert_eq!(data, PyStringData::Ucs1(b"hello, world"));
            assert_eq!(data.to_string(py).unwrap(), Cow::Borrowed("hello, world"));
            assert_eq!(data.to_string_lossy(), Cow::Borrowed("hello, world"));
        })
    }

    #[test]
    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    fn test_string_data_ucs1_invalid() {
        Python::attach(|py| {
            // 0xfe is not allowed in UTF-8.
            let buffer = b"f\xfe\0";
            let ptr = unsafe {
                crate::ffi::PyUnicode_FromKindAndData(
                    crate::ffi::PyUnicode_1BYTE_KIND as _,
                    buffer.as_ptr().cast(),
                    2,
                )
            };
            assert!(!ptr.is_null());
            let s = unsafe { ptr.assume_owned(py).cast_into_unchecked::<PyString>() };
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
    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    fn test_string_data_ucs2() {
        Python::attach(|py| {
            let s = py.eval(ffi::c_str!("'foo\\ud800'"), None, None).unwrap();
            let py_string = s.cast::<PyString>().unwrap();
            let data = unsafe { py_string.data().unwrap() };

            assert_eq!(data, PyStringData::Ucs2(&[102, 111, 111, 0xd800]));
            assert_eq!(
                data.to_string_lossy(),
                Cow::Owned::<str>("foo�".to_string())
            );
        })
    }

    #[test]
    #[cfg(all(not(any(Py_LIMITED_API, PyPy, GraalPy)), target_endian = "little"))]
    fn test_string_data_ucs2_invalid() {
        Python::attach(|py| {
            // U+FF22 (valid) & U+d800 (never valid)
            let buffer = b"\x22\xff\x00\xd8\x00\x00";
            let ptr = unsafe {
                crate::ffi::PyUnicode_FromKindAndData(
                    crate::ffi::PyUnicode_2BYTE_KIND as _,
                    buffer.as_ptr().cast(),
                    2,
                )
            };
            assert!(!ptr.is_null());
            let s = unsafe { ptr.assume_owned(py).cast_into_unchecked::<PyString>() };
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
    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    fn test_string_data_ucs4() {
        Python::attach(|py| {
            let s = "哈哈🐈";
            let py_string = PyString::new(py, s);
            let data = unsafe { py_string.data().unwrap() };

            assert_eq!(data, PyStringData::Ucs4(&[21704, 21704, 128008]));
            assert_eq!(data.to_string_lossy(), Cow::Owned::<str>(s.to_string()));
        })
    }

    #[test]
    #[cfg(all(not(any(Py_LIMITED_API, PyPy, GraalPy)), target_endian = "little"))]
    fn test_string_data_ucs4_invalid() {
        Python::attach(|py| {
            // U+20000 (valid) & U+d800 (never valid)
            let buffer = b"\x00\x00\x02\x00\x00\xd8\x00\x00\x00\x00\x00\x00";
            let ptr = unsafe {
                crate::ffi::PyUnicode_FromKindAndData(
                    crate::ffi::PyUnicode_4BYTE_KIND as _,
                    buffer.as_ptr().cast(),
                    2,
                )
            };
            assert!(!ptr.is_null());
            let s = unsafe { ptr.assume_owned(py).cast_into_unchecked::<PyString>() };
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
        Python::attach(|py| {
            let py_string1 = PyString::intern(py, "foo");
            assert_eq!(py_string1, "foo");

            let py_string2 = PyString::intern(py, "foo");
            assert_eq!(py_string2, "foo");

            assert_eq!(py_string1.as_ptr(), py_string2.as_ptr());

            let py_string3 = PyString::intern(py, "bar");
            assert_eq!(py_string3, "bar");

            assert_ne!(py_string1.as_ptr(), py_string3.as_ptr());
        });
    }

    #[test]
    fn test_py_to_str_utf8() {
        Python::attach(|py| {
            let s = "ascii 🐈";
            let py_string = PyString::new(py, s).unbind();

            #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
            assert_eq!(s, py_string.to_str(py).unwrap());

            assert_eq!(s, py_string.to_cow(py).unwrap());
        })
    }

    #[test]
    fn test_py_to_str_surrogate() {
        Python::attach(|py| {
            let py_string: Py<PyString> = py
                .eval(ffi::c_str!(r"'\ud800'"), None, None)
                .unwrap()
                .extract()
                .unwrap();

            #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
            assert!(py_string.to_str(py).is_err());

            assert!(py_string.to_cow(py).is_err());
        })
    }

    #[test]
    fn test_py_to_string_lossy() {
        Python::attach(|py| {
            let py_string: Py<PyString> = py
                .eval(ffi::c_str!(r"'🐈 Hello \ud800World'"), None, None)
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(py_string.to_string_lossy(py), "🐈 Hello ���World");
        })
    }

    #[test]
    fn test_comparisons() {
        Python::attach(|py| {
            let s = "hello, world";
            let py_string = PyString::new(py, s);

            assert_eq!(py_string, "hello, world");

            assert_eq!(py_string, s);
            assert_eq!(&py_string, s);
            assert_eq!(s, py_string);
            assert_eq!(s, &py_string);

            assert_eq!(py_string, *s);
            assert_eq!(&py_string, *s);
            assert_eq!(*s, py_string);
            assert_eq!(*s, &py_string);

            let py_string = py_string.as_borrowed();

            assert_eq!(py_string, s);
            assert_eq!(&py_string, s);
            assert_eq!(s, py_string);
            assert_eq!(s, &py_string);

            assert_eq!(py_string, *s);
            assert_eq!(*s, py_string);
        })
    }
}
