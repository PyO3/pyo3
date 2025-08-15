use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Borrowed;
use crate::py_result_ext::PyResultExt;
use crate::{ffi, Bound, PyAny, PyErr, PyResult, PyTypeCheck};

/// A Python iterator object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyIterator>`][crate::Py] or [`Bound<'py, PyIterator>`][Bound].
///
/// # Examples
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::ffi::c_str;
///
/// # fn main() -> PyResult<()> {
/// Python::attach(|py| -> PyResult<()> {
///     let list = py.eval(c_str!("iter([1, 2, 3, 4])"), None, None)?;
///     let numbers: PyResult<Vec<usize>> = list
///         .try_iter()?
///         .map(|i| i.and_then(|i|i.extract::<usize>()))
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

impl PyIterator {
    /// Builds an iterator for an iterable Python object; the equivalent of calling `iter(obj)` in Python.
    ///
    /// Usually it is more convenient to write [`obj.try_iter()`][crate::types::any::PyAnyMethods::try_iter],
    /// which is a more concise way of calling this function.
    pub fn from_object<'py>(obj: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyIterator>> {
        unsafe {
            ffi::PyObject_GetIter(obj.as_ptr())
                .assume_owned_or_err(obj.py())
                .cast_into_unchecked()
        }
    }
}

#[derive(Debug)]
#[cfg(all(not(PyPy), Py_3_10))]
pub enum PySendResult<'py> {
    Next(Bound<'py, PyAny>),
    Return(Bound<'py, PyAny>),
}

#[cfg(all(not(PyPy), Py_3_10))]
impl<'py> Bound<'py, PyIterator> {
    /// Sends a value into a python generator. This is the equivalent of calling `generator.send(value)` in Python.
    /// This resumes the generator and continues its execution until the next `yield` or `return` statement.
    /// If the generator exits without returning a value, this function returns a `StopException`.
    /// The first call to `send` must be made with `None` as the argument to start the generator, failing to do so will raise a `TypeError`.
    #[inline]
    pub fn send(&self, value: &Bound<'py, PyAny>) -> PyResult<PySendResult<'py>> {
        let py = self.py();
        let mut result = std::ptr::null_mut();
        match unsafe { ffi::PyIter_Send(self.as_ptr(), value.as_ptr(), &mut result) } {
            ffi::PySendResult::PYGEN_ERROR => Err(PyErr::fetch(py)),
            ffi::PySendResult::PYGEN_RETURN => Ok(PySendResult::Return(unsafe {
                result.assume_owned_unchecked(py)
            })),
            ffi::PySendResult::PYGEN_NEXT => Ok(PySendResult::Next(unsafe {
                result.assume_owned_unchecked(py)
            })),
        }
    }
}

impl<'py> Iterator for Bound<'py, PyIterator> {
    type Item = PyResult<Bound<'py, PyAny>>;

    /// Retrieves the next item from an iterator.
    ///
    /// Returns `None` when the iterator is exhausted.
    /// If an exception occurs, returns `Some(Err(..))`.
    /// Further `next()` calls after an exception occurs are likely
    /// to repeatedly result in the same exception.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Borrowed::from(&*self).next()
    }

    #[cfg(not(Py_LIMITED_API))]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = unsafe { ffi::PyObject_LengthHint(self.as_ptr(), 0) };
        (hint.max(0) as usize, None)
    }
}

impl<'py> Borrowed<'_, 'py, PyIterator> {
    // TODO: this method is on Borrowed so that &'py PyIterator can use this; once that
    // implementation is deleted this method should be moved to the `Bound<'py, PyIterator> impl
    fn next(self) -> Option<PyResult<Bound<'py, PyAny>>> {
        let py = self.py();

        match unsafe { ffi::PyIter_Next(self.as_ptr()).assume_owned_or_opt(py) } {
            Some(obj) => Some(Ok(obj)),
            None => PyErr::take(py).map(Err),
        }
    }
}

impl<'py> IntoIterator for &Bound<'py, PyIterator> {
    type Item = PyResult<Bound<'py, PyAny>>;
    type IntoIter = Bound<'py, PyIterator>;

    fn into_iter(self) -> Self::IntoIter {
        self.clone()
    }
}

impl PyTypeCheck for PyIterator {
    const NAME: &'static str = "Iterator";
    #[cfg(feature = "experimental-inspect")]
    const PYTHON_TYPE: &'static str = "collections.abc.Iterator";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        unsafe { ffi::PyIter_Check(object.as_ptr()) != 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::PyIterator;
    #[cfg(all(not(PyPy), Py_3_10))]
    use super::PySendResult;
    use crate::exceptions::PyTypeError;
    #[cfg(all(not(PyPy), Py_3_10))]
    use crate::types::PyNone;
    use crate::types::{PyAnyMethods, PyDict, PyList, PyListMethods};
    use crate::{ffi, IntoPyObject, Python};

