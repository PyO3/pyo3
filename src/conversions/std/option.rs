use crate::{
    conversion::IntoPyObject, types::any::PyAnyMethods, Bound, BoundObject, FromPyObject, PyAny,
    PyResult, Python,
};

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
