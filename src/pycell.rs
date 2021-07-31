//! Includes `PyCell` implementation.
use crate::exceptions::PyRuntimeError;
use crate::pyclass::PyClass;
use crate::pyclass_init::PyClassInitializer;
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::type_object::{PyLayout, PySizedLayout};
use crate::types::PyAny;
use crate::{class::impl_::PyClassBaseType, class::impl_::PyClassThreadChecker};
use crate::{
    conversion::{AsPyPointer, FromPyPointer, ToPyObject},
    ffi::PyBaseObject_Type,
    type_object::get_tp_free,
    PyTypeInfo,
};
use crate::{ffi, IntoPy, PyErr, PyNativeType, PyObject, PyResult, Python};
use std::cell::{Cell, UnsafeCell};
use std::fmt;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

/// Base layout of PyCell.
/// This is necessary for sharing BorrowFlag between parents and children.
#[doc(hidden)]
#[repr(C)]
pub struct PyCellBase<T> {
    ob_base: T,
    borrow_flag: Cell<BorrowFlag>,
}

unsafe impl<T, U> PyLayout<T> for PyCellBase<U> where U: PySizedLayout<T> {}

/// `PyCell` is the container type for [`PyClass`](../pyclass/trait.PyClass.html).
///
/// From Python side, `PyCell<T>` is the concrete layout of `T: PyClass` in the Python heap,
/// which means we can convert `*const PyClass<T>` to `*mut ffi::PyObject`.
///
/// From Rust side, `PyCell<T>` is the mutable container of `T`.
/// Since `PyCell<T: PyClass>` is always on the Python heap, we don't have the ownership of it.
/// Thus, to mutate the data behind `&PyCell<T>` safely, we employ the
/// [Interior Mutability Pattern](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
/// like [std::cell::RefCell](https://doc.rust-lang.org/std/cell/struct.RefCell.html).
///
/// `PyCell` implements `Deref<Target = PyAny>`, so you can also call methods from `PyAny`
/// when you have a `PyCell<T>`.
///
/// # Examples
///
/// In most cases, `PyCell` is hidden behind `#[pymethods]`.
/// However, you can construct `&PyCell` directly to test your pyclass in Rust code.
///
/// ```
/// # use pyo3::prelude::*;
/// #[pyclass]
/// struct Book {
///     #[pyo3(get)]
///     name: &'static str,
///     author: &'static str,
/// }
/// let book = Book {
///     name: "The Man in the High Castle",
///     author: "Philip Kindred Dick",
/// };
/// Python::with_gil(|py| {
///     let book_cell = PyCell::new(py, book).unwrap();
///     // `&PyCell` implements `ToPyObject`, so you can use it in a Python snippet
///     pyo3::py_run!(py, book_cell, "assert book_cell.name[-6:] == 'Castle'");
/// });
/// ```
/// You can use `slf: &PyCell<Self>` as an alternative `self` receiver of `#[pymethod]`,
/// though you rarely need it.
/// ```
/// # use pyo3::prelude::*;
/// use std::collections::HashMap;
/// #[pyclass]
/// #[derive(Default)]
/// struct Counter {
///     counter: HashMap<String, usize>
/// }
/// #[pymethods]
/// impl Counter {
///     // You can use &mut self here, but now we use &PyCell for demonstration
///     fn increment(slf: &PyCell<Self>, name: String) -> PyResult<usize> {
///         let mut slf_mut = slf.try_borrow_mut()?;
///         // Now a mutable reference exists so we cannot get another one
///         assert!(slf.try_borrow().is_err());
///         assert!(slf.try_borrow_mut().is_err());
///         let counter = slf_mut.counter.entry(name).or_insert(0);
///         *counter += 1;
///         Ok(*counter)
///     }
/// }
/// # Python::with_gil(|py| {
/// #     let counter = PyCell::new(py, Counter::default()).unwrap();
/// #     pyo3::py_run!(py, counter, "assert counter.increment('cat') == 1");
/// # });
/// ```
#[repr(C)]
pub struct PyCell<T: PyClass> {
    ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
    contents: PyCellContents<T>,
}

#[repr(C)]
pub(crate) struct PyCellContents<T: PyClass> {
    pub(crate) value: ManuallyDrop<UnsafeCell<T>>,
    pub(crate) thread_checker: T::ThreadChecker,
    pub(crate) dict: T::Dict,
    pub(crate) weakref: T::WeakRef,
}

