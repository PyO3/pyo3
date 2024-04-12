use crate::conversion::{IntoPyObject, IntoPyObjectExt};
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::types::list::{new_from_iter, try_new_from_iter};
use crate::types::PyList;
use crate::{Bound, IntoPy, PyAny, PyErr, PyObject, Python, ToPyObject};

impl<T> ToPyObject for [T]
where
    T: ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let mut iter = self.iter().map(|e| e.to_object(py));
        let list = new_from_iter(py, &mut iter);
        list.into()
    }
}

impl<T> ToPyObject for Vec<T>
where
    T: ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_slice().to_object(py)
    }
}

impl<T> IntoPy<PyObject> for Vec<T>
where
    T: IntoPy<PyObject>,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        let mut iter = self.into_iter().map(|e| e.into_py(py));
        let list = new_from_iter(py, &mut iter);
        list.into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::list_of(T::type_output())
    }
}

impl<'py, T> IntoPyObject<'py, PyList> for Vec<T>
where
    T: IntoPyObject<'py, PyAny>,
    PyErr: From<T::Error>,
{
    type Error = PyErr;

    fn into_pyobj(self, py: Python<'py>) -> Result<Bound<'py, PyList>, Self::Error> {
        let mut iter = self.into_iter().map(|e| {
            e.into_pyobject(py)
                .map(Bound::into_any)
                .map(Bound::unbind)
                .map_err(Into::into)
        });

        try_new_from_iter(py, &mut iter)
    }
}

impl<'py, T> IntoPyObject<'py, PyAny> for Vec<T>
where
    T: IntoPyObject<'py, PyAny>,
    PyErr: From<T::Error>,
{
    type Error = PyErr;

    fn into_pyobj(self, py: Python<'py>) -> Result<Bound<'py, PyAny>, Self::Error> {
        self.into_pyobject::<PyList, _>(py).map(Bound::into_any)
    }
}
