use std::iter::FusedIterator;

use crate::conversion::IntoPyObject;
use crate::ffi::{self, Py_ssize_t};
use crate::ffi_ptr_ext::FfiPtrExt;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::instance::Borrowed;
use crate::internal_tricks::get_ssize_index;
use crate::types::{any::PyAnyMethods, sequence::PySequenceMethods, PyList, PySequence};
use crate::{
    exceptions, Bound, BoundObject, FromPyObject, IntoPy, Py, PyAny, PyErr, PyObject, PyResult,
    Python, ToPyObject,
};

#[inline]
#[track_caller]
fn new_from_iter<'py>(
    py: Python<'py>,
    elements: &mut dyn ExactSizeIterator<Item = PyObject>,
) -> Bound<'py, PyTuple> {
    unsafe {
        // PyTuple_New checks for overflow but has a bad error message, so we check ourselves
        let len: Py_ssize_t = elements
            .len()
            .try_into()
            .expect("out of range integral type conversion attempted on `elements.len()`");

        let ptr = ffi::PyTuple_New(len);

        // - Panics if the ptr is null
        // - Cleans up the tuple if `convert` or the asserts panic
        let tup = ptr.assume_owned(py).downcast_into_unchecked();

        let mut counter: Py_ssize_t = 0;

        for obj in elements.take(len as usize) {
            #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
            ffi::PyTuple_SET_ITEM(ptr, counter, obj.into_ptr());
            #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
            ffi::PyTuple_SetItem(ptr, counter, obj.into_ptr());
            counter += 1;
        }

        assert!(elements.next().is_none(), "Attempted to create PyTuple but `elements` was larger than reported by its `ExactSizeIterator` implementation.");
        assert_eq!(len, counter, "Attempted to create PyTuple but `elements` was smaller than reported by its `ExactSizeIterator` implementation.");

        tup
    }
}

/// Represents a Python `tuple` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyTuple>`][crate::Py] or [`Bound<'py, PyTuple>`][Bound].
///
/// For APIs available on `tuple` objects, see the [`PyTupleMethods`] trait which is implemented for
/// [`Bound<'py, PyTuple>`][Bound].
#[repr(transparent)]
pub struct PyTuple(PyAny);

