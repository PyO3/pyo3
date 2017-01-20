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

use std::mem;
use ffi;
use python::{Python, PythonObject, ToPythonPointer, PyClone, PyDrop};
use conversion::{FromPyObject, ToPyObject};
use objects::{PyObject, PyList, PyTuple, PyIterator};
use ffi::Py_ssize_t;
use err;
use err::{PyErr, PyResult, result_from_owned_ptr, result_cast_from_owned_ptr};
use buffer;

/// Represents a reference to a python object supporting the sequence protocol.
pub struct PySequence(PyObject);

pyobject_newtype!(PySequence, PySequence_Check);

impl PySequence {
    /// Returns the number of objects in sequence. This is equivalent to Python `len()`.
    #[inline]
    pub fn len(&self, py: Python) -> PyResult<isize> {
        let v = unsafe { ffi::PySequence_Size(self.0.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(py))
        } else {
            Ok(v as isize)
        }
    }

    /// Return the concatenation of o1 and o2. Equivalent to python `o1 + o2`
    #[inline]
    pub fn concat(&self, py: Python, other: &PySequence) -> PyResult<PyObject> {
        unsafe {
            err::result_from_owned_ptr(py, ffi::PySequence_Concat(self.as_ptr(), other.as_ptr()))
        }
    }

    /// Return the result of repeating sequence object o count times.
    /// Equivalent to python `o * count`
    /// NB: Python accepts negative counts; it returns an empty Sequence.
    #[inline]
    pub fn repeat(&self, py: Python, count: isize) -> PyResult<PyObject> {
        unsafe {
            err::result_from_owned_ptr(py, ffi::PySequence_Repeat(self.as_ptr(), count as Py_ssize_t))
        }
    }

    /// Return the concatenation of o1 and o2 on success. Equivalent to python `o1 += o2`
    #[inline]
    pub fn in_place_concat(&self, py: Python, other: &PySequence) -> PyResult<PyObject> {
        unsafe {
            result_from_owned_ptr(py, ffi::PySequence_InPlaceConcat(self.as_ptr(), other.as_ptr()))
        }
    }

    /// Return the result of repeating sequence object o count times.
    /// Equivalent to python `o *= count`
    /// NB: Python accepts negative counts; it empties the Sequence.
    #[inline]
    pub fn in_place_repeat(&self, py: Python, count: isize) -> PyResult<PyObject> {
        unsafe {
            result_from_owned_ptr(py,
                ffi::PySequence_InPlaceRepeat(self.as_ptr(), count as Py_ssize_t))
        }
    }

    /// Return the ith element of the Sequence. Equivalent to python `o[index]`
    #[inline]
    pub fn get_item(&self, py: Python, index: isize) -> PyResult<PyObject> {
        unsafe {
            result_from_owned_ptr(py,
                ffi::PySequence_GetItem(self.as_ptr(), index as Py_ssize_t))
        }
    }

    /// Return the slice of sequence object o between begin and end.
    /// This is the equivalent of the Python expression `o[begin:end]`
    #[inline]
    pub fn get_slice(&self, py: Python, begin : isize, end : isize) -> PyResult<PyObject> {
        unsafe {
            result_from_owned_ptr(py,
                ffi::PySequence_GetSlice(self.as_ptr(), begin as Py_ssize_t, end as Py_ssize_t))
        }
    }

