#[cfg(feature = "experimental-inspect")]
use crate::inspect::{type_hint_union, PyStaticExpr};
#[cfg(feature = "experimental-inspect")]
use crate::type_object::PyTypeInfo;
#[cfg(feature = "experimental-inspect")]
use crate::types::PyNone;
use crate::{
    conversion::IntoPyObject, types::any::PyAnyMethods, BoundObject, FromPyObject, PyAny, Python,
};
use crate::{Borrowed, Bound};

impl<'py, T> IntoPyObject<'py> for Option<T>
where
    T: IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = T::Error;
    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = type_hint_union!(&T::OUTPUT_TYPE, PyNone::TYPE_HINT);

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
    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = <Option<&T>>::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.as_ref().into_pyobject(py)
    }
}

impl<'a, 'py, T> FromPyObject<'a, 'py> for Option<T>
where
    T: FromPyObject<'a, 'py>,
{
    type Error = T::Error;
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = type_hint_union!(T::INPUT_TYPE, PyNone::TYPE_HINT);

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if obj.is_none() {
            Ok(None)
        } else {
            obj.extract().map(Some)
        }
    }
}
