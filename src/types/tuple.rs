// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::ffi::{self, Py_ssize_t};
use crate::internal_tricks::get_ssize_index;
use crate::types::PySequence;
use crate::{
    exceptions, AsPyPointer, FromPyObject, IntoPy, IntoPyPointer, Py, PyAny, PyErr, PyObject,
    PyResult, PyTryFrom, Python, ToBorrowedObject, ToPyObject,
};

/// Represents a Python `tuple` object.
///
/// This type is immutable.
#[repr(transparent)]
pub struct PyTuple(PyAny);

pyobject_native_type_core!(PyTuple, ffi::PyTuple_Type, #checkfunction=ffi::PyTuple_Check);

impl PyTuple {
    /// Constructs a new tuple with the given elements.
    pub fn new<T, U>(py: Python, elements: impl IntoIterator<Item = T, IntoIter = U>) -> &PyTuple
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>,
    {
        let elements_iter = elements.into_iter();
        let len = elements_iter.len();
        unsafe {
            let ptr = ffi::PyTuple_New(len as Py_ssize_t);
            for (i, e) in elements_iter.enumerate() {
                #[cfg(not(any(Py_LIMITED_API, PyPy)))]
                ffi::PyTuple_SET_ITEM(ptr, i as Py_ssize_t, e.to_object(py).into_ptr());
                #[cfg(any(Py_LIMITED_API, PyPy))]
                ffi::PyTuple_SetItem(ptr, i as Py_ssize_t, e.to_object(py).into_ptr());
            }
            py.from_owned_ptr(ptr)
        }
    }

    /// Constructs an empty tuple (on the Python side, a singleton object).
    pub fn empty(py: Python) -> &PyTuple {
        unsafe { py.from_owned_ptr(ffi::PyTuple_New(0)) }
    }

    /// Gets the length of the tuple.
    pub fn len(&self) -> usize {
        unsafe {
            #[cfg(not(any(Py_LIMITED_API, PyPy)))]
            let size = ffi::PyTuple_GET_SIZE(self.as_ptr());
            #[cfg(any(Py_LIMITED_API, PyPy))]
            let size = ffi::PyTuple_Size(self.as_ptr());
            // non-negative Py_ssize_t should always fit into Rust uint
            size as usize
        }
    }

    /// Checks if the tuple is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `self` cast as a `PySequence`.
    pub fn as_sequence(&self) -> &PySequence {
        unsafe { PySequence::try_from_unchecked(self) }
    }

    /// Takes the slice `self[low:high]` and returns it as a new tuple.
    ///
    /// Indices must be nonnegative, and out-of-range indices are clipped to
    /// `self.len()`.
    pub fn get_slice(&self, low: usize, high: usize) -> &PyTuple {
        unsafe {
            self.py().from_owned_ptr(ffi::PyTuple_GetSlice(
                self.as_ptr(),
                get_ssize_index(low),
                get_ssize_index(high),
            ))
        }
    }

    #[deprecated(since = "0.15.0", note = "use self.get_slice instead")]
    /// Takes the slice `self[low:high]` and returns it as a new tuple.
    ///
    /// Indices must be nonnegative, and out-of-range indices are clipped to
    /// `self.len()`.
    pub fn slice(&self, low: isize, high: isize) -> &PyTuple {
        unsafe {
            self.py()
                .from_owned_ptr(ffi::PyTuple_GetSlice(self.as_ptr(), low, high))
        }
    }

    #[deprecated(
        since = "0.15.0",
        note = "use tuple.get_slice(low, tuple.len()) instead"
    )]
    /// Takes a slice of the tuple from `low` to the end and returns it as a new tuple.
    pub fn split_from(&self, low: usize) -> &PyTuple {
        unsafe {
            let ptr = ffi::PyTuple_GetSlice(
                self.as_ptr(),
                get_ssize_index(low),
                self.len() as Py_ssize_t,
            );
            self.py().from_owned_ptr(ptr)
        }
    }

    /// Gets the tuple item at the specified index.
    /// # Example
    /// ```
    /// use pyo3::{prelude::*, types::PyTuple};
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let ob = (1, 2, 3).to_object(py);
    ///     let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
    ///     let obj = tuple.get_item(0);
    ///     assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn get_item(&self, index: usize) -> PyResult<&PyAny> {
        unsafe {
            let item = ffi::PyTuple_GetItem(self.as_ptr(), index as Py_ssize_t);
            self.py().from_borrowed_ptr_or_err(item)
        }
    }

    /// Gets the tuple item at the specified index. Undefined behavior on bad index. Use with caution.
    ///
    /// # Safety
    ///
    /// Caller must verify that the index is within the bounds of the tuple.
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    pub unsafe fn get_item_unchecked(&self, index: usize) -> &PyAny {
        let item = ffi::PyTuple_GET_ITEM(self.as_ptr(), index as Py_ssize_t);
        self.py().from_borrowed_ptr(item)
    }

    /// Returns `self` as a slice of objects.
    #[cfg(not(Py_LIMITED_API))]
    pub fn as_slice(&self) -> &[&PyAny] {
        // This is safe because &PyAny has the same memory layout as *mut ffi::PyObject,
        // and because tuples are immutable.
        unsafe {
            let ptr = self.as_ptr() as *mut ffi::PyTupleObject;
            let slice = std::slice::from_raw_parts((*ptr).ob_item.as_ptr(), self.len());
            &*(slice as *const [*mut ffi::PyObject] as *const [&PyAny])
        }
    }

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    #[inline]
    pub fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToBorrowedObject,
    {
        self.as_sequence().contains(value)
    }

    /// Returns the first index `i` for which `self[i] == value`.
    ///
    /// This is equivalent to the Python expression `self.index(value)`.
    #[inline]
    pub fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: ToBorrowedObject,
    {
        self.as_sequence().index(value)
    }

    /// Returns an iterator over the tuple items.
    pub fn iter(&self) -> PyTupleIterator {
        PyTupleIterator {
            tuple: self,
            index: 0,
            length: self.len(),
        }
    }
}

