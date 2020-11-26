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

pyobject_native_var_type!(PyTuple, ffi::PyTuple_Type, ffi::PyTuple_Check);

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
            // non-negative Py_ssize_t should always fit into Rust uint
            ffi::PyTuple_Size(self.as_ptr()) as usize
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
            self.py()
                .from_borrowed_ptr(ffi::PyTuple_GetItem(self.as_ptr(), index as Py_ssize_t))
        }
    }

    /// Returns `self` as a slice of objects.
    ///
    /// Not available when compiled with Py_LIMITED_API.
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

tuple_conversion!(1, (ref0, 0, A));
tuple_conversion!(2, (ref0, 0, A), (ref1, 1, B));
tuple_conversion!(3, (ref0, 0, A), (ref1, 1, B), (ref2, 2, C));
tuple_conversion!(4, (ref0, 0, A), (ref1, 1, B), (ref2, 2, C), (ref3, 3, D));
tuple_conversion!(
    5,
    (ref0, 0, A),
    (ref1, 1, B),
    (ref2, 2, C),
    (ref3, 3, D),
    (ref4, 4, E)
);
tuple_conversion!(
    6,
    (ref0, 0, A),
    (ref1, 1, B),
    (ref2, 2, C),
    (ref3, 3, D),
    (ref4, 4, E),
    (ref5, 5, F)
);
tuple_conversion!(
    7,
    (ref0, 0, A),
    (ref1, 1, B),
    (ref2, 2, C),
    (ref3, 3, D),
    (ref4, 4, E),
    (ref5, 5, F),
    (ref6, 6, G)
);
tuple_conversion!(
    8,
    (ref0, 0, A),
    (ref1, 1, B),
    (ref2, 2, C),
    (ref3, 3, D),
    (ref4, 4, E),
    (ref5, 5, F),
    (ref6, 6, G),
    (ref7, 7, H)
);
tuple_conversion!(
    9,
    (ref0, 0, A),
    (ref1, 1, B),
    (ref2, 2, C),
    (ref3, 3, D),
    (ref4, 4, E),
    (ref5, 5, F),
    (ref6, 6, G),
    (ref7, 7, H),
    (ref8, 8, I)
);

#[cfg(test)]
mod test {
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
        assert_eq!(1, iter.next().unwrap().extract().unwrap());
        assert_eq!(2, iter.next().unwrap().extract().unwrap());
        assert_eq!(3, iter.next().unwrap().extract().unwrap());
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
}
