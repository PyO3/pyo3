// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::conversion::FromPy;
use crate::err::{PyErr, PyResult};
use crate::exceptions;
use crate::ffi::{self, Py_ssize_t};
use crate::instance::{AsPyRef, Py, PyNativeType};
use crate::object::PyObject;
use crate::types::PyAny;
use crate::AsPyPointer;
use crate::IntoPyPointer;
use crate::Python;
use crate::{FromPyObject, IntoPy, IntoPyObject, PyTryFrom, ToPyObject};
use std::slice;

/// Represents a Python `tuple` object.
#[repr(transparent)]
pub struct PyTuple(PyObject);

pyobject_native_type!(PyTuple, ffi::PyTuple_Type, ffi::PyTuple_Check);

impl PyTuple {
    /// Construct a new tuple with the given elements.
    pub fn new<'p, T, U>(
        py: Python<'p>,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> &'p PyTuple
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

    /// Retrieves the empty tuple.
    pub fn empty<'p>(py: Python<'p>) -> &'p PyTuple {
        unsafe { py.from_owned_ptr(ffi::PyTuple_New(0)) }
    }

    /// Gets the length of the tuple.
    pub fn len(&self) -> usize {
        unsafe {
            // non-negative Py_ssize_t should always fit into Rust uint
            ffi::PyTuple_GET_SIZE(self.as_ptr()) as usize
        }
    }

    /// Check if tuple is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Take a slice of the tuple pointed to by p from low to high and return it as a new tuple.
    pub fn slice(&self, low: isize, high: isize) -> Py<PyTuple> {
        unsafe { Py::from_owned_ptr_or_panic(ffi::PyTuple_GetSlice(self.as_ptr(), low, high)) }
    }

    /// Take a slice of the tuple pointed to by p from low and return it as a new tuple.
    pub fn split_from(&self, low: isize) -> Py<PyTuple> {
        unsafe {
            let ptr =
                ffi::PyTuple_GetSlice(self.as_ptr(), low, ffi::PyTuple_GET_SIZE(self.as_ptr()));
            Py::from_owned_ptr_or_panic(ptr)
        }
    }

    /// Gets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_item(&self, index: usize) -> &PyAny {
        // TODO: reconsider whether we should panic
        // It's quite inconsistent that this method takes `Python` when `len()` does not.
        assert!(index < self.len());
        unsafe {
            self.py()
                .from_borrowed_ptr(ffi::PyTuple_GET_ITEM(self.as_ptr(), index as Py_ssize_t))
        }
    }

    pub fn as_slice(&self) -> &[PyObject] {
        // This is safe because PyObject has the same memory layout as *mut ffi::PyObject,
        // and because tuples are immutable.
        // (We don't even need a Python token, thanks to immutability)
        unsafe {
            let ptr = self.as_ptr() as *mut ffi::PyTupleObject;
            let slice = slice::from_raw_parts((*ptr).ob_item.as_ptr(), self.len());
            &*(slice as *const [*mut ffi::PyObject] as *const [PyObject])
        }
    }

    /// Returns an iterator over the tuple items.
    pub fn iter(&self) -> PyTupleIterator {
        PyTupleIterator {
            py: self.py(),
            slice: self.as_slice(),
            index: 0,
        }
    }
}

/// Used by `PyTuple::iter()`.
pub struct PyTupleIterator<'a> {
    py: Python<'a>,
    slice: &'a [PyObject],
    index: usize,
}

impl<'a> Iterator for PyTupleIterator<'a> {
    type Item = &'a PyAny;

    #[inline]
    fn next(&mut self) -> Option<&'a PyAny> {
        if self.index < self.slice.len() {
            let item = self.slice[self.index].as_ref(self.py);
            self.index += 1;
            Some(item.into())
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

impl<'a> FromPy<&'a PyTuple> for Py<PyTuple> {
    fn from_py(tuple: &'a PyTuple, _py: Python) -> Py<PyTuple> {
        unsafe { Py::from_borrowed_ptr(tuple.as_ptr()) }
    }
}

fn wrong_tuple_length(t: &PyTuple, expected_length: usize) -> PyErr {
    let msg = format!(
        "Expected tuple of length {}, but got tuple of length {}.",
        expected_length,
        t.len()
    );
    exceptions::ValueError::py_err(msg)
}

macro_rules! tuple_conversion ({$length:expr,$(($refN:ident, $n:tt, $T:ident)),+} => {
    impl <$($T: ToPyObject),+> ToPyObject for ($($T,)+) {
        fn to_object(&self, py: Python) -> PyObject {
            unsafe {
                let ptr = ffi::PyTuple_New($length);
                $(ffi::PyTuple_SetItem(ptr, $n, self.$n.to_object(py).into_ptr());)+;
                PyObject::from_owned_ptr_or_panic(py, ptr)
            }
        }
    }
    impl <$($T: IntoPyObject),+> IntoPyObject for ($($T,)+) {
        fn into_object(self, py: Python) -> PyObject {
            unsafe {
                let ptr = ffi::PyTuple_New($length);
                $(ffi::PyTuple_SetItem(ptr, $n, self.$n.into_object(py).into_ptr());)+;
                PyObject::from_owned_ptr_or_panic(py, ptr)
            }
        }
    }

    impl <$($T: IntoPyObject),+> IntoPy<Py<PyTuple>> for ($($T,)+) {
        fn into_py(self, py: Python) -> Py<PyTuple> {
            unsafe {
                let ptr = ffi::PyTuple_New($length);
                $(ffi::PyTuple_SetItem(ptr, $n, self.$n.into_object(py).into_ptr());)+;
                Py::from_owned_ptr_or_panic(ptr)
            }
        }
    }

    impl<'s, $($T: FromPyObject<'s>),+> FromPyObject<'s> for ($($T,)+) {
        fn extract(obj: &'s PyAny) -> PyResult<Self>
        {
            let t = <PyTuple as PyTryFrom>::try_from(obj)?;
            let slice = t.as_slice();
            if t.len() == $length {
                Ok((
                    $(slice[$n].extract::<$T>(obj.py())?,)+
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
    use crate::instance::AsPyRef;
    use crate::objectprotocol::ObjectProtocol;
    use crate::types::PyAny;
    use crate::types::PyTuple;
    use crate::Python;
    use crate::{PyTryFrom, ToPyObject};
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
}
