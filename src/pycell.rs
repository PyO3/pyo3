//! PyO3's interior mutability primitive.
//!
//! Rust has strict aliasing rules - you can either have any number of immutable (shared) references or one mutable
//! reference. Python's ownership model is the complete opposite of that - any Python object
//! can be referenced any number of times, and mutation is allowed from any reference.
//!
//! PyO3 deals with these differences by employing the [Interior Mutability]
//! pattern. This requires that PyO3 enforces the borrowing rules and it has two mechanisms for
//! doing so:
//! - Statically it can enforce threadsafe access with the [`Python<'py>`](crate::Python) token.
//! All Rust code holding that token, or anything derived from it, can assume that they have
//! safe access to the Python interpreter's state. For this reason all the native Python objects
//! can be mutated through shared references.
//! - However, methods and functions in Rust usually *do* need `&mut` references. While PyO3 can
//! use the [`Python<'py>`](crate::Python) token to guarantee thread-safe access to them, it cannot
//! statically guarantee uniqueness of `&mut` references. As such those references have to be tracked
//! dynamically at runtime, using [`PyCell`] and the other types defined in this module. This works
//! similar to std's [`RefCell`](std::cell::RefCell) type.
//!
//! # When *not* to use PyCell
//!
//! Usually you can use `&mut` references as method and function receivers and arguments, and you
//! won't need to use [`PyCell`] directly:
//!
//! ```rust
//! use pyo3::prelude::*;
//!
//! #[pyclass]
//! struct Number {
//!     inner: u32,
//! }
//!
//! #[pymethods]
//! impl Number {
//!     fn increment(&mut self) {
//!         self.inner += 1;
//!     }
//! }
//! ```
//!
//! The [`#[pymethods]`](crate::pymethods) proc macro will generate this wrapper function (and more),
//! using [`PyCell`] under the hood:
//!
//! ```ignore
//! // This function is exported to Python.
//! unsafe extern "C" fn __wrap(slf: *mut PyObject, _args: *mut PyObject) -> *mut PyObject {
//!     pyo3::callback::handle_panic(|py| {
//!         let cell: &PyCell<Number> = py.from_borrowed_ptr(slf);
//!         let mut _ref: PyRefMut<Number> = cell.try_borrow_mut()?;
//!         let slf: &mut Number = &mut _ref;
//!         pyo3::callback::convert(py, Number::increment(slf))
//!     })
//! }
//!  ```
//!
//! # When to use PyCell
//! ## Using pyclasses from Rust
//!
//! However, we *do* need [`PyCell`] if we want to call its methods from Rust:
//! ```rust
//! # use pyo3::prelude::*;
//! #
//! # #[pyclass]
//! # struct Number {
//! #     inner: u32,
//! # }
//! #
//! # #[pymethods]
//! # impl Number {
//! #     fn increment(&mut self) {
//! #         self.inner += 1;
//! #     }
//! # }
//! # fn main() -> PyResult<()> {
//! Python::with_gil(|py| {
//!     let n = Py::new(py, Number { inner: 0 })?;
//!
//!     // We borrow the guard and then dereference
//!     // it to get a mutable reference to Number
//!     let mut guard: PyRefMut<'_, Number> = n.as_ref(py).borrow_mut();
//!     let n_mutable: &mut Number = &mut *guard;
//!
//!     n_mutable.increment();
//!
//!     // To avoid panics we must dispose of the
//!     // `PyRefMut` before borrowing again.
//!     drop(guard);
//!
//!     let n_immutable : &Number = &n.as_ref(py).borrow();
//!     assert_eq!(n_immutable.inner, 1);
//!
//!     Ok(())
//! })
//! # }
//! ```
//! ## Dealing with possibly overlapping mutable references
//!
//! It is also necessary to use [`PyCell`] if you can receive mutable arguments that may overlap.
//! Suppose the following function that swaps the values of two `Number`s:
//! ```
//! # use pyo3::prelude::*;
//! # #[pyclass]
//! # pub struct Number {
//! #     inner: u32,
//! # }
//! #[pyfunction]
//! fn swap_numbers(a: &mut Number, b: &mut Number) {
//!     std::mem::swap(&mut a.inner, &mut b.inner);
//! }
//! # use pyo3::AsPyPointer;
//! # fn main() {
//! #     Python::with_gil(|py|{
//! #         let n = Py::new(py, Number{inner: 35}).unwrap();
//! #         let n2 = n.clone_ref(py);
//! #         assert_eq!(n.as_ptr(), n2.as_ptr());
//! #         let fun = pyo3::wrap_pyfunction!(swap_numbers, py).unwrap();
//! #         fun.call1((n, n2)).expect_err("Managed to create overlapping mutable references. Note: this is undefined behaviour.");
//! #     });
//! # }
//! ```
//! When users pass in the same `Number` as both arguments, one of the mutable borrows will
//! fail and raise a `RuntimeError`:
//! ```text
//! >>> a = Number()
//! >>> swap_numbers(a, a)
//! Traceback (most recent call last):
//!   File "<stdin>", line 1, in <module>
//!   RuntimeError: Already borrowed
//! ```
//!
//! It is better to write that function like this:
//! ```rust
//! # use pyo3::prelude::*;
//! # #[pyclass]
//! # pub struct Number {
//! #     inner: u32,
//! # }
//! #[pyfunction]
//! fn swap_numbers(a: &PyCell<Number>, b: &PyCell<Number>) {
//!     // Check that the pointers are unequal
//!     if a.as_ptr() != b.as_ptr() {
//!         std::mem::swap(&mut a.borrow_mut().inner, &mut b.borrow_mut().inner);
//!     } else {
//!         // Do nothing - they are the same object, so don't need swapping.
//!     }
//! }
//! # use pyo3::AsPyPointer;
//! # fn main() {
//! #     // With duplicate numbers
//! #     Python::with_gil(|py|{
//! #         let n = Py::new(py, Number{inner: 35}).unwrap();
//! #         let n2 = n.clone_ref(py);
//! #         assert_eq!(n.as_ptr(), n2.as_ptr());
//! #         let fun = pyo3::wrap_pyfunction!(swap_numbers, py).unwrap();
//! #         fun.call1((n, n2)).unwrap();
//! #     });
//! #
//! #     // With two different numbers
//! #     Python::with_gil(|py|{
//! #         let n = Py::new(py, Number{inner: 35}).unwrap();
//! #         let n2 = Py::new(py, Number{inner: 42}).unwrap();
//! #         assert_ne!(n.as_ptr(), n2.as_ptr());
//! #         let fun = pyo3::wrap_pyfunction!(swap_numbers, py).unwrap();
//! #         fun.call1((&n, &n2)).unwrap();
//! #         let n: u32 = n.borrow(py).inner;
//! #         let n2: u32 = n2.borrow(py).inner;
//! #         assert_eq!(n, 42);
//! #         assert_eq!(n2, 35);
//! #     });
//! # }
//! ```
//! See the [guide] for more information.
//!
//! [guide]: https://pyo3.rs/latest/class.html#pycell-and-interior-mutability "PyCell and interior mutability"
//! [Interior Mutability]: https://doc.rust-lang.org/book/ch15-05-interior-mutability.html "RefCell<T> and the Interior Mutability Pattern - The Rust Programming Language"

