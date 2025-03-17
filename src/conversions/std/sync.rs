#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::types::PyAnyMethods;
use crate::{Bound, BoundObject, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python};
use std::sync::Arc;

// TODO find a better way (without the extra type parameters) to name the associated types in the trait.
impl<'py, A, T, O, E> IntoPyObject<'py> for Arc<A>
where
    for<'a> &'a A: IntoPyObject<'py, Target = T, Output = O, Error = E>,
    O: BoundObject<'py, T>,
    E: Into<PyErr>,
{
    type Target = T;
    type Output = O;
    type Error = E;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&*self).into_pyobject(py)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <&A as IntoPyObject<'py>>::type_output()
    }
}

impl<'a, 'py, T: 'a> IntoPyObject<'py> for &'a Arc<T>
where
    &'a T: IntoPyObject<'py>,
{
    type Target = <&'a T as IntoPyObject<'py>>::Target;
    type Output = <&'a T as IntoPyObject<'py>>::Output;
    type Error = <&'a T as IntoPyObject<'py>>::Error;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&**self).into_pyobject(py)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        <&'a T as IntoPyObject<'py>>::type_output()
    }
}

impl<'py, T> FromPyObject<'py> for Arc<T>
where
    T: FromPyObject<'py>,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.extract::<T>().map(Arc::new)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        T::type_input()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyInt;
    use crate::Python;

    #[test]
    fn test_arc_into_pyobject() {
        macro_rules! test_roundtrip {
            ($arc:expr) => {
                Python::with_gil(|py| {
                    let arc = $arc;
                    let obj: Bound<'_, PyInt> = arc.into_pyobject(py).unwrap();
                    assert_eq!(obj.extract::<i32>().unwrap(), 42);
                    let roundtrip = obj.extract::<Arc<i32>>().unwrap();
                    assert_eq!(&42, roundtrip.as_ref());
                });
            };
        }

        test_roundtrip!(Arc::new(42));
        test_roundtrip!(&Arc::new(42));
    }
}