    /// Assign object v to the ith element of o.
    /// Equivalent to Python statement `o[i] = v`
    #[inline]
    pub fn set_item(&self, py: Python, i: isize, v: &PyObject) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(py,
                ffi::PySequence_SetItem(self.as_ptr(), i as Py_ssize_t, v.as_ptr()))
        }
    }

    /// Delete the ith element of object o.
    /// Python statement `del o[i]`
    #[inline]
    pub fn del_item(&self, py: Python, i: isize) -> PyResult<()> {
        unsafe { 
            err::error_on_minusone(py,
                ffi::PySequence_DelItem(self.as_ptr(), i as Py_ssize_t))
        }
    }

    /// Assign the sequence object v to the slice in sequence object o from i1 to i2.
    /// This is the equivalent of the Python statement `o[i1:i2] = v`
    #[inline]
    pub fn set_slice(&self, py: Python, i1: isize, i2: isize, v: &PyObject) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(py,
                ffi::PySequence_SetSlice(self.as_ptr(), i1 as Py_ssize_t, i2 as Py_ssize_t, v.as_ptr()))
        }
    }

    /// Delete the slice in sequence object o from i1 to i2.
    /// equivalent of the Python statement `del o[i1:i2]`
    #[inline]
    pub fn del_slice(&self, py: Python, i1: isize, i2: isize) -> PyResult<()> {
        unsafe { 
            err::error_on_minusone(py,
                ffi::PySequence_DelSlice(self.as_ptr(), i1 as Py_ssize_t, i2 as Py_ssize_t))
        }
    }

    /// Return the number of occurrences of value in o, that is, return the number of keys for
    /// which `o[key] == value`
    #[inline]
    pub fn count<V>(&self, py: Python, value: V) -> PyResult<usize>
        where V: ToPyObject
    {
        let r = value.with_borrowed_ptr(py, |ptr| unsafe {
            ffi::PySequence_Count(self.as_ptr(), ptr)
        });
        if r == -1 {
            Err(PyErr::fetch(py))
        } else {
            Ok(r as usize)
        }
    }

   /// Determine if o contains value. this is equivalent to the Python expression `value in o`
    #[inline]
    pub fn contains<V>(&self, py: Python, value: V) -> PyResult<bool>
        where V: ToPyObject
    {
        let r = value.with_borrowed_ptr(py, |ptr| unsafe {
            ffi::PySequence_Contains(self.as_ptr(), ptr)
        });
        match r {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(PyErr::fetch(py))
        }
    }

    /// Return the first index i for which o[i] == value.
    /// This is equivalent to the Python expression `o.index(value)`
    #[inline]
    pub fn index<V>(&self, py: Python, value: V) -> PyResult<usize>
        where V: ToPyObject
    {
        let r = value.with_borrowed_ptr(py, |ptr| unsafe {
            ffi::PySequence_Index(self.as_ptr(), ptr)
        });
        if r == -1 {
            Err(PyErr::fetch(py))
        } else {
            Ok(r as usize)
        }
    }

    /// Return a fresh list based on the Sequence.
    #[inline]
    pub fn list(&self, py: Python) -> PyResult<PyList> {
        unsafe {
            result_cast_from_owned_ptr(py, ffi::PySequence_List(self.as_ptr()))
        }
    }

    /// Return a fresh tuple based on the Sequence.
    #[inline]
    pub fn tuple(&self, py: Python) -> PyResult<PyTuple> {
        unsafe {
            result_cast_from_owned_ptr(py, ffi::PySequence_Tuple(self.as_ptr()))
        }
    }

    #[inline]
    pub fn iter<'p>(&self, py: Python<'p>) -> PyResult<PyIterator<'p>> {
        use objectprotocol::ObjectProtocol;
        self.as_object().iter(py)
    }
}

#[cfg(not(feature="nightly"))]
impl <'source, T> FromPyObject<'source> for Vec<T>
    where for<'a> T: FromPyObject<'a>
{
    fn extract(py: Python, obj: &'source PyObject) -> PyResult<Self> {
        extract_sequence(py, obj)
    }
}

#[cfg(feature="nightly")]
impl <'source, T> FromPyObject<'source> for Vec<T>
    where for<'a> T: FromPyObject<'a>
{
    default fn extract(py: Python, obj: &'source PyObject) -> PyResult<Self> {
        extract_sequence(py, obj)
    }
}

