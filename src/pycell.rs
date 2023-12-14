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
//! ```rust
//! # use pyo3::prelude::*;
//! # #[pyclass]
//! # struct Number {
//! #    inner: u32,
//! # }
//! #
//! # #[pymethods]
//! # impl Number {
//! #    fn increment(&mut self) {
//! #        self.inner += 1;
//! #    }
//! # }
//! #
//! // The function which is exported to Python looks roughly like the following
//! unsafe extern "C" fn __pymethod_increment__(
//!     _slf: *mut pyo3::ffi::PyObject,
//!     _args: *mut pyo3::ffi::PyObject,
//! ) -> *mut pyo3::ffi::PyObject {
//!     use :: pyo3 as _pyo3;
//!     _pyo3::impl_::trampoline::noargs(_slf, _args, |py, _slf| {
//!         let _cell = py
//!             .from_borrowed_ptr::<_pyo3::PyAny>(_slf)
//!             .downcast::<_pyo3::PyCell<Number>>()?;
//!         let mut _ref = _cell.try_borrow_mut()?;
//!         let _slf: &mut Number = &mut *_ref;
//!         _pyo3::callback::convert(py, Number::increment(_slf))
//!     })
//! }
//! ```
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
//!     let n_immutable: &Number = &n.as_ref(py).borrow();
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
//! # fn main() {
//! #     Python::with_gil(|py| {
//! #         let n = Py::new(py, Number{inner: 35}).unwrap();
//! #         let n2 = n.clone_ref(py);
//! #         assert!(n.is(&n2));
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
//!     if !a.is(b) {
//!         std::mem::swap(&mut a.borrow_mut().inner, &mut b.borrow_mut().inner);
//!     } else {
//!         // Do nothing - they are the same object, so don't need swapping.
//!     }
//! }
//! # fn main() {
//! #     // With duplicate numbers
//! #     Python::with_gil(|py| {
//! #         let n = Py::new(py, Number{inner: 35}).unwrap();
//! #         let n2 = n.clone_ref(py);
//! #         assert!(n.is(&n2));
//! #         let fun = pyo3::wrap_pyfunction!(swap_numbers, py).unwrap();
//! #         fun.call1((n, n2)).unwrap();
//! #     });
//! #
//! #     // With two different numbers
//! #     Python::with_gil(|py| {
//! #         let n = Py::new(py, Number{inner: 35}).unwrap();
//! #         let n2 = Py::new(py, Number{inner: 42}).unwrap();
//! #         assert!(!n.is(&n2));
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
use crate::impl_::pyclass::{
    PyClassBaseType, PyClassDict, PyClassImpl, PyClassThreadChecker, PyClassWeakRef,
};
use crate::pyclass::{
    boolean_struct::{False, True},
    PyClass,
};
use crate::pyclass_init::PyClassInitializer;
use crate::type_object::{PyLayout, PySizedLayout};
use crate::types::PyAny;
use crate::{
    conversion::{AsPyPointer, FromPyPointer, ToPyObject},
    type_object::get_tp_free,
    PyTypeInfo,
};
use crate::{ffi, IntoPy, PyErr, PyNativeType, PyObject, PyResult, PyTypeCheck, Python};
use std::cell::UnsafeCell;
use std::fmt;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub(crate) mod impl_;
use impl_::{GetBorrowChecker, PyClassBorrowChecker, PyClassMutability};

/// Base layout of PyCell.
#[doc(hidden)]
#[repr(C)]
pub struct PyCellBase<T> {
    ob_base: T,
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
pub struct PyCell<T: PyClassImpl> {
    ob_base: <T::BaseType as PyClassBaseType>::LayoutAsBase,
    contents: PyCellContents<T>,
}

#[repr(C)]
pub(crate) struct PyCellContents<T: PyClassImpl> {
    pub(crate) value: ManuallyDrop<UnsafeCell<T>>,
    pub(crate) borrow_checker: <T::PyClassMutability as PyClassMutability>::Storage,
    pub(crate) thread_checker: T::ThreadChecker,
    pub(crate) dict: T::Dict,
    pub(crate) weakref: T::WeakRef,
}

unsafe impl<T: PyClass> PyNativeType for PyCell<T> {
    type AsRefSource = T;
}

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
    /// For frozen classes, the simpler [`get`][Self::get] is available.
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
    pub fn borrow_mut(&self) -> PyRefMut<'_, T>
    where
        T: PyClass<Frozen = False>,
    {
        self.try_borrow_mut().expect("Already borrowed")
    }