use crate::exceptions::PyRuntimeError;
use crate::impl_::pyclass::{PyClassBaseType, PyClassDict, PyClassThreadChecker, PyClassWeakRef};
use crate::pyclass::PyClass;
use crate::pyclass_init::PyClassInitializer;
use crate::type_object::{PyLayout, PySizedLayout};
use crate::types::PyAny;
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

/// A container type for (mutably) accessing [`PyClass`] values
///
/// `PyCell` autodereferences to [`PyAny`], so you can call `PyAny`'s methods on a `PyCell<T>`.
///
/// # Examples
///
/// This example demonstrates getting a mutable reference of the contained `PyClass`.
/// ```rust
/// use pyo3::prelude::*;
///
/// #[pyclass]
/// struct Number {
///     inner: u32,
/// }
///
/// #[pymethods]
/// impl Number {
///     fn increment(&mut self) {
///         self.inner += 1;
///     }
/// }
///
/// # fn main() -> PyResult<()> {
/// Python::with_gil(|py| {
///     let n = PyCell::new(py, Number { inner: 0 })?;
///
///     let n_mutable: &mut Number = &mut n.borrow_mut();
///     n_mutable.increment();
///
///     Ok(())
/// })
/// # }
/// ```
/// For more information on how, when and why (not) to use `PyCell` please see the
/// [module-level documentation](self).
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

unsafe impl<T: PyClass> PyNativeType for PyCell<T> {}

impl<T: PyClass> PyCell<T> {
    /// Makes a new `PyCell` on the Python heap and return the reference to it.
    ///
    /// In cases where the value in the cell does not need to be accessed immediately after
    /// creation, consider [`Py::new`](crate::Py::new) as a more efficient alternative.
    pub fn new(py: Python<'_>, value: impl Into<PyClassInitializer<T>>) -> PyResult<&Self> {
        unsafe {
            let initializer = value.into();
            let self_ = initializer.create_cell(py)?;
            FromPyPointer::from_owned_ptr_or_err(py, self_ as _)
        }
    }

