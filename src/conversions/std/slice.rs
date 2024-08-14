use std::borrow::Cow;

#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    conversion::IntoPyObject,
    types::{PyByteArray, PyByteArrayMethods, PyBytes, PyList},
    Bound, BoundObject, IntoPy, Py, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject,
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

impl<'a, 'py, T> IntoPyObject<'py> for &'a [T]
where
    &'a T: IntoPyObject<'py>,
    PyErr: From<<&'a T as IntoPyObject<'py>>::Error>,
{
    type Target = PyList;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    /// Turns [`&[u8]`](std::slice) into [`PyBytes`], all other `T`s will be turned into a [`PyList`]
    ///
    /// [`PyBytes`]: crate::types::PyBytes
    /// [`PyList`]: crate::types::PyList
    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let mut iter = self.into_iter().map(|e| {
            e.into_pyobject(py)
                .map(BoundObject::into_any)
                .map(BoundObject::unbind)
                .map_err(Into::into)
        });
        crate::types::list::try_new_from_iter(py, &mut iter)
    }
}

impl<'py> IntoPyObject<'py> for &'_ [u8] {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    /// Turns [`&[u8]`](std::slice) into [`PyBytes`]
    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new(py, self))
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

impl<'py, T> IntoPyObject<'py> for Cow<'_, [T]>
where
    T: Clone,
    for<'a> &'a T: IntoPyObject<'py>,
    for<'a> PyErr: From<<&'a T as IntoPyObject<'py>>::Error>,
{
    type Target = PyList;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    /// Turns [`&[u8]`](std::slice) into [`PyBytes`], all other `T`s will be turned into a [`PyList`]
    ///
    /// [`PyBytes`]: crate::types::PyBytes
    /// [`PyList`]: crate::types::PyList
    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let mut iter = self.iter().map(|e| {
            e.into_pyobject(py)
                .map(BoundObject::into_any)
                .map(BoundObject::unbind)
                .map_err(Into::into)
        });
        crate::types::list::try_new_from_iter(py, &mut iter)
    }
}

impl<'py> IntoPyObject<'py> for Cow<'_, [u8]> {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new(py, &self))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::{
        conversion::IntoPyObject,
        types::{any::PyAnyMethods, PyBytes, PyBytesMethods, PyList},
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

    #[test]
    fn test_slice_intopyobject_impl() {
        Python::with_gil(|py| {
            let bytes: &[u8] = b"foobar";
            let obj = bytes.into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyBytes>());
            assert_eq!(obj.as_bytes(), bytes);

            let nums: &[u16] = &[0, 1, 2, 3];
            let obj = nums.into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyList>());
        });
    }

    #[test]
    fn test_cow_intopyobject_impl() {
        Python::with_gil(|py| {
            let borrowed_bytes = Cow::<[u8]>::Borrowed(b"foobar");
            let obj = borrowed_bytes.clone().into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyBytes>());
            assert_eq!(obj.as_bytes(), &*borrowed_bytes);

            let owned_bytes = Cow::<[u8]>::Owned(b"foobar".to_vec());
            let obj = owned_bytes.clone().into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyBytes>());
            assert_eq!(obj.as_bytes(), &*owned_bytes);

            let borrowed_nums = Cow::<[u16]>::Borrowed(&[0, 1, 2, 3]);
            let obj = borrowed_nums.into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyList>());

            let owned_nums = Cow::<[u16]>::Owned(vec![0, 1, 2, 3]);
            let obj = owned_nums.into_pyobject(py).unwrap();
            assert!(obj.is_instance_of::<PyList>());
        });
    }
}