    /// Immutably borrows the value `T`, returning an error if the value is currently
    /// mutably borrowed. This borrow lasts as long as the returned `PyRef` exists.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// For frozen classes, the simpler [`get`][Self::get] is available.
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
        self.ensure_threadsafe();
        self.borrow_checker()
            .try_borrow()
            .map(|_| PyRef { inner: self })
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
    pub fn try_borrow_mut(&self) -> Result<PyRefMut<'_, T>, PyBorrowMutError>
    where
        T: PyClass<Frozen = False>,
    {
        self.ensure_threadsafe();
        self.borrow_checker()
            .try_borrow_mut()
            .map(|_| PyRefMut { inner: self })
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
        self.ensure_threadsafe();
        self.borrow_checker()
            .try_borrow_unguarded()
            .map(|_: ()| &*self.contents.value.get())
    }

    /// Provide an immutable borrow of the value `T` without acquiring the GIL.
    ///
    /// This is available if the class is [`frozen`][macro@crate::pyclass] and [`Sync`].
    ///
    /// While the GIL is usually required to get access to `&PyCell<T>`,
    /// compared to [`borrow`][Self::borrow] or [`try_borrow`][Self::try_borrow]
    /// this avoids any thread or borrow checking overhead at runtime.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// # use pyo3::prelude::*;
    ///
    /// #[pyclass(frozen)]
    /// struct FrozenCounter {
    ///     value: AtomicUsize,
    /// }
    ///
    /// Python::with_gil(|py| {
    ///     let counter = FrozenCounter { value: AtomicUsize::new(0) };
    ///
    ///     let cell = PyCell::new(py, counter).unwrap();
    ///
    ///     cell.get().value.fetch_add(1, Ordering::Relaxed);
    /// });
    /// ```
    pub fn get(&self) -> &T
    where
        T: PyClass<Frozen = True> + Sync,
    {
        // SAFETY: The class itself is frozen and `Sync` and we do not access anything but `self.contents.value`.
        unsafe { &*self.get_ptr() }
    }

    /// Replaces the wrapped value with a new one, returning the old value.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    #[inline]
    pub fn replace(&self, t: T) -> T
    where
        T: PyClass<Frozen = False>,
    {
        std::mem::replace(&mut *self.borrow_mut(), t)
    }

    /// Replaces the wrapped value with a new one computed from `f`, returning the old value.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn replace_with<F: FnOnce(&mut T) -> T>(&self, f: F) -> T
    where
        T: PyClass<Frozen = False>,
    {
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
    pub fn swap(&self, other: &Self)
    where
        T: PyClass<Frozen = False>,
    {
        std::mem::swap(&mut *self.borrow_mut(), &mut *other.borrow_mut())
    }

    pub(crate) fn get_ptr(&self) -> *mut T {
        self.contents.value.get()
    }

    /// Gets the offset of the dictionary from the start of the struct in bytes.
    pub(crate) fn dict_offset() -> ffi::Py_ssize_t {
        use memoffset::offset_of;

        let offset = offset_of!(PyCell<T>, contents) + offset_of!(PyCellContents<T>, dict);

        // Py_ssize_t may not be equal to isize on all platforms
        #[allow(clippy::useless_conversion)]
        offset.try_into().expect("offset should fit in Py_ssize_t")
    }

    /// Gets the offset of the weakref list from the start of the struct in bytes.
    pub(crate) fn weaklist_offset() -> ffi::Py_ssize_t {
        use memoffset::offset_of;

        let offset = offset_of!(PyCell<T>, contents) + offset_of!(PyCellContents<T>, weakref);

        // Py_ssize_t may not be equal to isize on all platforms
        #[allow(clippy::useless_conversion)]
        offset.try_into().expect("offset should fit in Py_ssize_t")
    }

    #[cfg(feature = "macros")]
    pub(crate) fn release_ref(&self) {
        self.borrow_checker().release_borrow();
    }

    #[cfg(feature = "macros")]
    pub(crate) fn release_mut(&self) {
        self.borrow_checker().release_borrow_mut();
    }
}

impl<T: PyClassImpl> PyCell<T> {
    fn borrow_checker(&self) -> &<T::PyClassMutability as PyClassMutability>::Checker {
        T::PyClassMutability::borrow_checker(self)
    }
}

unsafe impl<T: PyClassImpl> PyLayout<T> for PyCell<T> {}
impl<T: PyClass> PySizedLayout<T> for PyCell<T> {}

impl<T> PyTypeCheck for PyCell<T>
where
    T: PyClass,
{
    const NAME: &'static str = <T as PyTypeCheck>::NAME;

    fn type_check(object: &PyAny) -> bool {
        <T as PyTypeCheck>::type_check(object)
    }
}

unsafe impl<T: PyClass> AsPyPointer for PyCell<T> {
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
    pub fn py(&self) -> Python<'p> {
        self.inner.py()
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

impl<'p, T: PyClass> PyRef<'p, T> {
    /// Returns the raw FFI pointer represented by self.
    ///
    /// # Safety
    ///
    /// Callers are responsible for ensuring that the pointer does not outlive self.
    ///
    /// The reference is borrowed; callers should not decrease the reference count
    /// when they are finished with the pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::PyObject {
        self.inner.as_ptr()
    }

