use crate::ffi::{self, Py_ssize_t};
use crate::ffi_ptr_ext::FfiPtrExt;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::instance::Borrowed;
use crate::internal_tricks::get_ssize_index;
use crate::types::{any::PyAnyMethods, sequence::PySequenceMethods, PyList, PySequence};
use crate::{
    exceptions, Bound, FromPyObject, IntoPyObject, IntoPyObjectExt, PyAny, PyErr, PyResult, Python,
};
use std::iter::FusedIterator;
#[cfg(feature = "nightly")]
use std::num::NonZero;

#[inline]
#[track_caller]
fn try_new_from_iter<'py>(
    py: Python<'py>,
    mut elements: impl ExactSizeIterator<Item = PyResult<Bound<'py, PyAny>>>,
) -> PyResult<Bound<'py, PyTuple>> {
    unsafe {
        // PyTuple_New checks for overflow but has a bad error message, so we check ourselves
        let len: Py_ssize_t = elements
            .len()
            .try_into()
            .expect("out of range integral type conversion attempted on `elements.len()`");

        let ptr = ffi::PyTuple_New(len);

        // - Panics if the ptr is null
        // - Cleans up the tuple if `convert` or the asserts panic
        let tup = ptr.assume_owned(py).cast_into_unchecked();

        let mut counter: Py_ssize_t = 0;

        for obj in (&mut elements).take(len as usize) {
            #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
            ffi::PyTuple_SET_ITEM(ptr, counter, obj?.into_ptr());
            #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
            ffi::PyTuple_SetItem(ptr, counter, obj?.into_ptr());
            counter += 1;
        }

        assert!(elements.next().is_none(), "Attempted to create PyTuple but `elements` was larger than reported by its `ExactSizeIterator` implementation.");
        assert_eq!(len, counter, "Attempted to create PyTuple but `elements` was smaller than reported by its `ExactSizeIterator` implementation.");

        Ok(tup)
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
    /// If you want to create a [`PyTuple`] with elements of different or unknown types, create a Rust
    /// tuple with the given elements and convert it at once using [`into_pyobject()`][crate::IntoPyObject].
    /// (`IntoPyObject` is implemented for tuples of up to 12 elements.)
    ///
    /// To create a [`PyTuple`] from an iterable that doesn't implement [`ExactSizeIterator`],
    /// collect the elements into a `Vec` first.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyTuple;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     let elements: Vec<i32> = vec![0, 1, 2, 3, 4, 5];
    ///     let tuple = PyTuple::new(py, elements)?;
    ///     assert_eq!(format!("{:?}", tuple), "(0, 1, 2, 3, 4, 5)");
    ///
    ///     // alternative using `into_pyobject()`
    ///     let tuple = (0, "hello", true).into_pyobject(py)?;
    ///     assert_eq!(format!("{:?}", tuple), "(0, 'hello', True)");
    /// # Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if `element`'s [`ExactSizeIterator`] implementation is incorrect.
    /// All standard library structures implement this trait correctly, if they do, so calling this
    /// function using [`Vec`]`<T>` or `&[T]` will always succeed.
    #[track_caller]
    pub fn new<'py, T, U>(
        py: Python<'py>,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> PyResult<Bound<'py, PyTuple>>
    where
        T: IntoPyObject<'py>,
        U: ExactSizeIterator<Item = T>,
    {
        let elements = elements.into_iter().map(|e| e.into_bound_py_any(py));
        try_new_from_iter(py, elements)
    }

    /// Constructs an empty tuple (on the Python side, a singleton object).
    pub fn empty(py: Python<'_>) -> Bound<'_, PyTuple> {
        unsafe { ffi::PyTuple_New(0).assume_owned(py).cast_into_unchecked() }
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
    /// use pyo3::prelude::*;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| -> PyResult<()> {
    ///     let tuple = (1, 2, 3).into_pyobject(py)?;
    ///     let obj = tuple.get_item(0);
    ///     assert_eq!(obj?.extract::<i32>()?, 1);
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
        V: IntoPyObject<'py>;

    /// Returns the first index `i` for which `self[i] == value`.
    ///
    /// This is equivalent to the Python expression `self.index(value)`.
    fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: IntoPyObject<'py>;

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
        unsafe { self.cast_unchecked() }
    }

    fn into_sequence(self) -> Bound<'py, PySequence> {
        unsafe { self.cast_into_unchecked() }
    }

    fn get_slice(&self, low: usize, high: usize) -> Bound<'py, PyTuple> {
        unsafe {
            ffi::PyTuple_GetSlice(self.as_ptr(), get_ssize_index(low), get_ssize_index(high))
                .assume_owned(self.py())
                .cast_into_unchecked()
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
        unsafe { self.get_borrowed_item_unchecked(index).to_owned() }
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    unsafe fn get_borrowed_item_unchecked<'a>(&'a self, index: usize) -> Borrowed<'a, 'py, PyAny> {
        unsafe { self.as_borrowed().get_borrowed_item_unchecked(index) }
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
        V: IntoPyObject<'py>,
    {
        self.as_sequence().contains(value)
    }

    #[inline]
    fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: IntoPyObject<'py>,
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
        unsafe {
            ffi::PyTuple_GET_ITEM(self.as_ptr(), index as Py_ssize_t).assume_borrowed(self.py())
        }
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

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    #[inline]
    #[cfg(not(feature = "nightly"))]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let length = self.length.min(self.tuple.len());
        let target_index = self.index + n;
        if target_index < length {
            let item = unsafe {
                BorrowedTupleIterator::get_item(self.tuple.as_borrowed(), target_index).to_owned()
            };
            self.index = target_index + 1;
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    #[cfg(feature = "nightly")]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        let max_len = self.length.min(self.tuple.len());
        let currently_at = self.index;
        if currently_at >= max_len {
            if n == 0 {
                return Ok(());
            } else {
                return Err(unsafe { NonZero::new_unchecked(n) });
            }
        }

        let items_left = max_len - currently_at;
        if n <= items_left {
            self.index += n;
            Ok(())
        } else {
            self.index = max_len;
            let remainder = n - items_left;
            Err(unsafe { NonZero::new_unchecked(remainder) })
        }
    }
}

impl DoubleEndedIterator for BoundTupleIterator<'_> {
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

    #[inline]
    #[cfg(not(feature = "nightly"))]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let length_size = self.length.min(self.tuple.len());
        if self.index + n < length_size {
            let target_index = length_size - n - 1;
            let item = unsafe {
                BorrowedTupleIterator::get_item(self.tuple.as_borrowed(), target_index).to_owned()
            };
            self.length = target_index;
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    #[cfg(feature = "nightly")]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        let max_len = self.length.min(self.tuple.len());
        let currently_at = self.index;
        if currently_at >= max_len {
            if n == 0 {
                return Ok(());
            } else {
                return Err(unsafe { NonZero::new_unchecked(n) });
            }
        }

        let items_left = max_len - currently_at;
        if n <= items_left {
            self.length = max_len - n;
            Ok(())
        } else {
            self.length = currently_at;
            let remainder = n - items_left;
            Err(unsafe { NonZero::new_unchecked(remainder) })
        }
    }
}