index_impls!(PyTuple, "tuple", PyTuple::len, PyTuple::get_slice);

/// Used by `PyTuple::iter()`.
pub struct PyTupleIterator<'a> {
    tuple: &'a PyTuple,
    index: usize,
    length: usize,
}

impl<'a> Iterator for PyTupleIterator<'a> {
    type Item = &'a PyAny;

    #[inline]
    fn next(&mut self) -> Option<&'a PyAny> {
        if self.index < self.length {
            #[cfg(any(Py_LIMITED_API, PyPy))]
            let item = self.tuple.get_item(self.index).expect("tuple.get failed");
            #[cfg(not(any(Py_LIMITED_API, PyPy)))]
            let item = unsafe { self.tuple.get_item_unchecked(self.index) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.length.saturating_sub(self.index as usize),
            Some(self.length.saturating_sub(self.index as usize)),
        )
    }
}

impl<'a> ExactSizeIterator for PyTupleIterator<'a> {
    fn len(&self) -> usize {
        self.length - self.index
    }
}

impl<'a> IntoIterator for &'a PyTuple {
    type Item = &'a PyAny;
    type IntoIter = PyTupleIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn wrong_tuple_length(t: &PyTuple, expected_length: usize) -> PyErr {
    let msg = format!(
        "Expected tuple of length {}, but got tuple of length {}.",
        expected_length,
        t.len()
    );
    exceptions::PyValueError::new_err(msg)
}

macro_rules! tuple_conversion ({$length:expr,$(($refN:ident, $n:tt, $T:ident)),+} => {
    impl <$($T: ToPyObject),+> ToPyObject for ($($T,)+) {
        fn to_object(&self, py: Python) -> PyObject {
            unsafe {
                let ptr = ffi::PyTuple_New($length);
                $(ffi::PyTuple_SetItem(ptr, $n, self.$n.to_object(py).into_ptr());)+
                PyObject::from_owned_ptr(py, ptr)
            }
        }
    }
    impl <$($T: IntoPy<PyObject>),+> IntoPy<PyObject> for ($($T,)+) {
        fn into_py(self, py: Python) -> PyObject {
            unsafe {
                let ptr = ffi::PyTuple_New($length);
                $(ffi::PyTuple_SetItem(ptr, $n, self.$n.into_py(py).into_ptr());)+
                PyObject::from_owned_ptr(py, ptr)
            }
        }
    }

    impl <$($T: IntoPy<PyObject>),+> IntoPy<Py<PyTuple>> for ($($T,)+) {
        fn into_py(self, py: Python) -> Py<PyTuple> {
            unsafe {
                let ptr = ffi::PyTuple_New($length);
                $(ffi::PyTuple_SetItem(ptr, $n, self.$n.into_py(py).into_ptr());)+
                Py::from_owned_ptr(py, ptr)
            }
        }
    }

    impl<'s, $($T: FromPyObject<'s>),+> FromPyObject<'s> for ($($T,)+) {
        fn extract(obj: &'s PyAny) -> PyResult<Self>
        {
            let t = <PyTuple as PyTryFrom>::try_from(obj)?;
            if t.len() == $length {
                #[cfg(any(Py_LIMITED_API, PyPy))]
                return Ok(($(t.get_item($n)?.extract::<$T>()?,)+));

                #[cfg(not(any(Py_LIMITED_API, PyPy)))]
                unsafe {return Ok(($(t.get_item_unchecked($n).extract::<$T>()?,)+));}
            } else {
                Err(wrong_tuple_length(t, $length))
            }
        }
    }
});