    #[test]
    fn vec_iter() {
        Python::attach(|py| {
            let inst = vec![10, 20].into_pyobject(py).unwrap();
            let mut it = inst.try_iter().unwrap();
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
        let (obj, count) = Python::attach(|py| {
            let obj = vec![10, 20].into_pyobject(py).unwrap();
            let count = obj.get_refcnt();
            (obj.unbind(), count)
        });

        Python::attach(|py| {
            let inst = obj.bind(py);
            let mut it = inst.try_iter().unwrap();

            assert_eq!(
                10_i32,
                it.next().unwrap().unwrap().extract::<'_, i32>().unwrap()
            );
        });

        Python::attach(move |py| {
            assert_eq!(count, obj.get_refcnt(py));
        });
    }

    #[test]
    fn iter_item_refcnt() {
        Python::attach(|py| {
            let count;
            let obj = py.eval(ffi::c_str!("object()"), None, None).unwrap();
            let list = {
                let list = PyList::empty(py);
                list.append(10).unwrap();
                list.append(&obj).unwrap();
                count = obj.get_refcnt();
                list
            };

            {
                let mut it = list.iter();

                assert_eq!(10_i32, it.next().unwrap().extract::<'_, i32>().unwrap());
                assert!(it.next().unwrap().is(&obj));
                assert!(it.next().is_none());
            }
            assert_eq!(count, obj.get_refcnt());
        });
    }

    #[test]
    fn fibonacci_generator() {
        let fibonacci_generator = ffi::c_str!(
            r#"
def fibonacci(target):
    a = 1
    b = 1
    for _ in range(target):
        yield a
        a, b = b, a + b
"#
        );

        Python::attach(|py| {
            let context = PyDict::new(py);
            py.run(fibonacci_generator, None, Some(&context)).unwrap();

            let generator = py
                .eval(ffi::c_str!("fibonacci(5)"), None, Some(&context))
                .unwrap();
            for (actual, expected) in generator.try_iter().unwrap().zip(&[1, 1, 2, 3, 5]) {
                let actual = actual.unwrap().extract::<usize>().unwrap();
                assert_eq!(actual, *expected)
            }
        });
    }

    #[test]
    #[cfg(all(not(PyPy), Py_3_10))]
    fn send_generator() {
        let generator = ffi::c_str!(
            r#"
def gen():
    value = None
    while(True):
        value = yield value
        if value is None:
            return
"#
        );

        Python::attach(|py| {
            let context = PyDict::new(py);
            py.run(generator, None, Some(&context)).unwrap();

            let generator = py.eval(ffi::c_str!("gen()"), None, Some(&context)).unwrap();

            let one = 1i32.into_pyobject(py).unwrap();
            assert!(matches!(
                generator.try_iter().unwrap().send(&PyNone::get(py)).unwrap(),
                PySendResult::Next(value) if value.is_none()
            ));
            assert!(matches!(
                generator.try_iter().unwrap().send(&one).unwrap(),
                PySendResult::Next(value) if value.is(&one)
            ));
            assert!(matches!(
                generator.try_iter().unwrap().send(&PyNone::get(py)).unwrap(),
                PySendResult::Return(value) if value.is_none()
            ));
        });
    }

    #[test]
    fn fibonacci_generator_bound() {
        use crate::types::any::PyAnyMethods;
        use crate::Bound;

        let fibonacci_generator = ffi::c_str!(
            r#"
def fibonacci(target):
    a = 1
    b = 1
    for _ in range(target):
        yield a
        a, b = b, a + b
"#
        );

        Python::attach(|py| {
            let context = PyDict::new(py);
            py.run(fibonacci_generator, None, Some(&context)).unwrap();

            let generator: Bound<'_, PyIterator> = py
                .eval(ffi::c_str!("fibonacci(5)"), None, Some(&context))
                .unwrap()
                .cast_into()
                .unwrap();
            let mut items = vec![];
            for actual in &generator {
                let actual = actual.unwrap().extract::<usize>().unwrap();
                items.push(actual);
            }
            assert_eq!(items, [1, 1, 2, 3, 5]);
        });
    }

    #[test]
    fn int_not_iterable() {
        Python::attach(|py| {
            let x = 5i32.into_pyobject(py).unwrap();
            let err = PyIterator::from_object(&x).unwrap_err();

            assert!(err.is_instance_of::<PyTypeError>(py));
        });
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
            fn downcast_iterator(&mut self, obj: &crate::Bound<'_, crate::PyAny>) {
                self.failed = Some(obj.cast::<PyIterator>().unwrap_err().into());
            }
        }

        // Regression test for 2913
        Python::attach(|py| {
            let downcaster = crate::Py::new(py, Downcaster { failed: None }).unwrap();
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
        fn assert_iterator(obj: &crate::Bound<'_, crate::PyAny>) {
            assert!(obj.cast::<PyIterator>().is_ok())
        }

        // Regression test for 2913
        Python::attach(|py| {
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

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn length_hint_becomes_size_hint_lower_bound() {
        Python::attach(|py| {
            let list = py.eval(ffi::c_str!("[1, 2, 3]"), None, None).unwrap();
            let iter = list.try_iter().unwrap();
            let hint = iter.size_hint();
            assert_eq!(hint, (3, None));
        });
    }
}
