// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::slice;

use ffi::{self, Py_ssize_t};
use err::{PyErr, PyResult};
use instance::{Py, PyObjectWithToken};
use object::PyObject;
use objects::PyObjectRef;
use python::{Python, ToPyPointer, IntoPyPointer, PyDowncastFrom};
use conversion::{FromPyObject, ToPyObject, IntoPyTuple, IntoPyObject};
use super::exc;

/// Represents a Python `tuple` object.
pub struct PyTuple(PyObject);

pyobject_convert!(PyTuple);
pyobject_nativetype!(PyTuple, PyTuple_Type, PyTuple_Check);


impl PyTuple {

    /// Construct a new tuple with the given elements.
    pub fn new<T: ToPyObject>(py: Python, elements: &[T]) -> Py<PyTuple> {
        unsafe {
            let len = elements.len();
            let ptr = ffi::PyTuple_New(len as Py_ssize_t);
            for (i, e) in elements.iter().enumerate() {
                ffi::PyTuple_SetItem(ptr, i as Py_ssize_t, e.to_object(py).into_ptr());
            }
            Py::from_owned_ptr_or_panic(ptr)
        }
    }

    /// Retrieves the empty tuple.
    pub fn empty(_py: Python) -> Py<PyTuple> {
        unsafe {
            Py::from_owned_ptr_or_panic(ffi::PyTuple_New(0))
        }
    }

    /// Gets the length of the tuple.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe {
            // non-negative Py_ssize_t should always fit into Rust uint
            ffi::PyTuple_GET_SIZE(self.as_ptr()) as usize
        }
    }

    /// Gets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_item(&self, index: usize) -> &PyObjectRef {
        // TODO: reconsider whether we should panic
        // It's quite inconsistent that this method takes `Python` when `len()` does not.
        assert!(index < self.len());
        unsafe {
            self.py().cast_from_borrowed_ptr(
                ffi::PyTuple_GET_ITEM(self.as_ptr(), index as Py_ssize_t))
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[PyObject] {
        // This is safe because PyObject has the same memory layout as *mut ffi::PyObject,
        // and because tuples are immutable.
        // (We don't even need a Python token, thanks to immutability)
        unsafe {
            let ptr = self.as_ptr() as *mut ffi::PyTupleObject;
            PyObject::borrow_from_owned_ptr_slice(
                slice::from_raw_parts(
                    (*ptr).ob_item.as_ptr(), self.len()
                ))
        }
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<PyObject> {
        self.as_slice().iter()
    }
}

impl<'a> IntoPyTuple for &'a PyTuple {
    fn into_tuple(self, _py: Python) -> Py<PyTuple> {
        self.into()
    }
}

impl IntoPyTuple for Py<PyTuple> {
    fn into_tuple(self, _py: Python) -> Py<PyTuple> {
        self
    }
}

impl<'a> IntoPyTuple for &'a str {
    fn into_tuple(self, py: Python) -> Py<PyTuple> {
        unsafe {
            let ptr = ffi::PyTuple_New(1);
            ffi::PyTuple_SetItem(ptr, 0, self.into_object(py).into_ptr());
            Py::from_owned_ptr_or_panic(ptr)
        }
    }
}

fn wrong_tuple_length(py: Python, t: &PyTuple, expected_length: usize) -> PyErr {
    let msg = format!("Expected tuple of length {}, but got tuple of length {}.",
                      expected_length, t.len());
    PyErr::new_lazy_init(
        py.get_type::<exc::ValueError>(), Some(msg.into_object(py)))
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

    impl <$($T: IntoPyObject),+> IntoPyTuple for ($($T,)+) {
        fn into_tuple(self, py: Python) -> Py<PyTuple> {
            unsafe {
                let ptr = ffi::PyTuple_New($length);
                $(ffi::PyTuple_SetItem(ptr, $n, self.$n.into_object(py).into_ptr());)+;
                Py::from_owned_ptr_or_panic(ptr)
            }
        }
    }

    impl<'s, $($T: FromPyObject<'s>),+> FromPyObject<'s> for ($($T,)+) {
        fn extract(obj: &'s PyObjectRef) -> PyResult<Self>
        {
            let t = PyTuple::downcast_from(obj)?;
            let slice = t.as_slice();
            if t.len() == $length {
                Ok((
                    $( try!(slice[$n].extract::<$T>(obj.py())), )+
                ))
            } else {
                Err(wrong_tuple_length(obj.py(), t, $length))
            }
        }
    }
});