    /// Returns an owned raw FFI pointer represented by self.
    ///
    /// # Safety
    ///
    /// The reference is owned; when finished the caller should either transfer ownership
    /// of the pointer or decrease the reference count (e.g. with [`pyo3::ffi::Py_DecRef`](crate::ffi::Py_DecRef)).
    #[inline]
    pub fn into_ptr(self) -> *mut ffi::PyObject {
        self.inner.into_ptr()
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
        self.inner.borrow_checker().release_borrow()
    }
}

impl<T: PyClass> IntoPy<PyObject> for PyRef<'_, T> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.inner.as_ptr()) }
    }
}

impl<T: PyClass> IntoPy<PyObject> for &'_ PyRef<'_, T> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.inner.into_py(py)
    }
}

impl<'a, T: PyClass> std::convert::TryFrom<&'a PyCell<T>> for crate::PyRef<'a, T> {
    type Error = PyBorrowError;
    fn try_from(cell: &'a crate::PyCell<T>) -> Result<Self, Self::Error> {
        cell.try_borrow()
    }
}

unsafe impl<'a, T: PyClass> AsPyPointer for PyRef<'a, T> {
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
pub struct PyRefMut<'p, T: PyClass<Frozen = False>> {
    inner: &'p PyCell<T>,
}

impl<'p, T: PyClass<Frozen = False>> PyRefMut<'p, T> {
    /// Returns a `Python` token that is bound to the lifetime of the `PyRefMut`.
    pub fn py(&self) -> Python<'p> {
        self.inner.py()
    }
}

impl<'p, T, U> AsRef<U> for PyRefMut<'p, T>
where
    T: PyClass<BaseType = U, Frozen = False>,
    U: PyClass<Frozen = False>,
{
    fn as_ref(&self) -> &T::BaseType {
        unsafe { &*self.inner.ob_base.get_ptr() }
    }
}

impl<'p, T, U> AsMut<U> for PyRefMut<'p, T>
where
    T: PyClass<BaseType = U, Frozen = False>,
    U: PyClass<Frozen = False>,
{
    fn as_mut(&mut self) -> &mut T::BaseType {
        unsafe { &mut *self.inner.ob_base.get_ptr() }
    }
}

impl<'p, T: PyClass<Frozen = False>> PyRefMut<'p, T> {
    /// Returns the raw FFI pointer represented by self.
    ///
    /// # Safety
    ///
    /// Callers are responsible for ensuring that the pointer does not outlive self.
    ///
    /// The reference is borrowed; callers should not decrease the reference count
    /// when they are finished with the pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut ffi::PyObject {
        self.inner.as_ptr()
    }

    /// Returns an owned raw FFI pointer represented by self.
    ///
    /// # Safety
    ///
    /// The reference is owned; when finished the caller should either transfer ownership
    /// of the pointer or decrease the reference count (e.g. with [`pyo3::ffi::Py_DecRef`](crate::ffi::Py_DecRef)).
    #[inline]
    pub fn into_ptr(self) -> *mut ffi::PyObject {
        self.inner.into_ptr()
    }
}

