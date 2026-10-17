//! Python lists and related types.

use crate::err::{self, PyResult};
use crate::ffi::{self, Py_ssize_t};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::internal_tricks::get_ssize_index;
#[allow(unused_imports, reason = "used to build docs")]
use crate::platform::prelude::*;
use crate::types::sequence::PySequenceMethods;
use crate::types::{PySequence, PyTuple};
use crate::{Bound, BoundObject, IntoPyObject, IntoPyObjectExt, PyAny, PyErr, Python};
#[cfg(RustPython)]
use crate::{
    Py,
    sync::PyOnceLock,
    types::{PyType, PyTypeMethods},
};
use core::iter::FusedIterator;
#[cfg(feature = "nightly")]
use core::num::NonZero;

/// Represents a Python `list`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyList>`][crate::Py] or [`Bound<'py, PyList>`][Bound].
///
/// For APIs available on `list` objects, see the [`PyListMethods`] trait which is implemented for
/// [`Bound<'py, PyList>`][Bound].
#[repr(transparent)]
pub struct PyList(PyAny);

#[cfg(not(RustPython))]
pyobject_native_type_core!(
    PyList,
    pyobject_native_static_type_object!(ffi::PyList_Type),
    "builtins", "list",
    #checkfunction=ffi::PyList_Check
);

#[cfg(RustPython)]
pyobject_native_type_core!(
    PyList,
    |py| {
        static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        TYPE.import(py, "builtins", "list").unwrap().as_type_ptr()
    },
    "builtins",
    "list",
    #checkfunction=ffi::PyList_Check
);

#[cfg(Py_3_12)]
impl crate::impl_::pyclass::PyClassBaseType for PyList {
    type LayoutAsBase = crate::impl_::pycell::PyVariableClassObjectBase;
    type BaseNativeType = Self;
    type Initializer = crate::impl_::pyclass_init::PyNativeTypeInitializer<Self>;
    type PyClassMutability = crate::pycell::impl_::ImmutableClass;
    type Layout<T: crate::impl_::pyclass::PyClassImpl> =
        crate::impl_::pycell::PyVariableClassObject<T>;
}

impl PyList {
    /// Constructs a new list with the given elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyList;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     let elements: Vec<i32> = vec![0, 1, 2, 3, 4, 5];
    ///     let list = PyList::new(py, elements)?;
    ///     assert_eq!(format!("{:?}", list), "[0, 1, 2, 3, 4, 5]");
    /// # Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if `element`'s [`Iterator::size_hint`] implementation is incorrect.
    /// All standard library structures implement this trait correctly, if they do, so calling this
    /// function with (for example) [`Vec`]`<T>` or `&[T]` will always succeed.
    #[track_caller]
    pub fn new<'py, T>(
        py: Python<'py>,
        elements: impl IntoIterator<Item = T>,
    ) -> PyResult<Bound<'py, PyList>>
    where
        T: IntoPyObject<'py>,
    {
        let mut elements = elements.into_iter().map(|e| e.into_bound_py_any(py));
        let (min_len, _) = elements.size_hint();

        // PyList_New checks for overflow but has a bad error message, so we check ourselves
        let len: Py_ssize_t = min_len
            .try_into()
            .expect("out of range integral type conversion attempted on `elements.len()`");

        #[cfg(all(Py_LIMITED_API, Py_GIL_DISABLED, not(RustPython)))]
        if len >= list_extend::MIN_LEN && py.is_free_threaded_runtime() {
            return list_extend::new_from_iter(py, elements, len);
        }

        let list = unsafe { ffi::PyList_New(len).assume_owned(py).cast_into_unchecked() };

        let count = (&mut elements)
            .take(len as usize)
            .try_fold(0, |count, item| unsafe {
                #[cfg(not(Py_LIMITED_API))]
                ffi::PyList_SET_ITEM(list.as_ptr(), count, item?.into_ptr());
                #[cfg(Py_LIMITED_API)]
                ffi::PyList_SetItem(list.as_ptr(), count, item?.into_ptr());
                Ok::<_, PyErr>(count + 1)
            })?;

        assert_eq!(
            len, count,
            "Attempted to create PyList but `elements` was smaller than reported by its `size_hint` implementation."
        );

        elements.try_for_each(|item| list.append(item?))?;

        Ok(list)
    }

    /// Constructs a new empty list.
    pub fn empty(py: Python<'_>) -> Bound<'_, PyList> {
        unsafe { ffi::PyList_New(0).assume_owned(py).cast_into_unchecked() }
    }
}

/// Builds lists on free-threaded CPython through `PySequence_List` instead of `PyList_SetItem`.
///
/// Under the limited API, `PyList_SetItem` takes the list's critical section for every item
/// (https://github.com/python/cpython/issues/158660). `PySequence_List` builds the list from a
/// `pyo3.ListBuildIterator` instance. CPython's list-extension loop preallocates from the
/// instance's length callback and holds the list's critical section across the loop. CPython's
/// list-extension loop steals each new reference returned by the instance's `tp_iternext`
/// callback, avoiding an extra reference-count operation. The `tp_iternext` callback calls a Rust
/// closure stored in thread-local state instead of a PyO3 `__next__` trampoline.
///
/// `PyType_FromSpec` creates the `pyo3.ListBuildIterator` type without the `macros` feature.
/// A `#[pymethods]` `__next__` trampoline costs more per item than the critical section.
/// `pyo3.ListBuildIterator` instances are not GC-tracked, so `gc.get_objects()` cannot expose
/// an instance while `PyList::new` builds a list.
#[cfg(all(Py_LIMITED_API, Py_GIL_DISABLED, not(RustPython)))]
mod list_extend {
    use super::PyList;
    use crate::err::PyResult;
    use crate::exceptions::PySystemError;
    use crate::ffi::{self, Py_ssize_t};
    use crate::ffi_ptr_ext::FfiPtrExt;
    use crate::impl_::panic::PanicTrap;
    #[cfg(wip_feature_std)]
    use crate::panic::PanicException;
    use crate::sync::PyOnceLock;
    use crate::types::PyType;
    use crate::{Bound, Py, PyAny, PyErr, Python};
    use core::cell::Cell;
    use core::ptr::{self, NonNull};

    /// Below this conservative cutoff, allocating a `ListBuildIterator` may outweigh the
    /// per-item savings, so `PyList::new` continues to use `PyList_SetItem`.
    pub(super) const MIN_LEN: Py_ssize_t = 32;

    /// The `ItemProducer` closure returns a new reference, NULL on exhaustion, or NULL with a
    /// placeholder exception after an element conversion error.
    type ItemProducer = dyn FnMut() -> *mut ffi::PyObject;

    crate::platform::thread_local! {
        /// Item producer and length for a list currently being built on this thread.
        static ACTIVE_BUILD: Cell<Option<(NonNull<ItemProducer>, Py_ssize_t)>> = const { Cell::new(None) };
    }

    unsafe extern "C" fn next(_iterator: *mut ffi::PyObject) -> *mut ffi::PyObject {
        let trap = PanicTrap::new("uncaught panic while building a PyList");
        let item = match ACTIVE_BUILD.with(Cell::get) {
            // SAFETY: `new_from_iter` keeps the item producer alive while `PySequence_List`
            // runs. Without an active list build, `next` returns end-of-iteration.
            Some((producer, _)) => unsafe { (*producer.as_ptr())() },
            None => ptr::null_mut(),
        };
        trap.disarm();
        item
    }

