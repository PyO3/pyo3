use crate::inspect::types::TypeInfo;
use crate::types::list::new_from_iter;
use crate::{IntoPy, PyObject, Python, ToPyObject};

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

    fn type_output() -> TypeInfo {
        TypeInfo::list_of(T::type_output())
    }
}
