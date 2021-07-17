// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::{ffi, AsPyPointer, PyAny, PyErr, PyNativeType, PyResult, Python};
#[cfg(any(not(Py_LIMITED_API), Py_3_8))]
use crate::{PyDowncastError, PyTryFrom};

/// A Python iterator object.
///
/// # Examples
///
/// ```rust
/// # use pyo3::prelude::*;
/// use pyo3::types::PyIterator;
///
/// # fn main() -> PyResult<()> {
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// let list = py.eval("iter([1, 2, 3, 4])", None, None)?;
/// let numbers: PyResult<Vec<usize>> = list.iter()?.map(|i| i.and_then(PyAny::extract::<usize>)).collect();
/// let sum: usize = numbers?.iter().sum();
/// assert_eq!(sum, 10);
/// # Ok(())
/// # }
/// ```
#[repr(transparent)]
pub struct PyIterator(PyAny);
pyobject_native_type_named!(PyIterator);
#[cfg(any(not(Py_LIMITED_API), Py_3_8))]
pyobject_native_type_extract!(PyIterator);

impl PyIterator {
    /// Constructs a `PyIterator` from a Python iterable object.
    ///
    /// Equivalent to Python's built-in `iter` function.
    pub fn from_object<'p, T>(py: Python<'p>, obj: &T) -> PyResult<&'p PyIterator>
    where
        T: AsPyPointer,
    {
        unsafe { py.from_owned_ptr_or_err(ffi::PyObject_GetIter(obj.as_ptr())) }
    }
}

impl<'p> Iterator for &'p PyIterator {
    type Item = PyResult<&'p PyAny>;

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
                    Some(Err(PyErr::api_call_failed(py)))
                } else {
                    None
                }
            }
        }
    }
}

// PyIter_Check does not exist in the limited API until 3.8
#[cfg(any(not(Py_LIMITED_API), Py_3_8))]
#[cfg_attr(docsrs, doc(cfg(any(not(Py_LIMITED_API), Py_3_8))))]
impl<'v> PyTryFrom<'v> for PyIterator {
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v PyIterator, PyDowncastError<'v>> {
        let value = value.into();
        unsafe {
            if ffi::PyIter_Check(value.as_ptr()) != 0 {
                Ok(<PyIterator as PyTryFrom>::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value, "Iterator"))
            }
        }
    }

    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v PyIterator, PyDowncastError<'v>> {
        <PyIterator as PyTryFrom>::try_from(value)
    }

    #[inline]
    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v PyIterator {
        let ptr = value.into() as *const _ as *const PyIterator;
        &*ptr
    }
}

#[cfg(test)]
mod tests {
    use super::PyIterator;
    use crate::exceptions::PyTypeError;
    use crate::gil::GILPool;
    use crate::types::{PyDict, PyList};
    #[cfg(any(not(Py_LIMITED_API), Py_3_8))]
    use crate::{Py, PyAny, PyTryFrom};
    use crate::{Python, ToPyObject};
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
            count = obj.get_refcnt(py);
        }

        {
            let gil_guard = Python::acquire_gil();
            let py = gil_guard.python();
            let inst = obj.as_ref(py);
            let mut it = inst.iter().unwrap();

            assert_eq!(10, it.next().unwrap().unwrap().extract().unwrap());
        }
        assert_eq!(count, obj.get_refcnt(Python::acquire_gil().python()));
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
            count = none.get_refcnt(py);
            obj = l.to_object(py);
        }

        {
            let _pool = unsafe { GILPool::new() };
            let inst = obj.as_ref(py);
            let mut it = inst.iter().unwrap();

            assert_eq!(10, it.next().unwrap().unwrap().extract().unwrap());
            assert!(it.next().unwrap().unwrap().is_none());
        }
        assert_eq!(count, none.get_refcnt(py));
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

        let gil = Python::acquire_gil();
        let py = gil.python();

        let context = PyDict::new(py);
        py.run(fibonacci_generator, None, Some(context)).unwrap();

        let generator = py.eval("fibonacci(5)", None, Some(context)).unwrap();
        for (actual, expected) in generator.iter().unwrap().zip(&[1, 1, 2, 3, 5]) {
            let actual = actual.unwrap().extract::<usize>().unwrap();
            assert_eq!(actual, *expected)
        }
    }

    #[test]
    fn int_not_iterable() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let x = 5.to_object(py);
        let err = PyIterator::from_object(py, &x).unwrap_err();

        assert!(err.is_instance::<PyTypeError>(py))
    }

    #[test]
    #[cfg(any(not(Py_LIMITED_API), Py_3_8))]
    fn iterator_try_from() {
        let gil_guard = Python::acquire_gil();
        let py = gil_guard.python();
        let obj: Py<PyAny> = vec![10, 20].to_object(py).as_ref(py).iter().unwrap().into();
        let iter: &PyIterator = PyIterator::try_from(obj.as_ref(py)).unwrap();
        assert_eq!(obj, iter.into());
    }
}