    /// Immutably borrows the value `T`. This borrow lasts as long as the returned `PyRef` exists.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow`](#method.try_borrow).
    pub fn borrow(&self) -> PyRef<'_, T> {
        self.try_borrow().expect("Already mutably borrowed")
    }

    /// Mutably borrows the value `T`. This borrow lasts as long as the returned `PyRefMut` exists.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed. For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    pub fn borrow_mut(&self) -> PyRefMut<'_, T> {
        self.try_borrow_mut().expect("Already borrowed")
    }

    /// Immutably borrows the value `T`, returning an error if the value is currently
    /// mutably borrowed. This borrow lasts as long as the returned `PyRef` exists.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// #[pyclass]
    /// struct Class {}
    ///
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
    /// This borrow lasts as long as the returned `PyRefMut` exists.
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

    /// Replaces the wrapped value with a new one, returning the old value.
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

    /// Gets the offset of the dictionary from the start of the struct in bytes.
    pub(crate) fn dict_offset() -> ffi::Py_ssize_t {
        use std::convert::TryInto;
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
        offset.try_into().expect("offset should fit in Py_ssize_t")
    }

    /// Gets the offset of the weakref list from the start of the struct in bytes.
    pub(crate) fn weaklist_offset() -> ffi::Py_ssize_t {
        use std::convert::TryInto;
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
        offset.try_into().expect("offset should fit in Py_ssize_t")
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

/// A wrapper type for an immutably borrowed value from a [`PyCell`]`<T>`.
///
/// See the [`PyCell`] documentation for more information.
///
/// # Examples
///
/// You can use `PyRef` as an alternative to a `&self` receiver when
/// - you need to access the pointer of the `PyCell`, or
/// - you want to get a super class.
/// ```
/// # use pyo3::prelude::*;
/// #[pyclass(subclass)]
/// struct Parent {
///     basename: &'static str,
/// }
///
/// #[pyclass(extends=Parent)]
/// struct Child {
///     name: &'static str,
///  }
///
/// #[pymethods]
/// impl Child {
///     #[new]
///     fn new() -> (Self, Parent) {
///         (Child { name: "Caterpillar" }, Parent { basename: "Butterfly" })
///     }
///
///     fn format(slf: PyRef<'_, Self>) -> String {
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
///
/// See the [module-level documentation](self) for more information.
pub struct PyRef<'p, T: PyClass> {
    inner: &'p PyCell<T>,
}

impl<'p, T: PyClass> PyRef<'p, T> {
    /// Returns a `Python` token that is bound to the lifetime of the `PyRef`.
    pub fn py(&self) -> Python<'_> {
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
    /// Gets a `PyRef<T::BaseType>`.
    ///
    /// While `as_ref()` returns a reference of type `&T::BaseType`, this cannot be
    /// used to get the base of `T::BaseType`.
    ///
    /// But with the help of this method, you can get hold of instances of the
    /// super-superclass when needed.
    ///
    /// # Examples
    /// ```
    /// # use pyo3::prelude::*;
    /// #[pyclass(subclass)]
    /// struct Base1 {
    ///     name1: &'static str,
    /// }
    ///
    /// #[pyclass(extends=Base1, subclass)]
    /// struct Base2 {
    ///     name2: &'static str,
    /// }
    ///
    /// #[pyclass(extends=Base2)]
    /// struct Sub {
    ///     name3: &'static str,
    /// }
    ///
    /// #[pymethods]
    /// impl Sub {
    ///     #[new]
    ///     fn new() -> PyClassInitializer<Self> {
    ///         PyClassInitializer::from(Base1 { name1: "base1" })
    ///             .add_subclass(Base2 { name2: "base2" })
    ///             .add_subclass(Self { name3: "sub" })
    ///     }
    ///     fn name(slf: PyRef<'_, Self>) -> String {
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

/// A wrapper type for a mutably borrowed value from a[`PyCell`]`<T>`.
///
/// See the [module-level documentation](self) for more information.
pub struct PyRefMut<'p, T: PyClass> {
    inner: &'p PyCell<T>,
}

impl<'p, T: PyClass> PyRefMut<'p, T> {
    /// Returns a `Python` token that is bound to the lifetime of the `PyRefMut`.
    pub fn py(&self) -> Python<'_> {
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
    /// Gets a `PyRef<T::BaseType>`.
    ///
    /// See [`PyRef::into_super`] for more.
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
    fn into_py(self, py: Python<'_>) -> PyObject {
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

/// An error type returned by [`PyCell::try_borrow`].
///
/// If this error is allowed to bubble up into Python code it will raise a `RuntimeError`.
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

/// An error type returned by [`PyCell::try_borrow_mut`].
///
/// If this error is allowed to bubble up into Python code it will raise a `RuntimeError`.
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
    unsafe fn tp_dealloc(slf: *mut ffi::PyObject, py: Python<'_>);
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
    unsafe fn tp_dealloc(slf: *mut ffi::PyObject, py: Python<'_>) {
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
    unsafe fn tp_dealloc(slf: *mut ffi::PyObject, py: Python<'_>) {
        // Safety: Python only calls tp_dealloc when no references to the object remain.
        let cell = &mut *(slf as *mut PyCell<T>);
        ManuallyDrop::drop(&mut cell.contents.value);
        cell.contents.dict.clear_dict(py);
        cell.contents.weakref.clear_weakrefs(slf, py);
        <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(slf, py)
    }
}
