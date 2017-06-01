// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::slice;

use ffi::{self, Py_ssize_t};
use err::{PyErr, PyResult};
use pointers::{Ptr, PyPtr};
use python::{Python, ToPythonPointer, IntoPythonPointer};
use conversion::{FromPyObject, ToPyObject, ToPyTuple, IntoPyObject};
use objects::PyObject;
use native::PyNativeObject;
use token::PythonObjectWithGilToken;
use super::exc;

/// Represents a Python tuple object.
pub struct PyTuple<'p>(Ptr<'p>);
pub struct PyTuplePtr(PyPtr);
pyobject_nativetype!(PyTuple, PyTuple_Check, PyTuple_Type, PyTuplePtr);


impl<'p> PyTuple<'p> {

    /// Construct a new tuple with the given elements.
    pub fn new<T: ToPyObject>(py: Python<'p>, elements: &[T]) -> PyTuple<'p> {
        unsafe {
            let len = elements.len();
            let ptr = ffi::PyTuple_New(len as Py_ssize_t);
            for (i, e) in elements.iter().enumerate() {
                ffi::PyTuple_SetItem(ptr, i as Py_ssize_t, e.to_object(py).into_ptr());
            }
            PyTuple(Ptr::from_owned_ptr_or_panic(py, ptr))
        }
    }

    /// Construct a new tuple with the given raw pointer
    pub unsafe fn from_borrowed_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> PyTuple<'p> {
        PyTuple(Ptr::from_borrowed_ptr(py, ptr))
    }

    /// Retrieves the empty tuple.
    pub fn empty(py: Python<'p>) -> PyTuple<'p> {
        unsafe {
            PyTuple(Ptr::from_owned_ptr_or_panic(py, ffi::PyTuple_New(0)))
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
    pub fn get_item(&'p self, index: usize) -> PyObject<'p> {
        // TODO: reconsider whether we should panic
        // It's quite inconsistent that this method takes `Python` when `len()` does not.
        assert!(index < self.len());
        unsafe {
            PyObject::from_borrowed_ptr(
                self.gil(), ffi::PyTuple_GET_ITEM(self.as_ptr(), index as Py_ssize_t))
        }
    }

    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [PyObject] {
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

    //#[inline]
    //pub fn iter(&self) -> slice::Iter<PyObject> {
    //self.as_slice(py).iter()
    //}
}

impl<'a> ToPyTuple for PyTuple<'a> {
    fn to_py_tuple<'p>(&self, py: Python<'p>) -> PyTuple<'p> {
        unsafe { PyTuple(Ptr::from_borrowed_ptr(py, self.0.as_ptr())) }
    }
}

impl<'a> ToPyTuple for &'a str {
    fn to_py_tuple<'p>(&self, py: Python<'p>) -> PyTuple<'p> {
        PyTuple::new(py, &[py_coerce_expr!(self.to_object(py))])
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
        fn to_object<'p>(&self, py: Python<'p>) -> PyObject<'p> {
            PyTuple::new(py, &[
                $(py_coerce_expr!(self.$n.to_object(py)),)+
            ]).as_object()
        }
    }

    impl <$($T: ToPyObject),+> ToPyTuple for ($($T,)+) {
        fn to_py_tuple<'p>(&self, py: Python<'p>) -> PyTuple<'p> {
            PyTuple::new(py, &[
                $(py_coerce_expr!(self.$n.to_object(py)),)+
            ])
        }
    }

    impl<'s, $($T: FromPyObject<'s>),+> FromPyObject<'s> for ($($T,)+) {
        fn extract(obj: &'s PyObject<'s>) -> PyResult<Self>
        {
            let t = try!(obj.cast_as::<PyTuple>());
            let slice = t.as_slice();
            if t.len() == $length {
                Ok((
                    $( try!(slice[$n].extract::<$T>()), )+
                ))
            } else {
                Err(wrong_tuple_length(obj.gil(), t, $length))
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
/// use pyo3::ObjectProtocol;
/// let gil = pyo3::Python::acquire_gil();
/// let py = gil.python();
/// let os = py.import("os").unwrap();
/// let pid = os.call("get_pid", pyo3::NoArgs, None);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct NoArgs;

/// Converts `NoArgs` to an empty Python tuple.
impl ToPyObject for NoArgs {

    fn to_object<'p>(&self, py: Python<'p>) -> PyObject<'p> {
        PyTuple::empty(py).as_object()
    }
}

/// Converts `NoArgs` to an empty Python tuple.
impl ToPyTuple for NoArgs {

    fn to_py_tuple<'p>(&self, py: Python<'p>) -> PyTuple<'p> {
        PyTuple::empty(py)
    }
}

/// Converts `()` to an empty Python tuple.
impl ToPyTuple for () {

    fn to_py_tuple<'p>(&self, py: Python<'p>) -> PyTuple<'p> {
        PyTuple::empty(py)
    }
}


/// Returns `Ok(NoArgs)` if the input is an empty Python tuple.
/// Otherwise, returns an error.
pyobject_extract!(obj to NoArgs => {
    let t = try!(obj.cast_as::<PyTuple>());
    if t.len() == 0 {
        Ok(NoArgs)
    } else {
        Err(wrong_tuple_length(obj.gil(), t, 0))
    }
});


#[cfg(test)]
mod test {
    use PyTuple;
    use python::{Python, PyDowncastInto};
    use conversion::IntoPyObject;
    use conversion::ToPyObject;

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let tuple = PyTuple::downcast_into(py, (1, 2, 3).to_object(py).into_object(py)).unwrap();
        assert_eq!(3, tuple.len());
        assert_eq!((1, 2, 3), tuple.into_object(py).as_object(py).extract().unwrap());
    }
}