    unsafe extern "C" fn len(_iterator: *mut ffi::PyObject) -> Py_ssize_t {
        ACTIVE_BUILD.with(Cell::get).map_or(0, |(_, len)| len)
    }

    // TODO(subinterpreters): Cache the `ListBuildIterator` heap type per interpreter once PyO3
    // supports subinterpreters.
    fn get_iterator_type(py: Python<'_>) -> &Py<PyType> {
        static ITERATOR_TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        ITERATOR_TYPE.get_or_init(py, || {
            let mut slots = [
                ffi::PyType_Slot {
                    slot: ffi::Py_tp_iter,
                    pfunc: ffi::PyObject_SelfIter as *mut _,
                },
                ffi::PyType_Slot {
                    slot: ffi::Py_tp_iternext,
                    pfunc: next as *mut _,
                },
                ffi::PyType_Slot {
                    slot: ffi::Py_sq_length,
                    pfunc: len as *mut _,
                },
                ffi::PyType_Slot {
                    slot: 0,
                    pfunc: ptr::null_mut(),
                },
            ];
            let mut spec = ffi::PyType_Spec {
                name: c"pyo3.ListBuildIterator".as_ptr(),
                basicsize: 0,
                itemsize: 0,
                flags: ffi::Py_TPFLAGS_DEFAULT as _,
                slots: slots.as_mut_ptr(),
            };
            // SAFETY: `spec` and `slots` remain valid until `PyType_FromSpec` returns.
            unsafe {
                ffi::PyType_FromSpec(&mut spec)
                    .assume_owned_or_err(py)
                    .expect("failed to create the list build iterator type")
                    .cast_into_unchecked::<PyType>()
                    .unbind()
            }
        })
    }

    /// Restore the previous `ItemProducer` pointer after a nested list build.
    struct RestoreBuild(Option<(NonNull<ItemProducer>, Py_ssize_t)>);

    impl Drop for RestoreBuild {
        fn drop(&mut self) {
            ACTIVE_BUILD.with(|current| current.set(self.0));
        }
    }

    #[track_caller]
    pub(super) fn new_from_iter<'py>(
        py: Python<'py>,
        mut elements: impl Iterator<Item = PyResult<Bound<'py, PyAny>>>,
        len: Py_ssize_t,
    ) -> PyResult<Bound<'py, PyList>> {
        // Allocate a `ListBuildIterator` for this list. `ACTIVE_BUILD` holds the item producer.
        // A nested list build allocates another `ListBuildIterator` and restores the previous
        // item producer.
        // SAFETY: PyType_GenericAlloc returns a new reference or NULL with an exception set.
        let iterator = unsafe {
            ffi::PyType_GenericAlloc(get_iterator_type(py).as_ptr().cast(), 0)
                .assume_owned_or_err(py)?
        };
        let mut count: Py_ssize_t = 0;
        let mut failure: Option<PyErr> = None;
        let list = {
            let mut producer = || {
                // Catch panics before returning to CPython. PyErr::fetch resumes the panic below.
                // Without std, the PanicTrap in `next` aborts instead.
                #[cfg(wip_feature_std)]
                let next =
                    std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| elements.next()))
                        .unwrap_or_else(|payload| {
                            Some(Err(PanicException::from_panic_payload(payload)))
                        });
                #[cfg(not(wip_feature_std))]
                let next = elements.next();
                match next {
                    Some(Ok(item)) => {
                        count += 1;
                        return item.into_ptr();
                    }
                    None => return ptr::null_mut(),
                    // Keep the conversion error on the Rust side: `PySequence_List` treats
                    // `StopIteration` as exhaustion and could silently truncate the list.
                    Some(Err(err)) => failure = Some(err),
                }
                // `PySystemError` makes `PySequence_List` fail instead of treating the
                // conversion error as exhaustion.
                PySystemError::new_err("PyList::new: element conversion failed").restore(py);
                ptr::null_mut()
            };
            let producer: &mut (dyn FnMut() -> *mut ffi::PyObject + '_) = &mut producer;
            // SAFETY: `transmute` erases the borrow lifetime. `next` uses the producer pointer
            // only while `PySequence_List` runs. `RestoreBuild` restores the previous
            // `ACTIVE_BUILD` value before the producer closure is dropped.
            let producer: NonNull<ItemProducer> =
                unsafe { core::mem::transmute(NonNull::from(producer)) };
            let _restore =
                RestoreBuild(ACTIVE_BUILD.with(|current| current.replace(Some((producer, len)))));
            // SAFETY: PySequence_List returns a new list or NULL with an exception set.
            unsafe { ffi::PySequence_List(iterator.as_ptr()).assume_owned_or_err(py) }
        };
        if let Some(err) = failure {
            // `PyErr::fetch` resumes a caught panic stored as `PanicException`.
            err.restore(py);
            return Err(PyErr::fetch(py));
        }
        let list = list?;
        assert!(
            count >= len,
            "Attempted to create PyList but `elements` was smaller than reported by its `size_hint` implementation."
        );
        // SAFETY: PySequence_List returns a list.
        Ok(unsafe { list.cast_into_unchecked() })
    }
}

