#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::types::PyAnyMethods;
use crate::{Bound, FromPyObject, PyAny, PyResult};
use std::sync::Arc;

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
