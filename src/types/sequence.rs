// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::err::{self, PyDowncastError, PyErr, PyResult};
use crate::exceptions::PyValueError;
use crate::internal_tricks::get_ssize_index;
use crate::once_cell::GILOnceCell;
use crate::type_object::PyTypeInfo;
use crate::types::{PyAny, PyList, PyString, PyTuple, PyType};
use crate::{ffi, PyNativeType, ToPyObject};
use crate::{AsPyPointer, IntoPy, IntoPyPointer, Py, Python};
use crate::{FromPyObject, PyTryFrom};

static SEQUENCE_ABC: GILOnceCell<PyResult<Py<PyType>>> = GILOnceCell::new();

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
    pub fn len(&self) -> PyResult<usize> {
        let v = unsafe { ffi::PySequence_Size(self.as_ptr()) };
        if v == -1 {
            Err(PyErr::fetch(self.py()))
        } else {
            Ok(v as usize)
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
            self.py()
                .from_owned_ptr_or_err(ffi::PySequence_Concat(self.as_ptr(), other.as_ptr()))
        }
    }

    /// Returns the result of repeating a sequence object `count` times.
    ///
    /// This is equivalent to the Python expression `self * count`.
    #[inline]
    pub fn repeat(&self, count: usize) -> PyResult<&PySequence> {
        unsafe {
            self.py().from_owned_ptr_or_err(ffi::PySequence_Repeat(
                self.as_ptr(),
                get_ssize_index(count),
            ))
        }
    }

    /// Concatenates `self` and `other`, in place if possible.
    ///
    /// This is equivalent to the Python expression `self.__iadd__(other)`.
    ///
    /// The Python statement `self += other` is syntactic sugar for `self =
    /// self.__iadd__(other)`.  `__iadd__` should modify and return `self` if
    /// possible, but create and return a new object if not.
    #[inline]
    pub fn in_place_concat(&self, other: &PySequence) -> PyResult<&PySequence> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PySequence_InPlaceConcat(self.as_ptr(), other.as_ptr()))
        }
    }

    /// Repeats the sequence object `count` times and updates `self`, if possible.
    ///
    /// This is equivalent to the Python expression `self.__imul__(other)`.
    ///
    /// The Python statement `self *= other` is syntactic sugar for `self =
    /// self.__imul__(other)`.  `__imul__` should modify and return `self` if
    /// possible, but create and return a new object if not.
    #[inline]
    pub fn in_place_repeat(&self, count: usize) -> PyResult<&PySequence> {
        unsafe {
            self.py()
                .from_owned_ptr_or_err(ffi::PySequence_InPlaceRepeat(
                    self.as_ptr(),
                    get_ssize_index(count),
                ))
        }
    }

    /// Returns the `index`th element of the Sequence.
    ///
    /// This is equivalent to the Python expression `self[index]` without support of negative indices.
    #[inline]
    pub fn get_item(&self, index: usize) -> PyResult<&PyAny> {
        unsafe {
            self.py().from_owned_ptr_or_err(ffi::PySequence_GetItem(
                self.as_ptr(),
                get_ssize_index(index),
            ))
        }
    }

    /// Returns the slice of sequence object between `begin` and `end`.
    ///
    /// This is equivalent to the Python expression `self[begin:end]`.
    #[inline]
    pub fn get_slice(&self, begin: usize, end: usize) -> PyResult<&PySequence> {
        unsafe {
            self.py().from_owned_ptr_or_err(ffi::PySequence_GetSlice(
                self.as_ptr(),
                get_ssize_index(begin),
                get_ssize_index(end),
            ))
        }
    }

    /// Assigns object `item` to the `i`th element of self.
    ///
    /// This is equivalent to the Python statement `self[i] = v`.
    #[inline]
    pub fn set_item<I>(&self, i: usize, item: I) -> PyResult<()>
    where
        I: ToPyObject,
    {
        let py = self.py();
        unsafe {
            err::error_on_minusone(
                py,
                ffi::PySequence_SetItem(
                    self.as_ptr(),
                    get_ssize_index(i),
                    item.to_object(py).as_ptr(),
                ),
            )
        }
    }

    /// Deletes the `i`th element of self.
    ///
    /// This is equivalent to the Python statement `del self[i]`.
    #[inline]
    pub fn del_item(&self, i: usize) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PySequence_DelItem(self.as_ptr(), get_ssize_index(i)),
            )
        }
    }

    /// Assigns the sequence `v` to the slice of `self` from `i1` to `i2`.
    ///
    /// This is equivalent to the Python statement `self[i1:i2] = v`.
    #[inline]
    pub fn set_slice(&self, i1: usize, i2: usize, v: &PyAny) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PySequence_SetSlice(
                    self.as_ptr(),
                    get_ssize_index(i1),
                    get_ssize_index(i2),
                    v.as_ptr(),
                ),
            )
        }
    }

    /// Deletes the slice from `i1` to `i2` from `self`.
    ///
    /// This is equivalent to the Python statement `del self[i1:i2]`.
    #[inline]
    pub fn del_slice(&self, i1: usize, i2: usize) -> PyResult<()> {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PySequence_DelSlice(self.as_ptr(), get_ssize_index(i1), get_ssize_index(i2)),
            )
        }
    }

    /// Returns the number of occurrences of `value` in self, that is, return the
    /// number of keys for which `self[key] == value`.
    #[inline]
    #[cfg(not(PyPy))]
    pub fn count<V>(&self, value: V) -> PyResult<usize>
    where
        V: ToPyObject,
    {
        let r =
            unsafe { ffi::PySequence_Count(self.as_ptr(), value.to_object(self.py()).as_ptr()) };
        if r == -1 {
            Err(PyErr::fetch(self.py()))
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
        V: ToPyObject,
    {
        let r =
            unsafe { ffi::PySequence_Contains(self.as_ptr(), value.to_object(self.py()).as_ptr()) };
        match r {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(PyErr::fetch(self.py())),
        }
    }

    /// Returns the first index `i` for which `self[i] == value`.
    ///
    /// This is equivalent to the Python expression `self.index(value)`.
    #[inline]
    pub fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: ToPyObject,
    {
        let r =
            unsafe { ffi::PySequence_Index(self.as_ptr(), value.to_object(self.py()).as_ptr()) };
        if r == -1 {
            Err(PyErr::fetch(self.py()))
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

    /// Register a pyclass as a subclass of `collections.abc.Sequence` (from the Python standard
    /// library). This is equvalent to `collections.abc.Sequence.register(T)` in Python.
    /// This registration is required for a pyclass to be downcastable from `PyAny` to `PySequence`.
    pub fn register<T: PyTypeInfo>(py: Python<'_>) -> PyResult<()> {
        let ty = T::type_object(py);
        get_sequence_abc(py)?.call_method1("register", (ty,))?;
        Ok(())
    }
}

#[inline]
fn sequence_len(seq: &PySequence) -> usize {
    seq.len().expect("failed to get sequence length")
}

#[inline]
fn sequence_slice(seq: &PySequence, start: usize, end: usize) -> &PySequence {
    seq.get_slice(start, end)
        .expect("sequence slice operation failed")
}

index_impls!(PySequence, "sequence", sequence_len, sequence_slice);

impl<'a, T> FromPyObject<'a> for Vec<T>
where
    T: FromPyObject<'a>,
{
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        if let Ok(true) = obj.is_instance_of::<PyString>() {
            return Err(PyValueError::new_err("Can't extract `str` to `Vec`"));
        }
        extract_sequence(obj)
    }
}

fn extract_sequence<'s, T>(obj: &'s PyAny) -> PyResult<Vec<T>>
where
    T: FromPyObject<'s>,
{
    // Types that pass `PySequence_Check` usually implement enough of the sequence protocol
    // to support this function and if not, we will only fail extraction safely.
    let seq = unsafe {
        if ffi::PySequence_Check(obj.as_ptr()) != 0 {
            <PySequence as PyTryFrom>::try_from_unchecked(obj)
        } else {
            return Err(PyDowncastError::new(obj, "Sequence").into());
        }
    };

    let mut v = Vec::with_capacity(seq.len().unwrap_or(0) as usize);
    for item in seq.iter()? {
        v.push(item?.extract::<T>()?);
    }
    Ok(v)
}

fn get_sequence_abc(py: Python<'_>) -> Result<&PyType, PyErr> {
    SEQUENCE_ABC
        .get_or_init(py, || {
            Ok(py
                .import("collections.abc")?
                .getattr("Sequence")?
                .downcast::<PyType>()?
                .into_py(py))
        })
        .as_ref()
        .map_or_else(|e| Err(e.clone_ref(py)), |t| Ok(t.as_ref(py)))
}

impl<'v> PyTryFrom<'v> for PySequence {
    /// Downcasting to `PySequence` requires the concrete class to be a subclass (or registered
    /// subclass) of `collections.abc.Sequence` (from the Python standard library) - i.e.
    /// `isinstance(<class>, collections.abc.Sequence) == True`.
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v PySequence, PyDowncastError<'v>> {
        let value = value.into();

        // TODO: surface specific errors in this chain to the user
        if let Ok(abc) = get_sequence_abc(value.py()) {
            if value.is_instance(abc).unwrap_or(false) {
                unsafe { return Ok(<PySequence as PyTryFrom>::try_from_unchecked(value)) }
            }
        }

        Err(PyDowncastError::new(value, "Sequence"))
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

impl Py<PySequence> {
    /// Borrows a GIL-bound reference to the PySequence. By binding to the GIL lifetime, this
    /// allows the GIL-bound reference to not require `Python` for any of its methods.
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::{PyList, PySequence};
    /// # Python::with_gil(|py| {
    /// let seq: Py<PySequence> = PyList::empty(py).as_sequence().into();
    /// let seq: &PySequence = seq.as_ref(py);
    /// assert_eq!(seq.len().unwrap(), 0);
    /// # });
    /// ```
    pub fn as_ref<'py>(&'py self, _py: Python<'py>) -> &'py PySequence {
        let any = self.as_ptr() as *const PyAny;
        unsafe { PyNativeType::unchecked_downcast(&*any) }
    }

    /// Similar to [`as_ref`](#method.as_ref), and also consumes this `Py` and registers the
    /// Python object reference in PyO3's object storage. The reference count for the Python
    /// object will not be decreased until the GIL lifetime ends.
    pub fn into_ref(self, py: Python<'_>) -> &PySequence {
        unsafe { py.from_owned_ptr(self.into_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyList, PySequence};
    use crate::{AsPyPointer, Py, PyObject, PyTryFrom, Python, ToPyObject};

    fn get_object() -> PyObject {
        // Convenience function for getting a single unique object
        Python::with_gil(|py| {
            let obj = py.eval("object()", None, None).unwrap();

            obj.to_object(py)
        })
    }

    #[test]
    fn test_numbers_are_not_sequences() {
        Python::with_gil(|py| {
            let v = 42i32;
            assert!(<PySequence as PyTryFrom>::try_from(v.to_object(py).as_ref(py)).is_err());
        });
    }

    #[test]
    fn test_strings_are_sequences() {
        Python::with_gil(|py| {
            let v = "London Calling";
            assert!(<PySequence as PyTryFrom>::try_from(v.to_object(py).as_ref(py)).is_ok());
        });
    }

    #[test]
    fn test_strings_cannot_be_extracted_to_vec() {
        Python::with_gil(|py| {
            let v = "London Calling";
            let ob = v.to_object(py);

            assert!(ob.extract::<Vec<&str>>(py).is_err());
            assert!(ob.extract::<Vec<String>>(py).is_err());
            assert!(ob.extract::<Vec<char>>(py).is_err());
        });
    }

    #[test]
    fn test_seq_empty() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert_eq!(0, seq.len().unwrap());

            let needle = 7i32.to_object(py);
            assert!(!seq.contains(&needle).unwrap());
        });
    }

    #[test]
    fn test_seq_is_empty() {
        Python::with_gil(|py| {
            let list = vec![1].to_object(py);
            let seq = list.cast_as::<PySequence>(py).unwrap();
            assert!(!seq.is_empty().unwrap());
            let vec: Vec<u32> = Vec::new();
            let empty_list = vec.to_object(py);
            let empty_seq = empty_list.cast_as::<PySequence>(py).unwrap();
            assert!(empty_seq.is_empty().unwrap());
        });
    }

    #[test]
    fn test_seq_contains() {
        Python::with_gil(|py| {
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
        });
    }

    #[test]
    fn test_seq_get_item() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert_eq!(1, seq.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(1, seq.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(2, seq.get_item(2).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, seq.get_item(3).unwrap().extract::<i32>().unwrap());
            assert_eq!(5, seq.get_item(4).unwrap().extract::<i32>().unwrap());
            assert_eq!(8, seq.get_item(5).unwrap().extract::<i32>().unwrap());
            assert!(seq.get_item(10).is_err());
        });
    }

    #[test]
    fn test_seq_index_trait() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert_eq!(1, seq[0].extract::<i32>().unwrap());
            assert_eq!(1, seq[1].extract::<i32>().unwrap());
            assert_eq!(2, seq[2].extract::<i32>().unwrap());
        });
    }

    #[test]
    #[should_panic = "index 7 out of range for sequence"]
    fn test_seq_index_trait_panic() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            let _ = &seq[7];
        });
    }

    #[test]
    fn test_seq_index_trait_ranges() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert_eq!(vec![1, 2], seq[1..3].extract::<Vec<i32>>().unwrap());
            assert_eq!(Vec::<i32>::new(), seq[3..3].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![1, 2], seq[1..].extract::<Vec<i32>>().unwrap());
            assert_eq!(Vec::<i32>::new(), seq[3..].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![1, 1, 2], seq[..].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![1, 2], seq[1..=2].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![1, 1], seq[..2].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![1, 1], seq[..=1].extract::<Vec<i32>>().unwrap());
        })
    }

    #[test]
    #[should_panic = "range start index 5 out of range for sequence of length 3"]
    fn test_seq_index_trait_range_panic_start() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            seq[5..10].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    #[should_panic = "range end index 10 out of range for sequence of length 3"]
    fn test_seq_index_trait_range_panic_end() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            seq[1..10].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    #[should_panic = "slice index starts at 2 but ends at 1"]
    fn test_seq_index_trait_range_panic_wrong_order() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            #[allow(clippy::reversed_empty_ranges)]
            seq[2..1].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    #[should_panic = "range start index 8 out of range for sequence of length 3"]
    fn test_seq_index_trait_range_from_panic() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            seq[8..].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    fn test_seq_del_item() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert!(seq.del_item(10).is_err());
            assert_eq!(1, seq[0].extract::<i32>().unwrap());
            assert!(seq.del_item(0).is_ok());
            assert_eq!(1, seq[0].extract::<i32>().unwrap());
            assert!(seq.del_item(0).is_ok());
            assert_eq!(2, seq[0].extract::<i32>().unwrap());
            assert!(seq.del_item(0).is_ok());
            assert_eq!(3, seq[0].extract::<i32>().unwrap());
            assert!(seq.del_item(0).is_ok());
            assert_eq!(5, seq[0].extract::<i32>().unwrap());
            assert!(seq.del_item(0).is_ok());
            assert_eq!(8, seq[0].extract::<i32>().unwrap());
            assert!(seq.del_item(0).is_ok());
            assert_eq!(0, seq.len().unwrap());
            assert!(seq.del_item(0).is_err());
        });
    }

    #[test]
    fn test_seq_set_item() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert_eq!(2, seq[1].extract::<i32>().unwrap());
            assert!(seq.set_item(1, 10).is_ok());
            assert_eq!(10, seq[1].extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_seq_set_item_refcnt() {
        let obj = get_object();

        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 2];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert!(seq.set_item(1, &obj).is_ok());
            assert!(seq[1].as_ptr() == obj.as_ptr());
        });

        Python::with_gil(|py| {
            assert_eq!(1, obj.get_refcnt(py));
        });
    }

    #[test]
    fn test_seq_get_slice() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert_eq!(
                [1, 2, 3],
                seq.get_slice(1, 4).unwrap().extract::<[i32; 3]>().unwrap()
            );
            assert_eq!(
                [3, 5, 8],
                seq.get_slice(3, 100)
                    .unwrap()
                    .extract::<[i32; 3]>()
                    .unwrap()
            );
        });
    }

    #[test]
    fn test_set_slice() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
            let w: Vec<i32> = vec![7, 4];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            let ins = w.to_object(py);
            seq.set_slice(1, 4, ins.as_ref(py)).unwrap();
            assert_eq!([1, 7, 4, 5, 8], seq.extract::<[i32; 5]>().unwrap());
            seq.set_slice(3, 100, PyList::empty(py)).unwrap();
            assert_eq!([1, 7, 4], seq.extract::<[i32; 3]>().unwrap());
        });
    }

    #[test]
    fn test_del_slice() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            seq.del_slice(1, 4).unwrap();
            assert_eq!([1, 5, 8], seq.extract::<[i32; 3]>().unwrap());
            seq.del_slice(1, 100).unwrap();
            assert_eq!([1], seq.extract::<[i32; 1]>().unwrap());
        });
    }

    #[test]
    fn test_seq_index() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert_eq!(0, seq.index(1i32).unwrap());
            assert_eq!(2, seq.index(2i32).unwrap());
            assert_eq!(3, seq.index(3i32).unwrap());
            assert_eq!(4, seq.index(5i32).unwrap());
            assert_eq!(5, seq.index(8i32).unwrap());
            assert!(seq.index(42i32).is_err());
        });
    }

    #[test]
    #[cfg(not(PyPy))]
    fn test_seq_count() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert_eq!(2, seq.count(1i32).unwrap());
            assert_eq!(1, seq.count(2i32).unwrap());
            assert_eq!(1, seq.count(3i32).unwrap());
            assert_eq!(1, seq.count(5i32).unwrap());
            assert_eq!(1, seq.count(8i32).unwrap());
            assert_eq!(0, seq.count(42i32).unwrap());
        });
    }

    #[test]
    fn test_seq_iter() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 1, 2, 3, 5, 8];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            let mut idx = 0;
            for el in seq.iter().unwrap() {
                assert_eq!(v[idx], el.unwrap().extract::<i32>().unwrap());
                idx += 1;
            }
            assert_eq!(idx, v.len());
        });
    }

    #[test]
    fn test_seq_strings() {
        Python::with_gil(|py| {
            let v = vec!["It", "was", "the", "worst", "of", "times"];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();

            let bad_needle = "blurst".to_object(py);
            assert!(!seq.contains(bad_needle).unwrap());

            let good_needle = "worst".to_object(py);
            assert!(seq.contains(good_needle).unwrap());
        });
    }

    #[test]
    fn test_seq_concat() {
        Python::with_gil(|py| {
            let v: Vec<i32> = vec![1, 2, 3];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            let concat_seq = seq.concat(seq).unwrap();
            assert_eq!(6, concat_seq.len().unwrap());
            let concat_v: Vec<i32> = vec![1, 2, 3, 1, 2, 3];
            for (el, cc) in concat_seq.iter().unwrap().zip(concat_v) {
                assert_eq!(cc, el.unwrap().extract::<i32>().unwrap());
            }
        });
    }

    #[test]
    fn test_seq_concat_string() {
        Python::with_gil(|py| {
            let v = "string";
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            let concat_seq = seq.concat(seq).unwrap();
            assert_eq!(12, concat_seq.len().unwrap());
            let concat_v = "stringstring".to_owned();
            for (el, cc) in seq.iter().unwrap().zip(concat_v.chars()) {
                assert_eq!(cc, el.unwrap().extract::<char>().unwrap());
            }
        });
    }

    #[test]
    fn test_seq_repeat() {
        Python::with_gil(|py| {
            let v = vec!["foo", "bar"];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            let repeat_seq = seq.repeat(3).unwrap();
            assert_eq!(6, repeat_seq.len().unwrap());
            let repeated = vec!["foo", "bar", "foo", "bar", "foo", "bar"];
            for (el, rpt) in repeat_seq.iter().unwrap().zip(repeated.iter()) {
                assert_eq!(*rpt, el.unwrap().extract::<String>().unwrap());
            }
        });
    }

    #[test]
    fn test_seq_inplace() {
        Python::with_gil(|py| {
            let v = vec!["foo", "bar"];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            let rep_seq = seq.in_place_repeat(3).unwrap();
            assert_eq!(6, seq.len().unwrap());
            assert!(seq.is(rep_seq));

            let conc_seq = seq.in_place_concat(seq).unwrap();
            assert_eq!(12, seq.len().unwrap());
            assert!(seq.is(conc_seq));
        });
    }

    #[test]
    fn test_list_coercion() {
        Python::with_gil(|py| {
            let v = vec!["foo", "bar"];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert!(seq.list().is_ok());
        });
    }

    #[test]
    fn test_strings_coerce_to_lists() {
        Python::with_gil(|py| {
            let v = "foo";
            let ob = v.to_object(py);
            let seq = <PySequence as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
            assert!(seq.list().is_ok());
        });
    }

    #[test]
    fn test_tuple_coercion() {
        Python::with_gil(|py| {
            let v = ("foo", "bar");
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert!(seq.tuple().is_ok());
        });
    }

    #[test]
    fn test_lists_coerce_to_tuples() {
        Python::with_gil(|py| {
            let v = vec!["foo", "bar"];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            assert!(seq.tuple().is_ok());
        });
    }

    #[test]
    fn test_extract_tuple_to_vec() {
        Python::with_gil(|py| {
            let v: Vec<i32> = py.eval("(1, 2)", None, None).unwrap().extract().unwrap();
            assert!(v == [1, 2]);
        });
    }

    #[test]
    fn test_extract_range_to_vec() {
        Python::with_gil(|py| {
            let v: Vec<i32> = py
                .eval("range(1, 5)", None, None)
                .unwrap()
                .extract()
                .unwrap();
            assert!(v == [1, 2, 3, 4]);
        });
    }

    #[test]
    fn test_extract_bytearray_to_vec() {
        Python::with_gil(|py| {
            let v: Vec<u8> = py
                .eval("bytearray(b'abc')", None, None)
                .unwrap()
                .extract()
                .unwrap();
            assert!(v == b"abc");
        });
    }

    #[test]
    fn test_seq_try_from_unchecked() {
        Python::with_gil(|py| {
            let v = vec!["foo", "bar"];
            let ob = v.to_object(py);
            let seq = ob.cast_as::<PySequence>(py).unwrap();
            let type_ptr = seq.as_ref();
            let seq_from = unsafe { <PySequence as PyTryFrom>::try_from_unchecked(type_ptr) };
            assert!(seq_from.list().is_ok());
        });
    }

    #[test]
    fn test_as_ref() {
        Python::with_gil(|py| {
            let seq: Py<PySequence> = PyList::empty(py).as_sequence().into();
            let seq_ref: &PySequence = seq.as_ref(py);
            assert_eq!(seq_ref.len().unwrap(), 0);
        })
    }

    #[test]
    fn test_into_ref() {
        Python::with_gil(|py| {
            let bare_seq = PyList::empty(py).as_sequence();
            assert_eq!(bare_seq.get_refcnt(), 1);
            let seq: Py<PySequence> = bare_seq.into();
            assert_eq!(bare_seq.get_refcnt(), 2);
            let seq_ref = seq.into_ref(py);
            assert_eq!(seq_ref.len().unwrap(), 0);
            assert_eq!(seq_ref.get_refcnt(), 2);
        })
    }
}
