// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::{
    ffi,
    owned::PyOwned,
    types::{Any, Iterator},
    AsPyPointer, PyErr, PyResult, Python, objects::PyNativeObject
};
#[cfg(any(not(Py_LIMITED_API), Py_3_8))]
use crate::{
    objects::{PyAny, PyTryFrom},
    PyDowncastError,
};

/// A Python iterator object.
///
/// # Example
///
/// ```rust
/// # use pyo3::experimental::prelude::*;
/// use pyo3::experimental::objects::PyIterator;
///
/// # fn main() -> PyResult<()> {
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// let list = py.eval("iter([1, 2, 3, 4])", None, None)?;
/// let numbers: PyResult<Vec<usize>> = list.iter()?.map(|i| i.and_then(|any| any.extract())).collect();
/// let sum: usize = numbers?.iter().sum();
/// assert_eq!(sum, 10);
/// # Ok(())
/// # }
/// ```
#[repr(transparent)]
pub struct PyIterator<'py>(Iterator, Python<'py>);
pyo3_native_object!(PyIterator<'py>, Iterator, 'py);

impl<'py> PyIterator<'py> {
    /// Constructs a `PyIterator` from a Python iterable object.
    ///
    /// Equivalent to Python's built-in `iter` function.
    pub fn from_object<T>(py: Python<'py>, obj: &T) -> PyResult<PyOwned<'py, Iterator>>
    where
        T: AsPyPointer,
    {
        unsafe { PyOwned::from_raw_or_fetch_err(py, ffi::PyObject_GetIter(obj.as_ptr())) }
    }

    fn next(&self) -> Option<PyResult<PyOwned<'py, Any>>> {
        let py = self.py();

        match unsafe { PyOwned::from_raw(py, ffi::PyIter_Next(self.0.as_ptr())) } {
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

impl<'py> std::iter::Iterator for &'_ PyIterator<'py> {
    type Item = PyResult<PyOwned<'py, Any>>;

    /// Retrieves the next item from an iterator.
    ///
    /// Returns `None` when the iterator is exhausted.
    /// If an exception occurs, returns `Some(Err(..))`.
    /// Further `next()` calls after an exception occurs are likely
    /// to repeatedly result in the same exception.
    fn next(&mut self) -> Option<Self::Item> {
        (*self).next()
    }
}

impl<'py> std::iter::Iterator for PyOwned<'py, Iterator> {
    type Item = PyResult<PyOwned<'py, Any>>;
    fn next(&mut self) -> Option<Self::Item> {
        (**self).next()
    }
}

// PyIter_Check does not exist in the limited API until 3.8
#[cfg(any(not(Py_LIMITED_API), Py_3_8))]
impl<'a, 'py> PyTryFrom<'a, 'py> for PyIterator<'py> {
    fn try_from(value: &'a PyAny<'py>) -> Result<&'a Self, PyDowncastError<'py>> {
        unsafe {
            if ffi::PyIter_Check(value.as_ptr()) != 0 {
                Ok(<PyIterator as PyTryFrom>::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value.into_ty_ref(), "Iterator"))
            }
        }
    }

    fn try_from_exact(value: &'a PyAny<'py>) -> Result<&'a Self, PyDowncastError<'py>> {
        <PyIterator as PyTryFrom>::try_from(value)
    }

    #[inline]
    unsafe fn try_from_unchecked(value: &'a PyAny<'py>) -> &'a Self {
        let ptr = value as *const _ as *const PyIterator;
        &*ptr
    }
}

#[cfg(test)]
mod tests {
    use super::PyIterator;
    use crate::exceptions::PyTypeError;
    use crate::gil::GILPool;
    use crate::objects::{PyDict, PyList};
    #[cfg(any(not(Py_LIMITED_API), Py_3_8))]
    use crate::{Py, PyAny, objects::PyTryFrom};
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
        py.run(fibonacci_generator, None, Some(context.as_ref())).unwrap();

        let generator = py.eval("fibonacci(5)", None, Some(context.as_ref())).unwrap();
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
        let iter: &PyIterator = PyIterator::try_from(obj.as_object(py)).unwrap();
        assert_eq!(obj, iter.into());
    }
}
