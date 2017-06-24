// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use ffi;
use objects::PyObjectRef;
use python::{Python, ToPyPointer, IntoPyPointer};
use instance::PyObjectWithToken;
use err::{PyErr, PyResult, PyDowncastError};

/// A python iterator object.
///
/// Unlike other python objects, this class includes a `Python<'p>` token
/// so that `PyIterator` can implement the rust `Iterator` trait.
pub struct PyIterator<'p>(&'p PyObjectRef);


impl <'p> PyIterator<'p> {
    /// Constructs a `PyIterator` from a Python iterator object.
    pub fn from_object<T>(py: Python<'p>, obj: T)
                          -> Result<PyIterator<'p>, PyDowncastError<'p>>
        where T: IntoPyPointer
    {
        unsafe {
            let ptr = obj.into_ptr();
            if ffi::PyIter_Check(ptr) != 0 {
                Ok(PyIterator(py.cast_from_ptr(ptr)))
            } else {
                ffi::Py_DECREF(ptr);
                Err(PyDowncastError(py, None))
            }
        }
    }
}

impl <'p> Iterator for PyIterator<'p> {
    type Item = PyResult<&'p PyObjectRef>;

    /// Retrieves the next item from an iterator.
    /// Returns `None` when the iterator is exhausted.
    /// If an exception occurs, returns `Some(Err(..))`.
    /// Further `next()` calls after an exception occurs are likely
    /// to repeatedly result in the same exception.
    fn next(&mut self) -> Option<Self::Item> {
        let py = self.0.token();

        match unsafe {
            py.cast_from_ptr_or_opt(ffi::PyIter_Next(self.0.as_ptr())) }
        {
            Some(obj) => Some(Ok(obj)),
            None => {
                if PyErr::occurred(py) {
                    Some(Err(PyErr::fetch(py)))
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use instance::AsPyRef;
    use python::{Python, PyDowncastFrom};
    use conversion::ToPyObject;
    use objects::PyObjectRef;
    use objectprotocol::ObjectProtocol;

    #[test]
    fn vec_iter() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let obj = vec![10, 20].to_object(py);
        let inst = PyObjectRef::downcast_from(obj.as_ref(py)).unwrap();
        let mut it = inst.iter().unwrap();
        assert_eq!(10, it.next().unwrap().unwrap().extract().unwrap());
        assert_eq!(20, it.next().unwrap().unwrap().extract().unwrap());
        assert!(it.next().is_none());
    }
}