tuple_conversion!(1, (ref0, 0, A));
tuple_conversion!(2, (ref0, 0, A), (ref1, 1, B));
tuple_conversion!(3, (ref0, 0, A), (ref1, 1, B), (ref2, 2, C));
tuple_conversion!(4, (ref0, 0, A), (ref1, 1, B), (ref2, 2, C), (ref3, 3, D));
tuple_conversion!(5, (ref0, 0, A), (ref1, 1, B), (ref2, 2, C), (ref3, 3, D),
  (ref4, 4, E));
tuple_conversion!(6, (ref0, 0, A), (ref1, 1, B), (ref2, 2, C), (ref3, 3, D),
  (ref4, 4, E), (ref5, 5, F));
tuple_conversion!(7, (ref0, 0, A), (ref1, 1, B), (ref2, 2, C), (ref3, 3, D),
  (ref4, 4, E), (ref5, 5, F), (ref6, 6, G));
tuple_conversion!(8, (ref0, 0, A), (ref1, 1, B), (ref2, 2, C), (ref3, 3, D),
  (ref4, 4, E), (ref5, 5, F), (ref6, 6, G), (ref7, 7, H));
tuple_conversion!(9, (ref0, 0, A), (ref1, 1, B), (ref2, 2, C), (ref3, 3, D),
  (ref4, 4, E), (ref5, 5, F), (ref6, 6, G), (ref7, 7, H), (ref8, 8, I));

// Empty tuple:

/// An empty struct that represents the empty argument list.
/// Corresponds to the empty tuple `()` in Python.
///
/// # Example
/// ```
/// let gil = pyo3::Python::acquire_gil();
/// let py = gil.python();
/// let os = py.import("os").unwrap();
/// let pid = os.call("get_pid", pyo3::NoArgs, None);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct NoArgs;

/// Converts `NoArgs` to an empty Python tuple.
impl ToPyObject for NoArgs {

    fn to_object(&self, py: Python) -> PyObject {
        PyTuple::empty(py).into()
    }
}

impl IntoPyObject for NoArgs
{
    fn into_object(self, py: Python) -> PyObject {
        PyTuple::empty(py).into()
    }
}

/// Converts `NoArgs` to an empty Python tuple.
impl IntoPyTuple for NoArgs {

    fn into_tuple(self, py: Python) -> Py<PyTuple> {
        PyTuple::empty(py)
    }
}

/// Converts `()` to an empty Python tuple.
impl IntoPyTuple for () {

    fn into_tuple(self, py: Python) -> Py<PyTuple> {
        PyTuple::empty(py)
    }
}


/// Returns `Ok(NoArgs)` if the input is an empty Python tuple.
/// Otherwise, returns an error.
pyobject_extract!(py, obj to NoArgs => {
    let t = PyTuple::downcast_from(obj)?;
    if t.len() == 0 {
        Ok(NoArgs)
    } else {
        Err(wrong_tuple_length(obj.py(), t, 0))
    }
});


#[cfg(test)]
mod test {
    use PyTuple;
    use instance::AsPyRef;
    use python::{Python, PyDowncastFrom};
    use conversion::ToPyObject;
    use objects::PyObjectRef;
    use objectprotocol::ObjectProtocol;

    #[test]
    fn test_new() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyob = PyTuple::new(py, &[1, 2, 3]);
        let ob = pyob.as_ref(py);
        assert_eq!(3, ob.len());
        let ob: &PyObjectRef = ob.into();
        assert_eq!((1, 2, 3), ob.extract().unwrap());
    }

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = (1, 2, 3).to_object(py);
        let tuple = PyTuple::downcast_from(ob.as_ref(py)).unwrap();
        assert_eq!(3, tuple.len());
        let ob: &PyObjectRef = tuple.into();
        assert_eq!((1, 2, 3), ob.extract().unwrap());
    }
}
