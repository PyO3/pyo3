// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::err::{self, PyResult};
use crate::ffi::{self, Py_ssize_t};
use crate::{
    AsPyPointer, IntoPy, IntoPyPointer, PyAny, PyNativeType, PyObject, Python, ToBorrowedObject,
    ToPyObject,
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

    /// Gets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_item(&self, index: isize) -> &PyAny {
        assert!(index >= 0 && index < self.len() as isize);
        unsafe {
            #[cfg(not(Py_LIMITED_API))]
            let ptr = ffi::PyList_GET_ITEM(self.as_ptr(), index as Py_ssize_t);
            #[cfg(Py_LIMITED_API)]
            let ptr = ffi::PyList_GetItem(self.as_ptr(), index as Py_ssize_t);

            // PyList_GetItem return borrowed ptr; must make owned for safety (see #890).
            ffi::Py_INCREF(ptr);
            self.py().from_owned_ptr(ptr)
        }
    }

    /// Sets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn set_item<I>(&self, index: isize, item: I) -> PyResult<()>
    where
        I: ToPyObject,
    {
        unsafe {
            err::error_on_minusone(
                self.py(),
                ffi::PyList_SetItem(self.as_ptr(), index, item.to_object(self.py()).into_ptr()),
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
    /// Panics if the index is out of range.
    pub fn insert<I>(&self, index: isize, item: I) -> PyResult<()>
    where
        I: ToBorrowedObject,
    {
        item.with_borrowed_ptr(self.py(), |item| unsafe {
            err::error_on_minusone(self.py(), ffi::PyList_Insert(self.as_ptr(), index, item))
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
    index: isize,
}

impl<'a> Iterator for PyListIterator<'a> {
    type Item = &'a PyAny;

    #[inline]
    fn next(&mut self) -> Option<&'a PyAny> {
        if self.index < self.list.len() as isize {
            let item = self.list.get_item(self.index);
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
            len.saturating_sub(self.index as usize),
            Some(len.saturating_sub(self.index as usize)),
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
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[2, 3, 5, 7]);
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>().unwrap());
        assert_eq!(5, list.get_item(2).extract::<i32>().unwrap());
        assert_eq!(7, list.get_item(3).extract::<i32>().unwrap());
    }

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[1, 2, 3, 4]);
        assert_eq!(4, list.len());
    }

    #[test]
    fn test_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[2, 3, 5, 7]);
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>().unwrap());
        assert_eq!(5, list.get_item(2).extract::<i32>().unwrap());
        assert_eq!(7, list.get_item(3).extract::<i32>().unwrap());
    }

    #[test]
    #[should_panic]
    fn test_get_item_invalid() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[2, 3, 5, 7]);
        list.get_item(-1);
    }

    #[test]
    fn test_set_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[2, 3, 5, 7]);
        let val = 42i32.to_object(py);
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        list.set_item(0, val).unwrap();
        assert_eq!(42, list.get_item(0).extract::<i32>().unwrap());
    }

    #[test]
    fn test_set_item_refcnt() {
        let gil = Python::acquire_gil();
        let py = gil.python();

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
    }

    #[test]
    fn test_insert() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[2, 3, 5, 7]);
        let val = 42i32.to_object(py);
        assert_eq!(4, list.len());
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        list.insert(0, val).unwrap();
        assert_eq!(5, list.len());
        assert_eq!(42, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(2, list.get_item(1).extract::<i32>().unwrap());
    }

    #[test]
    fn test_insert_refcnt() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let cnt;
        {
            let _pool = unsafe { crate::GILPool::new() };
            let list = PyList::empty(py);
            let none = py.None();
            cnt = none.get_refcnt(py);
            list.insert(0, none).unwrap();
        }

        assert_eq!(cnt, py.None().get_refcnt(py));
    }

    #[test]
    fn test_append() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[2]);
        list.append(3).unwrap();
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>().unwrap());
    }

    #[test]
    fn test_append_refcnt() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let cnt;
        {
            let _pool = unsafe { crate::GILPool::new() };
            let list = PyList::empty(py);
            let none = py.None();
            cnt = none.get_refcnt(py);
            list.append(none).unwrap();
        }
        assert_eq!(cnt, py.None().get_refcnt(py));
    }

    #[test]
    fn test_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::new(py, &v);
        let mut idx = 0;
        for el in list.iter() {
            assert_eq!(v[idx], el.extract::<i32>().unwrap());
            idx += 1;
        }
        assert_eq!(idx, v.len());
    }

    #[test]
    fn test_iter_size_hint() {
        let gil = Python::acquire_gil();
        let py = gil.python();
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
    }

    #[test]
    fn test_into_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[1, 2, 3, 4]);
        for (i, item) in list.iter().enumerate() {
            assert_eq!((i + 1) as i32, item.extract::<i32>().unwrap());
        }
    }

    #[test]
    fn test_extract() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::new(py, &v);
        let v2 = list.as_ref().extract::<Vec<i32>>().unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_sort() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![7, 3, 2, 5];
        let list = PyList::new(py, &v);
        assert_eq!(7, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>().unwrap());
        assert_eq!(2, list.get_item(2).extract::<i32>().unwrap());
        assert_eq!(5, list.get_item(3).extract::<i32>().unwrap());
        list.sort().unwrap();
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>().unwrap());
        assert_eq!(5, list.get_item(2).extract::<i32>().unwrap());
        assert_eq!(7, list.get_item(3).extract::<i32>().unwrap());
    }

    #[test]
    fn test_reverse() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::new(py, &v);
        assert_eq!(2, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>().unwrap());
        assert_eq!(5, list.get_item(2).extract::<i32>().unwrap());
        assert_eq!(7, list.get_item(3).extract::<i32>().unwrap());
        list.reverse().unwrap();
        assert_eq!(7, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(5, list.get_item(1).extract::<i32>().unwrap());
        assert_eq!(3, list.get_item(2).extract::<i32>().unwrap());
        assert_eq!(2, list.get_item(3).extract::<i32>().unwrap());
    }

    #[test]
    fn test_array_into_py() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let array: PyObject = [1, 2].into_py(py);
        let list = <PyList as PyTryFrom>::try_from(array.as_ref(py)).unwrap();
        assert_eq!(1, list.get_item(0).extract::<i32>().unwrap());
        assert_eq!(2, list.get_item(1).extract::<i32>().unwrap());
    }
}