tuple_conversion!(1, (ref0, 0, T0));
tuple_conversion!(2, (ref0, 0, T0), (ref1, 1, T1));
tuple_conversion!(3, (ref0, 0, T0), (ref1, 1, T1), (ref2, 2, T2));
tuple_conversion!(
    4,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3)
);
tuple_conversion!(
    5,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4)
);
tuple_conversion!(
    6,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5)
);
tuple_conversion!(
    7,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6)
);
tuple_conversion!(
    8,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6),
    (ref7, 7, T7)
);
tuple_conversion!(
    9,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6),
    (ref7, 7, T7),
    (ref8, 8, T8)
);
tuple_conversion!(
    10,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6),
    (ref7, 7, T7),
    (ref8, 8, T8),
    (ref9, 9, T9)
);
tuple_conversion!(
    11,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6),
    (ref7, 7, T7),
    (ref8, 8, T8),
    (ref9, 9, T9),
    (ref10, 10, T10)
);

tuple_conversion!(
    12,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6),
    (ref7, 7, T7),
    (ref8, 8, T8),
    (ref9, 9, T9),
    (ref10, 10, T10),
    (ref11, 11, T11)
);

#[cfg(test)]
mod tests {
    use crate::types::{PyAny, PyTuple};
    use crate::{PyTryFrom, Python, ToPyObject};
    use std::collections::HashSet;