impl<'p, T, U> PyRefMut<'p, T>
where
    T: PyClass<BaseType = U, Frozen = False>,
    U: PyClass<Frozen = False>,
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

impl<'p, T: PyClass<Frozen = False>> Deref for PyRefMut<'p, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.inner.get_ptr() }
    }
}

impl<'p, T: PyClass<Frozen = False>> DerefMut for PyRefMut<'p, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.get_ptr() }
    }
}

impl<'p, T: PyClass<Frozen = False>> Drop for PyRefMut<'p, T> {
    fn drop(&mut self) {
        self.inner.borrow_checker().release_borrow_mut()
    }
}

impl<T: PyClass<Frozen = False>> IntoPy<PyObject> for PyRefMut<'_, T> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.inner.as_ptr()) }
    }
}

impl<T: PyClass<Frozen = False>> IntoPy<PyObject> for &'_ PyRefMut<'_, T> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.inner.into_py(py)
    }
}

unsafe impl<'a, T: PyClass<Frozen = False>> AsPyPointer for PyRefMut<'a, T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.inner.as_ptr()
    }
}

impl<'a, T: PyClass<Frozen = False>> std::convert::TryFrom<&'a PyCell<T>>
    for crate::PyRefMut<'a, T>
{
    type Error = PyBorrowMutError;
    fn try_from(cell: &'a crate::PyCell<T>) -> Result<Self, Self::Error> {
        cell.try_borrow_mut()
    }
}

impl<T: PyClass<Frozen = False> + fmt::Debug> fmt::Debug for PyRefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
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
    fn ensure_threadsafe(&self);
    /// Implementation of tp_dealloc.
    /// # Safety
    /// - slf must be a valid pointer to an instance of a T or a subclass.
    /// - slf must not be used after this call (as it will be freed).
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject);
}

impl<T, U> PyCellLayout<T> for PyCellBase<U>
where
    U: PySizedLayout<T>,
    T: PyTypeInfo,
{
    fn ensure_threadsafe(&self) {}
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        let type_obj = T::type_object_raw(py);
        // For `#[pyclass]` types which inherit from PyAny, we can just call tp_free
        if type_obj == std::ptr::addr_of_mut!(ffi::PyBaseObject_Type) {
            return get_tp_free(ffi::Py_TYPE(slf))(slf as _);
        }

        // More complex native types (e.g. `extends=PyDict`) require calling the base's dealloc.
        #[cfg(not(Py_LIMITED_API))]
        {
            if let Some(dealloc) = (*type_obj).tp_dealloc {
                // Before CPython 3.11 BaseException_dealloc would use Py_GC_UNTRACK which
                // assumes the exception is currently GC tracked, so we have to re-track
                // before calling the dealloc so that it can safely call Py_GC_UNTRACK.
                #[cfg(not(any(Py_3_11, PyPy)))]
                if ffi::PyType_FastSubclass(type_obj, ffi::Py_TPFLAGS_BASE_EXC_SUBCLASS) == 1 {
                    ffi::PyObject_GC_Track(slf.cast());
                }
                dealloc(slf as _);
            } else {
                get_tp_free(ffi::Py_TYPE(slf))(slf as _);
            }
        }

        #[cfg(Py_LIMITED_API)]
        unreachable!("subclassing native types is not possible with the `abi3` feature");
    }
}