pyobject_native_type_core!(PyTuple, pyobject_native_static_type_object!(ffi::PyTuple_Type), #checkfunction=ffi::PyTuple_Check);

impl PyTuple {
    /// Constructs a new tuple with the given elements.
    ///
    /// If you want to create a [`PyTuple`] with elements of different or unknown types, or from an
    /// iterable that doesn't implement [`ExactSizeIterator`], create a Rust tuple with the given
    /// elements and convert it at once using `into_py`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyTuple;
    ///
    /// # fn main() {
    /// Python::with_gil(|py| {
    ///     let elements: Vec<i32> = vec![0, 1, 2, 3, 4, 5];
    ///     let tuple = PyTuple::new(py, elements);
    ///     assert_eq!(format!("{:?}", tuple), "(0, 1, 2, 3, 4, 5)");
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if `element`'s [`ExactSizeIterator`] implementation is incorrect.
    /// All standard library structures implement this trait correctly, if they do, so calling this
    /// function using [`Vec`]`<T>` or `&[T]` will always succeed.
    #[track_caller]
    pub fn new<T, U>(
        py: Python<'_>,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> Bound<'_, PyTuple>
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>,
    {
        let mut elements = elements.into_iter().map(|e| e.to_object(py));
        new_from_iter(py, &mut elements)
    }

    /// Deprecated name for [`PyTuple::new`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyTuple::new`")]
    #[track_caller]
    #[inline]
    pub fn new_bound<T, U>(
        py: Python<'_>,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> Bound<'_, PyTuple>
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>,
    {
        PyTuple::new(py, elements)
    }

    /// Constructs an empty tuple (on the Python side, a singleton object).
    pub fn empty(py: Python<'_>) -> Bound<'_, PyTuple> {
        unsafe {
            ffi::PyTuple_New(0)
                .assume_owned(py)
                .downcast_into_unchecked()
        }
    }

    /// Deprecated name for [`PyTuple::empty`].
    #[deprecated(since = "0.23.0", note = "renamed to `PyTuple::empty`")]
    #[inline]
    pub fn empty_bound(py: Python<'_>) -> Bound<'_, PyTuple> {
        PyTuple::empty(py)
    }
}

/// Implementation of functionality for [`PyTuple`].
///
/// These methods are defined for the `Bound<'py, PyTuple>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyTuple")]
pub trait PyTupleMethods<'py>: crate::sealed::Sealed {
    /// Gets the length of the tuple.
    fn len(&self) -> usize;

    /// Checks if the tuple is empty.
    fn is_empty(&self) -> bool;

    /// Returns `self` cast as a `PySequence`.
    fn as_sequence(&self) -> &Bound<'py, PySequence>;

    /// Returns `self` cast as a `PySequence`.
    fn into_sequence(self) -> Bound<'py, PySequence>;

    /// Takes the slice `self[low:high]` and returns it as a new tuple.
    ///
    /// Indices must be nonnegative, and out-of-range indices are clipped to
    /// `self.len()`.
    fn get_slice(&self, low: usize, high: usize) -> Bound<'py, PyTuple>;

    /// Gets the tuple item at the specified index.
    /// # Example
    /// ```
    /// use pyo3::{prelude::*, types::PyTuple};
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| -> PyResult<()> {
    ///     let ob = (1, 2, 3).to_object(py);
    ///     let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();
    ///     let obj = tuple.get_item(0);
    ///     assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    fn get_item(&self, index: usize) -> PyResult<Bound<'py, PyAny>>;

    /// Like [`get_item`][PyTupleMethods::get_item], but returns a borrowed object, which is a slight performance optimization
    /// by avoiding a reference count change.
    fn get_borrowed_item<'a>(&'a self, index: usize) -> PyResult<Borrowed<'a, 'py, PyAny>>;

    /// Gets the tuple item at the specified index. Undefined behavior on bad index. Use with caution.
    ///
    /// # Safety
    ///
    /// Caller must verify that the index is within the bounds of the tuple.
    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    unsafe fn get_item_unchecked(&self, index: usize) -> Bound<'py, PyAny>;

    /// Like [`get_item_unchecked`][PyTupleMethods::get_item_unchecked], but returns a borrowed object,
    /// which is a slight performance optimization by avoiding a reference count change.
    ///
    /// # Safety
    ///
    /// Caller must verify that the index is within the bounds of the tuple.
    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    unsafe fn get_borrowed_item_unchecked<'a>(&'a self, index: usize) -> Borrowed<'a, 'py, PyAny>;

    /// Returns `self` as a slice of objects.
    #[cfg(not(any(Py_LIMITED_API, GraalPy)))]
    fn as_slice(&self) -> &[Bound<'py, PyAny>];

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToPyObject;

    /// Returns the first index `i` for which `self[i] == value`.
    ///
    /// This is equivalent to the Python expression `self.index(value)`.
    fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: ToPyObject;

    /// Returns an iterator over the tuple items.
    fn iter(&self) -> BoundTupleIterator<'py>;

    /// Like [`iter`][PyTupleMethods::iter], but produces an iterator which returns borrowed objects,
    /// which is a slight performance optimization by avoiding a reference count change.
    fn iter_borrowed<'a>(&'a self) -> BorrowedTupleIterator<'a, 'py>;

    /// Return a new list containing the contents of this tuple; equivalent to the Python expression `list(tuple)`.
    ///
    /// This method is equivalent to `self.as_sequence().to_list()` and faster than `PyList::new(py, self)`.
    fn to_list(&self) -> Bound<'py, PyList>;
}

impl<'py> PyTupleMethods<'py> for Bound<'py, PyTuple> {
    fn len(&self) -> usize {
        unsafe {
            #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
            let size = ffi::PyTuple_GET_SIZE(self.as_ptr());
            #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
            let size = ffi::PyTuple_Size(self.as_ptr());
            // non-negative Py_ssize_t should always fit into Rust uint
            size as usize
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn as_sequence(&self) -> &Bound<'py, PySequence> {
        unsafe { self.downcast_unchecked() }
    }

    fn into_sequence(self) -> Bound<'py, PySequence> {
        unsafe { self.into_any().downcast_into_unchecked() }
    }

    fn get_slice(&self, low: usize, high: usize) -> Bound<'py, PyTuple> {
        unsafe {
            ffi::PyTuple_GetSlice(self.as_ptr(), get_ssize_index(low), get_ssize_index(high))
                .assume_owned(self.py())
                .downcast_into_unchecked()
        }
    }

    fn get_item(&self, index: usize) -> PyResult<Bound<'py, PyAny>> {
        self.get_borrowed_item(index).map(Borrowed::to_owned)
    }

    fn get_borrowed_item<'a>(&'a self, index: usize) -> PyResult<Borrowed<'a, 'py, PyAny>> {
        self.as_borrowed().get_borrowed_item(index)
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    unsafe fn get_item_unchecked(&self, index: usize) -> Bound<'py, PyAny> {
        self.get_borrowed_item_unchecked(index).to_owned()
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    unsafe fn get_borrowed_item_unchecked<'a>(&'a self, index: usize) -> Borrowed<'a, 'py, PyAny> {
        self.as_borrowed().get_borrowed_item_unchecked(index)
    }

    #[cfg(not(any(Py_LIMITED_API, GraalPy)))]
    fn as_slice(&self) -> &[Bound<'py, PyAny>] {
        // SAFETY: self is known to be a tuple object, and tuples are immutable
        let items = unsafe { &(*self.as_ptr().cast::<ffi::PyTupleObject>()).ob_item };
        // SAFETY: Bound<'py, PyAny> has the same memory layout as *mut ffi::PyObject
        unsafe { std::slice::from_raw_parts(items.as_ptr().cast(), self.len()) }
    }

    #[inline]
    fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToPyObject,
    {
        self.as_sequence().contains(value)
    }

    #[inline]
    fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: ToPyObject,
    {
        self.as_sequence().index(value)
    }

    fn iter(&self) -> BoundTupleIterator<'py> {
        BoundTupleIterator::new(self.clone())
    }

    fn iter_borrowed<'a>(&'a self) -> BorrowedTupleIterator<'a, 'py> {
        self.as_borrowed().iter_borrowed()
    }

    fn to_list(&self) -> Bound<'py, PyList> {
        self.as_sequence()
            .to_list()
            .expect("failed to convert tuple to list")
    }
}