impl ExactSizeIterator for BoundTupleIterator<'_> {
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
        let item = unsafe { tuple.get_borrowed_item_unchecked(index) };
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

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }
}

impl DoubleEndedIterator for BorrowedTupleIterator<'_, '_> {
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

impl ExactSizeIterator for BorrowedTupleIterator<'_, '_> {
    fn len(&self) -> usize {
        self.length.saturating_sub(self.index)
    }
}

impl FusedIterator for BorrowedTupleIterator<'_, '_> {}

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
    impl <'py, $($T),+> IntoPyObject<'py> for ($($T,)+)
    where
        $($T: IntoPyObject<'py>,)+
    {
        type Target = PyTuple;
        type Output = Bound<'py, Self::Target>;
        type Error = PyErr;

        fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
            Ok(array_into_tuple(py, [$(self.$n.into_bound_py_any(py)?),+]))
        }

        #[cfg(feature = "experimental-inspect")]
        fn type_output() -> TypeInfo {
            TypeInfo::Tuple(Some(vec![$( $T::type_output() ),+]))
        }
    }

    impl <'a, 'py, $($T),+> IntoPyObject<'py> for &'a ($($T,)+)
    where
        $(&'a $T: IntoPyObject<'py>,)+
        $($T: 'a,)+ // MSRV
    {
        type Target = PyTuple;
        type Output = Bound<'py, Self::Target>;
        type Error = PyErr;

        fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
            Ok(array_into_tuple(py, [$(self.$n.into_bound_py_any(py)?),+]))
        }

        #[cfg(feature = "experimental-inspect")]
        fn type_output() -> TypeInfo {
            TypeInfo::Tuple(Some(vec![$( <&$T>::type_output() ),+]))
        }
    }