impl<T: PyClass> PyCell<T> {
    /// Get the offset of the dictionary from the start of the struct in bytes.
    #[cfg(not(all(Py_LIMITED_API, not(Py_3_9))))]
    pub(crate) fn dict_offset() -> Option<ffi::Py_ssize_t> {
        use std::convert::TryInto;
        if T::Dict::IS_DUMMY {
            None
        } else {
            #[cfg(addr_of)]
            let offset = {
                // With std::ptr::addr_of - can measure offset using uninit memory without UB.
                let cell = std::mem::MaybeUninit::<Self>::uninit();
                let base_ptr = cell.as_ptr();
                let dict_ptr = unsafe { std::ptr::addr_of!((*base_ptr).contents.dict) };
                unsafe { (dict_ptr as *const u8).offset_from(base_ptr as *const u8) }
            };
            #[cfg(not(addr_of))]
            let offset = {
                // No std::ptr::addr_of - need to take references to PyCell to measure offsets;
                // make a zero-initialised "fake" one so that referencing it is not UB.
                let mut cell = std::mem::MaybeUninit::<Self>::uninit();
                unsafe {
                    std::ptr::write_bytes(cell.as_mut_ptr(), 0, 1);
                }
                let cell = unsafe { cell.assume_init() };
                let dict_ptr = &cell.contents.dict;
                // offset_from wasn't stabilised until 1.47, so we also have to work around
                // that...
                let offset = (dict_ptr as *const _ as usize) - (&cell as *const _ as usize);
                // This isn't a valid cell, so ensure no Drop code runs etc.
                std::mem::forget(cell);
                offset
            };
            // Py_ssize_t may not be equal to isize on all platforms
            #[allow(clippy::useless_conversion)]
            Some(offset.try_into().expect("offset should fit in Py_ssize_t"))
        }
    }

    /// Get the offset of the weakref list from the start of the struct in bytes.
    #[cfg(not(all(Py_LIMITED_API, not(Py_3_9))))]
    pub(crate) fn weakref_offset() -> Option<ffi::Py_ssize_t> {
        use std::convert::TryInto;
        if T::WeakRef::IS_DUMMY {
            None
        } else {
            #[cfg(addr_of)]
            let offset = {
                // With std::ptr::addr_of - can measure offset using uninit memory without UB.
                let cell = std::mem::MaybeUninit::<Self>::uninit();
                let base_ptr = cell.as_ptr();
                let weaklist_ptr = unsafe { std::ptr::addr_of!((*base_ptr).contents.weakref) };
                unsafe { (weaklist_ptr as *const u8).offset_from(base_ptr as *const u8) }
            };
            #[cfg(not(addr_of))]
            let offset = {
                // No std::ptr::addr_of - need to take references to PyCell to measure offsets;
                // make a zero-initialised "fake" one so that referencing it is not UB.
                let mut cell = std::mem::MaybeUninit::<Self>::uninit();
                unsafe {
                    std::ptr::write_bytes(cell.as_mut_ptr(), 0, 1);
                }
                let cell = unsafe { cell.assume_init() };
                let weaklist_ptr = &cell.contents.weakref;
                // offset_from wasn't stabilised until 1.47, so we also have to work around
                // that...
                let offset = (weaklist_ptr as *const _ as usize) - (&cell as *const _ as usize);
                // This isn't a valid cell, so ensure no Drop code runs etc.
                std::mem::forget(cell);
                offset
            };
            // Py_ssize_t may not be equal to isize on all platforms
            #[allow(clippy::useless_conversion)]
            Some(offset.try_into().expect("offset should fit in Py_ssize_t"))
        }
    }
}

unsafe impl<T: PyClass> PyNativeType for PyCell<T> {}

impl<T: PyClass> PyCell<T> {
    /// Make a new `PyCell` on the Python heap and return the reference to it.
    ///
    /// In cases where the value in the cell does not need to be accessed immediately after
    /// creation, consider [`Py::new`](../instance/struct.Py.html#method.new) as a more efficient
    /// alternative.
    pub fn new(py: Python, value: impl Into<PyClassInitializer<T>>) -> PyResult<&Self> {
        unsafe {
            let initializer = value.into();
            let self_ = initializer.create_cell(py)?;
            FromPyPointer::from_owned_ptr_or_err(py, self_ as _)
        }
    }