/// Implementation of functionality for [`PyList`].
///
/// These methods are defined for the `Bound<'py, PyList>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyList")]
pub trait PyListMethods<'py>: crate::sealed::Sealed {
    /// Returns the length of the list.
    fn len(&self) -> usize;

    /// Checks if the list is empty.
    fn is_empty(&self) -> bool;

    /// Returns `self` cast as a `PySequence`.
    fn as_sequence(&self) -> &Bound<'py, PySequence>;

    /// Returns `self` cast as a `PySequence`.
    fn into_sequence(self) -> Bound<'py, PySequence>;

    /// Gets the list item at the specified index.
    /// # Example
    /// ```
    /// use pyo3::{prelude::*, types::PyList};
    /// Python::attach(|py| {
    ///     let list = PyList::new(py, [2, 3, 5, 7]).unwrap();
    ///     let obj = list.get_item(0);
    ///     assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 2);
    /// });
    /// ```
    fn get_item(&self, index: usize) -> PyResult<Bound<'py, PyAny>>;

    /// Gets the list item at the specified index without checking bounds or synchronizing access.
    /// Undefined behavior if index is out of bounds.
    ///
    /// # Safety
    ///
    /// - Caller must verify that the index is within the bounds of the list.
    /// - On the free-threaded build, caller must verify they have exclusive access to the list
    ///   via a lock or by holding the innermost critical section on the list.
    unsafe fn get_item_unchecked(&self, index: usize) -> Bound<'py, PyAny>;

    /// Takes the slice `self[low:high]` and returns it as a new list.
    ///
    /// Indices must be nonnegative, and out-of-range indices are clipped to
    /// `self.len()`.
    fn get_slice(&self, low: usize, high: usize) -> Bound<'py, PyList>;

    /// Sets the item at the specified index.
    ///
    /// Raises `IndexError` if the index is out of range.
    fn set_item<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: IntoPyObject<'py>;

    /// Deletes the `index`th element of self.
    ///
    /// This is equivalent to the Python statement `del self[i]`.
    fn del_item(&self, index: usize) -> PyResult<()>;

    /// Assigns the sequence `seq` to the slice of `self` from `low` to `high`.
    ///
    /// This is equivalent to the Python statement `self[low:high] = v`.
    fn set_slice(&self, low: usize, high: usize, seq: &Bound<'_, PyAny>) -> PyResult<()>;

    /// Deletes the slice from `low` to `high` from `self`.
    ///
    /// This is equivalent to the Python statement `del self[low:high]`.
    fn del_slice(&self, low: usize, high: usize) -> PyResult<()>;

    /// Appends an item to the list.
    fn append<I>(&self, item: I) -> PyResult<()>
    where
        I: IntoPyObject<'py>;

    /// Inserts an item at the specified index.
    ///
    /// If `index >= self.len()`, inserts at the end.
    fn insert<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: IntoPyObject<'py>;

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

    /// Returns an iterator over this list's items.
    fn iter(&self) -> BoundListIterator<'py>;

    /// Iterates over the contents of this list while holding a critical section on the list.
    /// This is useful when the GIL is disabled and the list is shared between threads.
    /// It is not guaranteed that the list will not be modified during iteration when the
    /// closure calls arbitrary Python code that releases the critical section held by the
    /// iterator. Otherwise, the list will not be modified during iteration.
    ///
    /// This is equivalent to for_each if the GIL is enabled.
    fn locked_for_each<F>(&self, closure: F) -> PyResult<()>
    where
        F: Fn(Bound<'py, PyAny>) -> PyResult<()>;

    /// Sorts the list in-place. Equivalent to the Python expression `l.sort()`.
    fn sort(&self) -> PyResult<()>;

    /// Reverses the list in-place. Equivalent to the Python expression `l.reverse()`.
    fn reverse(&self) -> PyResult<()>;

    /// Return a new tuple containing the contents of the list; equivalent to the Python expression `tuple(list)`.
    ///
    /// This method is equivalent to `self.as_sequence().to_tuple()` and faster than `PyTuple::new(py, this_list)`.
    fn to_tuple(&self) -> Bound<'py, PyTuple>;
}

impl<'py> PyListMethods<'py> for Bound<'py, PyList> {
    /// Returns the length of the list.
    fn len(&self) -> usize {
        let size = cfg_select! {
            // SAFETY: self is valid list object
            not(Py_LIMITED_API) => unsafe { ffi::PyList_GET_SIZE(self.as_ptr()) },
            // SAFETY: as above
            Py_LIMITED_API => unsafe { ffi::PyList_Size(self.as_ptr()) },
        };

        // non-negative Py_ssize_t should always fit into Rust usize
        size as usize
    }

    /// Checks if the list is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `self` cast as a `PySequence`.
    fn as_sequence(&self) -> &Bound<'py, PySequence> {
        unsafe { self.cast_unchecked() }
    }

    /// Returns `self` cast as a `PySequence`.
    fn into_sequence(self) -> Bound<'py, PySequence> {
        unsafe { self.cast_into_unchecked() }
    }

    /// Gets the list item at the specified index.
    /// # Example
    /// ```
    /// use pyo3::{prelude::*, types::PyList};
    /// Python::attach(|py| {
    ///     let list = PyList::new(py, [2, 3, 5, 7]).unwrap();
    ///     let obj = list.get_item(0);
    ///     assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 2);
    /// });
    /// ```
    fn get_item(&self, index: usize) -> PyResult<Bound<'py, PyAny>> {
        unsafe {
            ffi::compat::PyList_GetItemRef(self.as_ptr(), index as Py_ssize_t)
                .assume_owned_or_err(self.py())
        }
    }

    unsafe fn get_item_unchecked(&self, index: usize) -> Bound<'py, PyAny> {
        cfg_select! {
            // SAFETY:
            // - thread is attached to the interpreter
            // - self is a valid list object
            // - caller has guaranteed index is in bounds
            // - PyLIST_GET_ITEM is known to always return a borrowed pointer
            not(Py_LIMITED_API) => unsafe {
                ffi::PyList_GET_ITEM(self.as_ptr(), index as Py_ssize_t)
                    .assume_borrowed_unchecked(self.py())
            }.to_owned(),
            // SAFETY:
            // - thread is attached to the interpreter
            // - self is a valid list object
            // - caller has guaranteed index is in bounds
            // - PyList_GetItemRef is known to return an owned pointer in these conditions
            Py_LIMITED_API => unsafe {
                ffi::compat::PyList_GetItemRef(self.as_ptr(), index as Py_ssize_t)
                    .assume_owned_unchecked(self.py())
            }
        }
    }

    /// Takes the slice `self[low:high]` and returns it as a new list.
    ///
    /// Indices must be nonnegative, and out-of-range indices are clipped to
    /// `self.len()`.
    fn get_slice(&self, low: usize, high: usize) -> Bound<'py, PyList> {
        unsafe {
            ffi::PyList_GetSlice(self.as_ptr(), get_ssize_index(low), get_ssize_index(high))
                .assume_owned(self.py())
                .cast_into_unchecked()
        }
    }

    /// Sets the item at the specified index.
    ///
    /// Raises `IndexError` if the index is out of range.
    fn set_item<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: IntoPyObject<'py>,
    {
        let py = self.py();
        let item = item.into_bound_py_any(py)?;
        err::error_on_negative(py, unsafe {
            ffi::PyList_SetItem(self.as_ptr(), get_ssize_index(index), item.into_ptr())
        })
    }

    /// Deletes the `index`th element of self.
    ///
    /// This is equivalent to the Python statement `del self[i]`.
    #[inline]
    fn del_item(&self, index: usize) -> PyResult<()> {
        self.as_sequence().del_item(index)
    }

    /// Assigns the sequence `seq` to the slice of `self` from `low` to `high`.
    ///
    /// This is equivalent to the Python statement `self[low:high] = v`.
    #[inline]
    fn set_slice(&self, low: usize, high: usize, seq: &Bound<'_, PyAny>) -> PyResult<()> {
        err::error_on_negative(self.py(), unsafe {
            ffi::PyList_SetSlice(
                self.as_ptr(),
                get_ssize_index(low),
                get_ssize_index(high),
                seq.as_ptr(),
            )
        })
    }

    /// Deletes the slice from `low` to `high` from `self`.
    ///
    /// This is equivalent to the Python statement `del self[low:high]`.
    #[inline]
    fn del_slice(&self, low: usize, high: usize) -> PyResult<()> {
        self.as_sequence().del_slice(low, high)
    }

    /// Appends an item to the list.
    fn append<I>(&self, item: I) -> PyResult<()>
    where
        I: IntoPyObject<'py>,
    {
        let py = self.py();
        let item = item.into_pyobject_or_pyerr(py)?;
        err::error_on_negative(py, unsafe {
            ffi::PyList_Append(self.as_ptr(), item.as_ptr())
        })
    }

    /// Inserts an item at the specified index.
    ///
    /// If `index >= self.len()`, inserts at the end.
    fn insert<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: IntoPyObject<'py>,
    {
        let py = self.py();
        let item = item.into_pyobject_or_pyerr(py)?;
        err::error_on_negative(py, unsafe {
            ffi::PyList_Insert(self.as_ptr(), get_ssize_index(index), item.as_ptr())
        })
    }

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    #[inline]
    fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: IntoPyObject<'py>,
    {
        self.as_sequence().contains(value)
    }

    /// Returns the first index `i` for which `self[i] == value`.
    ///
    /// This is equivalent to the Python expression `self.index(value)`.
    #[inline]
    fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: IntoPyObject<'py>,
    {
        self.as_sequence().index(value)
    }

    /// Returns an iterator over this list's items.
    fn iter(&self) -> BoundListIterator<'py> {
        BoundListIterator::new(self.clone())
    }

    /// Iterates over a list while holding a critical section, calling a closure on each item
    fn locked_for_each<F>(&self, closure: F) -> PyResult<()>
    where
        F: Fn(Bound<'py, PyAny>) -> PyResult<()>,
    {
        crate::sync::critical_section::with_critical_section(self, || {
            self.iter().try_for_each(closure)
        })
    }

    /// Sorts the list in-place. Equivalent to the Python expression `l.sort()`.
    fn sort(&self) -> PyResult<()> {
        err::error_on_negative(self.py(), unsafe { ffi::PyList_Sort(self.as_ptr()) })
    }

    /// Reverses the list in-place. Equivalent to the Python expression `l.reverse()`.
    fn reverse(&self) -> PyResult<()> {
        err::error_on_negative(self.py(), unsafe { ffi::PyList_Reverse(self.as_ptr()) })
    }

    /// Return a new tuple containing the contents of the list; equivalent to the Python expression `tuple(list)`.
    ///
    /// This method is equivalent to `self.as_sequence().to_tuple()` and faster than `PyTuple::new(py, this_list)`.
    fn to_tuple(&self) -> Bound<'py, PyTuple> {
        unsafe {
            ffi::PyList_AsTuple(self.as_ptr())
                .assume_owned(self.py())
                .cast_into_unchecked()
        }
    }
}

