use std::borrow::Cow;

use err::PyResult;
use objects::{PyObject, PyString};
use python::{ToPyPointer, Python};
use conversion::{ToPyObject, IntoPyObject, RefFromPyObject};

/// Converts Rust `str` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for str {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}
impl<'a> IntoPyObject for &'a str {
    #[inline]
    fn into_object(self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

/// Converts Rust `Cow<str>` to Python object.
/// See `PyString::new` for details on the conversion.
impl<'a> ToPyObject for Cow<'a, str> {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

/// Converts Rust `String` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}
impl IntoPyObject for String {
    #[inline]
    fn into_object(self, py: Python) -> PyObject {
        PyString::new(py, &self).into()
    }
}

// /// Allows extracting strings from Python objects.
// /// Accepts Python `str` and `unicode` objects.
pyobject_extract!(py, obj to Cow<'source, str> => {
    try!(obj.cast_as::<PyString>(py)).to_string(py)
});


/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
pyobject_extract!(py, obj to String => {
    let s = try!(obj.cast_as::<PyString>(py));
    s.to_string(py).map(Cow::into_owned)
});


impl<'p> RefFromPyObject<'p> for str {
    fn with_extracted<F, R>(py: Python, obj: &'p PyObject, f: F) -> PyResult<R>
        where F: FnOnce(&str) -> R
    {
        let p = PyObject::from_borrowed_ptr(py, obj.as_ptr());
        let s = try!(p.extract::<Cow<str>>(py));
        Ok(f(&s))
    }
}
