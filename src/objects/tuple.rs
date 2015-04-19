// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std;
use python::{Python, PythonObject, ToPythonPointer};
use err::{self, PyResult, PyErr};
use super::object::PyObject;
use super::exc;
use ffi::{self, Py_ssize_t};
use conversion::{ToPyObject, FromPyObject};

pyobject_newtype!(PyTuple, PyTuple_Check, PyTuple_Type);

impl <'p> PyTuple<'p> {
    pub fn new(py: Python<'p>, elements: &[PyObject<'p>]) -> PyTuple<'p> {
        unsafe {
            let len = elements.len();
            let ptr = ffi::PyTuple_New(len as Py_ssize_t);
            let t = err::result_from_owned_ptr(py, ptr).unwrap().unchecked_cast_into::<PyTuple>();
            for (i, e) in elements.iter().enumerate() {
                ffi::PyTuple_SET_ITEM(ptr, i as Py_ssize_t, e.clone().steal_ptr());
            }
            t
        }
    }

    /// Retrieves the empty tuple.
    pub fn empty(py: Python<'p>) -> PyTuple<'p> {
        unsafe {
            err::result_from_owned_ptr(py, ffi::PyTuple_New(0)).unwrap().unchecked_cast_into::<PyTuple>()
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust uint
        unsafe {
            ffi::PyTuple_GET_SIZE(self.as_ptr()) as usize
        }
    }
    
    #[inline]
    pub fn as_slice<'a>(&'a self) -> &'a [PyObject<'p>] {
        // This is safe because PyObject has the same memory layout as *mut ffi::PyObject,
        // and because tuples are immutable.
        unsafe {
            let ptr = self.as_ptr() as *mut ffi::PyTupleObject;
            std::mem::transmute(std::raw::Slice {
                data: (*ptr).ob_item.as_ptr(),
                len: self.len()
            })
        }
    }
    
    #[inline]
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, PyObject<'p>> {
        self.as_slice().iter()
    }
}

impl<'p> std::ops::Index<usize> for PyTuple<'p> {
    type Output = PyObject<'p>;

    #[inline]
    fn index<'a>(&'a self, index: usize) -> &'a PyObject<'p> {
        // use as_slice() to use the normal Rust bounds checking when indexing
        &self.as_slice()[index]
    }
}

impl<'p, 'a> std::iter::IntoIterator for &'a PyTuple<'p> {
    type Item = &'a PyObject<'p>;
    type IntoIter = std::slice::Iter<'a, PyObject<'p>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}


fn wrong_tuple_length<'p>(t: &PyTuple<'p>, expected_length: usize) -> PyErr<'p> {
    let py = t.python();
    let msg = format!("Expected tuple of length {}, but got tuple of length {}.", expected_length, t.len());
    PyErr::new_lazy_init(py.get_type::<exc::ValueError>(), Some(msg.to_py_object(py)))
}

macro_rules! id (($a:expr) => ($a));

macro_rules! tuple_conversion ({$length:expr,$(($refN:ident, $n:tt, $T:ident)),+} => (
    impl <'p, $($T: ToPyObject<'p>),+> ToPyObject<'p> for ($($T,)+) {
        type ObjectType = PyTuple<'p>;

        fn to_py_object(&self, py: Python<'p>) -> PyTuple<'p> {
            PyTuple::new(py, &[
                $(id!(self.$n.to_py_object(py)).into_object(),)+
            ])
        }

        fn into_py_object(self, py: Python<'p>) -> PyTuple<'p> {
            PyTuple::new(py, &[
                $(id!(self.$n.into_py_object(py)).into_object(),)+
            ])
        }
    }

    impl <'p, 's, $($T: FromPyObject<'p, 's>),+> FromPyObject<'p, 's> for ($($T,)+) {
        fn from_py_object(s : &'s PyObject<'p>) -> PyResult<'p, ($($T,)+)> {
            let t = try!(s.cast_as::<PyTuple>());
            match t.as_slice() {
                [$(ref $refN,)+] => Ok((
                    $(try!($refN.extract::<$T>()),)+
                )),
                _ => Err(wrong_tuple_length(t, 2))
            }
        }
    }
));

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

pub struct NoArgs;

impl <'p> ToPyObject<'p> for NoArgs {
    type ObjectType = PyTuple<'p>;

    fn to_py_object(&self, py: Python<'p>) -> PyTuple<'p> {
        PyTuple::empty(py)
    }
}

impl <'p, 's> FromPyObject<'p, 's> for NoArgs {
    fn from_py_object(s : &'s PyObject<'p>) -> PyResult<'p, NoArgs> {
        let t = try!(s.cast_as::<PyTuple>());
        if t.len() == 0 {
            Ok(NoArgs)
        } else {
            Err(wrong_tuple_length(t, 0))
        }
    }
}

