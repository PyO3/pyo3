// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::err::{self, PyDowncastError, PyErr, PyResult};
use crate::ffi::{self, Py_ssize_t};
use crate::instance::PyNativeType;
use crate::types::{PyAny, PyList, PyTuple};
use crate::AsPyPointer;
use crate::{FromPyObject, PyTryFrom, ToBorrowedObject};

/// Represents a reference to a Python object supporting the sequence protocol.
#[repr(transparent)]
pub struct PySequence(PyAny);
pyobject_native_type_named!(PySequence);
pyobject_native_type_extract!(PySequence);

impl PySequence {
    /// Returns the number of objects in sequence.
    ///
    /// This is equivalent to the Python expression `len(self)`.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> PyResult<isize> {
        let v = unsafe { ffi::PySequence_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::api_call_failed(self.py()))
        } else {
            Ok(v as isize)
        }
    }

    /// Returns whether the sequence is empty.
    #[inline]
    pub fn is_empty(&self) -> PyResult<bool> {
        self.len().map(|l| l == 0)
    }

    /// Returns the concatenation of `self` and `other`.
    ///
    /// This is equivalent to the Python expression `self + other`.
    #[inline]
    pub fn concat(&self, other: &PySequence) -> PyResult<&PySequence> {
        unsafe {
            let ptr = self
                .py()
                .from_owned_ptr_or_err::<PyAny>(ffi::PySequence_Concat(
                    self.as_ptr(),
                    other.as_ptr(),
                ))?;
            Ok(&*(ptr as *const PyAny as *const PySequence))
        }
    }

    /// Returns the result of repeating a sequence object `count` times.
    ///
    /// This is equivalent to the Python expression `self * count`.
    /// NB: Python accepts negative counts; it returns an empty Sequence.
    #[inline]
    pub fn repeat(&self, count: isize) -> PyResult<&PySequence> {
        unsafe {
            let ptr = self
                .py()
                .from_owned_ptr_or_err::<PyAny>(ffi::PySequence_Repeat(
                    self.as_ptr(),
                    count as Py_ssize_t,
                ))?;
            Ok(&*(ptr as *const PyAny as *const PySequence))
        }
    }

    /// Concatenates `self` and `other` in place.
    ///
    /// This is equivalent to the Python statement `self += other`.
    #[inline]
    pub fn in_place_concat(&self, other: &PySequence) -> PyResult<()> {
        unsafe {
            let ptr = ffi::PySequence_InPlaceConcat(self.as_ptr(), other.as_ptr());
            if ptr.is_null() {
                Err(PyErr::api_call_failed(self.py()))
            } else {
                Ok(())
            }
        }
    }

    /// Repeats the sequence object `count` times and updates `self`.
    ///
    /// This is equivalent to the Python statement `self *= count`.
    /// NB: Python accepts negative counts; it empties the Sequence.
    #[inline]
    pub fn in_place_repeat(&self, count: isize) -> PyResult<()> {
        unsafe {
            let ptr = ffi::PySequence_InPlaceRepeat(self.as_ptr(), count as Py_ssize_t);
            if ptr.is_null() {
                Err(PyErr::api_call_failed(self.py()))
            } else {
                Ok(())
            }
        }
    }

    /// Returns the `index`th element of the Sequence.
    ///
    /// This is equivalent to the Python expression `self[index]`.
    #[inline]
    pub fn get_item(&self, index: isize) -> PyResult<&PyAny> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PySequence_GetItem(self.as_ptr(), index as Py_ssize_t))
        }
    }

    /// Returns the slice of sequence object between `begin` and `end`.
    ///
    /// This is equivalent to the Python expression `self[begin:end]`.
    #[inline]
    pub fn get_slice(&self, begin: isize, end: isize) -> PyResult<&PyAny> {
        unsafe {
            self.py().from_owned_ptr_or_err(ffi::PySequence_GetSlice(
                self.as_ptr(),
                begin as Py_ssize_t,
                end as Py_ssize_t,
            ))
        }
    }

    /// Assigns object `item` to the `i`th element of self.
    ///
    /// This is equivalent to the Python statement `self[i] = v`.
    #[inline]
    pub fn set_item<I>(&self, i: isize, item: I) -> PyResult<()>
    where
        I: ToBorrowedObject,
    {
        unsafe {
            item.with_borrowed_ptr(self.py(), |item| {
                err::error_on_minusone(
                    self.py(),
                    ffi::PySequence_SetItem(self.as_ptr(), i as Py_ssize_t, item),
                )
            })
        }
    }

    /// Deletes the `i`th element of self.
    ///
    /// This is equivalent to the Python statement `del self[i]`.
    #[inline]
    pub fn del_item(&self, i: isize) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PySequence_DelItem(self.as_ptr(), i as Py_ssize_t),
            )
        }
    }

    /// Assigns the sequence `v` to the slice of `self` from `i1` to `i2`.
    ///
    /// This is equivalent to the Python statement `self[i1:i2] = v`.
    #[inline]
    pub fn set_slice(&self, i1: isize, i2: isize, v: &PyAny) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PySequence_SetSlice(
                    self.as_ptr(),
                    i1 as Py_ssize_t,
                    i2 as Py_ssize_t,
                    v.as_ptr(),
                ),
            )
        }
    }

    /// Deletes the slice from `i1` to `i2` from `self`.
    ///
    /// This is equivalent to the Python statement `del self[i1:i2]`.
    #[inline]
    pub fn del_slice(&self, i1: isize, i2: isize) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PySequence_DelSlice(self.as_ptr(), i1 as Py_ssize_t, i2 as Py_ssize_t),
            )
        }
    }

    /// Returns the number of occurrences of `value` in self, that is, return the
    /// number of keys for which `self[key] == value`.
    #[inline]
    #[cfg(not(PyPy))]
    #[cfg_attr(docsrs, doc(cfg(not(PyPy))))]
    pub fn count<V>(&self, value: V) -> PyResult<usize>
    where
        V: ToBorrowedObject,
    {
        let r = value.with_borrowed_ptr(self.py(), |ptr| unsafe {
            ffi::PySequence_Count(self.as_ptr(), ptr)
        });
        if r == -1 {
            Err(PyErr::api_call_failed(self.py()))
        } else {
            Ok(r as usize)
        }
    }

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    #[inline]
    pub fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToBorrowedObject,
    {
        let r = value.with_borrowed_ptr(self.py(), |ptr| unsafe {
            ffi::PySequence_Contains(self.as_ptr(), ptr)
        });
        match r {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(PyErr::api_call_failed(self.py())),
        }
    }

    /// Returns the first index `i` for which `self[i] == value`.
    ///
    /// This is equivalent to the Python expression `self.index(value)`.
    #[inline]
    pub fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: ToBorrowedObject,
    {
        let r = value.with_borrowed_ptr(self.py(), |ptr| unsafe {
            ffi::PySequence_Index(self.as_ptr(), ptr)
        });
        if r == -1 {
            Err(PyErr::api_call_failed(self.py()))
        } else {
            Ok(r as usize)
        }
    }

    /// Returns a fresh list based on the Sequence.
    #[inline]
    pub fn list(&self) -> PyResult<&PyList> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PySequence_List(self.as_ptr()))
        }
    }

    /// Returns a fresh tuple based on the Sequence.
    #[inline]
    pub fn tuple(&self) -> PyResult<&PyTuple> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PySequence_Tuple(self.as_ptr()))
        }
    }
}