    #[test]
    fn test_new() {
        Python::with_gil(|py| {
            let ob = PyTuple::new(py, &[1, 2, 3]);
            assert_eq!(3, ob.len());
            let ob: &PyAny = ob.into();
            assert_eq!((1, 2, 3), ob.extract().unwrap());

            let mut map = HashSet::new();
            map.insert(1);
            map.insert(2);
            PyTuple::new(py, &map);
        });
    }

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(3, tuple.len());
            let ob: &PyAny = tuple.into();
            assert_eq!((1, 2, 3), ob.extract().unwrap());
        });
    }

    #[test]
    fn test_slice() {
        Python::with_gil(|py| {
            let tup = PyTuple::new(py, &[2, 3, 5, 7]);
            let slice = tup.get_slice(1, 3);
            assert_eq!(2, slice.len());
            let slice = tup.get_slice(1, 7);
            assert_eq!(3, slice.len());
        });
    }

    #[test]
    fn test_iter() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(3, tuple.len());
            let mut iter = tuple.iter();

            assert_eq!(iter.size_hint(), (3, Some(3)));

            assert_eq!(1, iter.next().unwrap().extract().unwrap());
            assert_eq!(iter.size_hint(), (2, Some(2)));

            assert_eq!(2, iter.next().unwrap().extract().unwrap());
            assert_eq!(iter.size_hint(), (1, Some(1)));

            assert_eq!(3, iter.next().unwrap().extract().unwrap());
            assert_eq!(iter.size_hint(), (0, Some(0)));
        });
    }

    #[test]
    fn test_into_iter() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(3, tuple.len());

            for (i, item) in tuple.iter().enumerate() {
                assert_eq!(i + 1, item.extract().unwrap());
            }
        });
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_as_slice() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();

            let slice = tuple.as_slice();
            assert_eq!(3, slice.len());
            assert_eq!(1, slice[0].extract().unwrap());
            assert_eq!(2, slice[1].extract().unwrap());
            assert_eq!(3, slice[2].extract().unwrap());
        });
    }

    #[test]
    fn test_tuple_lengths_up_to_12() {
        Python::with_gil(|py| {
            let t0 = (0,).to_object(py);
            let t1 = (0, 1).to_object(py);
            let t2 = (0, 1, 2).to_object(py);
            let t3 = (0, 1, 2, 3).to_object(py);
            let t4 = (0, 1, 2, 3, 4).to_object(py);
            let t5 = (0, 1, 2, 3, 4, 5).to_object(py);
            let t6 = (0, 1, 2, 3, 4, 5, 6).to_object(py);
            let t7 = (0, 1, 2, 3, 4, 5, 6, 7).to_object(py);
            let t8 = (0, 1, 2, 3, 4, 5, 6, 7, 8).to_object(py);
            let t9 = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9).to_object(py);
            let t10 = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10).to_object(py);
            let t11 = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11).to_object(py);

            assert_eq!(t0.extract::<(i32,)>(py).unwrap(), (0,));
            assert_eq!(t1.extract::<(i32, i32)>(py).unwrap(), (0, 1,));
            assert_eq!(t2.extract::<(i32, i32, i32)>(py).unwrap(), (0, 1, 2,));
            assert_eq!(
                t3.extract::<(i32, i32, i32, i32,)>(py).unwrap(),
                (0, 1, 2, 3,)
            );
            assert_eq!(
                t4.extract::<(i32, i32, i32, i32, i32,)>(py).unwrap(),
                (0, 1, 2, 3, 4,)
            );
            assert_eq!(
                t5.extract::<(i32, i32, i32, i32, i32, i32,)>(py).unwrap(),
                (0, 1, 2, 3, 4, 5,)
            );
            assert_eq!(
                t6.extract::<(i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6,)
            );
            assert_eq!(
                t7.extract::<(i32, i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7,)
            );
            assert_eq!(
                t8.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8,)
            );
            assert_eq!(
                t9.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8, 9,)
            );
            assert_eq!(
                t10.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,)
            );
            assert_eq!(
                t11.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,)
            );
        })
    }

    #[test]
    fn test_tuple_get_item_invalid_index() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            let obj = tuple.get_item(5);
            assert!(obj.is_err());
            assert_eq!(
                obj.unwrap_err().to_string(),
                "IndexError: tuple index out of range"
            );
        });
    }

    #[test]
    fn test_tuple_get_item_sanity() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            let obj = tuple.get_item(0);
            assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    #[test]
    fn test_tuple_get_item_unchecked_sanity() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            let obj = unsafe { tuple.get_item_unchecked(0) };
            assert_eq!(obj.extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_tuple_index_trait() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(1, tuple[0].extract::<i32>().unwrap());
            assert_eq!(2, tuple[1].extract::<i32>().unwrap());
            assert_eq!(3, tuple[2].extract::<i32>().unwrap());
        });
    }

    #[test]
    #[should_panic]
    fn test_tuple_index_trait_panic() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            let _ = &tuple[7];
        });
    }

    #[test]
    fn test_tuple_index_trait_ranges() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(vec![2, 3], tuple[1..3].extract::<Vec<i32>>().unwrap());
            assert_eq!(
                Vec::<i32>::new(),
                tuple[3..3].extract::<Vec<i32>>().unwrap()
            );
            assert_eq!(vec![2, 3], tuple[1..].extract::<Vec<i32>>().unwrap());
            assert_eq!(Vec::<i32>::new(), tuple[3..].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![1, 2, 3], tuple[..].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![2, 3], tuple[1..=2].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![1, 2], tuple[..2].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![1, 2], tuple[..=1].extract::<Vec<i32>>().unwrap());
        })
    }

    #[test]
    #[should_panic = "range start index 5 out of range for tuple of length 3"]
    fn test_tuple_index_trait_range_panic_start() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            tuple[5..10].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    #[should_panic = "range end index 10 out of range for tuple of length 3"]
    fn test_tuple_index_trait_range_panic_end() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            tuple[1..10].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    #[should_panic = "slice index starts at 2 but ends at 1"]
    fn test_tuple_index_trait_range_panic_wrong_order() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            #[allow(clippy::reversed_empty_ranges)]
            tuple[2..1].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    #[should_panic = "range start index 8 out of range for tuple of length 3"]
    fn test_tuple_index_trait_range_from_panic() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            tuple[8..].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    fn test_tuple_contains() {
        Python::with_gil(|py| {
            let ob = (1, 1, 2, 3, 5, 8).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(6, tuple.len());

            let bad_needle = 7i32.to_object(py);
            assert!(!tuple.contains(&bad_needle).unwrap());

            let good_needle = 8i32.to_object(py);
            assert!(tuple.contains(&good_needle).unwrap());

            let type_coerced_needle = 8f32.to_object(py);
            assert!(tuple.contains(&type_coerced_needle).unwrap());
        });
    }

    #[test]
    fn test_tuple_index() {
        Python::with_gil(|py| {
            let ob = (1, 1, 2, 3, 5, 8).to_object(py);
            let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert_eq!(0, tuple.index(1i32).unwrap());
            assert_eq!(2, tuple.index(2i32).unwrap());
            assert_eq!(3, tuple.index(3i32).unwrap());
            assert_eq!(4, tuple.index(5i32).unwrap());
            assert_eq!(5, tuple.index(8i32).unwrap());
            assert!(tuple.index(42i32).is_err());
        });
    }
}
