// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::conversion::PyTryFrom;
use crate::{ffi, AsPyPointer, PyAny, PyDowncastError, PyErr, PyNativeType, PyResult, Python};

/// A Python iterator object.
///
/// Unlike other Python objects, this class includes a `Python<'p>` token
/// so that `PyIterator` can implement the Rust `Iterator` trait.
///
/// # Example
///
/// ```rust
/// # use pyo3::prelude::*;
/// use pyo3::types::PyIterator;
///
/// # fn main() -> PyResult<()> {
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// let list = py.eval("iter([1, 2, 3, 4])", None, None)?;
/// let numbers: PyResult<Vec<usize>> = list.iter()?.map(|i| i.and_then(ObjectProtocol::extract::<usize>)).collect();
/// let sum: usize = numbers?.iter().sum();
/// assert_eq!(sum, 10);
/// # Ok(())
/// # }
/// ```
pub struct PyIterator<'p>(PyAny<'p>);

pyobject_native_type_named!(PyIterator);
pyobject_native_type_extract!(PyIterator);
pyobject_native_newtype!(PyIterator);

impl<'p> PyIterator<'p> {
    /// Constructs a `PyIterator` from a Python iterator object.
    pub fn from_object<T>(py: Python<'p>, obj: &T) -> Result<PyIterator<'p>, PyDowncastError>
    where
        T: AsPyPointer,
    {
        unsafe {
            let ptr = ffi::PyObject_GetIter(obj.as_ptr());
            // Returns NULL if an object cannot be iterated.
            if ptr.is_null() {
                PyErr::fetch(py);
                return Err(PyDowncastError);
            }

            if ffi::PyIter_Check(ptr) != 0 {
                Ok(py.from_owned_ptr(ptr))
            } else {
                Err(PyDowncastError)
            }
        }
    }
}

impl<'p> Iterator for PyIterator<'p> {
    type Item = PyResult<PyAny<'p>>;

    /// Retrieves the next item from an iterator.
    ///
    /// Returns `None` when the iterator is exhausted.
    /// If an exception occurs, returns `Some(Err(..))`.
    /// Further `next()` calls after an exception occurs are likely
    /// to repeatedly result in the same exception.
    fn next(&mut self) -> Option<Self::Item> {
        let py = self.0.py();

        match unsafe { py.from_owned_ptr_or_opt(ffi::PyIter_Next(self.0.as_ptr())) } {
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

impl<'py> PyTryFrom<'py> for PyIterator<'py> {
    fn try_from<'a>(value: &'a PyAny<'py>) -> Result<&'a PyIterator<'py>, PyDowncastError> {
        unsafe {
            if ffi::PyIter_Check(value.as_ptr()) != 0 {
                Ok(<PyIterator as PyTryFrom>::try_from_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    fn try_from_exact<'a>(value: &'a PyAny<'py>) -> Result<&'a PyIterator<'py>, PyDowncastError> {
        <PyIterator as PyTryFrom>::try_from(value)
    }

    #[inline]
    unsafe fn try_from_unchecked<'a>(value: &'a PyAny<'py>) -> &'a PyIterator<'py> {
        let ptr = value as *const _ as *const PyIterator;
        &*ptr
    }
}

#[cfg(test)]
mod tests {
    use crate::gil::GILPool;
    use crate::instance::AsPyRef;
    use crate::objectprotocol::ObjectProtocol;
    use crate::types::{PyDict, PyList};
    use crate::GILGuard;
    use crate::Python;
    use crate::ToPyObject;
    use indoc::indoc;

    #[test]
    fn vec_iter() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let obj = vec![10, 20].to_object(py);
        let inst = obj.as_ref(py);
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
            let inst = obj.as_ref(py);
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
            let _pool = unsafe { GILPool::new() };
            let l = PyList::empty(py);
            none = py.None();
            l.append(10).unwrap();
            l.append(&none).unwrap();
            count = none.get_refcnt();
            obj = l.to_object(py);
        }

        {
            let _pool = unsafe { GILPool::new() };
            let inst = obj.as_ref(py);
            let mut it = inst.iter().unwrap();

            assert_eq!(10, it.next().unwrap().unwrap().extract().unwrap());
            assert!(it.next().unwrap().unwrap().is_none());
        }
        assert_eq!(count, none.get_refcnt());
    }

    #[test]
    fn fibonacci_generator() {
        let fibonacci_generator = indoc!(
            r#"
            def fibonacci(target):
                a = 1
                b = 1
                for _ in range(target):
                    yield a
                    a, b = b, a + b
        "#
        );

        let gil = GILGuard::acquire();
        let py = gil.python();

        let context = PyDict::new(py);
        py.run(fibonacci_generator, None, Some(&context)).unwrap();

        let generator = py.eval("fibonacci(5)", None, Some(&context)).unwrap();
        for (actual, expected) in generator.iter().unwrap().zip(&[1, 1, 2, 3, 5]) {
            let actual = actual.unwrap().extract::<usize>().unwrap();
            assert_eq!(actual, *expected)
        }
    }
}
