use python::{PythonObject, PythonObjectWithCheckedDowncast};
use objects::PyObject;
use err::{PyErr, PyResult};
use pyptr::PyPtr;
use ffi;

pythonobject_newtype_only_pythonobject!(PyIterator);

impl <'p> PythonObjectWithCheckedDowncast<'p> for PyIterator<'p> {
    #[inline]
    fn downcast_from<'a>(o: &'a PyObject<'p>) -> Option<&'a PyIterator<'p>> {
        unsafe {
            if ffi::PyIter_Check(o.as_ptr()) {
                Some(PythonObject::unchecked_downcast_from(o))
            } else {
                None
            }
        }
    }
}

impl <'p> PyIterator<'p> {
    /// Retrieves the next item from an iterator.
    /// Returns None when the iterator is exhausted.
    #[inline]
    pub fn iter_next(&self) -> PyResult<'p, Option<PyPtr<'p, PyObject<'p>>>> {
        let py = self.python();
        let r = unsafe { ffi::PyIter_Next(self.as_ptr()) };
        if r.is_null() {
            if PyErr::occurred(py) {
                Err(PyErr::fetch(py))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(unsafe { PyPtr::from_owned_ptr(py, r) }))
        }
    }
}


