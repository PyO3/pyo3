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
use python::{Python, PythonObject, ToPythonPointer, PyClone};
use conversion::ToPyObject;
use objects::{PyObject, PyList, PyTuple};
use ffi::Py_ssize_t;
use err;
use err::{PyErr, PyResult, result_from_owned_ptr, result_cast_from_owned_ptr};

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
    pub fn concat(&self, other: &PySequence, py: Python) -> PyResult<PyObject> {
        unsafe {
            err::result_from_owned_ptr(py, ffi::PySequence_Concat(self.as_ptr(), other.as_ptr()))
        }
    }

    /// Return the result of repeating sequence object o count times.
    /// Equivalent to python `o * count`
    /// NB: Python accepts negative counts; it returns an empty Sequence.
    #[inline]
    pub fn repeat(&self, count: isize, py: Python) -> PyResult<PyObject> {
        unsafe {
            err::result_from_owned_ptr(py, ffi::PySequence_Repeat(self.as_ptr(), count as Py_ssize_t))
        }
    }

    /// Return the concatenation of o1 and o2 on success. Equivalent to python `o1 += o2`
    #[inline]
    pub fn in_place_concat(&self, other: &PySequence, py: Python) -> PyResult<PyObject> {
        unsafe {
            result_from_owned_ptr(py, ffi::PySequence_InPlaceConcat(self.as_ptr(), other.as_ptr()))
        }
    }

    /// Return the result of repeating sequence object o count times.
    /// Equivalent to python `o *= count`
    /// NB: Python accepts negative counts; it empties the Sequence.
    #[inline]
    pub fn in_place_repeat(&self, count: isize, py: Python) -> PyResult<PyObject> {
        unsafe {
            result_from_owned_ptr(py,
                ffi::PySequence_InPlaceRepeat(self.as_ptr(), count as Py_ssize_t))
        }
    }

    /// Return the ith element of the Sequence. Equivalent to python `o[index]`
    #[inline]
    pub fn get_item(&self, index: isize, py: Python) -> PyResult<PyObject> {
        unsafe {
            result_from_owned_ptr(py,
                ffi::PySequence_GetItem(self.as_ptr(), index as Py_ssize_t))
        }
    }

    /// Return the slice of sequence object o between begin and end.
    /// This is the equivalent of the Python expression `o[begin:end]`
    #[inline]
    pub fn get_slice(&self, begin : isize, end : isize, py: Python) -> PyResult<PyObject> {
        unsafe {
            result_from_owned_ptr(py,
                ffi::PySequence_GetSlice(self.as_ptr(), begin as Py_ssize_t, end as Py_ssize_t))
        }
    }

    /// Assign object v to the ith element of o.
    /// Equivalent to Python statement `o[i] = v`
    #[inline]
    pub fn set_item(&self, i: isize, v: &PyObject, py: Python) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(py,
                ffi::PySequence_SetItem(self.as_ptr(), i as Py_ssize_t, v.as_ptr()))
        }
    }

    /// Delete the ith element of object o.
    /// Python statement `del o[i]`
    #[inline]
    pub fn del_item(&self, i: isize, py: Python) -> PyResult<()> {
        unsafe { 
            err::error_on_minusone(py,
                ffi::PySequence_DelItem(self.as_ptr(), i as Py_ssize_t))
        }
    }

    /// Assign the sequence object v to the slice in sequence object o from i1 to i2.
    /// This is the equivalent of the Python statement `o[i1:i2] = v`
    #[inline]
    pub fn set_slice(&self, i1: isize, i2: isize, v: &PyObject, py: Python) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(py,
                ffi::PySequence_SetSlice(self.as_ptr(), i1 as Py_ssize_t, i2 as Py_ssize_t, v.as_ptr()))
        }
    }

    /// Delete the slice in sequence object o from i1 to i2.
    /// equivalent of the Python statement `del o[i1:i2]`
    #[inline]
    pub fn del_slice(&self, i1: isize, i2: isize, py: Python) -> PyResult<()> {
        unsafe { 
            err::error_on_minusone(py,
                ffi::PySequence_DelSlice(self.as_ptr(), i1 as Py_ssize_t, i2 as Py_ssize_t))
        }
    }

    /// Return the number of occurrences of value in o, that is, return the number of keys for
    /// which `o[key] == value`
    #[inline]
    pub fn count<V>(&self, value: V, py: Python) -> PyResult<usize>
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
    pub fn contains<V>(&self, value: V, py: Python) -> PyResult<bool>
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
    pub fn index<V>(&self, value: V, py: Python) -> PyResult<usize>
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
    pub fn iter<'p>(&self, py: Python<'p>) -> PySequenceIterator<'p> {
        PySequenceIterator {
            sequence: self.clone_ref(py),
            index: 0,
            py: py
        }
    }
    
    #[inline]
    pub fn into_iter<'p>(self, py: Python<'p>) -> PySequenceIterator<'p> {
        PySequenceIterator {
            sequence: self,
            index: 0,
            py: py
        }
    }
}

pub struct PySequenceIterator<'p> {
    sequence : PySequence,
    index : isize,
    py : Python<'p>
}

impl <'p> Iterator for PySequenceIterator<'p> {
    // TODO: reconsider error reporting; maybe this should be Item = PyResult<PyObject>?
    type Item = PyObject;