impl<'a, T> FromPyObject<'a> for Vec<T>
where
    T: FromPyObject<'a>,
{
    #[cfg(not(feature = "nightly"))]
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        extract_sequence(obj)
    }
    #[cfg(feature = "nightly")]
    default fn extract(obj: &'a PyAny) -> PyResult<Self> {
        extract_sequence(obj)
    }
}

#[cfg(feature = "nightly")]
impl<'source, T> FromPyObject<'source> for Vec<T>
where
    for<'a> T: FromPyObject<'a> + crate::buffer::Element,
{
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        // first try buffer protocol
        if let Ok(buf) = crate::buffer::PyBuffer::get(obj) {
            if buf.dimensions() == 1 {
                if let Ok(v) = buf.to_vec(obj.py()) {
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

fn extract_sequence<'s, T>(obj: &'s PyAny) -> PyResult<Vec<T>>
where
    T: FromPyObject<'s>,
{
    let seq = <PySequence as PyTryFrom>::try_from(obj)?;
    let mut v = Vec::with_capacity(seq.len().unwrap_or(0) as usize);
    for item in seq.iter()? {
        v.push(item?.extract::<T>()?);
    }
    Ok(v)
}

impl<'v> PyTryFrom<'v> for PySequence {
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v PySequence, PyDowncastError<'v>> {
        let value = value.into();
        unsafe {
            if ffi::PySequence_Check(value.as_ptr()) != 0 {
                Ok(<PySequence as PyTryFrom>::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value, "Sequence"))
            }
        }
    }

    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v PySequence, PyDowncastError<'v>> {
        <PySequence as PyTryFrom>::try_from(value)
    }

    #[inline]
    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v PySequence {
        let ptr = value.into() as *const _ as *const PySequence;
        &*ptr
    }
}