    /// Immutably borrows the value `T`. This borrow lasts untill the returned `PyRef` exists.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow`](#method.try_borrow).
    pub fn borrow(&self) -> PyRef<'_, T> {
        self.try_borrow().expect("Already mutably borrowed")
    }

    /// Mutably borrows the value `T`. This borrow lasts untill the returned `PyRefMut` exists.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    pub fn borrow_mut(&self) -> PyRefMut<'_, T> {
        self.try_borrow_mut().expect("Already borrowed")
    }

    /// Immutably borrows the value `T`, returning an error if the value is currently
    /// mutably borrowed. This borrow lasts untill the returned `PyRef` exists.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// #[pyclass]
    /// struct Class {}
    /// Python::with_gil(|py| {
    ///     let c = PyCell::new(py, Class {}).unwrap();
    ///     {
    ///         let m = c.borrow_mut();
    ///         assert!(c.try_borrow().is_err());
    ///     }
    ///
    ///     {
    ///         let m = c.borrow();
    ///         assert!(c.try_borrow().is_ok());
    ///     }
    /// });
    /// ```
    pub fn try_borrow(&self) -> Result<PyRef<'_, T>, PyBorrowError> {
        let flag = self.get_borrow_flag();
        if flag == BorrowFlag::HAS_MUTABLE_BORROW {
            Err(PyBorrowError { _private: () })
        } else {
            self.set_borrow_flag(flag.increment());
            Ok(PyRef { inner: self })
        }
    }

    /// Mutably borrows the value `T`, returning an error if the value is currently borrowed.
    /// This borrow lasts untill the returned `PyRefMut` exists.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// #[pyclass]
    /// struct Class {}
    /// Python::with_gil(|py| {
    ///     let c = PyCell::new(py, Class {}).unwrap();
    ///     {
    ///         let m = c.borrow();
    ///         assert!(c.try_borrow_mut().is_err());
    ///     }
    ///
    ///     assert!(c.try_borrow_mut().is_ok());
    /// });
    /// ```
    pub fn try_borrow_mut(&self) -> Result<PyRefMut<'_, T>, PyBorrowMutError> {
        if self.get_borrow_flag() != BorrowFlag::UNUSED {
            Err(PyBorrowMutError { _private: () })
        } else {
            self.set_borrow_flag(BorrowFlag::HAS_MUTABLE_BORROW);
            Ok(PyRefMut { inner: self })
        }
    }

    /// Immutably borrows the value `T`, returning an error if the value is
    /// currently mutably borrowed.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it does not return a `PyRef`,
    /// thus leaving the borrow flag untouched. Mutably borrowing the `PyCell`
    /// while the reference returned by this method is alive is undefined behaviour.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// #[pyclass]
    /// struct Class {}
    /// Python::with_gil(|py| {
    ///     let c = PyCell::new(py, Class {}).unwrap();
    ///
    ///     {
    ///         let m = c.borrow_mut();
    ///         assert!(unsafe { c.try_borrow_unguarded() }.is_err());
    ///     }
    ///
    ///     {
    ///         let m = c.borrow();
    ///         assert!(unsafe { c.try_borrow_unguarded() }.is_ok());
    ///     }
    /// });
    /// ```
    pub unsafe fn try_borrow_unguarded(&self) -> Result<&T, PyBorrowError> {
        if self.get_borrow_flag() == BorrowFlag::HAS_MUTABLE_BORROW {
            Err(PyBorrowError { _private: () })
        } else {
            Ok(&*self.contents.value.get())
        }
    }

    /// Replaces the wrapped value with a new one, returning the old value,
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    #[inline]
    pub fn replace(&self, t: T) -> T {
        std::mem::replace(&mut *self.borrow_mut(), t)
    }

    /// Replaces the wrapped value with a new one computed from `f`, returning the old value.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn replace_with<F: FnOnce(&mut T) -> T>(&self, f: F) -> T {
        let mut_borrow = &mut *self.borrow_mut();
        let replacement = f(mut_borrow);
        std::mem::replace(mut_borrow, replacement)
    }

    /// Swaps the wrapped value of `self` with the wrapped value of `other`.
    ///
    /// # Panics
    ///
    /// Panics if the value in either `PyCell` is currently borrowed.
    #[inline]
    pub fn swap(&self, other: &Self) {
        std::mem::swap(&mut *self.borrow_mut(), &mut *other.borrow_mut())
    }

    fn get_ptr(&self) -> *mut T {
        self.contents.value.get()
    }
}

