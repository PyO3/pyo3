// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use buffer;
use ffi::{self, Py_ssize_t};
use err::{self, PyErr, PyResult, PyDowncastError};
use object::PyObject;
use instance::PyObjectWithToken;
use python::ToPyPointer;
use conversion::{FromPyObject, ToBorrowedObject, PyTryFrom};
use objects::{PyObjectRef, PyList, PyTuple};
use objectprotocol::ObjectProtocol;


/// Represents a reference to a python object supporting the sequence protocol.
pub struct PySequence(PyObject);

pyobject_nativetype!(PySequence);
pyobject_downcast!(PySequence, PySequence_Check);


#[cfg_attr(feature = "cargo-clippy", allow(len_without_is_empty))]
impl PySequence {
    /// Returns the number of objects in sequence. This is equivalent to Python `len()`.
    #[inline]
    pub fn len(&self) -> PyResult<isize> {
        let v = unsafe { ffi::PySequence_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(v as isize)
        }
    }

    /// Return the concatenation of o1 and o2. Equivalent to python `o1 + o2`
    #[inline]
    pub fn concat(&self, other: &PySequence) -> PyResult<&PySequence> {
        unsafe {
            let ptr = self.py().from_owned_ptr_or_err::<PyObjectRef>(
                ffi::PySequence_Concat(self.as_ptr(), other.as_ptr()))?;
            Ok(std::mem::transmute(ptr))
        }
    }

    /// Return the result of repeating sequence object o count times.
    /// Equivalent to python `o * count`
    /// NB: Python accepts negative counts; it returns an empty Sequence.
    #[inline]
    pub fn repeat(&self, count: isize) -> PyResult<&PySequence> {
        unsafe {
            let ptr = self.py().from_owned_ptr_or_err::<PyObjectRef>(
                ffi::PySequence_Repeat(self.as_ptr(), count as Py_ssize_t))?;
            Ok(std::mem::transmute(ptr))
        }
    }

    /// Concatenate of o1 and o2 on success. Equivalent to python `o1 += o2`
    #[inline]
    pub fn in_place_concat(&self, other: &PySequence) -> PyResult<()> {
        unsafe {
            let ptr = ffi::PySequence_InPlaceConcat(self.as_ptr(), other.as_ptr());
            if ptr.is_null() {
                Err(PyErr::fetch(self.py()))
            } else {
                Ok(())
            }
        }
    }

    /// Repeate sequence object o count times and store in self.
    /// Equivalent to python `o *= count`
    /// NB: Python accepts negative counts; it empties the Sequence.
    #[inline]
    pub fn in_place_repeat(&self, count: isize) -> PyResult<()> {
        unsafe {
            let ptr = ffi::PySequence_InPlaceRepeat(self.as_ptr(), count as Py_ssize_t);
            if ptr.is_null() {
                Err(PyErr::fetch(self.py()))
            } else {
                Ok(())
            }
        }
    }

    /// Return the ith element of the Sequence. Equivalent to python `o[index]`
    #[inline]
    pub fn get_item(&self, index: isize) -> PyResult<&PyObjectRef> {
        unsafe {
            self.py().from_owned_ptr_or_err(
                ffi::PySequence_GetItem(self.as_ptr(), index as Py_ssize_t))
        }
    }

    /// Return the slice of sequence object o between begin and end.
    /// This is the equivalent of the Python expression `o[begin:end]`
    #[inline]
    pub fn get_slice(&self, begin: isize, end: isize) -> PyResult<&PyObjectRef> {
        unsafe {
            self.py().from_owned_ptr_or_err(
                ffi::PySequence_GetSlice(
                    self.as_ptr(), begin as Py_ssize_t, end as Py_ssize_t))
        }
    }

    /// Assign object v to the ith element of o.
    /// Equivalent to Python statement `o[i] = v`
    #[inline]
    pub fn set_item<I>(&self, i: isize, item: I) -> PyResult<()> where I: ToBorrowedObject {
        unsafe {
            item.with_borrowed_ptr(self.py(), |item| {
                err::error_on_minusone(
                    self.py(), ffi::PySequence_SetItem(self.as_ptr(), i as Py_ssize_t, item))
            })
        }
    }

