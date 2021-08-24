// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::err::{self, PyResult};
use crate::ffi::{self, Py_ssize_t};
use crate::internal_tricks::get_ssize_index;
use crate::{
    AsPyPointer, IntoPy, IntoPyPointer, PyAny, PyObject, Python, ToBorrowedObject, ToPyObject,
};

/// Represents a Python `list`.
#[repr(transparent)]
pub struct PyList(PyAny);

pyobject_native_type_core!(PyList, ffi::PyList_Type, #checkfunction=ffi::PyList_Check);

#[inline]
unsafe fn new_from_iter<T>(
    elements: impl ExactSizeIterator<Item = T>,
    convert: impl Fn(T) -> PyObject,
) -> *mut ffi::PyObject {
    let ptr = ffi::PyList_New(elements.len() as Py_ssize_t);
    for (i, e) in elements.enumerate() {
        let obj = convert(e).into_ptr();
        #[cfg(not(Py_LIMITED_API))]
        ffi::PyList_SET_ITEM(ptr, i as Py_ssize_t, obj);
        #[cfg(Py_LIMITED_API)]
        ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj);
    }
    ptr
}

impl PyList {
    /// Constructs a new list with the given elements.
    pub fn new<T, U>(py: Python<'_>, elements: impl IntoIterator<Item = T, IntoIter = U>) -> &PyList
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>,
    {
        unsafe {
            py.from_owned_ptr::<PyList>(new_from_iter(elements.into_iter(), |e| e.to_object(py)))
        }
    }

    /// Constructs a new empty list.
    pub fn empty(py: Python) -> &PyList {
        unsafe { py.from_owned_ptr::<PyList>(ffi::PyList_New(0)) }
    }

    /// Returns the length of the list.
    pub fn len(&self) -> usize {
        unsafe {
            #[cfg(not(Py_LIMITED_API))]
            let size = ffi::PyList_GET_SIZE(self.as_ptr());
            #[cfg(Py_LIMITED_API)]
            let size = ffi::PyList_Size(self.as_ptr());

            // non-negative Py_ssize_t should always fit into Rust usize
            size as usize
        }
    }

    /// Checks if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the list item at the specified index.
    /// # Example
    /// ```
    /// use pyo3::{prelude::*, types::PyList};
    /// Python::with_gil(|py| {
    ///     let list = PyList::new(py, &[2, 3, 5, 7]);
    ///     let obj = list.get_item(0);
    ///     assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 2);
    /// });
    /// ```
    pub fn get_item(&self, index: usize) -> PyResult<&PyAny> {
        unsafe {
            let item = ffi::PyList_GetItem(self.as_ptr(), index as Py_ssize_t);
            // PyList_GetItem return borrowed ptr; must make owned for safety (see #890).
            ffi::Py_XINCREF(item);
            self.py().from_owned_ptr_or_err(item)
        }
    }

    /// Gets the list item at the specified index. Undefined behavior on bad index. Use with caution.
    ///
    /// # Safety
    ///
    /// Caller must verify that the index is within the bounds of the list.
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    #[cfg_attr(docsrs, doc(cfg(not(any(Py_LIMITED_API, PyPy)))))]
    pub unsafe fn get_item_unchecked(&self, index: usize) -> &PyAny {
        let item = ffi::PyList_GET_ITEM(self.as_ptr(), index as Py_ssize_t);
        // PyList_GET_ITEM return borrowed ptr; must make owned for safety (see #890).
        ffi::Py_XINCREF(item);
        self.py().from_owned_ptr(item)
    }

    /// Takes the slice `self[low:high]` and returns it as a new list.
    ///
    /// Indices must be nonnegative, and out-of-range indices are clipped to
    /// `self.len()`.
    pub fn get_slice(&self, low: usize, high: usize) -> &PyList {
        unsafe {
            self.py().from_owned_ptr(ffi::PyList_GetSlice(
                self.as_ptr(),
                get_ssize_index(low),
                get_ssize_index(high),
            ))
        }
    }

    /// Sets the item at the specified index.
    ///
    /// Raises `IndexError` if the index is out of range.
    pub fn set_item<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: ToPyObject,
    {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PyList_SetItem(
                    self.as_ptr(),
                    get_ssize_index(index),
                    item.to_object(self.py()).into_ptr(),
                ),
            )
        }
    }

    /// Appends an item to the list.
    pub fn append<I>(&self, item: I) -> PyResult<()>
    where
        I: ToBorrowedObject,
    {
        item.with_borrowed_ptr(self.py(), |item| unsafe {
            err::error_on_minusone(self.py(), ffi::PyList_Append(self.as_ptr(), item))
        })
    }

    /// Inserts an item at the specified index.
    ///
    /// If `index >= self.len()`, inserts at the end.
    pub fn insert<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: ToBorrowedObject,
    {
        item.with_borrowed_ptr(self.py(), |item| unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PyList_Insert(self.as_ptr(), get_ssize_index(index), item),
            )
        })
    }

    /// Returns an iterator over this list's items.
    pub fn iter(&self) -> PyListIterator {
        PyListIterator {
            list: self,
            index: 0,
        }
    }

    /// Sorts the list in-place. Equivalent to the Python expression `l.sort()`.
    pub fn sort(&self) -> PyResult<()> {
        unsafe { err::error_on_minusone(self.py(), ffi::PyList_Sort(self.as_ptr())) }
    }

    /// Reverses the list in-place. Equivalent to the Python expression `l.reverse()`.
    pub fn reverse(&self) -> PyResult<()> {
        unsafe { err::error_on_minusone(self.py(), ffi::PyList_Reverse(self.as_ptr())) }
    }
}