unsafe impl<T: PyClass> PyLayout<T> for PyCell<T> {}
impl<T: PyClass> PySizedLayout<T> for PyCell<T> {}

impl<T: PyClass> AsPyPointer for PyCell<T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        (self as *const _) as *mut _
    }
}

impl<T: PyClass> ToPyObject for &PyCell<T> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

impl<T: PyClass> AsRef<PyAny> for PyCell<T> {
    fn as_ref(&self) -> &PyAny {
        unsafe { self.py().from_borrowed_ptr(self.as_ptr()) }
    }
}

impl<T: PyClass> Deref for PyCell<T> {
    type Target = PyAny;

    fn deref(&self) -> &PyAny {
        unsafe { self.py().from_borrowed_ptr(self.as_ptr()) }
    }
}

impl<T: PyClass + fmt::Debug> fmt::Debug for PyCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.try_borrow() {
            Ok(borrow) => f.debug_struct("RefCell").field("value", &borrow).finish(),
            Err(_) => {
                struct BorrowedPlaceholder;
                impl fmt::Debug for BorrowedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("<borrowed>")
                    }
                }
                f.debug_struct("RefCell")
                    .field("value", &BorrowedPlaceholder)
                    .finish()
            }
        }
    }
}

/// Wraps a borrowed reference to a value in a `PyCell<T>`.
///
/// See the [`PyCell`](struct.PyCell.html) documentation for more.
/// # Examples
/// You can use `PyRef` as an alternative of `&self` receiver when
/// - You need to access the pointer of `PyCell`.
/// - You want to get super class.
/// ```
/// # use pyo3::prelude::*;
/// #[pyclass(subclass)]
/// struct Parent {
///     basename: &'static str,
/// }
/// #[pyclass(extends=Parent)]
/// struct Child {
///     name: &'static str,
///  }
/// #[pymethods]
/// impl Child {
///     #[new]
///     fn new() -> (Self, Parent) {
///         (Child { name: "Caterpillar" }, Parent { basename: "Butterfly" })
///     }
///     fn format(slf: PyRef<Self>) -> String {
///         // We can get *mut ffi::PyObject from PyRef
///         use pyo3::AsPyPointer;
///         let refcnt = unsafe { pyo3::ffi::Py_REFCNT(slf.as_ptr()) };
///         // We can get &Self::BaseType by as_ref
///         let basename = slf.as_ref().basename;
///         format!("{}(base: {}, cnt: {})", slf.name, basename, refcnt)
///     }
/// }
/// # Python::with_gil(|py| {
/// #     let sub = PyCell::new(py, Child::new()).unwrap();
/// #     pyo3::py_run!(py, sub, "assert sub.format() == 'Caterpillar(base: Butterfly, cnt: 3)'");
/// # });
/// ```
pub struct PyRef<'p, T: PyClass> {
    inner: &'p PyCell<T>,
}

impl<'p, T: PyClass> PyRef<'p, T> {
    /// Returns `Python` token.
    /// This function is safe since PyRef has the same lifetime as a `GILGuard`.
    pub fn py(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }
}

impl<'p, T, U> AsRef<U> for PyRef<'p, T>
where
    T: PyClass<BaseType = U>,
    U: PyClass,
{
    fn as_ref(&self) -> &T::BaseType {
        unsafe { &*self.inner.ob_base.get_ptr() }
    }
}

impl<'p, T, U> PyRef<'p, T>
where
    T: PyClass<BaseType = U>,
    U: PyClass,
{
    /// Get `PyRef<T::BaseType>`.
    /// You can use this method to get super class of super class.
    ///
    /// # Examples
    /// ```
    /// # use pyo3::prelude::*;
    /// #[pyclass(subclass)]
    /// struct Base1 {
    ///     name1: &'static str,
    /// }
    /// #[pyclass(extends=Base1, subclass)]
    /// struct Base2 {
    ///     name2: &'static str,
    ///  }
    /// #[pyclass(extends=Base2)]
    /// struct Sub {
    ///     name3: &'static str,
    ///  }
    /// #[pymethods]
    /// impl Sub {
    ///     #[new]
    ///     fn new() -> PyClassInitializer<Self> {
    ///         PyClassInitializer::from(Base1{ name1: "base1" })
    ///             .add_subclass(Base2 { name2: "base2" })
    ///             .add_subclass(Self { name3: "sub" })
    ///     }
    ///     fn name(slf: PyRef<Self>) -> String {
    ///         let subname = slf.name3;
    ///         let super_ = slf.into_super();
    ///         format!("{} {} {}", super_.as_ref().name1, super_.name2, subname)
    ///     }
    /// }
    /// # Python::with_gil(|py| {
    /// #     let sub = PyCell::new(py, Sub::new()).unwrap();
    /// #     pyo3::py_run!(py, sub, "assert sub.name() == 'base1 base2 sub'")
    /// # });
    /// ```
    pub fn into_super(self) -> PyRef<'p, U> {
        let PyRef { inner } = self;
        std::mem::forget(self);
        PyRef {
            inner: &inner.ob_base,
        }
    }
}