// New types for type checking when using BoundListIterator associated methods, like
// BoundListIterator::next_unsynchronized.
struct Index(usize);
struct Length(usize);

/// Used by `PyList::iter()`.
pub struct BoundListIterator<'py> {
    list: Bound<'py, PyList>,
    index: Index,
    length: Length,
}

impl<'py> BoundListIterator<'py> {
    fn new(list: Bound<'py, PyList>) -> Self {
        Self {
            index: Index(0),
            length: Length(list.len()),
            list,
        }
    }

    /// # Safety
    ///
    /// The caller must hold an active critical section on this list.
    #[inline]
    unsafe fn next_unsynchronized(
        index: &mut Index,
        length: &mut Length,
        list: &Bound<'py, PyList>,
    ) -> Option<Bound<'py, PyAny>> {
        let length = length.0.min(list.len());
        let my_index = index.0;

        if index.0 < length {
            // SAFETY: index.0 < list.len() guarantees in bounds
            let item = unsafe { list.get_item_unchecked(my_index) };
            index.0 += 1;
            Some(item)
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// The caller must hold an active critical section on this list.
    #[inline]
    #[cfg(not(feature = "nightly"))]
    unsafe fn nth_unsynchronized(
        index: &mut Index,
        length: &mut Length,
        list: &Bound<'py, PyList>,
        n: usize,
    ) -> Option<Bound<'py, PyAny>> {
        let current_length = length.0.min(list.len());
        if let Some(target_index) = index.0.checked_add(n) {
            if target_index < current_length {
                // SAFETY: target_index < current_length guarantees in bounds
                let item = unsafe { list.get_item_unchecked(target_index) };
                // +1 cannot overflow as target_index < current_length
                index.0 = target_index + 1;
                return Some(item);
            }
        }

        // n overflows the remaining length of the list;
        // nth must exhaust all remaining items
        index.0 = current_length;
        None
    }

    /// # Safety
    ///
    /// The caller must hold an active critical section on this list.
    #[inline]
    unsafe fn next_back_unsynchronized(
        index: &mut Index,
        length: &mut Length,
        list: &Bound<'py, PyList>,
    ) -> Option<Bound<'py, PyAny>> {
        let current_length = length.0.min(list.len());

        if index.0 < current_length {
            // SAFETY: index.0 < list.len() guarantees in bounds
            let item = unsafe { list.get_item_unchecked(current_length - 1) };
            length.0 = current_length - 1;
            Some(item)
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// The caller must hold an active critical section on this list.
    #[inline]
    #[cfg(not(feature = "nightly"))]
    unsafe fn nth_back_unsynchronized(
        index: &mut Index,
        length: &mut Length,
        list: &Bound<'py, PyList>,
        n: usize,
    ) -> Option<Bound<'py, PyAny>> {
        let current_length = length.0.min(list.len());
        if let Some(index_after_item) = current_length.checked_sub(n) {
            if index.0 < index_after_item {
                // -1 cannot underflow as index_after_item > index
                let target_index = index_after_item - 1;
                // SAFETY: target_index is < current_length
                let item = unsafe { list.get_item_unchecked(target_index) };
                length.0 = target_index;
                return Some(item);
            }
        }

        // n overflows the remaining length of the tuple;
        // nth must exhaust all remaining items
        length.0 = index.0;
        None
    }

    fn with_critical_section<R>(
        &mut self,
        f: impl FnOnce(&mut Index, &mut Length, &Bound<'py, PyList>) -> R,
    ) -> R {
        let Self {
            index,
            length,
            list,
        } = self;
        crate::sync::critical_section::with_critical_section(list, || f(index, length, list))
    }
}

impl<'py> Iterator for BoundListIterator<'py> {
    type Item = Bound<'py, PyAny>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: with_critical_section locks the list
        self.with_critical_section(|index, length, list| unsafe {
            Self::next_unsynchronized(index, length, list)
        })
    }

