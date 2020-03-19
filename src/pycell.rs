//! Includes `PyCell` implementation.
use crate::conversion::{AsPyPointer, FromPyPointer, ToPyObject};
use crate::pyclass_init::PyClassInitializer;
use crate::pyclass_slots::{PyClassDict, PyClassWeakRef};
use crate::type_object::{PyBorrowFlagLayout, PyDowncastImpl, PyLayout, PySizedLayout, PyTypeInfo};
use crate::{ffi, FromPy, PyAny, PyClass, PyErr, PyNativeType, PyObject, PyResult, Python};
use std::cell::{Cell, UnsafeCell};
use std::fmt;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

/// Base layout of PyCell.
/// This is necessary for sharing BorrowFlag between parents and children.
#[doc(hidden)]
#[repr(C)]
pub struct PyCellBase<T: PyTypeInfo> {
    ob_base: T::Layout,
    borrow_flag: Cell<BorrowFlag>,
}

unsafe impl<T> PyLayout<T> for PyCellBase<T>
where
    T: PyTypeInfo + PyNativeType,
    T::Layout: PySizedLayout<T>,
{
    const IS_NATIVE_TYPE: bool = true;
}

// Thes impls ensures `PyCellBase` can be a base type.
impl<T> PySizedLayout<T> for PyCellBase<T>
where
    T: PyTypeInfo + PyNativeType,
    T::Layout: PySizedLayout<T>,
{
}

unsafe impl<T> PyBorrowFlagLayout<T> for PyCellBase<T>
where
    T: PyTypeInfo + PyNativeType,
    T::Layout: PySizedLayout<T>,
{
}

/// Inner type of `PyCell` without dict slots and reference counter.
/// This struct has two usages:
/// 1. As an inner type of `PyRef` and `PyRefMut`.
/// 2. When `#[pyclass(extends=Base)]` is specified, `PyCellInner<Base>` is used as a base layout.
#[doc(hidden)]
#[repr(C)]
pub struct PyCellInner<T: PyClass> {
    ob_base: T::BaseLayout,
    value: ManuallyDrop<UnsafeCell<T>>,
}

impl<T: PyClass> AsPyPointer for PyCellInner<T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        (self as *const _) as *mut _
    }
}

unsafe impl<T: PyClass> PyLayout<T> for PyCellInner<T> {
    const IS_NATIVE_TYPE: bool = false;
    fn get_super(&mut self) -> Option<&mut T::BaseLayout> {
        Some(&mut self.ob_base)
    }
    unsafe fn py_init(&mut self, value: T) {
        self.value = ManuallyDrop::new(UnsafeCell::new(value));
    }
    unsafe fn py_drop(&mut self, py: Python) {
        ManuallyDrop::drop(&mut self.value);
        self.ob_base.py_drop(py);
    }
}

// These impls ensures `PyCellInner` can be a base type.
impl<T: PyClass> PySizedLayout<T> for PyCellInner<T> {}
unsafe impl<T: PyClass> PyBorrowFlagLayout<T> for PyCellInner<T> {}

impl<T: PyClass> PyCellInner<T> {
    unsafe fn get_ptr(&self) -> *mut T {
        self.value.get()
    }
    fn get_borrow_flag(&self) -> BorrowFlag {
        let base = (&self.ob_base) as *const _ as *const PyCellBase<T::BaseNativeType>;
        unsafe { (*base).borrow_flag.get() }
    }
    fn set_borrow_flag(&self, flag: BorrowFlag) {
        let base = (&self.ob_base) as *const _ as *const PyCellBase<T::BaseNativeType>;
        unsafe { (*base).borrow_flag.set(flag) }
    }
}

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
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// let book = Book {
///     name: "The Man in the High Castle",
///     author: "Philip Kindred Dick",
/// };
/// let book_cell = PyCell::new(py, book).unwrap();
/// // you can expose PyCell to Python snippets
/// pyo3::py_run!(py, book_cell, "assert book_cell.name[-6:] == 'Castle'");
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
/// # let gil = Python::acquire_gil();
/// # let py = gil.python();
/// # let counter = PyCell::new(py, Counter::default()).unwrap();
/// # pyo3::py_run!(py, counter, "assert counter.increment('cat') == 1");
/// ```
#[repr(C)]
pub struct PyCell<T: PyClass> {
    inner: PyCellInner<T>,
    dict: T::Dict,
    weakref: T::WeakRef,
}