#[cfg(feature="nightly")]
impl <'source, T> FromPyObject<'source> for Vec<T>
    where for<'a> T: FromPyObject<'a> + buffer::Element + Copy
{
    fn extract(py: Python, obj: &'source PyObject) -> PyResult<Self> {
        // first try buffer protocol
        if let Ok(buf) = buffer::PyBuffer::get(py, obj) {
            if buf.dimensions() == 1 {
                if let Ok(v) = buf.to_vec::<T>(py) {
                    buf.release_ref(py);
                    return Ok(v);
                }
            }
            buf.release_ref(py);
        }
        // fall back to sequence protocol
        extract_sequence(py, obj)
    }
}

fn extract_sequence<T>(py: Python, obj: &PyObject) -> PyResult<Vec<T>>
    where for<'a> T: FromPyObject<'a>
{
    let seq = try!(obj.cast_as::<PySequence>(py));
    let mut v = Vec::new();
    for item in try!(seq.iter(py)) {
        let item = try!(item);
        v.push(try!(T::extract(py, &item)));
        item.release_ref(py);
    }
    Ok(v)
}

#[cfg(test)]
mod test {
    use std;
    use python::{Python, PythonObject};
    use conversion::ToPyObject;
    use objects::{PySequence, PyList, PyTuple, PyIterator};

