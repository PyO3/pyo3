// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::ffi::{self, Py_ssize_t};
use crate::{
    exceptions, AsPyPointer, FromPyObject, IntoPy, IntoPyPointer, Py, PyAny, PyErr, PyNativeType,
    PyObject, PyResult, PyTryFrom, Python, ToPyObject,
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

    /// Takes a slice of the tuple pointed from `low` to `high` and returns it as a new tuple.
    pub fn slice(&self, low: isize, high: isize) -> &PyTuple {
        unsafe {
            self.py()
                .from_owned_ptr(ffi::PyTuple_GetSlice(self.as_ptr(), low, high))
        }
    }

    /// Takes a slice of the tuple from `low` to the end and returns it as a new tuple.
    pub fn split_from(&self, low: isize) -> &PyTuple {
        unsafe {
            let ptr = ffi::PyTuple_GetSlice(self.as_ptr(), low, self.len() as Py_ssize_t);
            self.py().from_owned_ptr(ptr)
        }
    }

    /// Gets the tuple item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_item(&self, index: usize) -> &PyAny {
        assert!(index < self.len());
        unsafe {
            #[cfg(not(any(Py_LIMITED_API, PyPy)))]
            let item = ffi::PyTuple_GET_ITEM(self.as_ptr(), index as Py_ssize_t);
            #[cfg(any(Py_LIMITED_API, PyPy))]
            let item = ffi::PyTuple_GetItem(self.as_ptr(), index as Py_ssize_t);

            self.py().from_borrowed_ptr(item)
        }
    }

    /// Returns `self` as a slice of objects.
    #[cfg(not(Py_LIMITED_API))]
    #[cfg_attr(docsrs, doc(cfg(not(Py_LIMITED_API))))]
    pub fn as_slice(&self) -> &[&PyAny] {
        // This is safe because &PyAny has the same memory layout as *mut ffi::PyObject,
        // and because tuples are immutable.
        unsafe {
            let ptr = self.as_ptr() as *mut ffi::PyTupleObject;
            let slice = std::slice::from_raw_parts((*ptr).ob_item.as_ptr(), self.len());
            &*(slice as *const [*mut ffi::PyObject] as *const [&PyAny])
        }
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
            let item = self.tuple.get_item(self.index);
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
                Ok((
                    $(t.get_item($n).extract::<$T>()?,)+
                ))
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
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = PyTuple::new(py, &[1, 2, 3]);
        assert_eq!(3, ob.len());
        let ob: &PyAny = ob.into();
        assert_eq!((1, 2, 3), ob.extract().unwrap());

        let mut map = HashSet::new();
        map.insert(1);
        map.insert(2);
        PyTuple::new(py, &map);
    }

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = (1, 2, 3).to_object(py);
        let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(3, tuple.len());
        let ob: &PyAny = tuple.into();
        assert_eq!((1, 2, 3), ob.extract().unwrap());
    }

    #[test]
    fn test_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
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
    }

    #[test]
    fn test_into_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = (1, 2, 3).to_object(py);
        let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
        assert_eq!(3, tuple.len());

        for (i, item) in tuple.iter().enumerate() {
            assert_eq!(i + 1, item.extract().unwrap());
        }
    }

    #[test]
    #[cfg(not(Py_LIMITED_API))]
    fn test_as_slice() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = (1, 2, 3).to_object(py);
        let tuple = <PyTuple as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();

        let slice = tuple.as_slice();
        assert_eq!(3, slice.len());
        assert_eq!(1, slice[0].extract().unwrap());
        assert_eq!(2, slice[1].extract().unwrap());
        assert_eq!(3, slice[2].extract().unwrap());
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
}