impl<'a, 'py> Borrowed<'a, 'py, PyTuple> {
    fn get_borrowed_item(self, index: usize) -> PyResult<Borrowed<'a, 'py, PyAny>> {
        unsafe {
            ffi::PyTuple_GetItem(self.as_ptr(), index as Py_ssize_t)
                .assume_borrowed_or_err(self.py())
        }
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    unsafe fn get_borrowed_item_unchecked(self, index: usize) -> Borrowed<'a, 'py, PyAny> {
        ffi::PyTuple_GET_ITEM(self.as_ptr(), index as Py_ssize_t).assume_borrowed(self.py())
    }

    pub(crate) fn iter_borrowed(self) -> BorrowedTupleIterator<'a, 'py> {
        BorrowedTupleIterator::new(self)
    }
}

/// Used by `PyTuple::into_iter()`.
pub struct BoundTupleIterator<'py> {
    tuple: Bound<'py, PyTuple>,
    index: usize,
    length: usize,
}

impl<'py> BoundTupleIterator<'py> {
    fn new(tuple: Bound<'py, PyTuple>) -> Self {
        let length = tuple.len();
        BoundTupleIterator {
            tuple,
            index: 0,
            length,
        }
    }
}

impl<'py> Iterator for BoundTupleIterator<'py> {
    type Item = Bound<'py, PyAny>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            let item = unsafe {
                BorrowedTupleIterator::get_item(self.tuple.as_borrowed(), self.index).to_owned()
            };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'py> DoubleEndedIterator for BoundTupleIterator<'py> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            let item = unsafe {
                BorrowedTupleIterator::get_item(self.tuple.as_borrowed(), self.length - 1)
                    .to_owned()
            };
            self.length -= 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<'py> ExactSizeIterator for BoundTupleIterator<'py> {
    fn len(&self) -> usize {
        self.length.saturating_sub(self.index)
    }
}

impl FusedIterator for BoundTupleIterator<'_> {}

impl<'py> IntoIterator for Bound<'py, PyTuple> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundTupleIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        BoundTupleIterator::new(self)
    }
}

impl<'py> IntoIterator for &Bound<'py, PyTuple> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundTupleIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Used by `PyTuple::iter_borrowed()`.
pub struct BorrowedTupleIterator<'a, 'py> {
    tuple: Borrowed<'a, 'py, PyTuple>,
    index: usize,
    length: usize,
}

impl<'a, 'py> BorrowedTupleIterator<'a, 'py> {
    fn new(tuple: Borrowed<'a, 'py, PyTuple>) -> Self {
        let length = tuple.len();
        BorrowedTupleIterator {
            tuple,
            index: 0,
            length,
        }
    }