    #[test]
    fn test_numbers_are_not_sequences() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = 42i32;
        assert!(v.to_py_object(py).into_object().cast_into::<PySequence>(py).is_err());
    }

    #[test]
    fn test_strings_are_sequences() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "London Calling";
        assert!(v.to_py_object(py).into_object().cast_into::<PySequence>(py).is_ok());
    }
    #[test]
    fn test_seq_empty() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert_eq!(0, seq.len(py).unwrap());

        let needle = 7i32.to_py_object(py).into_object();
        assert_eq!(false, seq.contains(py, &needle).unwrap());
    }

    #[test]
    fn test_seq_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert_eq!(6, seq.len(py).unwrap());

        let bad_needle = 7i32.to_py_object(py).into_object();
        assert_eq!(false, seq.contains(py, &bad_needle).unwrap());

        let good_needle = 8i32.to_py_object(py).into_object();
        assert_eq!(true, seq.contains(py, &good_needle).unwrap());

        let type_coerced_needle = 8f32.to_py_object(py).into_object();
        assert_eq!(true, seq.contains(py, &type_coerced_needle).unwrap());
    }

    #[test]
    fn test_seq_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert_eq!(1, seq.get_item(py, 0).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(1, seq.get_item(py, 1).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(2, seq.get_item(py, 2).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(3, seq.get_item(py, 3).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(5, seq.get_item(py, 4).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(8, seq.get_item(py, 5).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(8, seq.get_item(py, -1).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(5, seq.get_item(py, -2).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(3, seq.get_item(py, -3).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(2, seq.get_item(py, -4).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(1, seq.get_item(py, -5).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.get_item(py, 10).is_err());
    }

    // fn test_get_slice() {}
    // fn test_set_slice() {}
    // fn test_del_slice() {}

    #[test]
    fn test_seq_del_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert!(seq.del_item(py, 10).is_err());
        assert_eq!(1, seq.get_item(py, 0).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(py, 0).is_ok());
        assert_eq!(1, seq.get_item(py, 0).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(py, 0).is_ok());
        assert_eq!(2, seq.get_item(py, 0).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(py, 0).is_ok());
        assert_eq!(3, seq.get_item(py, 0).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(py, 0).is_ok());
        assert_eq!(5, seq.get_item(py, 0).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(py, 0).is_ok());
        assert_eq!(8, seq.get_item(py, 0).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(py, 0).is_ok());
        assert_eq!(0, seq.len(py).unwrap());
        assert!(seq.del_item(py, 0).is_err());
    }

    #[test]
    fn test_seq_index() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert_eq!(0, seq.index(py, 1i32).unwrap());
        assert_eq!(2, seq.index(py, 2i32).unwrap());
        assert_eq!(3, seq.index(py, 3i32).unwrap());
        assert_eq!(4, seq.index(py, 5i32).unwrap());
        assert_eq!(5, seq.index(py, 8i32).unwrap());
        assert!(seq.index(py, 42i32).is_err());
    }

    #[test]
    fn test_seq_count() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert_eq!(2, seq.count(py, 1i32).unwrap());
        assert_eq!(1, seq.count(py, 2i32).unwrap());
        assert_eq!(1, seq.count(py, 3i32).unwrap());
        assert_eq!(1, seq.count(py, 5i32).unwrap());
        assert_eq!(1, seq.count(py, 8i32).unwrap());
        assert_eq!(0, seq.count(py, 42i32).unwrap());
    }

    #[test]
    fn test_seq_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        let mut idx = 0;
        for el in seq.iter(py).unwrap() {
            assert_eq!(v[idx], el.unwrap().extract::<i32>(py).unwrap());
            idx += 1;
        }
        assert_eq!(idx, v.len());
    }

    #[test]
    fn test_seq_strings() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec!["It", "was", "the", "worst", "of", "times"];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();

        let bad_needle = "blurst".to_py_object(py);
        assert_eq!(false, seq.contains(py, bad_needle).unwrap());

        let good_needle = "worst".to_py_object(py);
        assert_eq!(true, seq.contains(py, good_needle).unwrap());
    }

    #[test]
    fn test_seq_concat() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 2, 3];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        let concat_seq = seq.concat(py, &seq).unwrap().cast_into::<PySequence>(py).unwrap();
        assert_eq!(6, concat_seq.len(py).unwrap());
        let concat_v : Vec<i32> = vec![1, 2, 3, 1, 2, 3];
        for (el, cc) in seq.iter(py).unwrap().zip(concat_v) {
            assert_eq!(cc, el.unwrap().extract::<i32>(py).unwrap());
        }
    }

    #[test]
    fn test_seq_concat_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "string";
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        let concat_seq = seq.concat(py, &seq).unwrap().cast_into::<PySequence>(py).unwrap();
        assert_eq!(12, concat_seq.len(py).unwrap());
        /*let concat_v = "stringstring".to_owned();
        for (el, cc) in seq.iter(py).unwrap().zip(concat_v.chars()) {
            assert_eq!(cc, el.unwrap().extract::<char>(py).unwrap()); //TODO: extract::<char>() is not implemented
        }*/
    }

    #[test]
    fn test_seq_repeat() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec!["foo", "bar"];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        let repeat_seq = seq.repeat(py, 3).unwrap().cast_into::<PySequence>(py).unwrap();
        assert_eq!(6, repeat_seq.len(py).unwrap());
        let repeated = vec!["foo", "bar", "foo", "bar", "foo", "bar"];
        for (el, rpt) in seq.iter(py).unwrap().zip(repeated.iter()) {
            assert_eq!(*rpt, el.unwrap().extract::<String>(py).unwrap());
        }
    }

    #[test]
    fn test_list_coercion() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec!["foo", "bar"];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert!(seq.list(py).is_ok());
    }

    #[test]
    fn test_strings_coerce_to_lists() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "foo";
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert!(seq.list(py).is_ok());
    }

    #[test]
    fn test_tuple_coercion() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = ("foo", "bar");
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert!(seq.tuple(py).is_ok());
    }

    #[test]
    fn test_lists_coerce_to_tuples() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec!["foo", "bar"];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert!(seq.tuple(py).is_ok());
    }

    #[test]
    fn test_extract_tuple_to_vec() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = py.eval("(1, 2)", None, None).unwrap().extract(py).unwrap();
        assert!(v == [1, 2]);
    }

    #[test]
    fn test_extract_range_to_vec() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = py.eval("range(1, 5)", None, None).unwrap().extract(py).unwrap();
        assert!(v == [1, 2, 3, 4]);
    }
    
    #[test]
    fn test_extract_bytearray_to_vec() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<u8> = py.eval("bytearray(b'abc')", None, None).unwrap().extract(py).unwrap();
        assert!(v == b"abc");
    }
}
