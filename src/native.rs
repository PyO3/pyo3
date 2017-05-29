
// use python::{Python, ToPythonPointer, PythonObjectWithCheckedDowncast};
// use err::{PyErr, PyResult};
// use ppptr::pptr;
// use pyptr::Py;
use typeob::PyTypeInfo;
// use conversion::{ToPyObject, FromPyObject};


pub trait PyNativeObject : PyTypeInfo {}


/*impl<'a, T: Sized> FromPyObject<'a> for T
    where T: PyNativeObject + PythonObjectWithCheckedDowncast
{
    /// Extracts `Self` from the source `Py<PyObject>`.
    fn extract<S>(py: &'a Py<'a, S>) -> PyResult<Self> where S: PyTypeInfo
    {
        <T as PythonObjectWithCheckedDowncast>
            ::downcast_from(py.clone_ref()).map_err(|e| e.into())
    }
}*/

/*impl<T> ::IntoPyObject for T where T: PyNativeObject
{
    #[inline]
    default fn into_object(self, py: Python) -> ::PyPtr<PyObject>
    {
        unsafe { ::std::mem::transmute(self) }
    }
}*/
