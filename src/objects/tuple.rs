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

use python::{Python, PythonObject, ToPythonPointer};
use err::{self, PyResult, PyErr};
use super::object::PyObject;
use super::exc;
use ffi::{self, Py_ssize_t};
use conversion::{ToPyObject, ExtractPyObject};

/// Represents a Python tuple object.
pub struct PyTuple<'p>(PyObject<'p>);

pyobject_newtype!(PyTuple, PyTuple_Check, PyTuple_Type);

impl <'p> PyTuple<'p> {
    /// Construct a new tuple with the given elements.
    pub fn new(py: Python<'p>, elements: &[PyObject<'p>]) -> PyTuple<'p> {
        unsafe {
            let len = elements.len();
            let ptr = ffi::PyTuple_New(len as Py_ssize_t);
            let t = err::result_cast_from_owned_ptr::<PyTuple>(py, ptr).unwrap();
            for (i, e) in elements.iter().enumerate() {
                ffi::PyTuple_SetItem(ptr, i as Py_ssize_t, e.clone().steal_ptr());
            }
            t
        }
    }

    /// Retrieves the empty tuple.
    pub fn empty(py: Python<'p>) -> PyTuple<'p> {
        unsafe {
            err::result_cast_from_owned_ptr::<PyTuple>(py, ffi::PyTuple_New(0)).unwrap()
        }
    }

    /// Gets the length of the tuple.
    #[inline]
    pub fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust uint
        unsafe {
            ffi::PyTuple_Size(self.as_ptr()) as usize
        }
    }

    /// Gets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_item(&self, index: usize) -> PyObject<'p> {
        assert!(index < self.len());
        unsafe {
            PyObject::from_borrowed_ptr(self.python(), ffi::PyTuple_GetItem(self.as_ptr(), index as Py_ssize_t))
        }
    }

    /* Disabled for now; we might want to change the PyObject memory layout for
       compatiblity with stable Rust.
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
    */
}

impl <'p> IntoIterator for PyTuple<'p> {
    type Item = PyObject<'p>;
    type IntoIter = PyTupleIterator<'p>;

    #[inline]
    fn into_iter(self) -> PyTupleIterator<'p> {
        PyTupleIterator { index: 0, len: self.len(), tuple: self }
    }
}

impl <'a, 'p> IntoIterator for &'a PyTuple<'p> {
    type Item = PyObject<'p>;
    type IntoIter = PyTupleIterator<'p>;

    #[inline]
    fn into_iter(self) -> PyTupleIterator<'p> {
        PyTupleIterator { index: 0, len: self.len(), tuple: self.clone() }
    }
}

/// Used by `impl IntoIterator for &PyTuple`.
pub struct PyTupleIterator<'p> {
    tuple: PyTuple<'p>,
    index: usize,
    len: usize
}

impl <'p> Iterator for PyTupleIterator<'p> {
    type Item = PyObject<'p>;

    #[inline]
    fn next(&mut self) -> Option<PyObject<'p>> {
        if self.index < self.len {
            let item = self.tuple.get_item(self.index);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl <'p> ExactSizeIterator for PyTupleIterator<'p> {
    #[inline]
    fn len(&self) -> usize {
        return self.len;
    }
}

fn wrong_tuple_length<'p>(t: &PyTuple<'p>, expected_length: usize) -> PyErr<'p> {
    let py = t.python();
    let msg = format!("Expected tuple of length {}, but got tuple of length {}.", expected_length, t.len());
    PyErr::new_lazy_init(py.get_type::<exc::ValueError>(), Some(msg.to_py_object(py).into_object()))
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

    /* TODO: reimplement this without slice matching
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
    */
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

/// An empty struct that represents the empty argument list.
/// Corresponds to the empty tuple `()` in Python.
///
/// # Example
/// ```
/// let gil_guard = cpython::Python::acquire_gil();
/// let py = gil_guard.python();
/// let os = py.import("os").unwrap();
/// let pid = os.call("get_pid", cpython::NoArgs, None);
/// ```
pub struct NoArgs;

/// Converts `NoArgs` to an empty Python tuple.
impl <'p> ToPyObject<'p> for NoArgs {
    type ObjectType = PyTuple<'p>;

    fn to_py_object(&self, py: Python<'p>) -> PyTuple<'p> {
        PyTuple::empty(py)
    }
}

/// Returns `Ok(NoArgs)` if the input is an empty Python tuple.
/// Otherwise, returns an error.
extract!(obj to NoArgs => {
    let t = try!(obj.cast_as::<PyTuple>());
    if t.len() == 0 {
        Ok(NoArgs)
    } else {
        Err(wrong_tuple_length(t, 0))
    }
});

