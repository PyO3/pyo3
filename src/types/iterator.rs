// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::{ffi, AsPyPointer, PyAny, PyErr, PyResult, Python};
use crate::{PyDowncastError, PyTryFrom};

/// A Python iterator object.
///
/// # Examples
///
/// ```rust
/// use pyo3::prelude::*;
///
/// # fn main() -> PyResult<()> {
/// Python::with_gil(|py| -> PyResult<()> {
///     let list = py.eval("iter([1, 2, 3, 4])", None, None)?;
///     let numbers: PyResult<Vec<usize>> = list
///         .iter()?
///         .map(|i| i.and_then(PyAny::extract::<usize>))
///         .collect();
///     let sum: usize = numbers?.iter().sum();
///     assert_eq!(sum, 10);
///     Ok(())
/// })
/// # }
/// ```
#[repr(transparent)]
pub struct PyIterator(PyAny);
pyobject_native_type_named!(PyIterator);
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
            None => PyErr::take(py).map(Err),
        }
    }
}

// PyIter_Check does not exist in the limited API until 3.8
impl<'v> PyTryFrom<'v> for PyIterator {
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v PyIterator, PyDowncastError<'v>> {
        let value = value.into();
        unsafe {
            if ffi::PyIter_Check(value.as_ptr()) != 0 {
                Ok(value.downcast_unchecked())
            } else {
                Err(PyDowncastError::new(value, "Iterator"))
            }
        }
    }

    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v PyIterator, PyDowncastError<'v>> {
        value.into().downcast()
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
    use crate::{Py, PyAny, Python, ToPyObject};

    #[test]
    fn vec_iter() {
        Python::with_gil(|py| {
            let obj = vec![10, 20].to_object(py);
            let inst = obj.as_ref(py);
            let mut it = inst.iter().unwrap();
            assert_eq!(
                10_i32,
                it.next().unwrap().unwrap().extract::<'_, i32>().unwrap()
            );
            assert_eq!(
                20_i32,
                it.next().unwrap().unwrap().extract::<'_, i32>().unwrap()
            );
            assert!(it.next().is_none());
        });
    }

    #[test]
    fn iter_refcnt() {
        let (obj, count) = Python::with_gil(|py| {
            let obj = vec![10, 20].to_object(py);
            let count = obj.get_refcnt(py);
            (obj, count)
        });

        Python::with_gil(|py| {
            let inst = obj.as_ref(py);
            let mut it = inst.iter().unwrap();

            assert_eq!(
                10_i32,
                it.next().unwrap().unwrap().extract::<'_, i32>().unwrap()
            );
        });

        Python::with_gil(|py| {
            assert_eq!(count, obj.get_refcnt(py));
        });
    }

    #[test]
    fn iter_item_refcnt() {
        Python::with_gil(|py| {
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

                assert_eq!(
                    10_i32,
                    it.next().unwrap().unwrap().extract::<'_, i32>().unwrap()
                );
                assert!(it.next().unwrap().unwrap().is_none());
            }
            assert_eq!(count, none.get_refcnt(py));
        });
    }

    #[test]
    fn fibonacci_generator() {
        let fibonacci_generator = r#"
def fibonacci(target):
    a = 1
    b = 1
    for _ in range(target):
        yield a
        a, b = b, a + b
"#;

        Python::with_gil(|py| {
            let context = PyDict::new(py);
            py.run(fibonacci_generator, None, Some(context)).unwrap();

            let generator = py.eval("fibonacci(5)", None, Some(context)).unwrap();
            for (actual, expected) in generator.iter().unwrap().zip(&[1, 1, 2, 3, 5]) {
                let actual = actual.unwrap().extract::<usize>().unwrap();
                assert_eq!(actual, *expected)
            }
        });
    }

    #[test]
    fn int_not_iterable() {
        Python::with_gil(|py| {
            let x = 5.to_object(py);
            let err = PyIterator::from_object(py, &x).unwrap_err();

            assert!(err.is_instance_of::<PyTypeError>(py));
        });
    }

    #[test]

    fn iterator_try_from() {
        Python::with_gil(|py| {
            let obj: Py<PyAny> = vec![10, 20].to_object(py).as_ref(py).iter().unwrap().into();
            let iter: &PyIterator = obj.downcast(py).unwrap();
            assert!(obj.is(iter));
        });
    }

    #[test]
    fn test_as_ref() {
        Python::with_gil(|py| {
            let iter: Py<PyIterator> = PyAny::iter(PyList::empty(py)).unwrap().into();
            let mut iter_ref: &PyIterator = iter.as_ref(py);
            assert!(iter_ref.next().is_none());
        })
    }

    #[test]
    fn test_into_ref() {
        Python::with_gil(|py| {
            let bare_iter = PyAny::iter(PyList::empty(py)).unwrap();
            assert_eq!(bare_iter.get_refcnt(), 1);
            let iter: Py<PyIterator> = bare_iter.into();
            assert_eq!(bare_iter.get_refcnt(), 2);
            let mut iter_ref = iter.into_ref(py);
            assert!(iter_ref.next().is_none());
            assert_eq!(iter_ref.get_refcnt(), 2);
        })
    }

    #[test]
    #[cfg(feature = "macros")]
    fn python_class_not_iterator() {
        use crate::PyErr;

        #[crate::pyclass(crate = "crate")]
        struct Downcaster {
            failed: Option<PyErr>,
        }

        #[crate::pymethods(crate = "crate")]
        impl Downcaster {
            fn downcast_iterator(&mut self, obj: &PyAny) {
                self.failed = Some(obj.downcast::<PyIterator>().unwrap_err().into());
            }
        }

        // Regression test for 2913
        Python::with_gil(|py| {
            let downcaster = Py::new(py, Downcaster { failed: None }).unwrap();
            crate::py_run!(
                py,
                downcaster,
                r#"
                    from collections.abc import Sequence

                    class MySequence(Sequence):
                        def __init__(self):
                            self._data = [1, 2, 3]

                        def __getitem__(self, index):
                            return self._data[index]

                        def __len__(self):
                            return len(self._data)

                    downcaster.downcast_iterator(MySequence())
                "#
            );

            assert_eq!(
                downcaster.borrow_mut(py).failed.take().unwrap().to_string(),
                "TypeError: 'MySequence' object cannot be converted to 'Iterator'"
            );
        });
    }

    #[test]
    #[cfg(feature = "macros")]
    fn python_class_iterator() {
        #[crate::pyfunction(crate = "crate")]
        fn assert_iterator(obj: &PyAny) {
            assert!(obj.downcast::<PyIterator>().is_ok())
        }

        // Regression test for 2913
        Python::with_gil(|py| {
            let assert_iterator = crate::wrap_pyfunction!(assert_iterator, py).unwrap();
            crate::py_run!(
                py,
                assert_iterator,
                r#"
                    class MyIter:
                        def __next__(self):
                            raise StopIteration

                    assert_iterator(MyIter())
                "#
            );
        });
    }
}