#[cfg(test)]
mod test {
    use crate::types::PySequence;
    use crate::AsPyPointer;
    use crate::Python;
    use crate::{PyObject, PyTryFrom, ToPyObject};

    fn get_object() -> PyObject {
        // Convenience function for getting a single unique object
        let gil = Python::acquire_gil();
        let py = gil.python();

        let obj = py.eval("object()", None, None).unwrap();

        obj.to_object(py)
    }

    #[test]
    fn test_numbers_are_not_sequences() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = 42i32;
        assert!(<PySequence as PyTryFrom>::try_from(v.to_object(py).as_ref(py)).is_err());
    }

    #[test]
    fn test_strings_are_sequences() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = "London Calling";
        assert!(<PySequence as PyTryFrom>::try_from(v.to_object(py).as_ref(py)).is_ok());
    }
    #[test]
    fn test_seq_empty() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = vec![];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        assert_eq!(0, seq.len().unwrap());

        let needle = 7i32.to_object(py);
        assert!(!seq.contains(&needle).unwrap());
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
        assert!(!seq.contains(&bad_needle).unwrap());

        let good_needle = 8i32.to_object(py);
        assert!(seq.contains(&good_needle).unwrap());

        let type_coerced_needle = 8f32.to_object(py);
        assert!(seq.contains(&type_coerced_needle).unwrap());
    }

    #[test]
    fn test_seq_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
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
        let obj = get_object();
        {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let v: Vec<i32> = vec![1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert!(seq.set_item(1, &obj).is_ok());
            assert!(seq.get_item(1).unwrap().as_ptr() == obj.as_ptr());
        }
        {
            let gil = Python::acquire_gil();
            let py = gil.python();
            assert_eq!(1, obj.get_refcnt(py));
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
    #[cfg(not(PyPy))]
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
        assert!(!seq.contains(bad_needle).unwrap());

        let good_needle = "worst".to_object(py);
        assert!(seq.contains(good_needle).unwrap());
    }

    #[test]
    fn test_seq_concat() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<i32> = vec![1, 2, 3];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        let concat_seq = seq.concat(&seq).unwrap();
        assert_eq!(6, concat_seq.len().unwrap());
        let concat_v: Vec<i32> = vec![1, 2, 3, 1, 2, 3];
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
        let seq = <PySequence as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
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
        let v: Vec<i32> = py
            .eval("range(1, 5)", None, None)
            .unwrap()
            .extract()
            .unwrap();
        assert!(v == [1, 2, 3, 4]);
    }

    #[test]
    fn test_extract_bytearray_to_vec() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v: Vec<u8> = py
            .eval("bytearray(b'abc')", None, None)
            .unwrap()
            .extract()
            .unwrap();
        assert!(v == b"abc");
    }

    #[test]
    fn test_seq_try_from_unchecked() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec!["foo", "bar"];
        let ob = v.to_object(py);
        let seq = ob.cast_as::<PySequence>(py).unwrap();
        let type_ptr = seq.as_ref();
        let seq_from = unsafe { <PySequence as PyTryFrom>::try_from_unchecked(type_ptr) };
        assert!(seq_from.list().is_ok());
    }

    #[test]
    fn test_is_empty() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = vec![1].to_object(py);
        let seq = list.cast_as::<PySequence>(py).unwrap();
        assert!(!seq.is_empty().unwrap());
        let vec: Vec<u32> = Vec::new();
        let empty_list = vec.to_object(py);
        let empty_seq = empty_list.cast_as::<PySequence>(py).unwrap();
        assert!(empty_seq.is_empty().unwrap());
    }
}