    unsafe fn get_item(
        tuple: Borrowed<'a, 'py, PyTuple>,
        index: usize,
    ) -> Borrowed<'a, 'py, PyAny> {
        #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
        let item = tuple.get_borrowed_item(index).expect("tuple.get failed");
        #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
        let item = tuple.get_borrowed_item_unchecked(index);
        item
    }
}

impl<'a, 'py> Iterator for BorrowedTupleIterator<'a, 'py> {
    type Item = Borrowed<'a, 'py, PyAny>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            let item = unsafe { Self::get_item(self.tuple, self.index) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, 'py> DoubleEndedIterator for BorrowedTupleIterator<'a, 'py> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            let item = unsafe { Self::get_item(self.tuple, self.length - 1) };
            self.length -= 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<'a, 'py> ExactSizeIterator for BorrowedTupleIterator<'a, 'py> {
    fn len(&self) -> usize {
        self.length.saturating_sub(self.index)
    }
}

impl FusedIterator for BorrowedTupleIterator<'_, '_> {}

impl IntoPy<Py<PyTuple>> for Bound<'_, PyTuple> {
    fn into_py(self, _: Python<'_>) -> Py<PyTuple> {
        self.unbind()
    }
}

impl IntoPy<Py<PyTuple>> for &'_ Bound<'_, PyTuple> {
    fn into_py(self, _: Python<'_>) -> Py<PyTuple> {
        self.clone().unbind()
    }
}

#[cold]
fn wrong_tuple_length(t: &Bound<'_, PyTuple>, expected_length: usize) -> PyErr {
    let msg = format!(
        "expected tuple of length {}, but got tuple of length {}",
        expected_length,
        t.len()
    );
    exceptions::PyValueError::new_err(msg)
}

macro_rules! tuple_conversion ({$length:expr,$(($refN:ident, $n:tt, $T:ident)),+} => {
    impl <$($T: ToPyObject),+> ToPyObject for ($($T,)+) {
        fn to_object(&self, py: Python<'_>) -> PyObject {
            array_into_tuple(py, [$(self.$n.to_object(py)),+]).into()
        }
    }
    impl <$($T: IntoPy<PyObject>),+> IntoPy<PyObject> for ($($T,)+) {
        fn into_py(self, py: Python<'_>) -> PyObject {
            array_into_tuple(py, [$(self.$n.into_py(py)),+]).into()
        }

        #[cfg(feature = "experimental-inspect")]
fn type_output() -> TypeInfo {
            TypeInfo::Tuple(Some(vec![$( $T::type_output() ),+]))
        }
    }

    impl <'py, $($T),+> IntoPyObject<'py> for ($($T,)+)
    where
        $($T: IntoPyObject<'py>,)+
        PyErr: $(From<$T::Error> + )+
    {
        type Target = PyTuple;
        type Output = Bound<'py, Self::Target>;
        type Error = PyErr;

        fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
            Ok(array_into_tuple(py, [$(self.$n.into_pyobject(py)?.into_any().unbind()),+]).into_bound(py))
        }
    }

    impl <$($T: IntoPy<PyObject>),+> IntoPy<Py<PyTuple>> for ($($T,)+) {
        fn into_py(self, py: Python<'_>) -> Py<PyTuple> {
            array_into_tuple(py, [$(self.$n.into_py(py)),+])
        }

        #[cfg(feature = "experimental-inspect")]
        fn type_output() -> TypeInfo {
            TypeInfo::Tuple(Some(vec![$( $T::type_output() ),+]))
        }
    }

    impl<'py, $($T: FromPyObject<'py>),+> FromPyObject<'py> for ($($T,)+) {
        fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self>
        {
            let t = obj.downcast::<PyTuple>()?;
            if t.len() == $length {
                #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
                return Ok(($(t.get_borrowed_item($n)?.extract::<$T>()?,)+));

                #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
                unsafe {return Ok(($(t.get_borrowed_item_unchecked($n).extract::<$T>()?,)+));}
            } else {
                Err(wrong_tuple_length(t, $length))
            }
        }

        #[cfg(feature = "experimental-inspect")]
fn type_input() -> TypeInfo {
            TypeInfo::Tuple(Some(vec![$( $T::type_input() ),+]))
        }
    }
});

