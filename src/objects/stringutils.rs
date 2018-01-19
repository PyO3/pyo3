// Copyright (c) 2017-present PyO3 Project and Contributors
use std::borrow::Cow;

use err::PyResult;
use python::Python;
use object::PyObject;
use objects::{PyObjectRef, PyString};
use instance::PyObjectWithToken;
use conversion::{ToPyObject, IntoPyObject, PyTryFrom};

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
impl<'a> IntoPyObject for &'a String {
    #[inline]
    fn into_object(self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'source> ::FromPyObject<'source> for Cow<'source, str> {
    fn extract(ob: &'source PyObjectRef) -> PyResult<Self> {
        PyString::try_from(ob)?.to_string()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl<'a> ::FromPyObject<'a> for &'a str {
    fn extract(ob: &'a PyObjectRef) -> PyResult<Self> {
        let s: Cow<'a, str> = ::FromPyObject::extract(ob)?;
        match s {
            Cow::Borrowed(r) => Ok(r),
            Cow::Owned(r) => {
                let r = ob.py().register_any(r);
                Ok(r.as_str())
            }
        }
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
pyobject_extract!(obj to String => {
    <PyString as PyTryFrom>::try_from(obj)?.to_string().map(Cow::into_owned)
});