    impl<'py, $($T),+> crate::call::private::Sealed for ($($T,)+) where $($T: IntoPyObject<'py>,)+ {}
    impl<'py, $($T),+> crate::call::PyCallArgs<'py> for ($($T,)+)
    where
        $($T: IntoPyObject<'py>,)+
    {
        #[cfg(all(Py_3_9, not(any(PyPy, GraalPy, Py_LIMITED_API))))]
        fn call(
            self,
            function: Borrowed<'_, 'py, PyAny>,
            kwargs: Borrowed<'_, '_, crate::types::PyDict>,
            _: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            let py = function.py();
            // We need this to drop the arguments correctly.
            let args_bound = [$(self.$n.into_bound_py_any(py)?,)*];
            // Prepend one null argument for `PY_VECTORCALL_ARGUMENTS_OFFSET`.
            let mut args = [std::ptr::null_mut(), $(args_bound[$n].as_ptr()),*];
            unsafe {
                ffi::PyObject_VectorcallDict(
                    function.as_ptr(),
                    args.as_mut_ptr().add(1),
                    $length + ffi::PY_VECTORCALL_ARGUMENTS_OFFSET,
                    kwargs.as_ptr(),
                )
                .assume_owned_or_err(py)
            }
        }

        #[cfg(all(not(any(PyPy, GraalPy)), any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_12)))]
        fn call_positional(
            self,
            function: Borrowed<'_, 'py, PyAny>,
            _: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            let py = function.py();
            // We need this to drop the arguments correctly.
            let args_bound = [$(self.$n.into_bound_py_any(py)?,)*];

            #[cfg(not(Py_LIMITED_API))]
            if $length == 1 {
                return unsafe {
                    ffi::PyObject_CallOneArg(
                       function.as_ptr(),
                       args_bound[0].as_ptr()
                    )
                    .assume_owned_or_err(py)
                };
            }

            // Prepend one null argument for `PY_VECTORCALL_ARGUMENTS_OFFSET`.
            let mut args = [std::ptr::null_mut(), $(args_bound[$n].as_ptr()),*];
            unsafe {
                ffi::PyObject_Vectorcall(
                    function.as_ptr(),
                    args.as_mut_ptr().add(1),
                    $length + ffi::PY_VECTORCALL_ARGUMENTS_OFFSET,
                    std::ptr::null_mut(),
                )
                .assume_owned_or_err(py)
            }
        }

        #[cfg(all(not(any(PyPy, GraalPy)), any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_12)))]
        fn call_method_positional(
            self,
            object: Borrowed<'_, 'py, PyAny>,
            method_name: Borrowed<'_, 'py, crate::types::PyString>,
            _: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            let py = object.py();
            // We need this to drop the arguments correctly.
            let args_bound = [$(self.$n.into_bound_py_any(py)?,)*];

            #[cfg(not(Py_LIMITED_API))]
            if $length == 1 {
                return unsafe {
                    ffi::PyObject_CallMethodOneArg(
                            object.as_ptr(),
                            method_name.as_ptr(),
                            args_bound[0].as_ptr(),
                    )
                    .assume_owned_or_err(py)
                };
            }

            let mut args = [object.as_ptr(), $(args_bound[$n].as_ptr()),*];
            unsafe {
                ffi::PyObject_VectorcallMethod(
                    method_name.as_ptr(),
                    args.as_mut_ptr(),
                    // +1 for the receiver.
                    1 + $length + ffi::PY_VECTORCALL_ARGUMENTS_OFFSET,
                    std::ptr::null_mut(),
                )
                .assume_owned_or_err(py)
            }

        }

        #[cfg(not(all(Py_3_9, not(any(PyPy, GraalPy, Py_LIMITED_API)))))]
        fn call(
            self,
            function: Borrowed<'_, 'py, PyAny>,
            kwargs: Borrowed<'_, 'py, crate::types::PyDict>,
            token: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            self.into_pyobject_or_pyerr(function.py())?.call(function, kwargs, token)
        }

        #[cfg(not(all(not(any(PyPy, GraalPy)), any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_12))))]
        fn call_positional(
            self,
            function: Borrowed<'_, 'py, PyAny>,
            token: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            self.into_pyobject_or_pyerr(function.py())?.call_positional(function, token)
        }

        #[cfg(not(all(not(any(PyPy, GraalPy)), any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_12))))]
        fn call_method_positional(
            self,
            object: Borrowed<'_, 'py, PyAny>,
            method_name: Borrowed<'_, 'py, crate::types::PyString>,
            token: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            self.into_pyobject_or_pyerr(object.py())?.call_method_positional(object, method_name, token)
        }
    }

    impl<'a, 'py, $($T),+> crate::call::private::Sealed for &'a ($($T,)+) where $(&'a $T: IntoPyObject<'py>,)+ $($T: 'a,)+ /*MSRV */ {}
    impl<'a, 'py, $($T),+> crate::call::PyCallArgs<'py> for &'a ($($T,)+)
    where
        $(&'a $T: IntoPyObject<'py>,)+
        $($T: 'a,)+ // MSRV
    {
        #[cfg(all(Py_3_9, not(any(PyPy, GraalPy, Py_LIMITED_API))))]
        fn call(
            self,
            function: Borrowed<'_, 'py, PyAny>,
            kwargs: Borrowed<'_, '_, crate::types::PyDict>,
            _: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            let py = function.py();
            // We need this to drop the arguments correctly.
            let args_bound = [$(self.$n.into_bound_py_any(py)?,)*];
            // Prepend one null argument for `PY_VECTORCALL_ARGUMENTS_OFFSET`.
            let mut args = [std::ptr::null_mut(), $(args_bound[$n].as_ptr()),*];
            unsafe {
                ffi::PyObject_VectorcallDict(
                    function.as_ptr(),
                    args.as_mut_ptr().add(1),
                    $length + ffi::PY_VECTORCALL_ARGUMENTS_OFFSET,
                    kwargs.as_ptr(),
                )
                .assume_owned_or_err(py)
            }
        }

        #[cfg(all(not(any(PyPy, GraalPy)), any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_12)))]
        fn call_positional(
            self,
            function: Borrowed<'_, 'py, PyAny>,
            _: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            let py = function.py();
            // We need this to drop the arguments correctly.
            let args_bound = [$(self.$n.into_bound_py_any(py)?,)*];

            #[cfg(not(Py_LIMITED_API))]
            if $length == 1 {
                return unsafe {
                    ffi::PyObject_CallOneArg(
                       function.as_ptr(),
                       args_bound[0].as_ptr()
                    )
                    .assume_owned_or_err(py)
                };
            }

            // Prepend one null argument for `PY_VECTORCALL_ARGUMENTS_OFFSET`.
            let mut args = [std::ptr::null_mut(), $(args_bound[$n].as_ptr()),*];
            unsafe {
                ffi::PyObject_Vectorcall(
                    function.as_ptr(),
                    args.as_mut_ptr().add(1),
                    $length + ffi::PY_VECTORCALL_ARGUMENTS_OFFSET,
                    std::ptr::null_mut(),
                )
                .assume_owned_or_err(py)
            }
        }

        #[cfg(all(not(any(PyPy, GraalPy)), any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_12)))]
        fn call_method_positional(
            self,
            object: Borrowed<'_, 'py, PyAny>,
            method_name: Borrowed<'_, 'py, crate::types::PyString>,
            _: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            let py = object.py();
            // We need this to drop the arguments correctly.
            let args_bound = [$(self.$n.into_bound_py_any(py)?,)*];

            #[cfg(not(Py_LIMITED_API))]
            if $length == 1 {
                return unsafe {
                    ffi::PyObject_CallMethodOneArg(
                            object.as_ptr(),
                            method_name.as_ptr(),
                            args_bound[0].as_ptr(),
                    )
                    .assume_owned_or_err(py)
                };
            }

            let mut args = [object.as_ptr(), $(args_bound[$n].as_ptr()),*];
            unsafe {
                ffi::PyObject_VectorcallMethod(
                    method_name.as_ptr(),
                    args.as_mut_ptr(),
                    // +1 for the receiver.
                    1 + $length + ffi::PY_VECTORCALL_ARGUMENTS_OFFSET,
                    std::ptr::null_mut(),
                )
                .assume_owned_or_err(py)
            }
        }

        #[cfg(not(all(Py_3_9, not(any(PyPy, GraalPy, Py_LIMITED_API)))))]
        fn call(
            self,
            function: Borrowed<'_, 'py, PyAny>,
            kwargs: Borrowed<'_, 'py, crate::types::PyDict>,
            token: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            self.into_pyobject_or_pyerr(function.py())?.call(function, kwargs, token)
        }

        #[cfg(not(all(not(any(PyPy, GraalPy)), any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_12))))]
        fn call_positional(
            self,
            function: Borrowed<'_, 'py, PyAny>,
            token: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            self.into_pyobject_or_pyerr(function.py())?.call_positional(function, token)
        }

        #[cfg(not(all(not(any(PyPy, GraalPy)), any(all(Py_3_9, not(Py_LIMITED_API)), Py_3_12))))]
        fn call_method_positional(
            self,
            object: Borrowed<'_, 'py, PyAny>,
            method_name: Borrowed<'_, 'py, crate::types::PyString>,
            token: crate::call::private::Token,
        ) -> PyResult<Bound<'py, PyAny>> {
            self.into_pyobject_or_pyerr(object.py())?.call_method_positional(object, method_name, token)
        }
    }

    impl<'py, $($T: FromPyObject<'py>),+> FromPyObject<'py> for ($($T,)+) {
        fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self>
        {
            let t = obj.cast::<PyTuple>()?;
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

fn array_into_tuple<'py, const N: usize>(
    py: Python<'py>,
    array: [Bound<'py, PyAny>; N],
) -> Bound<'py, PyTuple> {
    unsafe {
        let ptr = ffi::PyTuple_New(N.try_into().expect("0 < N <= 12"));
        let tup = ptr.assume_owned(py).cast_into_unchecked();
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
    use crate::{IntoPyObject, Python};
    use std::collections::HashSet;
    #[cfg(feature = "nightly")]
    use std::num::NonZero;
    use std::ops::Range;
    #[test]
    fn test_new() {
        Python::attach(|py| {
            let ob = PyTuple::new(py, [1, 2, 3]).unwrap();
            assert_eq!(3, ob.len());
            let ob = ob.as_any();
            assert_eq!((1, 2, 3), ob.extract().unwrap());

            let mut map = HashSet::new();
            map.insert(1);
            map.insert(2);
            PyTuple::new(py, map).unwrap();
        });
    }

    #[test]
    fn test_len() {
        Python::attach(|py| {
            let ob = (1, 2, 3).into_pyobject(py).unwrap();
            let tuple = ob.cast::<PyTuple>().unwrap();
            assert_eq!(3, tuple.len());
            assert!(!tuple.is_empty());
            let ob = tuple.as_any();
            assert_eq!((1, 2, 3), ob.extract().unwrap());
        });
    }

    #[test]
    fn test_empty() {
        Python::attach(|py| {
            let tuple = PyTuple::empty(py);
            assert!(tuple.is_empty());
            assert_eq!(0, tuple.len());
        });
    }

    #[test]
    fn test_slice() {
        Python::attach(|py| {
            let tup = PyTuple::new(py, [2, 3, 5, 7]).unwrap();
            let slice = tup.get_slice(1, 3);
            assert_eq!(2, slice.len());
            let slice = tup.get_slice(1, 7);
            assert_eq!(3, slice.len());
        });
    }

    #[test]
    fn test_iter() {
        Python::attach(|py| {
            let ob = (1, 2, 3).into_pyobject(py).unwrap();
            let tuple = ob.cast::<PyTuple>().unwrap();
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
        Python::attach(|py| {
            let ob = (1, 2, 3).into_pyobject(py).unwrap();
            let tuple = ob.cast::<PyTuple>().unwrap();
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
        Python::attach(|py| {
            let tuple = PyTuple::new(py, [1, 2, 3]).unwrap();
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
        Python::attach(|py| {
            let tuple = PyTuple::new(py, [1, 2, 3]).unwrap();
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
        Python::attach(|py| {
            let ob = (1, 2, 3).into_pyobject(py).unwrap();
            let tuple = ob.cast::<PyTuple>().unwrap();
            assert_eq!(3, tuple.len());

            for (i, item) in tuple.iter().enumerate() {
                assert_eq!(i + 1, item.extract::<'_, usize>().unwrap());
            }
        });
    }

    #[test]
    fn test_into_iter_bound() {
        Python::attach(|py| {
            let tuple = (1, 2, 3).into_pyobject(py).unwrap();
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
        Python::attach(|py| {
            let ob = (1, 2, 3).into_pyobject(py).unwrap();
            let tuple = ob.cast::<PyTuple>().unwrap();

            let slice = tuple.as_slice();
            assert_eq!(3, slice.len());
            assert_eq!(1_i32, slice[0].extract::<'_, i32>().unwrap());
            assert_eq!(2_i32, slice[1].extract::<'_, i32>().unwrap());
            assert_eq!(3_i32, slice[2].extract::<'_, i32>().unwrap());
        });
    }

    #[test]
    fn test_tuple_lengths_up_to_12() {
        Python::attach(|py| {
            let t0 = (0,).into_pyobject(py).unwrap();
            let t1 = (0, 1).into_pyobject(py).unwrap();
            let t2 = (0, 1, 2).into_pyobject(py).unwrap();
            let t3 = (0, 1, 2, 3).into_pyobject(py).unwrap();
            let t4 = (0, 1, 2, 3, 4).into_pyobject(py).unwrap();
            let t5 = (0, 1, 2, 3, 4, 5).into_pyobject(py).unwrap();
            let t6 = (0, 1, 2, 3, 4, 5, 6).into_pyobject(py).unwrap();
            let t7 = (0, 1, 2, 3, 4, 5, 6, 7).into_pyobject(py).unwrap();
            let t8 = (0, 1, 2, 3, 4, 5, 6, 7, 8).into_pyobject(py).unwrap();
            let t9 = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9).into_pyobject(py).unwrap();
            let t10 = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
                .into_pyobject(py)
                .unwrap();
            let t11 = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
                .into_pyobject(py)
                .unwrap();

            assert_eq!(t0.extract::<(i32,)>().unwrap(), (0,));
            assert_eq!(t1.extract::<(i32, i32)>().unwrap(), (0, 1,));
            assert_eq!(t2.extract::<(i32, i32, i32)>().unwrap(), (0, 1, 2,));
            assert_eq!(
                t3.extract::<(i32, i32, i32, i32,)>().unwrap(),
                (0, 1, 2, 3,)
            );
            assert_eq!(
                t4.extract::<(i32, i32, i32, i32, i32,)>().unwrap(),
                (0, 1, 2, 3, 4,)
            );
            assert_eq!(
                t5.extract::<(i32, i32, i32, i32, i32, i32,)>().unwrap(),
                (0, 1, 2, 3, 4, 5,)
            );
            assert_eq!(
                t6.extract::<(i32, i32, i32, i32, i32, i32, i32,)>()
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6,)
            );
            assert_eq!(
                t7.extract::<(i32, i32, i32, i32, i32, i32, i32, i32,)>()
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7,)
            );
            assert_eq!(
                t8.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32,)>()
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8,)
            );
            assert_eq!(
                t9.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,)>()
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8, 9,)
            );
            assert_eq!(
                t10.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,)>()
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,)
            );
            assert_eq!(
                t11.extract::<(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32,)>()
                    .unwrap(),
                (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,)
            );
        })
    }

    #[test]
    fn test_tuple_get_item_invalid_index() {
        Python::attach(|py| {
            let ob = (1, 2, 3).into_pyobject(py).unwrap();
            let tuple = ob.cast::<PyTuple>().unwrap();
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
        Python::attach(|py| {
            let ob = (1, 2, 3).into_pyobject(py).unwrap();
            let tuple = ob.cast::<PyTuple>().unwrap();
            let obj = tuple.get_item(0);
            assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 1);
        });
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    #[test]
    fn test_tuple_get_item_unchecked_sanity() {
        Python::attach(|py| {
            let ob = (1, 2, 3).into_pyobject(py).unwrap();
            let tuple = ob.cast::<PyTuple>().unwrap();
            let obj = unsafe { tuple.get_item_unchecked(0) };
            assert_eq!(obj.extract::<i32>().unwrap(), 1);
        });
    }

    #[test]
    fn test_tuple_contains() {
        Python::attach(|py| {
            let ob = (1, 1, 2, 3, 5, 8).into_pyobject(py).unwrap();
            let tuple = ob.cast::<PyTuple>().unwrap();
            assert_eq!(6, tuple.len());

            let bad_needle = 7i32.into_pyobject(py).unwrap();
            assert!(!tuple.contains(&bad_needle).unwrap());

            let good_needle = 8i32.into_pyobject(py).unwrap();
            assert!(tuple.contains(&good_needle).unwrap());

            let type_coerced_needle = 8f32.into_pyobject(py).unwrap();
            assert!(tuple.contains(&type_coerced_needle).unwrap());
        });
    }

    #[test]
    fn test_tuple_index() {
        Python::attach(|py| {
            let ob = (1, 1, 2, 3, 5, 8).into_pyobject(py).unwrap();
            let tuple = ob.cast::<PyTuple>().unwrap();
            assert_eq!(0, tuple.index(1i32).unwrap());
            assert_eq!(2, tuple.index(2i32).unwrap());
            assert_eq!(3, tuple.index(3i32).unwrap());
            assert_eq!(4, tuple.index(5i32).unwrap());
            assert_eq!(5, tuple.index(8i32).unwrap());
            assert!(tuple.index(42i32).is_err());
        });
    }

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
        Python::attach(|py| {
            let iter = FaultyIter(0..usize::MAX, 73);
            let _tuple = PyTuple::new(py, iter);
        })
    }

    #[test]
    #[should_panic(
        expected = "Attempted to create PyTuple but `elements` was smaller than reported by its `ExactSizeIterator` implementation."
    )]
    fn too_short_iterator() {
        Python::attach(|py| {
            let iter = FaultyIter(0..35, 73);
            let _tuple = PyTuple::new(py, iter);
        })
    }

    #[test]
    #[should_panic(
        expected = "out of range integral type conversion attempted on `elements.len()`"
    )]
    fn overflowing_size() {
        Python::attach(|py| {
            let iter = FaultyIter(0..0, usize::MAX);

            let _tuple = PyTuple::new(py, iter);
        })
    }

    #[test]
    fn bad_intopyobject_doesnt_cause_leaks() {
        use crate::types::PyInt;
        use std::convert::Infallible;
        use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

        static NEEDS_DESTRUCTING_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct Bad(usize);

        impl Drop for Bad {
            fn drop(&mut self) {
                NEEDS_DESTRUCTING_COUNT.fetch_sub(1, SeqCst);
            }
        }

        impl<'py> IntoPyObject<'py> for Bad {
            type Target = PyInt;
            type Output = crate::Bound<'py, Self::Target>;
            type Error = Infallible;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                // This panic should not lead to a memory leak
                assert_ne!(self.0, 42);
                self.0.into_pyobject(py)
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

        Python::attach(|py| {
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

    #[test]
    fn bad_intopyobject_doesnt_cause_leaks_2() {
        use crate::types::PyInt;
        use std::convert::Infallible;
        use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

        static NEEDS_DESTRUCTING_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct Bad(usize);

        impl Drop for Bad {
            fn drop(&mut self) {
                NEEDS_DESTRUCTING_COUNT.fetch_sub(1, SeqCst);
            }
        }

        impl<'py> IntoPyObject<'py> for &Bad {
            type Target = PyInt;
            type Output = crate::Bound<'py, Self::Target>;
            type Error = Infallible;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                // This panic should not lead to a memory leak
                assert_ne!(self.0, 3);
                self.0.into_pyobject(py)
            }
        }

        let s = (Bad(1), Bad(2), Bad(3), Bad(4));
        NEEDS_DESTRUCTING_COUNT.store(4, SeqCst);
        Python::attach(|py| {
            std::panic::catch_unwind(|| {
                let _tuple = (&s).into_pyobject(py).unwrap();
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
        Python::attach(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3]).unwrap();
            let list = tuple.to_list();
            let list_expected = PyList::new(py, vec![1, 2, 3]).unwrap();
            assert!(list.eq(list_expected).unwrap());
        })
    }

    #[test]
    fn test_tuple_as_sequence() {
        Python::attach(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3]).unwrap();
            let sequence = tuple.as_sequence();
            assert!(tuple.get_item(0).unwrap().eq(1).unwrap());
            assert!(sequence.get_item(0).unwrap().eq(1).unwrap());

            assert_eq!(tuple.len(), 3);
            assert_eq!(sequence.len().unwrap(), 3);
        })
    }

    #[test]
    fn test_tuple_into_sequence() {
        Python::attach(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3]).unwrap();
            let sequence = tuple.into_sequence();
            assert!(sequence.get_item(0).unwrap().eq(1).unwrap());
            assert_eq!(sequence.len().unwrap(), 3);
        })
    }

    #[test]
    fn test_bound_tuple_get_item() {
        Python::attach(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3, 4]).unwrap();

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

    #[test]
    fn test_bound_tuple_nth() {
        Python::attach(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3, 4]).unwrap();
            let mut iter = tuple.iter();
            assert_eq!(iter.nth(1).unwrap().extract::<i32>().unwrap(), 2);
            assert_eq!(iter.nth(1).unwrap().extract::<i32>().unwrap(), 4);
            assert!(iter.nth(1).is_none());

            let tuple = PyTuple::new(py, Vec::<i32>::new()).unwrap();
            let mut iter = tuple.iter();
            iter.next();
            assert!(iter.nth(1).is_none());

            let tuple = PyTuple::new(py, vec![1, 2, 3]).unwrap();
            let mut iter = tuple.iter();
            assert!(iter.nth(10).is_none());

            let tuple = PyTuple::new(py, vec![6, 7, 8, 9, 10]).unwrap();
            let mut iter = tuple.iter();
            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 6);
            assert_eq!(iter.nth(2).unwrap().extract::<i32>().unwrap(), 9);
            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 10);

            let mut iter = tuple.iter();
            assert_eq!(iter.nth_back(1).unwrap().extract::<i32>().unwrap(), 9);
            assert_eq!(iter.nth(2).unwrap().extract::<i32>().unwrap(), 8);
            assert!(iter.next().is_none());
        });
    }

    #[test]
    fn test_bound_tuple_nth_back() {
        Python::attach(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3, 4, 5]).unwrap();
            let mut iter = tuple.iter();
            assert_eq!(iter.nth_back(0).unwrap().extract::<i32>().unwrap(), 5);
            assert_eq!(iter.nth_back(1).unwrap().extract::<i32>().unwrap(), 3);
            assert!(iter.nth_back(2).is_none());

            let tuple = PyTuple::new(py, Vec::<i32>::new()).unwrap();
            let mut iter = tuple.iter();
            assert!(iter.nth_back(0).is_none());
            assert!(iter.nth_back(1).is_none());

            let tuple = PyTuple::new(py, vec![1, 2, 3]).unwrap();
            let mut iter = tuple.iter();
            assert!(iter.nth_back(5).is_none());

            let tuple = PyTuple::new(py, vec![1, 2, 3, 4, 5]).unwrap();
            let mut iter = tuple.iter();
            iter.next_back(); // Consume the last element
            assert_eq!(iter.nth_back(1).unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(iter.next_back().unwrap().extract::<i32>().unwrap(), 2);
            assert_eq!(iter.nth_back(0).unwrap().extract::<i32>().unwrap(), 1);

            let tuple = PyTuple::new(py, vec![1, 2, 3, 4, 5]).unwrap();
            let mut iter = tuple.iter();
            assert_eq!(iter.nth_back(1).unwrap().extract::<i32>().unwrap(), 4);
            assert_eq!(iter.nth_back(2).unwrap().extract::<i32>().unwrap(), 1);

            let mut iter2 = tuple.iter();
            iter2.next_back();
            assert_eq!(iter2.nth_back(1).unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(iter2.next_back().unwrap().extract::<i32>().unwrap(), 2);

            let mut iter3 = tuple.iter();
            iter3.nth(1);
            assert_eq!(iter3.nth_back(2).unwrap().extract::<i32>().unwrap(), 3);
            assert!(iter3.nth_back(0).is_none());
        });
    }

    #[cfg(feature = "nightly")]
    #[test]
    fn test_bound_tuple_advance_by() {
        Python::attach(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3, 4, 5]).unwrap();
            let mut iter = tuple.iter();

            assert_eq!(iter.advance_by(2), Ok(()));
            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(iter.advance_by(0), Ok(()));
            assert_eq!(iter.advance_by(100), Err(NonZero::new(98).unwrap()));
            assert!(iter.next().is_none());

            let mut iter2 = tuple.iter();
            assert_eq!(iter2.advance_by(6), Err(NonZero::new(1).unwrap()));

            let mut iter3 = tuple.iter();
            assert_eq!(iter3.advance_by(5), Ok(()));

            let mut iter4 = tuple.iter();
            assert_eq!(iter4.advance_by(0), Ok(()));
            assert_eq!(iter4.next().unwrap().extract::<i32>().unwrap(), 1);
        })
    }

    #[cfg(feature = "nightly")]
    #[test]
    fn test_bound_tuple_advance_back_by() {
        Python::attach(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3, 4, 5]).unwrap();
            let mut iter = tuple.iter();

            assert_eq!(iter.advance_back_by(2), Ok(()));
            assert_eq!(iter.next_back().unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(iter.advance_back_by(0), Ok(()));
            assert_eq!(iter.advance_back_by(100), Err(NonZero::new(98).unwrap()));
            assert!(iter.next_back().is_none());

            let mut iter2 = tuple.iter();
            assert_eq!(iter2.advance_back_by(6), Err(NonZero::new(1).unwrap()));

            let mut iter3 = tuple.iter();
            assert_eq!(iter3.advance_back_by(5), Ok(()));

            let mut iter4 = tuple.iter();
            assert_eq!(iter4.advance_back_by(0), Ok(()));
            assert_eq!(iter4.next_back().unwrap().extract::<i32>().unwrap(), 5);
        })
    }

    #[test]
    fn test_iter_last() {
        Python::attach(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3]).unwrap();
            let last = tuple.iter().last();
            assert_eq!(last.unwrap().extract::<i32>().unwrap(), 3);
        })
    }

    #[test]
    fn test_iter_count() {
        Python::attach(|py| {
            let tuple = PyTuple::new(py, vec![1, 2, 3]).unwrap();
            assert_eq!(tuple.iter().count(), 3);
        })
    }
}
