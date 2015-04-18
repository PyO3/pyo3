use python::{PythonObject, PythonObjectWithCheckedDowncast, ToPythonPointer};
use objects::PyObject;
use err::{PyErr, PyResult};
use ffi;

pyobject_newtype!(PyIterator, PyIter_Check);

impl <'p> PyIterator<'p> {
    /// Retrieves the next item from an iterator.
    /// Returns `None` when the iterator is exhausted.
    pub fn iter_next(&self) -> PyResult<'p, Option<PyObject<'p>>> {
        let py = self.python();
        match unsafe { PyObject::from_owned_ptr_opt(py, ffi::PyIter_Next(self.as_ptr())) } {
            Some(obj) => Ok(Some(obj)),
            None => {
                if PyErr::occurred(py) {
                    Err(PyErr::fetch(py))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