impl<'p, T: PyClass> Deref for PyRef<'p, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.inner.get_ptr() }
    }
}

impl<'p, T: PyClass> Drop for PyRef<'p, T> {
    fn drop(&mut self) {
        let flag = self.inner.get_borrow_flag();
        self.inner.set_borrow_flag(flag.decrement())
    }
}

impl<T: PyClass> IntoPy<PyObject> for PyRef<'_, T> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.inner.as_ptr()) }
    }
}

impl<'a, T: PyClass> std::convert::TryFrom<&'a PyCell<T>> for crate::PyRef<'a, T> {
    type Error = PyBorrowError;
    fn try_from(cell: &'a crate::PyCell<T>) -> Result<Self, Self::Error> {
        cell.try_borrow()
    }
}

impl<'a, T: PyClass> AsPyPointer for PyRef<'a, T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.inner.as_ptr()
    }
}

impl<T: PyClass + fmt::Debug> fmt::Debug for PyRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

/// Wraps a mutable borrowed reference to a value in a `PyCell<T>`.
///
/// See the [`PyCell`](struct.PyCell.html) and [`PyRef`](struct.PyRef.html) documentations for more.
pub struct PyRefMut<'p, T: PyClass> {
    inner: &'p PyCell<T>,
}

impl<'p, T: PyClass> PyRefMut<'p, T> {
    /// Returns `Python` token.
    /// This function is safe since PyRefMut has the same lifetime as a `GILGuard`.
    pub fn py(&self) -> Python {
        unsafe { Python::assume_gil_acquired() }
    }
}

impl<'p, T, U> AsRef<U> for PyRefMut<'p, T>
where
    T: PyClass<BaseType = U>,
    U: PyClass,
{
    fn as_ref(&self) -> &T::BaseType {
        unsafe { &*self.inner.ob_base.get_ptr() }
    }
}

impl<'p, T, U> AsMut<U> for PyRefMut<'p, T>
where
    T: PyClass<BaseType = U>,
    U: PyClass,
{
    fn as_mut(&mut self) -> &mut T::BaseType {
        unsafe { &mut *self.inner.ob_base.get_ptr() }
    }
}

impl<'p, T, U> PyRefMut<'p, T>
where
    T: PyClass<BaseType = U>,
    U: PyClass,
{
    /// Get `PyRef<T::BaseType>`.
    /// See  [`PyRef::into_super`](struct.PyRef.html#method.into_super) for more.
    pub fn into_super(self) -> PyRefMut<'p, U> {
        let PyRefMut { inner } = self;
        std::mem::forget(self);
        PyRefMut {
            inner: &inner.ob_base,
        }
    }
}

impl<'p, T: PyClass> Deref for PyRefMut<'p, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.inner.get_ptr() }
    }
}

impl<'p, T: PyClass> DerefMut for PyRefMut<'p, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.get_ptr() }
    }
}

impl<'p, T: PyClass> Drop for PyRefMut<'p, T> {
    fn drop(&mut self) {
        self.inner.set_borrow_flag(BorrowFlag::UNUSED)
    }
}

impl<T: PyClass> IntoPy<PyObject> for PyRefMut<'_, T> {
    fn into_py(self, py: Python) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.inner.as_ptr()) }
    }
}

impl<'a, T: PyClass> AsPyPointer for PyRefMut<'a, T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.inner.as_ptr()
    }
}

impl<'a, T: PyClass> std::convert::TryFrom<&'a PyCell<T>> for crate::PyRefMut<'a, T> {
    type Error = PyBorrowMutError;
    fn try_from(cell: &'a crate::PyCell<T>) -> Result<Self, Self::Error> {
        cell.try_borrow_mut()
    }
}