    /// Delete the ith element of object o.
    /// Python statement `del o[i]`
    #[inline]
    pub fn del_item(&self, i: isize) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(
                self.py(), ffi::PySequence_DelItem(self.as_ptr(), i as Py_ssize_t))
        }
    }

    /// Assign the sequence object v to the slice in sequence object o from i1 to i2.
    /// This is the equivalent of the Python statement `o[i1:i2] = v`
    #[inline]
    pub fn set_slice(&self, i1: isize, i2: isize, v: &PyObjectRef) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(
                self.py(), ffi::PySequence_SetSlice(
                    self.as_ptr(), i1 as Py_ssize_t, i2 as Py_ssize_t, v.as_ptr()))
        }
    }

    /// Delete the slice in sequence object o from i1 to i2.
    /// equivalent of the Python statement `del o[i1:i2]`
    #[inline]
    pub fn del_slice(&self, i1: isize, i2: isize) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PySequence_DelSlice(self.as_ptr(), i1 as Py_ssize_t, i2 as Py_ssize_t))
        }
    }

    /// Return the number of occurrences of value in o, that is, return the number of keys for
    /// which `o[key] == value`
    #[inline]
    pub fn count<V>(&self, value: V) -> PyResult<usize> where V: ToBorrowedObject
    {
        let r = value.with_borrowed_ptr(self.py(), |ptr| unsafe {
            ffi::PySequence_Count(self.as_ptr(), ptr)
        });
        if r == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(r as usize)
        }
    }

    /// Determine if o contains value. this is equivalent to the Python expression `value in o`
    #[inline]
    pub fn contains<V>(&self, value: V) -> PyResult<bool> where V: ToBorrowedObject
    {
        let r = value.with_borrowed_ptr(self.py(), |ptr| unsafe {
            ffi::PySequence_Contains(self.as_ptr(), ptr)
        });
        match r {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(PyErr::fetch(self.py()))
        }
    }

    /// Return the first index i for which o[i] == value.
    /// This is equivalent to the Python expression `o.index(value)`
    #[inline]
    pub fn index<V>(&self, value: V) -> PyResult<usize> where V: ToBorrowedObject
    {
        let r = value.with_borrowed_ptr(self.py(), |ptr| unsafe {
            ffi::PySequence_Index(self.as_ptr(), ptr)
        });
        if r == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(r as usize)
        }
    }

    /// Return a fresh list based on the Sequence.
    #[inline]
    pub fn list(&self) -> PyResult<&PyList> {
        unsafe {
            self.py().from_owned_ptr_or_err(ffi::PySequence_List(self.as_ptr()))
        }
    }

    /// Return a fresh tuple based on the Sequence.
    #[inline]
    pub fn tuple(&self) -> PyResult<&PyTuple> {
        unsafe {
            self.py().from_owned_ptr_or_err(ffi::PySequence_Tuple(self.as_ptr()))
        }
    }
}


impl<'a, T> FromPyObject<'a> for Vec<T> where T: FromPyObject<'a>
{
    default fn extract(obj: &'a PyObjectRef) -> PyResult<Self> {
        extract_sequence(obj)
    }
}

impl <'source, T> FromPyObject<'source> for Vec<T>
    where for<'a> T: FromPyObject<'a> + buffer::Element + Copy
{
    fn extract(obj: &'source PyObjectRef) -> PyResult<Self> {
        // first try buffer protocol
        if let Ok(buf) = buffer::PyBuffer::get(obj.py(), obj) {
            if buf.dimensions() == 1 {
                if let Ok(v) = buf.to_vec::<T>(obj.py()) {
                    buf.release(obj.py());
                    return Ok(v);
                }
            }
            buf.release(obj.py());
        }
        // fall back to sequence protocol
        extract_sequence(obj)
    }
}

fn extract_sequence<'s, T>(obj: &'s PyObjectRef) -> PyResult<Vec<T>> where T: FromPyObject<'s>
{
    let seq = PySequence::try_from(obj)?;
    let mut v = Vec::with_capacity(seq.len().unwrap_or(0) as usize);
    for item in seq.iter()? {
        v.push(item?.extract::<T>()?);
    }
    Ok(v)
}

impl PyTryFrom for PySequence
{
    type Error = PyDowncastError;

    fn try_from(value: &PyObjectRef) -> Result<&PySequence, Self::Error> {
        unsafe {
            if ffi::PySequence_Check(value.as_ptr()) != 0 {
                let ptr = value as *const _ as *mut u8 as *mut PySequence;
                Ok(&*ptr)
            } else {
                Err(PyDowncastError)
            }
        }
    }