fn array_into_tuple<const N: usize>(py: Python<'_>, array: [PyObject; N]) -> Py<PyTuple> {
    unsafe {
        let ptr = ffi::PyTuple_New(N.try_into().expect("0 < N <= 12"));
        let tup = Py::from_owned_ptr(py, ptr);
        for (index, obj) in array.into_iter().enumerate() {
            #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
            ffi::PyTuple_SET_ITEM(ptr, index as ffi::Py_ssize_t, obj.into_ptr());
            #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
            ffi::PyTuple_SetItem(ptr, index as ffi::Py_ssize_t, obj.into_ptr());
        }
        tup
    }
}

tuple_conversion!(1, (ref0, 0, T0));
tuple_conversion!(2, (ref0, 0, T0), (ref1, 1, T1));
tuple_conversion!(3, (ref0, 0, T0), (ref1, 1, T1), (ref2, 2, T2));
tuple_conversion!(
    4,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3)
);
tuple_conversion!(
    5,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4)
);
tuple_conversion!(
    6,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5)
);
tuple_conversion!(
    7,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6)
);
tuple_conversion!(
    8,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6),
    (ref7, 7, T7)
);
tuple_conversion!(
    9,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6),
    (ref7, 7, T7),
    (ref8, 8, T8)
);
tuple_conversion!(
    10,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6),
    (ref7, 7, T7),
    (ref8, 8, T8),
    (ref9, 9, T9)
);
tuple_conversion!(
    11,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6),
    (ref7, 7, T7),
    (ref8, 8, T8),
    (ref9, 9, T9),
    (ref10, 10, T10)
);

tuple_conversion!(
    12,
    (ref0, 0, T0),
    (ref1, 1, T1),
    (ref2, 2, T2),
    (ref3, 3, T3),
    (ref4, 4, T4),
    (ref5, 5, T5),
    (ref6, 6, T6),
    (ref7, 7, T7),
    (ref8, 8, T8),
    (ref9, 9, T9),
    (ref10, 10, T10),
    (ref11, 11, T11)
);

#[cfg(test)]
mod tests {
    use crate::types::{any::PyAnyMethods, tuple::PyTupleMethods, PyList, PyTuple};
    use crate::{Python, ToPyObject};
    use std::collections::HashSet;