impl<T: PyClass> PyCell<T> {
    /// Make new `PyCell` on the Python heap and returns the reference of it.
    ///
    pub fn new(py: Python, value: impl Into<PyClassInitializer<T>>) -> PyResult<&Self>
    where
        T::BaseLayout: PyBorrowFlagLayout<T::BaseType>,
    {
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
    /// let gil = Python::acquire_gil();
    /// let py = gil.python();
    /// let c = PyCell::new(py, Class {}).unwrap();
    /// {
    ///     let m = c.borrow_mut();
    ///     assert!(c.try_borrow().is_err());
    /// }
    ///
    /// {
    ///     let m = c.borrow();
    ///     assert!(c.try_borrow().is_ok());
    /// }
    /// ```
    pub fn try_borrow(&self) -> Result<PyRef<'_, T>, PyBorrowError> {
        let flag = self.inner.get_borrow_flag();
        if flag == BorrowFlag::HAS_MUTABLE_BORROW {
            Err(PyBorrowError { _private: () })
        } else {
            self.inner.set_borrow_flag(flag.increment());
            Ok(PyRef { inner: &self.inner })
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
    /// let gil = Python::acquire_gil();
    /// let py = gil.python();
    /// let c = PyCell::new(py, Class {}).unwrap();
    /// {
    ///     let m = c.borrow();
    ///     assert!(c.try_borrow_mut().is_err());
    /// }
    ///
    /// assert!(c.try_borrow_mut().is_ok());
    /// ```
    pub fn try_borrow_mut(&self) -> Result<PyRefMut<'_, T>, PyBorrowMutError> {
        if self.inner.get_borrow_flag() != BorrowFlag::UNUSED {
            Err(PyBorrowMutError { _private: () })
        } else {
            self.inner.set_borrow_flag(BorrowFlag::HAS_MUTABLE_BORROW);
            Ok(PyRefMut { inner: &self.inner })
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
    /// let gil = Python::acquire_gil();
    /// let py = gil.python();
    /// let c = PyCell::new(py, Class {}).unwrap();
    ///
    /// {
    ///     let m = c.borrow_mut();
    ///     assert!(unsafe { c.try_borrow_unguarded() }.is_err());
    /// }
    ///
    /// {
    ///     let m = c.borrow();
    ///     assert!(unsafe { c.try_borrow_unguarded() }.is_ok());
    /// }
    /// ```
    pub unsafe fn try_borrow_unguarded(&self) -> Result<&T, PyBorrowError> {
        if self.inner.get_borrow_flag() == BorrowFlag::HAS_MUTABLE_BORROW {
            Err(PyBorrowError { _private: () })
        } else {
            Ok(&*self.inner.value.get())
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

    /// Allocates new PyCell without initilizing value.
    /// Requires `T::BaseLayout: PyBorrowFlagLayout<T::BaseType>` to ensure `self` has a borrow flag.
    pub(crate) unsafe fn internal_new(py: Python) -> PyResult<*mut Self>
    where
        T::BaseLayout: PyBorrowFlagLayout<T::BaseType>,
    {
        let base = T::alloc(py);
        if base.is_null() {
            return Err(PyErr::fetch(py));
        }
        let base = base as *mut PyCellBase<T::BaseNativeType>;
        (*base).borrow_flag = Cell::new(BorrowFlag::UNUSED);
        let self_ = base as *mut Self;
        (*self_).dict = T::Dict::new();
        (*self_).weakref = T::WeakRef::new();
        Ok(self_)
    }
}

unsafe impl<T: PyClass> PyLayout<T> for PyCell<T> {
    const IS_NATIVE_TYPE: bool = false;
    fn get_super(&mut self) -> Option<&mut T::BaseLayout> {
        Some(&mut self.inner.ob_base)
    }
    unsafe fn py_init(&mut self, value: T) {
        self.inner.value = ManuallyDrop::new(UnsafeCell::new(value));
    }
    unsafe fn py_drop(&mut self, py: Python) {
        ManuallyDrop::drop(&mut self.inner.value);
        self.dict.clear_dict(py);
        self.weakref.clear_weakrefs(self.as_ptr(), py);
        self.inner.ob_base.py_drop(py);
    }
}

unsafe impl<T: PyClass> PyDowncastImpl for PyCell<T> {
    unsafe fn unchecked_downcast(obj: &PyAny) -> &Self {
        &*(obj.as_ptr() as *const Self)
    }
    private_impl! {}
}

impl<T: PyClass> AsPyPointer for PyCell<T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.inner.as_ptr()
    }
}

impl<T: PyClass> ToPyObject for &PyCell<T> {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
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
/// # Example
/// You can use `PyRef` as an alternative of `&self` receiver when
/// - You need to access the pointer of `PyCell`.
/// - You want to get super class.
/// ```
/// # use pyo3::prelude::*;
/// #[pyclass]
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
/// # let gil = Python::acquire_gil();
/// # let py = gil.python();
/// # let sub = PyCell::new(py, Child::new()).unwrap();
/// # pyo3::py_run!(py, sub, "assert sub.format() == 'Caterpillar(base: Butterfly, cnt: 4)'");
/// ```
pub struct PyRef<'p, T: PyClass> {
    inner: &'p PyCellInner<T>,
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
    T: PyClass + PyTypeInfo<BaseType = U, BaseLayout = PyCellInner<U>>,
    U: PyClass,
{
    fn as_ref(&self) -> &T::BaseType {
        unsafe { &*self.inner.ob_base.get_ptr() }
    }
}

impl<'p, T, U> PyRef<'p, T>
where
    T: PyClass + PyTypeInfo<BaseType = U, BaseLayout = PyCellInner<U>>,
    U: PyClass,
{
    /// Get `PyRef<T::BaseType>`.
    /// You can use this method to get super class of super class.
    ///
    /// # Examples
    /// ```
    /// # use pyo3::prelude::*;
    /// #[pyclass]
    /// struct Base1 {
    ///     name1: &'static str,
    /// }
    /// #[pyclass(extends=Base1)]
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
    /// # let gil = Python::acquire_gil();
    /// # let py = gil.python();
    /// # let sub = PyCell::new(py, Sub::new()).unwrap();
    /// # pyo3::py_run!(py, sub, "assert sub.name() == 'base1 base2 sub'")
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

impl<'p, T: PyClass> FromPy<PyRef<'p, T>> for PyObject {
    fn from_py(pyref: PyRef<'p, T>, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, pyref.inner.as_ptr()) }
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
    inner: &'p PyCellInner<T>,
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
    T: PyClass + PyTypeInfo<BaseType = U, BaseLayout = PyCellInner<U>>,
    U: PyClass,
{
    fn as_ref(&self) -> &T::BaseType {
        unsafe { &*self.inner.ob_base.get_ptr() }
    }
}

impl<'p, T, U> AsMut<U> for PyRefMut<'p, T>
where
    T: PyClass + PyTypeInfo<BaseType = U, BaseLayout = PyCellInner<U>>,
    U: PyClass,
{
    fn as_mut(&mut self) -> &mut T::BaseType {
        unsafe { &mut *self.inner.ob_base.get_ptr() }
    }
}

impl<'p, T, U> PyRefMut<'p, T>
where
    T: PyClass + PyTypeInfo<BaseType = U, BaseLayout = PyCellInner<U>>,
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

impl<'p, T: PyClass> FromPy<PyRefMut<'p, T>> for PyObject {
    fn from_py(pyref: PyRefMut<'p, T>, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, pyref.inner.as_ptr()) }
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

#[derive(Copy, Clone, Eq, PartialEq)]
struct BorrowFlag(usize);

impl BorrowFlag {
    const UNUSED: BorrowFlag = BorrowFlag(0);
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

pyo3_exception!(PyBorrowError, crate::exceptions::RuntimeError);
pyo3_exception!(PyBorrowMutError, crate::exceptions::RuntimeError);