impl<T: PyClassImpl> PyCellLayout<T> for PyCell<T>
where
    <T::BaseType as PyClassBaseType>::LayoutAsBase: PyCellLayout<T::BaseType>,
{
    fn ensure_threadsafe(&self) {
        self.contents.thread_checker.ensure();
        self.ob_base.ensure_threadsafe();
    }
    unsafe fn tp_dealloc(py: Python<'_>, slf: *mut ffi::PyObject) {
        // Safety: Python only calls tp_dealloc when no references to the object remain.
        let cell = &mut *(slf as *mut PyCell<T>);
        if cell.contents.thread_checker.can_drop(py) {
            ManuallyDrop::drop(&mut cell.contents.value);
        }
        cell.contents.dict.clear_dict(py);
        cell.contents.weakref.clear_weakrefs(slf, py);
        <T::BaseType as PyClassBaseType>::LayoutAsBase::tp_dealloc(py, slf)
    }
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {

    use super::*;

    #[crate::pyclass]
    #[pyo3(crate = "crate")]
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    struct SomeClass(i32);

    #[test]
    fn pycell_replace() {
        Python::with_gil(|py| {
            let cell = PyCell::new(py, SomeClass(0)).unwrap();
            assert_eq!(*cell.borrow(), SomeClass(0));

            let previous = cell.replace(SomeClass(123));
            assert_eq!(previous, SomeClass(0));
            assert_eq!(*cell.borrow(), SomeClass(123));
        })
    }

    #[test]
    #[should_panic(expected = "Already borrowed: PyBorrowMutError")]
    fn pycell_replace_panic() {
        Python::with_gil(|py| {
            let cell = PyCell::new(py, SomeClass(0)).unwrap();
            let _guard = cell.borrow();

            cell.replace(SomeClass(123));
        })
    }

    #[test]
    fn pycell_replace_with() {
        Python::with_gil(|py| {
            let cell = PyCell::new(py, SomeClass(0)).unwrap();
            assert_eq!(*cell.borrow(), SomeClass(0));

            let previous = cell.replace_with(|value| {
                *value = SomeClass(2);
                SomeClass(123)
            });
            assert_eq!(previous, SomeClass(2));
            assert_eq!(*cell.borrow(), SomeClass(123));
        })
    }

    #[test]
    #[should_panic(expected = "Already borrowed: PyBorrowMutError")]
    fn pycell_replace_with_panic() {
        Python::with_gil(|py| {
            let cell = PyCell::new(py, SomeClass(0)).unwrap();
            let _guard = cell.borrow();

            cell.replace_with(|_| SomeClass(123));
        })
    }

    #[test]
    fn pycell_swap() {
        Python::with_gil(|py| {
            let cell = PyCell::new(py, SomeClass(0)).unwrap();
            let cell2 = PyCell::new(py, SomeClass(123)).unwrap();
            assert_eq!(*cell.borrow(), SomeClass(0));
            assert_eq!(*cell2.borrow(), SomeClass(123));

            cell.swap(cell2);
            assert_eq!(*cell.borrow(), SomeClass(123));
            assert_eq!(*cell2.borrow(), SomeClass(0));
        })
    }

    #[test]
    #[should_panic(expected = "Already borrowed: PyBorrowMutError")]
    fn pycell_swap_panic() {
        Python::with_gil(|py| {
            let cell = PyCell::new(py, SomeClass(0)).unwrap();
            let cell2 = PyCell::new(py, SomeClass(123)).unwrap();

            let _guard = cell.borrow();
            cell.swap(cell2);
        })
    }

    #[test]
    #[should_panic(expected = "Already borrowed: PyBorrowMutError")]
    fn pycell_swap_panic_other_borrowed() {
        Python::with_gil(|py| {
            let cell = PyCell::new(py, SomeClass(0)).unwrap();
            let cell2 = PyCell::new(py, SomeClass(123)).unwrap();

            let _guard = cell2.borrow();
            cell.swap(cell2);
        })
    }

    #[test]
    fn test_as_ptr() {
        Python::with_gil(|py| {
            let cell = PyCell::new(py, SomeClass(0)).unwrap();
            let ptr = cell.as_ptr();

            assert_eq!(cell.borrow().as_ptr(), ptr);
            assert_eq!(cell.borrow_mut().as_ptr(), ptr);
        })
    }

    #[test]
    fn test_into_ptr() {
        Python::with_gil(|py| {
            let cell = PyCell::new(py, SomeClass(0)).unwrap();
            let ptr = cell.as_ptr();

            assert_eq!(cell.borrow().into_ptr(), ptr);
            unsafe { ffi::Py_DECREF(ptr) };

            assert_eq!(cell.borrow_mut().into_ptr(), ptr);
            unsafe { ffi::Py_DECREF(ptr) };
        })
    }
}