    #[inline]
    #[cfg(not(feature = "nightly"))]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        // SAFETY: with_critical_section locks the list
        self.with_critical_section(|index, length, list| unsafe {
            Self::nth_unsynchronized(index, length, list, n)
        })
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
    #[cfg(all(Py_GIL_DISABLED, not(Py_LIMITED_API), not(feature = "nightly")))]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.with_critical_section(|index, length, list| {
            let mut accum = init;
            // SAFETY: with_critical_section locks the list
            while let Some(x) = unsafe { Self::next_unsynchronized(index, length, list) } {
                accum = f(accum, x);
            }
            accum
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(Py_LIMITED_API), feature = "nightly"))]
    fn try_fold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> R,
        R: core::ops::Try<Output = B>,
    {
        self.with_critical_section(|index, length, list| {
            let mut accum = init;
            // SAFETY: with_critical_section locks the list
            while let Some(x) = unsafe { Self::next_unsynchronized(index, length, list) } {
                accum = f(accum, x)?
            }
            R::from_output(accum)
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(Py_LIMITED_API), not(feature = "nightly")))]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        self.with_critical_section(|index, length, list| {
            // SAFETY: with_critical_section locks the list
            while let Some(x) = unsafe { Self::next_unsynchronized(index, length, list) } {
                if !f(x) {
                    return false;
                }
            }
            true
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(Py_LIMITED_API), not(feature = "nightly")))]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        self.with_critical_section(|index, length, list| {
            // SAFETY: with_critical_section locks the list
            while let Some(x) = unsafe { Self::next_unsynchronized(index, length, list) } {
                if f(x) {
                    return true;
                }
            }
            false
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(Py_LIMITED_API), not(feature = "nightly")))]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        self.with_critical_section(|index, length, list| {
            // SAFETY: with_critical_section locks the list
            while let Some(x) = unsafe { Self::next_unsynchronized(index, length, list) } {
                if predicate(&x) {
                    return Some(x);
                }
            }
            None
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(Py_LIMITED_API), not(feature = "nightly")))]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        self.with_critical_section(|index, length, list| {
            // SAFETY: with_critical_section locks the list
            while let Some(x) = unsafe { Self::next_unsynchronized(index, length, list) } {
                if let found @ Some(_) = f(x) {
                    return found;
                }
            }
            None
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(Py_LIMITED_API), not(feature = "nightly")))]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        self.with_critical_section(|index, length, list| {
            let mut acc = 0;
            // SAFETY: with_critical_section locks the list
            while let Some(x) = unsafe { Self::next_unsynchronized(index, length, list) } {
                if predicate(x) {
                    return Some(acc);
                }
                acc += 1;
            }
            None
        })
    }

    #[inline]
    #[cfg(feature = "nightly")]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        self.with_critical_section(|index, length, list| {
            let current_length = length.0.min(list.len());
            let items_left = current_length.saturating_sub(index.0);
            if let Some(overflow) = NonZero::new(n.saturating_sub(items_left)) {
                // n overflows the remaining length of the list; advance_by must exhaust all remaining items
                index.0 = current_length;
                return Err(overflow);
            }

            // cannot overflow as length - index >= n
            index.0 += n;
            Ok(())
        })
    }
}

impl DoubleEndedIterator for BoundListIterator<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.with_critical_section(|index, length, list| unsafe {
            Self::next_back_unsynchronized(index, length, list)
        })
    }

    #[inline]
    #[cfg(not(feature = "nightly"))]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        // SAFETY: with_critical_section locks the list
        self.with_critical_section(|index, length, list| unsafe {
            Self::nth_back_unsynchronized(index, length, list, n)
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(Py_LIMITED_API), not(feature = "nightly")))]
    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.with_critical_section(|index, length, list| {
            let mut accum = init;
            // SAFETY: with_critical_section locks the list
            while let Some(x) = unsafe { Self::next_back_unsynchronized(index, length, list) } {
                accum = f(accum, x);
            }
            accum
        })
    }

    #[inline]
    #[cfg(all(Py_GIL_DISABLED, not(Py_LIMITED_API), feature = "nightly"))]
    fn try_rfold<B, F, R>(&mut self, init: B, mut f: F) -> R
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> R,
        R: core::ops::Try<Output = B>,
    {
        self.with_critical_section(|index, length, list| {
            let mut accum = init;
            // SAFETY: with_critical_section locks the list
            while let Some(x) = unsafe { Self::next_back_unsynchronized(index, length, list) } {
                accum = f(accum, x)?
            }
            R::from_output(accum)
        })
    }

    #[inline]
    #[cfg(feature = "nightly")]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        self.with_critical_section(|index, length, list| {
            let current_length = length.0.min(list.len());
            let items_left = current_length.saturating_sub(index.0);
            if let Some(overflow) = NonZero::new(n.saturating_sub(items_left)) {
                // n overflows the remaining length of the list; advance_back_by must exhaust all remaining items
                length.0 = index.0;
                return Err(overflow);
            }

            // cannot overflow as current_length - index >= n
            length.0 = current_length - n;
            Ok(())
        })
    }
}

impl ExactSizeIterator for BoundListIterator<'_> {
    fn len(&self) -> usize {
        self.length.0.saturating_sub(self.index.0)
    }
}

impl FusedIterator for BoundListIterator<'_> {}

impl<'py> IntoIterator for Bound<'py, PyList> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundListIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        BoundListIterator::new(self)
    }
}