/// Used by `PyList::iter()`.
pub struct PyListIterator<'a> {
    list: &'a PyList,
    index: usize,
}

impl<'a> Iterator for PyListIterator<'a> {
    type Item = &'a PyAny;

    #[inline]
    fn next(&mut self) -> Option<&'a PyAny> {
        if self.index < self.list.len() {
            #[cfg(any(Py_LIMITED_API, PyPy))]
            let item = self.list.get_item(self.index).expect("tuple.get failed");
            #[cfg(not(any(Py_LIMITED_API, PyPy)))]
            let item = unsafe { self.list.get_item_unchecked(self.index) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.list.len();

        (
            len.saturating_sub(self.index),
            Some(len.saturating_sub(self.index)),
        )
    }
}

impl<'a> std::iter::IntoIterator for &'a PyList {
    type Item = &'a PyAny;
    type IntoIter = PyListIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> ToPyObject for [T]
where
    T: ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_owned_ptr(py, new_from_iter(self.iter(), |e| e.to_object(py))) }
    }
}

impl<T> ToPyObject for Vec<T>
where
    T: ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_slice().to_object(py)
    }
}

impl<T> IntoPy<PyObject> for Vec<T>
where
    T: IntoPy<PyObject>,
{
    fn into_py(self, py: Python) -> PyObject {
        unsafe { PyObject::from_owned_ptr(py, new_from_iter(self.into_iter(), |e| e.into_py(py))) }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::PyList;
    use crate::Python;
    use crate::{IntoPy, PyObject, PyTryFrom, ToPyObject};

    #[test]
    fn test_new() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[2, 3, 5, 7]);
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(5, list.get_item(2).unwrap().extract::<i32>().unwrap());
            assert_eq!(7, list.get_item(3).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[1, 2, 3, 4]);
            assert_eq!(4, list.len());
        });
    }

    #[test]
    fn test_get_item() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[2, 3, 5, 7]);
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(5, list.get_item(2).unwrap().extract::<i32>().unwrap());
            assert_eq!(7, list.get_item(3).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_get_slice() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[2, 3, 5, 7]);
            let slice = list.get_slice(1, 3);
            assert_eq!(2, slice.len());
            let slice = list.get_slice(1, 7);
            assert_eq!(3, slice.len());
        });
    }

    #[test]
    fn test_set_item() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[2, 3, 5, 7]);
            let val = 42i32.to_object(py);
            let val2 = 42i32.to_object(py);
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            list.set_item(0, val).unwrap();
            assert_eq!(42, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert!(list.set_item(10, val2).is_err());
        });
    }

    #[test]
    fn test_set_item_refcnt() {
        Python::with_gil(|py| {
            let cnt;
            {
                let _pool = unsafe { crate::GILPool::new() };
                let v = vec![2];
                let ob = v.to_object(py);
                let list = <PyList as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();
                let none = py.None();
                cnt = none.get_refcnt(py);
                list.set_item(0, none).unwrap();
            }

            assert_eq!(cnt, py.None().get_refcnt(py));
        });
    }

    #[test]
    fn test_insert() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[2, 3, 5, 7]);
            let val = 42i32.to_object(py);
            let val2 = 43i32.to_object(py);
            assert_eq!(4, list.len());
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            list.insert(0, val).unwrap();
            list.insert(1000, val2).unwrap();
            assert_eq!(6, list.len());
            assert_eq!(42, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(2, list.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(43, list.get_item(5).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_insert_refcnt() {
        Python::with_gil(|py| {
            let cnt;
            {
                let _pool = unsafe { crate::GILPool::new() };
                let list = PyList::empty(py);
                let none = py.None();
                cnt = none.get_refcnt(py);
                list.insert(0, none).unwrap();
            }

            assert_eq!(cnt, py.None().get_refcnt(py));
        });
    }

    #[test]
    fn test_append() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[2]);
            list.append(3).unwrap();
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(1).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_append_refcnt() {
        Python::with_gil(|py| {
            let cnt;
            {
                let _pool = unsafe { crate::GILPool::new() };
                let list = PyList::empty(py);
                let none = py.None();
                cnt = none.get_refcnt(py);
                list.append(none).unwrap();
            }
            assert_eq!(cnt, py.None().get_refcnt(py));
        });
    }

    #[test]
    fn test_iter() {
        Python::with_gil(|py| {
            let v = vec![2, 3, 5, 7];
            let list = PyList::new(py, &v);
            let mut idx = 0;
            for el in list.iter() {
                assert_eq!(v[idx], el.extract::<i32>().unwrap());
                idx += 1;
            }
            assert_eq!(idx, v.len());
        });
    }

    #[test]
    fn test_iter_size_hint() {
        Python::with_gil(|py| {
            let v = vec![2, 3, 5, 7];
            let ob = v.to_object(py);
            let list = <PyList as PyTryFrom>::try_from(ob.as_ref(py)).unwrap();

            let mut iter = list.iter();
            assert_eq!(iter.size_hint(), (v.len(), Some(v.len())));
            iter.next();
            assert_eq!(iter.size_hint(), (v.len() - 1, Some(v.len() - 1)));

            // Exhust iterator.
            for _ in &mut iter {}

            assert_eq!(iter.size_hint(), (0, Some(0)));
        });
    }

    #[test]
    fn test_into_iter() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[1, 2, 3, 4]);
            for (i, item) in list.iter().enumerate() {
                assert_eq!((i + 1) as i32, item.extract::<i32>().unwrap());
            }
        });
    }

    #[test]
    fn test_extract() {
        Python::with_gil(|py| {
            let v = vec![2, 3, 5, 7];
            let list = PyList::new(py, &v);
            let v2 = list.as_ref().extract::<Vec<i32>>().unwrap();
            assert_eq!(v, v2);
        });
    }

    #[test]
    fn test_sort() {
        Python::with_gil(|py| {
            let v = vec![7, 3, 2, 5];
            let list = PyList::new(py, &v);
            assert_eq!(7, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(2, list.get_item(2).unwrap().extract::<i32>().unwrap());
            assert_eq!(5, list.get_item(3).unwrap().extract::<i32>().unwrap());
            list.sort().unwrap();
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(5, list.get_item(2).unwrap().extract::<i32>().unwrap());
            assert_eq!(7, list.get_item(3).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_reverse() {
        Python::with_gil(|py| {
            let v = vec![2, 3, 5, 7];
            let list = PyList::new(py, &v);
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(5, list.get_item(2).unwrap().extract::<i32>().unwrap());
            assert_eq!(7, list.get_item(3).unwrap().extract::<i32>().unwrap());
            list.reverse().unwrap();
            assert_eq!(7, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(5, list.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(2).unwrap().extract::<i32>().unwrap());
            assert_eq!(2, list.get_item(3).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_array_into_py() {
        Python::with_gil(|py| {
            let array: PyObject = [1, 2].into_py(py);
            let list = <PyList as PyTryFrom>::try_from(array.as_ref(py)).unwrap();
            assert_eq!(1, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(2, list.get_item(1).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_list_get_item_invalid_index() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[2, 3, 5, 7]);
            let obj = list.get_item(5);
            assert!(obj.is_err());
            assert_eq!(
                obj.unwrap_err().to_string(),
                "IndexError: list index out of range"
            );
        });
    }

    #[test]
    fn test_list_get_item_sanity() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[2, 3, 5, 7]);
            let obj = list.get_item(0);
            assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    #[test]
    fn test_list_get_item_unchecked_sanity() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[2, 3, 5, 7]);
            let obj = unsafe { list.get_item_unchecked(0) };
            assert_eq!(obj.extract::<i32>().unwrap(), 2);
        });
    }
}