impl<T: PyClass + fmt::Debug> fmt::Debug for PyRefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*(self.deref()), f)
    }
}

#[doc(hidden)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BorrowFlag(usize);

impl BorrowFlag {
    pub(crate) const UNUSED: BorrowFlag = BorrowFlag(0);
    const HAS_MUTABLE_BORROW: BorrowFlag = BorrowFlag(usize::max_value());
    const fn increment(self) -> Self {
        Self(self.0 + 1)
    }
    const fn decrement(self) -> Self {
        Self(self.0 - 1)
    }
}

/// An error returned by [`PyCell::try_borrow`](struct.PyCell.html#method.try_borrow).
///
/// In Python, you can catch this error by `except RuntimeError`.
pub struct PyBorrowError {
    _private: (),
}

impl fmt::Debug for PyBorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PyBorrowError").finish()
    }
}

impl fmt::Display for PyBorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("Already mutably borrowed", f)
    }
}

impl From<PyBorrowError> for PyErr {
    fn from(other: PyBorrowError) -> Self {
        PyRuntimeError::new_err(other.to_string())
    }
}

/// An error returned by [`PyCell::try_borrow_mut`](struct.PyCell.html#method.try_borrow_mut).
///
/// In Python, you can catch this error by `except RuntimeError`.
pub struct PyBorrowMutError {
    _private: (),
}

impl fmt::Debug for PyBorrowMutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PyBorrowMutError").finish()
    }
}

impl fmt::Display for PyBorrowMutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("Already borrowed", f)
    }
}

impl From<PyBorrowMutError> for PyErr {
    fn from(other: PyBorrowMutError) -> Self {
        PyRuntimeError::new_err(other.to_string())
    }
}

#[doc(hidden)]
pub trait PyCellLayout<T>: PyLayout<T> {
    fn get_borrow_flag(&self) -> BorrowFlag;
    fn set_borrow_flag(&self, flag: BorrowFlag);
    /// Implementation of tp_dealloc.
    /// # Safety
    /// - slf must be a valid pointer to an instance of a T or a subclass.
    /// - slf must not be used after this call (as it will be freed).
    unsafe fn tp_dealloc(slf: *mut ffi::PyObject, py: Python);
}

impl<T, U> PyCellLayout<T> for PyCellBase<U>
where
    U: PySizedLayout<T>,
    T: PyTypeInfo,
{
    fn get_borrow_flag(&self) -> BorrowFlag {
        self.borrow_flag.get()
    }
    fn set_borrow_flag(&self, flag: BorrowFlag) {
        self.borrow_flag.set(flag)
    }
    unsafe fn tp_dealloc(slf: *mut ffi::PyObject, py: Python) {
        // For `#[pyclass]` types which inherit from PyAny, we can just call tp_free
        if T::type_object_raw(py) == &mut PyBaseObject_Type {
            return get_tp_free(ffi::Py_TYPE(slf))(slf as _);
        }

        // More complex native types (e.g. `extends=PyDict`) require calling the base's dealloc.
        #[cfg(not(Py_LIMITED_API))]
        {
            if let Some(dealloc) = (*T::type_object_raw(py)).tp_dealloc {
                dealloc(slf as _);
            } else {
                get_tp_free(ffi::Py_TYPE(slf))(slf as _);
            }
        }

        #[cfg(Py_LIMITED_API)]
        unreachable!("subclassing native types is not possible with the `abi3` feature");
    }
}

impl<T: PyClass> PyCellLayout<T> for PyCell<T>
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyCellLayout<T::BaseType>,
{
    fn get_borrow_flag(&self) -> BorrowFlag {
        self.contents.thread_checker.ensure();
        self.ob_base.get_borrow_flag()
    }
    fn set_borrow_flag(&self, flag: BorrowFlag) {
        self.ob_base.set_borrow_flag(flag)
    }
    unsafe fn tp_dealloc(slf: *mut ffi::PyObject, py: Python) {
        // Safety: Python only calls tp_dealloc when no references to the object remain.
        let cell = &mut *(slf as *mut PyCell<T>);
        ManuallyDrop::drop(&mut cell.contents.value);
        cell.contents.dict.clear_dict(py);
        cell.contents.weakref.clear_weakrefs(slf, py);
        <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(slf, py)
    }
}