impl<'py> IntoIterator for &Bound<'py, PyList> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundListIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::platform::prelude::*;
    use crate::types::any::PyAnyMethods;
    use crate::types::list::PyListMethods;
    use crate::types::sequence::PySequenceMethods;
    use crate::types::{PyList, PyTuple};
    use crate::{IntoPyObject, PyResult, Python};
    #[cfg(feature = "nightly")]
    use core::num::NonZero;

    #[test]
    fn test_new() {
        Python::attach(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]).unwrap();
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(5, list.get_item(2).unwrap().extract::<i32>().unwrap());
            assert_eq!(7, list.get_item(3).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_new_iterator_path() {
        Python::attach(|py| {
            for len in [0, 1, 32] {
                let expected: Vec<usize> = (0..len).collect();
                let objects: Vec<_> = expected
                    .iter()
                    .map(|value| value.into_pyobject(py).unwrap())
                    .collect();
                for list in [PyList::new(py, &expected), PyList::new(py, &objects)] {
                    assert_eq!(list.unwrap().extract::<Vec<usize>>().unwrap(), expected);
                }
                // Converting each `&expected` item builds an inner list.
                let nested = PyList::new(py, vec![&expected; 32]).unwrap();
                assert_eq!(
                    nested.extract::<Vec<Vec<usize>>>().unwrap(),
                    vec![expected; 32]
                );
            }
        });
    }

    #[test]
    fn new_reports_stop_iteration_from_conversion() {
        use crate::exceptions::PyStopIteration;
        use crate::{Bound, PyAny, PyErr};

        // `PySequence_List` treats `StopIteration` as exhaustion, but `PyList::new` must report
        // the same exception when an item conversion fails with `StopIteration`.
        struct Stop;

        impl<'py> IntoPyObject<'py> for Stop {
            type Target = PyAny;
            type Output = Bound<'py, PyAny>;
            type Error = PyErr;

            fn into_pyobject(self, _py: Python<'py>) -> PyResult<Self::Output> {
                Err(PyStopIteration::new_err(()))
            }
        }

        Python::attach(|py| {
            let err = PyList::new(py, (0..32).map(|_| Stop)).unwrap_err();
            assert!(err.is_instance_of::<PyStopIteration>(py));
        });
    }

    #[test]
    fn test_len() {
        Python::attach(|py| {
            let list = PyList::new(py, [1, 2, 3, 4]).unwrap();
            assert_eq!(4, list.len());
        });
    }

    #[test]
    fn test_get_item() {
        Python::attach(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]).unwrap();
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(5, list.get_item(2).unwrap().extract::<i32>().unwrap());
            assert_eq!(7, list.get_item(3).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_get_slice() {
        Python::attach(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]).unwrap();
            let slice = list.get_slice(1, 3);
            assert_eq!(2, slice.len());
            let slice = list.get_slice(1, 7);
            assert_eq!(3, slice.len());
        });
    }

    #[test]
    fn test_set_item() {
        Python::attach(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]).unwrap();
            let val = 42i32.into_pyobject(py).unwrap();
            let val2 = 42i32.into_pyobject(py).unwrap();
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            list.set_item(0, val).unwrap();
            assert_eq!(42, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert!(list.set_item(10, val2).is_err());
        });
    }

    #[test]
    fn test_set_item_refcnt() {
        Python::attach(|py| {
            let obj = py.eval(c"object()", None, None).unwrap();
            let cnt;
            {
                let v = vec![2];
                let ob = v.into_pyobject(py).unwrap();
                let list = ob.cast::<PyList>().unwrap();
                cnt = obj._get_refcnt();
                list.set_item(0, &obj).unwrap();
            }

            assert_eq!(cnt, obj._get_refcnt());
        });
    }

    #[test]
    fn test_insert() {
        Python::attach(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]).unwrap();
            let val = 42i32.into_pyobject(py).unwrap();
            let val2 = 43i32.into_pyobject(py).unwrap();
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
        Python::attach(|py| {
            let cnt;
            let obj = py.eval(c"object()", None, None).unwrap();
            {
                let list = PyList::empty(py);
                cnt = obj._get_refcnt();
                list.insert(0, &obj).unwrap();
            }

            assert_eq!(cnt, obj._get_refcnt());
        });
    }

    #[test]
    fn test_append() {
        Python::attach(|py| {
            let list = PyList::new(py, [2]).unwrap();
            list.append(3).unwrap();
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(1).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_append_refcnt() {
        Python::attach(|py| {
            let cnt;
            let obj = py.eval(c"object()", None, None).unwrap();
            {
                let list = PyList::empty(py);
                cnt = obj._get_refcnt();
                list.append(&obj).unwrap();
            }
            assert_eq!(cnt, obj._get_refcnt());
        });
    }

    #[test]
    fn test_iter() {
        Python::attach(|py| {
            let v = vec![2, 3, 5, 7];
            let list = PyList::new(py, &v).unwrap();
            let mut idx = 0;
            for el in list {
                assert_eq!(v[idx], el.extract::<i32>().unwrap());
                idx += 1;
            }
            assert_eq!(idx, v.len());
        });
    }

    #[test]
    fn test_iter_size_hint() {
        Python::attach(|py| {
            let v = vec![2, 3, 5, 7];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            assert_eq!(iter.size_hint(), (v.len(), Some(v.len())));
            iter.next();
            assert_eq!(iter.size_hint(), (v.len() - 1, Some(v.len() - 1)));

            // Exhaust iterator.
            for _ in &mut iter {}

            assert_eq!(iter.size_hint(), (0, Some(0)));
        });
    }

    #[test]
    fn test_iter_rev() {
        Python::attach(|py| {
            let v = vec![2, 3, 5, 7];
            let ob = v.into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter().rev();

            assert_eq!(iter.size_hint(), (4, Some(4)));

            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 7);
            assert_eq!(iter.size_hint(), (3, Some(3)));

            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 5);
            assert_eq!(iter.size_hint(), (2, Some(2)));

            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(iter.size_hint(), (1, Some(1)));

            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 2);
            assert_eq!(iter.size_hint(), (0, Some(0)));

            assert!(iter.next().is_none());
            assert!(iter.next().is_none());
        });
    }

    #[test]
    fn test_iter_all() {
        Python::attach(|py| {
            let list = PyList::new(py, [true, true, true]).unwrap();
            assert!(list.iter().all(|x| x.extract::<bool>().unwrap()));

            let list = PyList::new(py, [true, false, true]).unwrap();
            assert!(!list.iter().all(|x| x.extract::<bool>().unwrap()));
        });
    }

    #[test]
    fn test_iter_any() {
        Python::attach(|py| {
            let list = PyList::new(py, [true, true, true]).unwrap();
            assert!(list.iter().any(|x| x.extract::<bool>().unwrap()));

            let list = PyList::new(py, [true, false, true]).unwrap();
            assert!(list.iter().any(|x| x.extract::<bool>().unwrap()));

            let list = PyList::new(py, [false, false, false]).unwrap();
            assert!(!list.iter().any(|x| x.extract::<bool>().unwrap()));
        });
    }

    #[test]
    fn test_iter_find() {
        Python::attach(|py: Python<'_>| {
            let list = PyList::new(py, ["hello", "world"]).unwrap();
            assert_eq!(
                Some("world".to_string()),
                list.iter()
                    .find(|v| v.extract::<String>().unwrap() == "world")
                    .map(|v| v.extract::<String>().unwrap())
            );
            assert_eq!(
                None,
                list.iter()
                    .find(|v| v.extract::<String>().unwrap() == "foobar")
                    .map(|v| v.extract::<String>().unwrap())
            );
        });
    }

    #[test]
    fn test_iter_position() {
        Python::attach(|py: Python<'_>| {
            let list = PyList::new(py, ["hello", "world"]).unwrap();
            assert_eq!(
                Some(1),
                list.iter()
                    .position(|v| v.extract::<String>().unwrap() == "world")
            );
            assert_eq!(
                None,
                list.iter()
                    .position(|v| v.extract::<String>().unwrap() == "foobar")
            );
        });
    }

    #[test]
    fn test_iter_fold() {
        Python::attach(|py: Python<'_>| {
            let list = PyList::new(py, [1, 2, 3]).unwrap();
            let sum = list
                .iter()
                .fold(0, |acc, v| acc + v.extract::<usize>().unwrap());
            assert_eq!(sum, 6);
        });
    }

    #[test]
    fn test_iter_fold_out_of_bounds() {
        Python::attach(|py: Python<'_>| {
            let list = PyList::new(py, [1, 2, 3]).unwrap();
            let sum = list.iter().fold(0, |_, _| {
                // clear the list to create a pathological fold operation
                // that mutates the list as it processes it
                for _ in 0..3 {
                    list.del_item(0).unwrap();
                }
                -5
            });
            assert_eq!(sum, -5);
            assert_eq!(list.len(), 0);
        });
    }

    #[test]
    fn test_iter_rfold() {
        Python::attach(|py: Python<'_>| {
            let list = PyList::new(py, [1, 2, 3]).unwrap();
            let sum = list
                .iter()
                .rfold(0, |acc, v| acc + v.extract::<usize>().unwrap());
            assert_eq!(sum, 6);
        });
    }

    #[test]
    fn test_iter_try_fold() {
        Python::attach(|py: Python<'_>| {
            let list = PyList::new(py, [1, 2, 3]).unwrap();
            let sum = list
                .iter()
                .try_fold(0, |acc, v| PyResult::Ok(acc + v.extract::<usize>()?))
                .unwrap();
            assert_eq!(sum, 6);

            let list = PyList::new(py, ["foo", "bar"]).unwrap();
            assert!(
                list.iter()
                    .try_fold(0, |acc, v| PyResult::Ok(acc + v.extract::<usize>()?))
                    .is_err()
            );
        });
    }

    #[test]
    fn test_iter_try_rfold() {
        Python::attach(|py: Python<'_>| {
            let list = PyList::new(py, [1, 2, 3]).unwrap();
            let sum = list
                .iter()
                .try_rfold(0, |acc, v| PyResult::Ok(acc + v.extract::<usize>()?))
                .unwrap();
            assert_eq!(sum, 6);

            let list = PyList::new(py, ["foo", "bar"]).unwrap();
            assert!(
                list.iter()
                    .try_rfold(0, |acc, v| PyResult::Ok(acc + v.extract::<usize>()?))
                    .is_err()
            );
        });
    }

    #[test]
    fn test_into_iter() {
        Python::attach(|py| {
            let list = PyList::new(py, [1, 2, 3, 4]).unwrap();
            for (i, item) in list.iter().enumerate() {
                assert_eq!((i + 1) as i32, item.extract::<i32>().unwrap());
            }
        });
    }

    #[test]
    fn test_into_iter_bound() {
        use crate::types::any::PyAnyMethods;

        Python::attach(|py| {
            let list = PyList::new(py, [1, 2, 3, 4]).unwrap();
            let mut items = vec![];
            for item in &list {
                items.push(item.extract::<i32>().unwrap());
            }
            assert_eq!(items, vec![1, 2, 3, 4]);
        });
    }

    #[test]
    fn test_as_sequence() {
        Python::attach(|py| {
            let list = PyList::new(py, [1, 2, 3, 4]).unwrap();

            assert_eq!(list.as_sequence().len().unwrap(), 4);
            assert_eq!(
                list.as_sequence()
                    .get_item(1)
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                2
            );
        });
    }

    #[test]
    fn test_into_sequence() {
        Python::attach(|py| {
            let list = PyList::new(py, [1, 2, 3, 4]).unwrap();

            let sequence = list.into_sequence();

            assert_eq!(sequence.len().unwrap(), 4);
            assert_eq!(sequence.get_item(1).unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn test_extract() {
        Python::attach(|py| {
            let v = vec![2, 3, 5, 7];
            let list = PyList::new(py, &v).unwrap();
            let v2 = list.as_any().extract::<Vec<i32>>().unwrap();
            assert_eq!(v, v2);
        });
    }

    #[test]
    fn test_sort() {
        Python::attach(|py| {
            let v = vec![7, 3, 2, 5];
            let list = PyList::new(py, &v).unwrap();
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
        Python::attach(|py| {
            let v = vec![2, 3, 5, 7];
            let list = PyList::new(py, &v).unwrap();
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
    fn test_array_into_pyobject() {
        Python::attach(|py| {
            let array = [1, 2].into_pyobject(py).unwrap();
            let list = array.cast::<PyList>().unwrap();
            assert_eq!(1, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(2, list.get_item(1).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_list_get_item_invalid_index() {
        Python::attach(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]).unwrap();
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
        Python::attach(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]).unwrap();
            let obj = list.get_item(0);
            assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn test_list_get_item_unchecked_sanity() {
        Python::attach(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]).unwrap();
            let obj = unsafe { list.get_item_unchecked(0) };
            assert_eq!(obj.extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn test_list_del_item() {
        Python::attach(|py| {
            let list = PyList::new(py, [1, 1, 2, 3, 5, 8]).unwrap();
            assert!(list.del_item(10).is_err());
            assert_eq!(1, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(1, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(3, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(5, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(8, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(0, list.len());
            assert!(list.del_item(0).is_err());
        });
    }

    #[test]
    fn test_list_set_slice() {
        Python::attach(|py| {
            let list = PyList::new(py, [1, 1, 2, 3, 5, 8]).unwrap();
            let ins = PyList::new(py, [7, 4]).unwrap();
            list.set_slice(1, 4, &ins).unwrap();
            assert_eq!([1, 7, 4, 5, 8], list.extract::<[i32; 5]>().unwrap());
            list.set_slice(3, 100, &PyList::empty(py)).unwrap();
            assert_eq!([1, 7, 4], list.extract::<[i32; 3]>().unwrap());
        });
    }

    #[test]
    fn test_list_del_slice() {
        Python::attach(|py| {
            let list = PyList::new(py, [1, 1, 2, 3, 5, 8]).unwrap();
            list.del_slice(1, 4).unwrap();
            assert_eq!([1, 5, 8], list.extract::<[i32; 3]>().unwrap());
            list.del_slice(1, 100).unwrap();
            assert_eq!([1], list.extract::<[i32; 1]>().unwrap());
        });
    }

    #[test]
    fn test_list_contains() {
        Python::attach(|py| {
            let list = PyList::new(py, [1, 1, 2, 3, 5, 8]).unwrap();
            assert_eq!(6, list.len());

            let bad_needle = 7i32.into_pyobject(py).unwrap();
            assert!(!list.contains(&bad_needle).unwrap());

            let good_needle = 8i32.into_pyobject(py).unwrap();
            assert!(list.contains(&good_needle).unwrap());

            let type_coerced_needle = 8f32.into_pyobject(py).unwrap();
            assert!(list.contains(&type_coerced_needle).unwrap());
        });
    }

    #[test]
    fn test_list_index() {
        Python::attach(|py| {
            let list = PyList::new(py, [1, 1, 2, 3, 5, 8]).unwrap();
            assert_eq!(0, list.index(1i32).unwrap());
            assert_eq!(2, list.index(2i32).unwrap());
            assert_eq!(3, list.index(3i32).unwrap());
            assert_eq!(4, list.index(5i32).unwrap());
            assert_eq!(5, list.index(8i32).unwrap());
            assert!(list.index(42i32).is_err());
        });
    }

    use core::ops::Range;

    // An iterator that lies about its `size_hint` implementation.
    // See https://github.com/PyO3/pyo3/issues/2118
    struct FaultyIter(Range<usize>, usize);

    impl Iterator for FaultyIter {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.1, Some(self.1))
        }
    }

    #[test]
    #[should_panic(
        expected = "Attempted to create PyList but `elements` was smaller than reported by its `size_hint` implementation."
    )]
    fn too_short_iterator() {
        Python::attach(|py| {
            let iter = FaultyIter(0..35, 73);
            let _list = PyList::new(py, iter).unwrap();
        })
    }

    #[test]
    #[should_panic(
        expected = "out of range integral type conversion attempted on `elements.len()`"
    )]
    fn overflowing_size() {
        Python::attach(|py| {
            let iter = FaultyIter(0..0, usize::MAX);

            let _list = PyList::new(py, iter).unwrap();
        })
    }

    #[test]
    #[cfg(panic = "unwind")]
    fn bad_intopyobject_doesnt_cause_leaks() {
        use crate::types::PyInt;
        use core::convert::Infallible;
        use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};
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

            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.1, Some(self.1))
            }
        }

        Python::attach(|py| {
            std::panic::catch_unwind(|| {
                let iter = FaultyIter(0..50, 50);
                let _list = PyList::new(py, iter).unwrap();
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
    fn test_list_to_tuple() {
        Python::attach(|py| {
            let list = PyList::new(py, vec![1, 2, 3]).unwrap();
            let tuple = list.to_tuple();
            let tuple_expected = PyTuple::new(py, vec![1, 2, 3]).unwrap();
            assert!(tuple.eq(tuple_expected).unwrap());
        })
    }

    #[test]
    fn test_iter_nth() {
        Python::attach(|py| {
            let v = vec![6, 7, 8, 9, 10];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            iter.next();
            assert_eq!(iter.nth(1).unwrap().extract::<i32>().unwrap(), 8);
            assert_eq!(iter.nth(1).unwrap().extract::<i32>().unwrap(), 10);
            assert!(iter.nth(1).is_none());

            let v: Vec<i32> = vec![];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            iter.next();
            assert!(iter.nth(1).is_none());

            let v = vec![1, 2, 3];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            assert!(iter.nth(10).is_none());

            let v = vec![6, 7, 8, 9, 10];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();
            let mut iter = list.iter();
            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 6);
            assert_eq!(iter.nth(2).unwrap().extract::<i32>().unwrap(), 9);
            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 10);

            let mut iter = list.iter();
            assert_eq!(iter.nth_back(1).unwrap().extract::<i32>().unwrap(), 9);
            assert_eq!(iter.nth(2).unwrap().extract::<i32>().unwrap(), 8);
            assert!(iter.next().is_none());

            // nth consumes all elements in the list, even on `None` return
            let mut iter = list.iter();
            assert!(iter.nth(100).is_none());
            assert!(iter.next().is_none());
            assert!(iter.next_back().is_none());

            // nth should not overflow the iterator
            // a naive implementation of nth will overflow if number of advanced
            // elements plus N overflows usize::MAX
            let mut iter = list.iter();
            assert!(iter.next().is_some());
            assert!(iter.nth(usize::MAX).is_none());
        });
    }

    #[test]
    fn test_iter_nth_back() {
        Python::attach(|py| {
            let v = vec![1, 2, 3, 4, 5];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            assert_eq!(iter.nth_back(0).unwrap().extract::<i32>().unwrap(), 5);
            assert_eq!(iter.nth_back(1).unwrap().extract::<i32>().unwrap(), 3);
            assert!(iter.nth_back(2).is_none());

            let v: Vec<i32> = vec![];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            assert!(iter.nth_back(0).is_none());
            assert!(iter.nth_back(1).is_none());

            let v = vec![1, 2, 3];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            assert!(iter.nth_back(5).is_none());

            let v = vec![1, 2, 3, 4, 5];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            iter.next_back(); // Consume the last element
            assert_eq!(iter.nth_back(1).unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(iter.next_back().unwrap().extract::<i32>().unwrap(), 2);
            assert_eq!(iter.nth_back(0).unwrap().extract::<i32>().unwrap(), 1);

            let v = vec![1, 2, 3, 4, 5];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            assert_eq!(iter.nth_back(1).unwrap().extract::<i32>().unwrap(), 4);
            assert_eq!(iter.nth_back(2).unwrap().extract::<i32>().unwrap(), 1);

            let mut iter2 = list.iter();
            iter2.next_back();
            assert_eq!(iter2.nth_back(1).unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(iter2.next_back().unwrap().extract::<i32>().unwrap(), 2);

            let mut iter3 = list.iter();
            iter3.nth(1);
            assert_eq!(iter3.nth_back(2).unwrap().extract::<i32>().unwrap(), 3);
            assert!(iter3.nth_back(0).is_none());

            // nth_back consumes all elements in the list, even on `None` return
            let mut iter4 = list.iter();
            assert!(iter4.nth_back(100).is_none());
            assert!(iter4.next_back().is_none());
            assert!(iter4.next().is_none());

            // nth_back should not overflow with usize::MAX
            let mut iter5 = list.iter();
            iter5.nth(1); //
            assert!(iter5.nth_back(usize::MAX).is_none());
        });
    }

    #[cfg(feature = "nightly")]
    #[test]
    fn test_iter_advance_by() {
        Python::attach(|py| {
            let v = vec![1, 2, 3, 4, 5];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            assert_eq!(iter.advance_by(2), Ok(()));
            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(iter.advance_by(0), Ok(()));
            assert_eq!(iter.advance_by(100), Err(NonZero::new(98).unwrap()));

            let mut iter2 = list.iter();
            assert_eq!(iter2.advance_by(6), Err(NonZero::new(1).unwrap()));

            let mut iter3 = list.iter();
            assert_eq!(iter3.advance_by(5), Ok(()));

            let mut iter4 = list.iter();
            assert_eq!(iter4.advance_by(0), Ok(()));
            assert_eq!(iter4.next().unwrap().extract::<i32>().unwrap(), 1);

            // advance_by should not overflow with usize::MAX
            // - first advanc will overflow by MAX - len, and will exhaust the iterator
            // - second advance will overflow by MAX
            let mut iter5 = list.iter();
            assert_eq!(
                iter5.advance_by(usize::MAX),
                Err(NonZero::new(usize::MAX - list.len()).unwrap())
            );
            assert_eq!(
                iter5.advance_by(usize::MAX),
                Err(NonZero::new(usize::MAX).unwrap())
            );
        })
    }

    #[cfg(feature = "nightly")]
    #[test]
    fn test_iter_advance_back_by() {
        Python::attach(|py| {
            let v = vec![1, 2, 3, 4, 5];
            let ob = (&v).into_pyobject(py).unwrap();
            let list = ob.cast::<PyList>().unwrap();

            let mut iter = list.iter();
            assert_eq!(iter.advance_back_by(2), Ok(()));
            assert_eq!(iter.next_back().unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(iter.advance_back_by(0), Ok(()));
            assert_eq!(iter.advance_back_by(100), Err(NonZero::new(98).unwrap()));

            let mut iter2 = list.iter();
            assert_eq!(iter2.advance_back_by(6), Err(NonZero::new(1).unwrap()));

            let mut iter3 = list.iter();
            assert_eq!(iter3.advance_back_by(5), Ok(()));

            let mut iter4 = list.iter();
            assert_eq!(iter4.advance_back_by(0), Ok(()));
            assert_eq!(iter4.next_back().unwrap().extract::<i32>().unwrap(), 5);

            // advance_back_by should not overflow with usize::MAX
            // - first advance will overflow by MAX - len, and will exhaust the iterator
            // - second advance will overflow by MAX
            let mut iter5 = list.iter();
            assert_eq!(
                iter5.advance_back_by(usize::MAX),
                Err(NonZero::new(usize::MAX - list.len()).unwrap())
            );
            assert_eq!(
                iter5.advance_back_by(usize::MAX),
                Err(NonZero::new(usize::MAX).unwrap())
            );
        })
    }

    #[test]
    fn test_iter_last() {
        Python::attach(|py| {
            let list = PyList::new(py, vec![1, 2, 3]).unwrap();
            let last = list.iter().last();
            assert_eq!(last.unwrap().extract::<i32>().unwrap(), 3);
        })
    }

    #[test]
    fn test_iter_count() {
        Python::attach(|py| {
            let list = PyList::new(py, vec![1, 2, 3]).unwrap();
            assert_eq!(list.iter().count(), 3);
        })
    }

    #[test]
    fn test_new_from_non_exact_iter() {
        Python::attach(|py| {
            let iter = (0..5)
                .filter(|_| true) // Filter does not implement ExactSizeIterator
                .map(|item| item.into_pyobject(py).unwrap());

            assert!(
                matches!(iter.size_hint(), (0, _)),
                "size_hint lower bound should be 0 because we do not now the final size after filter"
            );

            let list = PyList::new(py, iter).unwrap();
            assert_eq!(
                list.len(),
                5,
                "list should contain all elements even though size_hint is 0"
            );
        })
    }

    #[test]
    fn test_new_from_non_exact_iter_with_lower_bound() {
        Python::attach(|py| {
            for len in [0, 32] {
                let iter = (0..len).chain((len..len + 3).filter(|_| true));
                assert_eq!(iter.size_hint().0, len);
                let list = PyList::new(py, iter).unwrap();
                assert_eq!(
                    list.extract::<Vec<usize>>().unwrap(),
                    (0..len + 3).collect::<Vec<_>>()
                );
            }
        })
    }
}
