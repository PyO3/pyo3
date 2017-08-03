// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use ffi;
use objects::PyObjectRef;
use python::{Python, ToPyPointer};
use instance::PyObjectWithToken;
use err::{PyErr, PyResult, PyDowncastError};

/// A python iterator object.
///
/// Unlike other python objects, this class includes a `Python<'p>` token
/// so that `PyIterator` can implement the rust `Iterator` trait.
pub struct PyIterator<'p>(&'p PyObjectRef);


impl <'p> PyIterator<'p> {
    /// Constructs a `PyIterator` from a Python iterator object.
    pub fn from_object<T>(py: Python<'p>, obj: &T) -> Result<PyIterator<'p>, PyDowncastError>
        where T: ToPyPointer
    {
        unsafe {
            let ptr = ffi::PyObject_GetIter(obj.as_ptr());

            if ffi::PyIter_Check(ptr) != 0 {
                // this is not right, but this cause of segfault check #71
                Ok(PyIterator(py.from_borrowed_ptr(ptr)))
            } else {
                Err(PyDowncastError)
            }
        }
    }
}

impl<'p> Iterator for PyIterator<'p> {
    type Item = PyResult<&'p PyObjectRef>;

    /// Retrieves the next item from an iterator.
    /// Returns `None` when the iterator is exhausted.
    /// If an exception occurs, returns `Some(Err(..))`.
    /// Further `next()` calls after an exception occurs are likely
    /// to repeatedly result in the same exception.
    fn next(&mut self) -> Option<Self::Item> {
        let py = self.0.py();

        match unsafe {
            py.from_owned_ptr_or_opt(ffi::PyIter_Next(self.0.as_ptr())) }
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

/// Dropping a `PyIterator` instance decrements the reference count on the object by 1.
impl<'p> Drop for PyIterator<'p> {

    fn drop(&mut self) {
        unsafe { ffi::Py_DECREF(self.0.as_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use instance::AsPyRef;
    use python::Python;
    use pythonrun::GILPool;
    use conversion::{PyTryFrom, ToPyObject};
    use objects::{PyObjectRef, PyList};
    use objectprotocol::ObjectProtocol;

    #[test]
    fn vec_iter() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let obj = vec![10, 20].to_object(py);
        let inst = PyObjectRef::try_from(obj.as_ref(py)).unwrap();
        let mut it = inst.iter().unwrap();
        assert_eq!(10, it.next().unwrap().unwrap().extract().unwrap());
        assert_eq!(20, it.next().unwrap().unwrap().extract().unwrap());
        assert!(it.next().is_none());
    }

    #[test]
    fn iter_refcnt() {
        let obj;
        let count;
        {
            let gil_guard = Python::acquire_gil();
            let py = gil_guard.python();
            obj = vec![10, 20].to_object(py);
            count = obj.get_refcnt();
        }

        {
            let gil_guard = Python::acquire_gil();
            let py = gil_guard.python();
            let inst = PyObjectRef::try_from(obj.as_ref(py)).unwrap();
            let mut it = inst.iter().unwrap();

            assert_eq!(10, it.next().unwrap().unwrap().extract().unwrap());
        }
        assert_eq!(count, obj.get_refcnt());
    }

    #[test]
    fn iter_item_refcnt() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();

        let obj;
        let none;
        let count;
        {
            let _pool = GILPool::new();
            let l = PyList::empty(py);
            none = py.None();
            l.append(10).unwrap();
            l.append(&none).unwrap();
            count = none.get_refcnt();
            obj = l.to_object(py);
        }

        {
            let _pool = GILPool::new();
            let inst = PyObjectRef::try_from(obj.as_ref(py)).unwrap();
            let mut it = inst.iter().unwrap();

            assert_eq!(10, it.next().unwrap().unwrap().extract().unwrap());
            assert!(it.next().unwrap().unwrap().is_none());
        }
        assert_eq!(count, none.get_refcnt());
    }
}