    fn try_from_exact(value: &PyObjectRef) -> Result<&PySequence, Self::Error> {
        PySequence::try_from(value)
    }

    fn try_from_mut(value: &PyObjectRef) -> Result<&mut PySequence, Self::Error> {
        unsafe {
            if ffi::PySequence_Check(value.as_ptr()) != 0 {
                let ptr = value as *const _ as *mut u8 as *mut PySequence;
                Ok(&mut *ptr)
            } else {
                Err(PyDowncastError)
            }
        }
    }

    fn try_from_mut_exact(value: &PyObjectRef) -> Result<&mut PySequence, Self::Error> {
        PySequence::try_from_mut(value)
    }
}


#[cfg(test)]
mod test {
    use instance::AsPyRef;
    use python::Python;
    use conversion::{PyTryFrom, ToPyObject};
    use objects::PySequence;
    use objectprotocol::ObjectProtocol;

    #[test]
    fn test_numbers_are_not_sequences() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = 42i32;
        assert!(PySequence::try_from(v.to_object(py).as_ref(py)).is_err());
    }

    #[test]
    fn test_strings_are_sequences() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "London Calling";
        assert!(PySequence::try_from(v.to_object(py).as_ref(py)).is_ok());
    }
    #[test]
    fn test_seq_empty() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert_eq!(0, seq.len().unwrap());

        let needle = 7i32.to_object(py);
        assert_eq!(false, seq.contains(&needle).unwrap());
    }

    #[test]
    fn test_seq_contains() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert_eq!(6, seq.len().unwrap());

        let bad_needle = 7i32.to_object(py);
        assert_eq!(false, seq.contains(&bad_needle).unwrap());

        let good_needle = 8i32.to_object(py);
        assert_eq!(true, seq.contains(&good_needle).unwrap());

        let type_coerced_needle = 8f32.to_object(py);
        assert_eq!(true, seq.contains(&type_coerced_needle).unwrap());
    }

    #[test]
    fn test_seq_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert_eq!(1, seq.get_item(0).unwrap().extract::<i32>().unwrap());
        assert_eq!(1, seq.get_item(1).unwrap().extract::<i32>().unwrap());
        assert_eq!(2, seq.get_item(2).unwrap().extract::<i32>().unwrap());
        assert_eq!(3, seq.get_item(3).unwrap().extract::<i32>().unwrap());
        assert_eq!(5, seq.get_item(4).unwrap().extract::<i32>().unwrap());
        assert_eq!(8, seq.get_item(5).unwrap().extract::<i32>().unwrap());
        assert_eq!(8, seq.get_item(-1).unwrap().extract::<i32>().unwrap());
        assert_eq!(5, seq.get_item(-2).unwrap().extract::<i32>().unwrap());
        assert_eq!(3, seq.get_item(-3).unwrap().extract::<i32>().unwrap());
        assert_eq!(2, seq.get_item(-4).unwrap().extract::<i32>().unwrap());
        assert_eq!(1, seq.get_item(-5).unwrap().extract::<i32>().unwrap());
        assert!(seq.get_item(10).is_err());
    }

    // fn test_get_slice() {}
    // fn test_set_slice() {}
    // fn test_del_slice() {}

    #[test]
    fn test_seq_del_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert!(seq.del_item(10).is_err());
        assert_eq!(1, seq.get_item(0).unwrap().extract::<i32>().unwrap());
        assert!(seq.del_item(0).is_ok());
        assert_eq!(1, seq.get_item(0).unwrap().extract::<i32>().unwrap());
        assert!(seq.del_item(0).is_ok());
        assert_eq!(2, seq.get_item(0).unwrap().extract::<i32>().unwrap());
        assert!(seq.del_item(0).is_ok());
        assert_eq!(3, seq.get_item(0).unwrap().extract::<i32>().unwrap());
        assert!(seq.del_item(0).is_ok());
        assert_eq!(5, seq.get_item(0).unwrap().extract::<i32>().unwrap());
        assert!(seq.del_item(0).is_ok());
        assert_eq!(8, seq.get_item(0).unwrap().extract::<i32>().unwrap());
        assert!(seq.del_item(0).is_ok());
        assert_eq!(0, seq.len().unwrap());
        assert!(seq.del_item(0).is_err());
    }

    #[test]
    fn test_seq_set_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = vec![1, 2];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert_eq!(2, seq.get_item(1).unwrap().extract::<i32>().unwrap());
        assert!(seq.set_item(1, 10).is_ok());
        assert_eq!(10, seq.get_item(1).unwrap().extract::<i32>().unwrap());
    }

    #[test]
    fn test_seq_set_item_refcnt() {
        let cnt;
        {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let v: Vec<i32> = vec![1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            let none = py.None();
            cnt = none.get_refcnt();
            assert!(seq.set_item(1, none).is_ok());
            assert!(seq.get_item(1).unwrap().is_none());
        }
        {
            let gil = Python::acquire_gil();
            let py = gil.python();
            assert_eq!(cnt, py.None().get_refcnt());
        }
    }

    #[test]
    fn test_seq_index() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert_eq!(0, seq.index(1i32).unwrap());
        assert_eq!(2, seq.index(2i32).unwrap());
        assert_eq!(3, seq.index(3i32).unwrap());
        assert_eq!(4, seq.index(5i32).unwrap());
        assert_eq!(5, seq.index(8i32).unwrap());
        assert!(seq.index(42i32).is_err());
    }

    #[test]
    fn test_seq_count() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert_eq!(2, seq.count(1i32).unwrap());
        assert_eq!(1, seq.count(2i32).unwrap());
        assert_eq!(1, seq.count(3i32).unwrap());
        assert_eq!(1, seq.count(5i32).unwrap());
        assert_eq!(1, seq.count(8i32).unwrap());
        assert_eq!(0, seq.count(42i32).unwrap());
    }

    #[test]
    fn test_seq_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        let mut idx = 0;
        for el in seq.iter().unwrap() {
            assert_eq!(v[idx], el.unwrap().extract::<i32>().unwrap());
            idx += 1;
        }
        assert_eq!(idx, v.len());
    }

    #[test]
    fn test_seq_strings() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec!["It", "was", "the", "worst", "of", "times"];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();

        let bad_needle = "blurst".to_object(py);
        assert_eq!(false, seq.contains(bad_needle).unwrap());

        let good_needle = "worst".to_object(py);
        assert_eq!(true, seq.contains(good_needle).unwrap());
    }

    #[test]
    fn test_seq_concat() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v : Vec<i32> = vec![1, 2, 3];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        let concat_seq = seq.concat(&seq).unwrap();
        assert_eq!(6, concat_seq.len().unwrap());
        let concat_v : Vec<i32> = vec![1, 2, 3, 1, 2, 3];
        for (el, cc) in concat_seq.iter().unwrap().zip(concat_v) {
            assert_eq!(cc, el.unwrap().extract::<i32>().unwrap());
        }
    }

    #[test]
    fn test_seq_concat_string() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "string";
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        let concat_seq = seq.concat(&seq).unwrap();
        assert_eq!(12, concat_seq.len().unwrap());
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
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        let repeat_seq = seq.repeat(3).unwrap();
        assert_eq!(6, repeat_seq.len().unwrap());
        let repeated = vec!["foo", "bar", "foo", "bar", "foo", "bar"];
        for (el, rpt) in repeat_seq.iter().unwrap().zip(repeated.iter()) {
            assert_eq!(*rpt, el.unwrap().extract::<String>().unwrap());
        }
    }

    #[test]
    fn test_list_coercion() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec!["foo", "bar"];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert!(seq.list().is_ok());
    }

    #[test]
    fn test_strings_coerce_to_lists() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "foo";
        let ob = v.to_object(py);
        let seq = PySequence::try_from(ob.as_ref(py)).unwrap();
        assert!(seq.list().is_ok());
    }

    #[test]
    fn test_tuple_coercion() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = ("foo", "bar");
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert!(seq.tuple().is_ok());
    }

    #[test]
    fn test_lists_coerce_to_tuples() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec!["foo", "bar"];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert!(seq.tuple().is_ok());
    }

    #[test]
    fn test_extract_tuple_to_vec() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = py.eval("(1, 2)", None, None).unwrap().extract().unwrap();
        assert!(v == [1, 2]);
    }

    #[test]
    fn test_extract_range_to_vec() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = py.eval("range(1, 5)", None, None).unwrap().extract().unwrap();
        assert!(v == [1, 2, 3, 4]);
    }

    #[test]
    fn test_extract_bytearray_to_vec() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<u8> = py.eval("bytearray(b'abc')", None, None).unwrap().extract().unwrap();
        assert!(v == b"abc");
    }
}
