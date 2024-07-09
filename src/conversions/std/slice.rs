use std::{borrow::Cow, convert::Infallible};

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    conversion::IntoPyObject,
    types::{PyByteArray, PyByteArrayMethods, PyBytes},
    Bound, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject,
};

impl<'a> IntoPy<PyObject> for &'a [u8] {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyBytes::new(py, self).unbind().into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("bytes")
    }
}

impl<'py> IntoPyObject<'py> for &[u8] {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new_bound(py, self))
    }
}

impl<'a> crate::conversion::FromPyObjectBound<'a, '_> for &'a [u8] {
    fn from_py_object_bound(obj: crate::Borrowed<'a, '_, PyAny>) -> PyResult<Self> {
        Ok(obj.downcast::<PyBytes>()?.as_bytes())
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

/// Special-purpose trait impl to efficiently handle both `bytes` and `bytearray`
///
/// If the source object is a `bytes` object, the `Cow` will be borrowed and
/// pointing into the source object, and no copying or heap allocations will happen.
/// If it is a `bytearray`, its contents will be copied to an owned `Cow`.
impl<'a> crate::conversion::FromPyObjectBound<'a, '_> for Cow<'a, [u8]> {
    fn from_py_object_bound(ob: crate::Borrowed<'a, '_, PyAny>) -> PyResult<Self> {
        if let Ok(bytes) = ob.downcast::<PyBytes>() {
            return Ok(Cow::Borrowed(bytes.as_bytes()));
        }

        let byte_array = ob.downcast::<PyByteArray>()?;
        Ok(Cow::Owned(byte_array.to_vec()))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

impl ToPyObject for Cow<'_, [u8]> {
    fn to_object(&self, py: Python<'_>) -> Py<PyAny> {
        PyBytes::new(py, self.as_ref()).into()
    }
}

impl IntoPy<Py<PyAny>> for Cow<'_, [u8]> {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        self.to_object(py)
    }
}

impl<'py> IntoPyObject<'py> for Cow<'_, [u8]> {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new_bound(py, &self))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::{
        types::{any::PyAnyMethods, PyBytes},
        Python, ToPyObject,
    };

    #[test]
    fn test_extract_bytes() {
        Python::with_gil(|py| {
            let py_bytes = py.eval_bound("b'Hello Python'", None, None).unwrap();
            let bytes: &[u8] = py_bytes.extract().unwrap();
            assert_eq!(bytes, b"Hello Python");
        });
    }

    #[test]
    fn test_cow_impl() {
        Python::with_gil(|py| {
            let bytes = py.eval_bound(r#"b"foobar""#, None, None).unwrap();
            let cow = bytes.extract::<Cow<'_, [u8]>>().unwrap();
            assert_eq!(cow, Cow::<[u8]>::Borrowed(b"foobar"));

            let byte_array = py
                .eval_bound(r#"bytearray(b"foobar")"#, None, None)
                .unwrap();
            let cow = byte_array.extract::<Cow<'_, [u8]>>().unwrap();
            assert_eq!(cow, Cow::<[u8]>::Owned(b"foobar".to_vec()));

            let something_else_entirely = py.eval_bound("42", None, None).unwrap();
            something_else_entirely
                .extract::<Cow<'_, [u8]>>()
                .unwrap_err();

            let cow = Cow::<[u8]>::Borrowed(b"foobar").to_object(py);
            assert!(cow.bind(py).is_instance_of::<PyBytes>());

            let cow = Cow::<[u8]>::Owned(b"foobar".to_vec()).to_object(py);
            assert!(cow.bind(py).is_instance_of::<PyBytes>());
        });
    }
}