    #[test]
    fn test_new() {
        Python::with_gil(|py| {
            let ob = PyTuple::new(py, [1, 2, 3]);
            assert_eq!(3, ob.len());
            let ob = ob.as_any();
            assert_eq!((1, 2, 3), ob.extract().unwrap());

            let mut map = HashSet::new();
            map.insert(1);
            map.insert(2);
            PyTuple::new(py, map);
        });
    }

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();
            assert_eq!(3, tuple.len());
            assert!(!tuple.is_empty());
            let ob = tuple.as_any();
            assert_eq!((1, 2, 3), ob.extract().unwrap());
        });
    }

    #[test]
    fn test_empty() {
        Python::with_gil(|py| {
            let tuple = PyTuple::empty(py);
            assert!(tuple.is_empty());
            assert_eq!(0, tuple.len());
        });
    }

    #[test]
    fn test_slice() {
        Python::with_gil(|py| {
            let tup = PyTuple::new(py, [2, 3, 5, 7]);
            let slice = tup.get_slice(1, 3);
            assert_eq!(2, slice.len());
            let slice = tup.get_slice(1, 7);
            assert_eq!(3, slice.len());
        });
    }

    #[test]
    fn test_iter() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();
            assert_eq!(3, tuple.len());
            let mut iter = tuple.iter();

            assert_eq!(iter.size_hint(), (3, Some(3)));

            assert_eq!(1_i32, iter.next().unwrap().extract::<'_, i32>().unwrap());
            assert_eq!(iter.size_hint(), (2, Some(2)));

            assert_eq!(2_i32, iter.next().unwrap().extract::<'_, i32>().unwrap());
            assert_eq!(iter.size_hint(), (1, Some(1)));

            assert_eq!(3_i32, iter.next().unwrap().extract::<'_, i32>().unwrap());
            assert_eq!(iter.size_hint(), (0, Some(0)));

            assert!(iter.next().is_none());
            assert!(iter.next().is_none());
        });
    }

    #[test]
    fn test_iter_rev() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();
            assert_eq!(3, tuple.len());
            let mut iter = tuple.iter().rev();

            assert_eq!(iter.size_hint(), (3, Some(3)));

            assert_eq!(3_i32, iter.next().unwrap().extract::<'_, i32>().unwrap());
            assert_eq!(iter.size_hint(), (2, Some(2)));

            assert_eq!(2_i32, iter.next().unwrap().extract::<'_, i32>().unwrap());
            assert_eq!(iter.size_hint(), (1, Some(1)));

            assert_eq!(1_i32, iter.next().unwrap().extract::<'_, i32>().unwrap());
            assert_eq!(iter.size_hint(), (0, Some(0)));

            assert!(iter.next().is_none());
            assert!(iter.next().is_none());
        });
    }

    #[test]
    fn test_bound_iter() {
        Python::with_gil(|py| {
            let tuple = PyTuple::new(py, [1, 2, 3]);
            assert_eq!(3, tuple.len());
            let mut iter = tuple.iter();

            assert_eq!(iter.size_hint(), (3, Some(3)));

            assert_eq!(1, iter.next().unwrap().extract::<i32>().unwrap());
            assert_eq!(iter.size_hint(), (2, Some(2)));

            assert_eq!(2, iter.next().unwrap().extract::<i32>().unwrap());
            assert_eq!(iter.size_hint(), (1, Some(1)));

            assert_eq!(3, iter.next().unwrap().extract::<i32>().unwrap());
            assert_eq!(iter.size_hint(), (0, Some(0)));

            assert!(iter.next().is_none());
            assert!(iter.next().is_none());
        });
    }

    #[test]
    fn test_bound_iter_rev() {
        Python::with_gil(|py| {
            let tuple = PyTuple::new(py, [1, 2, 3]);
            assert_eq!(3, tuple.len());
            let mut iter = tuple.iter().rev();

            assert_eq!(iter.size_hint(), (3, Some(3)));

            assert_eq!(3, iter.next().unwrap().extract::<i32>().unwrap());
            assert_eq!(iter.size_hint(), (2, Some(2)));

            assert_eq!(2, iter.next().unwrap().extract::<i32>().unwrap());
            assert_eq!(iter.size_hint(), (1, Some(1)));

            assert_eq!(1, iter.next().unwrap().extract::<i32>().unwrap());
            assert_eq!(iter.size_hint(), (0, Some(0)));

            assert!(iter.next().is_none());
            assert!(iter.next().is_none());
        });
    }

    #[test]
    fn test_into_iter() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();
            assert_eq!(3, tuple.len());

            for (i, item) in tuple.iter().enumerate() {
                assert_eq!(i + 1, item.extract::<'_, usize>().unwrap());
            }
        });
    }

    #[test]
    fn test_into_iter_bound() {
        use crate::Bound;

        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple: &Bound<'_, PyTuple> = ob.downcast_bound(py).unwrap();
            assert_eq!(3, tuple.len());

            let mut items = vec![];
            for item in tuple {
                items.push(item.extract::<usize>().unwrap());
            }
            assert_eq!(items, vec![1, 2, 3]);
        });
    }

    #[test]
    #[cfg(not(any(Py_LIMITED_API, GraalPy)))]
    fn test_as_slice() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();

            let slice = tuple.as_slice();
            assert_eq!(3, slice.len());
            assert_eq!(1_i32, slice[0].extract::<'_, i32>().unwrap());
            assert_eq!(2_i32, slice[1].extract::<'_, i32>().unwrap());
            assert_eq!(3_i32, slice[2].extract::<'_, i32>().unwrap());
        });
    }

    #[test]
    fn test_tuple_lengths_up_to_12() {
        Python::with_gil(|py| {
            let t0 = (0,).to_object(py);
            let t1 = (0, 1).to_object(py);
            let t2 = (0, 1, 2).to_object(py);
            let t3 = (0, 1, 2, 3).to_object(py);
            let t4 = (0, 1, 2, 3, 4).to_object(py);
            let t5 = (0, 1, 2, 3, 4, 5).to_object(py);
            let t6 = (0, 1, 2, 3, 4, 5, 6).to_object(py);
            let t7 = (0, 1, 2, 3, 4, 5, 6, 7).to_object(py);
            let t8 = (0, 1, 2, 3, 4, 5, 6, 7, 8).to_object(py);
            let t9 = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9).to_object(py);
            let t10 = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10).to_object(py);
            let t11 = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11).to_object(py);

            assert_eq!(t0.extract::<(i32,)>(py).unwrap(), (0,));
            assert_eq!(t1.extract::<(i32, i32)>(py).unwrap(), (0, 1,));
            assert_eq!(t2.extract::<(i32, i32, i32)>(py).unwrap(), (0, 1, 2,));
            assert_eq!(
                t3.extract::<(i32, i32, i32, i32,)>(py).unwrap(),
                (0, 1, 2, 3,)
            );
            assert_eq!(
                t4.extract::<(i32, i32, i32, i32, i32,)>(py).unwrap(),
                (0, 1, 2, 3, 4,)
            );
            assert_eq!(
                t5.extract::<(i32, i32, i32, i32, i32, i32,)>(py).unwrap(),
                (0, 1, 2, 3, 4, 5,)
            );
            assert_eq!(
                t6.extract::<(i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6,)
            );
            assert_eq!(
                t7.extract::<(i32, i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7,)
            );
            assert_eq!(
                t8.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8,)
            );
            assert_eq!(
                t9.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8, 9,)
            );
            assert_eq!(
                t10.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,)
            );
            assert_eq!(
                t11.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,)>(py)
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,)
            );
        })
    }

    #[test]
    fn test_tuple_get_item_invalid_index() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();
            let obj = tuple.get_item(5);
            assert!(obj.is_err());
            assert_eq!(
                obj.unwrap_err().to_string(),
                "IndexError: tuple index out of range"
            );
        });
    }

    #[test]
    fn test_tuple_get_item_sanity() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();
            let obj = tuple.get_item(0);
            assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    #[test]
    fn test_tuple_get_item_unchecked_sanity() {
        Python::with_gil(|py| {
            let ob = (1, 2, 3).to_object(py);
            let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();
            let obj = unsafe { tuple.get_item_unchecked(0) };
            assert_eq!(obj.extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_tuple_contains() {
        Python::with_gil(|py| {
            let ob = (1, 1, 2, 3, 5, 8).to_object(py);
            let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();
            assert_eq!(6, tuple.len());

            let bad_needle = 7i32.to_object(py);
            assert!(!tuple.contains(&bad_needle).unwrap());

            let good_needle = 8i32.to_object(py);
            assert!(tuple.contains(&good_needle).unwrap());

            let type_coerced_needle = 8f32.to_object(py);
            assert!(tuple.contains(&type_coerced_needle).unwrap());
        });
    }

    #[test]
    fn test_tuple_index() {
        Python::with_gil(|py| {
            let ob = (1, 1, 2, 3, 5, 8).to_object(py);
            let tuple = ob.downcast_bound::<PyTuple>(py).unwrap();
            assert_eq!(0, tuple.index(1i32).unwrap());
            assert_eq!(2, tuple.index(2i32).unwrap());
            assert_eq!(3, tuple.index(3i32).unwrap());
            assert_eq!(4, tuple.index(5i32).unwrap());
            assert_eq!(5, tuple.index(8i32).unwrap());
            assert!(tuple.index(42i32).is_err());
        });
    }

    use std::ops::Range;

    // An iterator that lies about its `ExactSizeIterator` implementation.
    // See https://github.com/PyO3/pyo3/issues/2118
    struct FaultyIter(Range<usize>, usize);

    impl Iterator for FaultyIter {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }
    }

    impl ExactSizeIterator for FaultyIter {
        fn len(&self) -> usize {
            self.1
        }
    }

    #[test]
    #[should_panic(
        expected = "Attempted to create PyTuple but `elements` was larger than reported by its `ExactSizeIterator` implementation."
    )]
    fn too_long_iterator() {
        Python::with_gil(|py| {
            let iter = FaultyIter(0..usize::MAX, 73);
            let _tuple = PyTuple::new(py, iter);
        })
    }

    #[test]
    #[should_panic(
        expected = "Attempted to create PyTuple but `elements` was smaller than reported by its `ExactSizeIterator` implementation."
    )]
    fn too_short_iterator() {
        Python::with_gil(|py| {
            let iter = FaultyIter(0..35, 73);
            let _tuple = PyTuple::new(py, iter);
        })
    }

    #[test]
    #[should_panic(
        expected = "out of range integral type conversion attempted on `elements.len()`"
    )]
    fn overflowing_size() {
        Python::with_gil(|py| {
            let iter = FaultyIter(0..0, usize::MAX);

            let _tuple = PyTuple::new(py, iter);
        })
    }

    #[cfg(feature = "macros")]
    #[test]
    fn bad_clone_mem_leaks() {
        use crate::{IntoPy, Py, PyAny};
        use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

        static NEEDS_DESTRUCTING_COUNT: AtomicUsize = AtomicUsize::new(0);

        #[crate::pyclass]
        #[pyo3(crate = "crate")]
        struct Bad(usize);

        impl Clone for Bad {
            fn clone(&self) -> Self {
                // This panic should not lead to a memory leak
                assert_ne!(self.0, 42);
                NEEDS_DESTRUCTING_COUNT.fetch_add(1, SeqCst);

                Bad(self.0)
            }
        }

        impl Drop for Bad {
            fn drop(&mut self) {
                NEEDS_DESTRUCTING_COUNT.fetch_sub(1, SeqCst);
            }
        }

        impl ToPyObject for Bad {
            fn to_object(&self, py: Python<'_>) -> Py<PyAny> {
                self.to_owned().into_py(py)
            }
        }

        struct FaultyIter(Range<usize>, usize);

        impl Iterator for FaultyIter {
            type Item = Bad;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.next().map(|i| {
                    NEEDS_DESTRUCTING_COUNT.fetch_add(1, SeqCst);
                    Bad(i)
                })
            }
        }

        impl ExactSizeIterator for FaultyIter {
            fn len(&self) -> usize {
                self.1
            }
        }

        Python::with_gil(|py| {
            std::panic::catch_unwind(|| {
                let iter = FaultyIter(0..50, 50);
                let _tuple = PyTuple::new(py, iter);
            })
            .unwrap_err();
        });

        assert_eq!(
            NEEDS_DESTRUCTING_COUNT.load(SeqCst),
            0,
            "Some destructors did not run"
        );
    }

    #[cfg(feature = "macros")]
    #[test]
    fn bad_clone_mem_leaks_2() {
        use crate::{IntoPy, Py, PyAny};
        use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

        static NEEDS_DESTRUCTING_COUNT: AtomicUsize = AtomicUsize::new(0);

        #[crate::pyclass]
        #[pyo3(crate = "crate")]
        struct Bad(usize);

        impl Clone for Bad {
            fn clone(&self) -> Self {
                // This panic should not lead to a memory leak
                assert_ne!(self.0, 3);
                NEEDS_DESTRUCTING_COUNT.fetch_add(1, SeqCst);

                Bad(self.0)
            }
        }

        impl Drop for Bad {
            fn drop(&mut self) {
                NEEDS_DESTRUCTING_COUNT.fetch_sub(1, SeqCst);
            }
        }

        impl ToPyObject for Bad {
            fn to_object(&self, py: Python<'_>) -> Py<PyAny> {
                self.to_owned().into_py(py)
            }
        }

        let s = (Bad(1), Bad(2), Bad(3), Bad(4));
        NEEDS_DESTRUCTING_COUNT.store(4, SeqCst);
        Python::with_gil(|py| {
            std::panic::catch_unwind(|| {
                let _tuple: Py<PyAny> = s.to_object(py);
            })
            .unwrap_err();
        });
        drop(s);

        assert_eq!(
            NEEDS_DESTRUCTING_COUNT.load(SeqCst),
            0,
            "Some destructors did not run"
        );
    }

    #[test]
    fn test_tuple_to_list() {
        Python::with_gil(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3]);
            let list = tuple.to_list();
            let list_expected = PyList::new(py, vec![1, 2, 3]);
            assert!(list.eq(list_expected).unwrap());
        })
    }

    #[test]
    fn test_tuple_as_sequence() {
        Python::with_gil(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3]);
            let sequence = tuple.as_sequence();
            assert!(tuple.get_item(0).unwrap().eq(1).unwrap());
            assert!(sequence.get_item(0).unwrap().eq(1).unwrap());

            assert_eq!(tuple.len(), 3);
            assert_eq!(sequence.len().unwrap(), 3);
        })
    }

    #[test]
    fn test_tuple_into_sequence() {
        Python::with_gil(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3]);
            let sequence = tuple.into_sequence();
            assert!(sequence.get_item(0).unwrap().eq(1).unwrap());
            assert_eq!(sequence.len().unwrap(), 3);
        })
    }

    #[test]
    fn test_bound_tuple_get_item() {
        Python::with_gil(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3, 4]);

            assert_eq!(tuple.len(), 4);
            assert_eq!(tuple.get_item(0).unwrap().extract::<i32>().unwrap(), 1);
            assert_eq!(
                tuple
                    .get_borrowed_item(1)
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                2
            );
            #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
            {
                assert_eq!(
                    unsafe { tuple.get_item_unchecked(2) }
                        .extract::<i32>()
                        .unwrap(),
                    3
                );
                assert_eq!(
                    unsafe { tuple.get_borrowed_item_unchecked(3) }
                        .extract::<i32>()
                        .unwrap(),
                    4
                );
            }
        })
    }
}
