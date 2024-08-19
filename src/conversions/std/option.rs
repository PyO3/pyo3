use crate::{
    conversion::IntoPyObject, ffi, types::any::PyAnyMethods, AsPyPointer, Bound, BoundObject,
    FromPyObject, IntoPy, PyAny, PyObject, PyResult, Python, ToPyObject,
};

/// `Option::Some<T>` is converted like `T`.
/// `Option::None` is converted to Python `None`.
impl<T> ToPyObject for Option<T>
where
    T: ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_ref()
            .map_or_else(|| py.None(), |val| val.to_object(py))
    }
}

impl<T> IntoPy<PyObject> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.map_or_else(|| py.None(), |val| val.into_py(py))
    }
}

impl<'py, T> IntoPyObject<'py> for Option<T>
where
    T: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = T::Error;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.map_or_else(
            || Ok(py.None().into_bound(py)),
            |val| {
                val.into_pyobject(py)
                    .map(BoundObject::into_any)
                    .map(BoundObject::into_bound)
            },
        )
    }
}

impl<'a, 'py, T> IntoPyObject<'py> for &'a Option<T>
where
    &'a T: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = <&'a T as IntoPyObject<'py>>::Error;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_ref().into_pyobject(py)
    }
}

impl<'py, T> FromPyObject<'py> for Option<T>
where
    T: FromPyObject<'py>,
{
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        if obj.is_none() {
            Ok(None)
        } else {
            obj.extract().map(Some)
        }
    }
}

/// Convert `None` into a null pointer.
unsafe impl<T> AsPyPointer for Option<T>
where
    T: AsPyPointer,
{
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.as_ref()
            .map_or_else(std::ptr::null_mut, |t| t.as_ptr())
    }
}

#[cfg(test)]
mod tests {
    use crate::{PyObject, Python};

    #[test]
    fn test_option_as_ptr() {
        Python::with_gil(|py| {
            use crate::AsPyPointer;
            let mut option: Option<PyObject> = None;
            assert_eq!(option.as_ptr(), std::ptr::null_mut());

            let none = py.None();
            option = Some(none.clone_ref(py));

            let ref_cnt = none.get_refcnt(py);
            assert_eq!(option.as_ptr(), none.as_ptr());

            // Ensure ref count not changed by as_ptr call
            assert_eq!(none.get_refcnt(py), ref_cnt);
        });
    }
}