    fn next(&mut self) -> Option<PyObject> {
        // can't report any errors in underlying size check so we panic.
        let len = self.sequence.len(self.py).unwrap();
        if self.index < len {
            match self.sequence.get_item(self.index, self.py) {
                Ok(item) => {
                    self.index += 1;
                    Some(item)
                },
                Err(_) => None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use std;
    use python::{Python, PythonObject};
    use conversion::ToPyObject;
    use objects::{PySequence, PyList, PyTuple};

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
        assert_eq!(false, seq.contains(&needle, py).unwrap());
    }

    #[test]
    fn test_seq_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert_eq!(6, seq.len(py).unwrap());

        let bad_needle = 7i32.to_py_object(py).into_object();
        assert_eq!(false, seq.contains(&bad_needle, py).unwrap());

        let good_needle = 8i32.to_py_object(py).into_object();
        assert_eq!(true, seq.contains(&good_needle, py).unwrap());

        let type_coerced_needle = 8f32.to_py_object(py).into_object();
        assert_eq!(true, seq.contains(&type_coerced_needle, py).unwrap());
    }

    #[test]
    fn test_seq_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert_eq!(1, seq.get_item(0, py).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(1, seq.get_item(1, py).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(2, seq.get_item(2, py).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(3, seq.get_item(3, py).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(5, seq.get_item(4, py).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(8, seq.get_item(5, py).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(8, seq.get_item(-1, py).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(5, seq.get_item(-2, py).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(3, seq.get_item(-3, py).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(2, seq.get_item(-4, py).unwrap().extract::<i32>(py).unwrap());
        assert_eq!(1, seq.get_item(-5, py).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.get_item(10, py).is_err());
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
        assert!(seq.del_item(10, py).is_err());
        assert_eq!(1, seq.get_item(0, py).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(0, py).is_ok());
        assert_eq!(1, seq.get_item(0, py).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(0, py).is_ok());
        assert_eq!(2, seq.get_item(0, py).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(0, py).is_ok());
        assert_eq!(3, seq.get_item(0, py).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(0, py).is_ok());
        assert_eq!(5, seq.get_item(0, py).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(0, py).is_ok());
        assert_eq!(8, seq.get_item(0, py).unwrap().extract::<i32>(py).unwrap());
        assert!(seq.del_item(0, py).is_ok());
        assert_eq!(0, seq.len(py).unwrap());
        assert!(seq.del_item(0, py).is_err());
    }

    #[test]
    fn test_seq_index() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert_eq!(0, seq.index(1i32, py).unwrap());
        assert_eq!(2, seq.index(2i32, py).unwrap());
        assert_eq!(3, seq.index(3i32, py).unwrap());
        assert_eq!(4, seq.index(5i32, py).unwrap());
        assert_eq!(5, seq.index(8i32, py).unwrap());
        assert!(seq.index(42i32, py).is_err());
    }

    #[test]
    fn test_seq_count() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        assert_eq!(2, seq.count(1i32, py).unwrap());
        assert_eq!(1, seq.count(2i32, py).unwrap());
        assert_eq!(1, seq.count(3i32, py).unwrap());
        assert_eq!(1, seq.count(5i32, py).unwrap());
        assert_eq!(1, seq.count(8i32, py).unwrap());
        assert_eq!(0, seq.count(42i32, py).unwrap());
    }

/*
    #[test]
    fn test_seq_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        let mut idx = 0;
        for el in seq {
            assert_eq!(v[idx], el.extract::<i32>(py).unwrap());
            idx += 1;
        }
        assert_eq!(idx, v.len());
    }

    #[test]
    fn test_seq_into_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>().unwrap();
        let mut idx = 0;
        for el in seq.into_iter() {
            assert_eq!(v[idx], el.extract::<i32>().unwrap());
            idx += 1;
        }
        assert_eq!(idx, v.len());
    }
*/

    #[test]
    fn test_seq_strings() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec!["It", "was", "the", "worst", "of", "times"];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();

        let bad_needle = "blurst".to_py_object(py);
        assert_eq!(false, seq.contains(bad_needle, py).unwrap());

        let good_needle = "worst".to_py_object(py);
        assert_eq!(true, seq.contains(good_needle, py).unwrap());
    }

    #[test]
    fn test_seq_concat() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 2, 3];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        let concat_seq = seq.concat(&seq, py).unwrap().cast_into::<PySequence>(py).unwrap();
        assert_eq!(6, concat_seq.len(py).unwrap());
        let concat_v : Vec<i32> = vec![1, 2, 3, 1, 2, 3];
        for (el, cc) in seq.into_iter(py).zip(concat_v) {
            assert_eq!(cc, el.extract::<i32>(py).unwrap());
        }
    }

    #[test]
    fn test_seq_concat_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "string";
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        let concat_seq = seq.concat(&seq, py).unwrap().cast_into::<PySequence>(py).unwrap();
        assert_eq!(12, concat_seq.len(py).unwrap());
        /*let concat_v = "stringstring".to_owned();
        for (el, cc) in seq.into_iter(py).zip(concat_v.chars()) {
            assert_eq!(cc, el.extract::<char>(py).unwrap()); TODO: extract::<char>() is not implemented
        }*/
    }

    #[test]
    fn test_seq_repeat() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec!["foo", "bar"];
        let seq = v.to_py_object(py).into_object().cast_into::<PySequence>(py).unwrap();
        let repeat_seq = seq.repeat(3, py).unwrap().cast_into::<PySequence>(py).unwrap();
        assert_eq!(6, repeat_seq.len(py).unwrap());
        let repeated = vec!["foo", "bar", "foo", "bar", "foo", "bar"];
        for (el, rpt) in seq.into_iter(py).zip(repeated.iter()) {
            assert_eq!(*rpt, el.extract::<String>(py).unwrap());
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
}
