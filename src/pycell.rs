//! PyO3's interior mutability primitive.
//!
//! Rust has strict aliasing rules - you can either have any number of immutable (shared) references or one mutable
//! reference. Python's ownership model is the complete opposite of that - any Python object
//! can be referenced any number of times, and mutation is allowed from any reference.
//!
//! PyO3 deals with these differences by employing the [Interior Mutability]
//! pattern. This requires that PyO3 enforces the borrowing rules and it has two mechanisms for
//! doing so:
//! - Statically it can enforce thread-safe access with the [`Python<'py>`](crate::Python) token.
//!   All Rust code holding that token, or anything derived from it, can assume that they have
//!   safe access to the Python interpreter's state. For this reason all the native Python objects
//!   can be mutated through shared references.
//! - However, methods and functions in Rust usually *do* need `&mut` references. While PyO3 can
//!   use the [`Python<'py>`](crate::Python) token to guarantee thread-safe access to them, it cannot
//!   statically guarantee uniqueness of `&mut` references. As such those references have to be tracked
//!   dynamically at runtime, using `PyCell` and the other types defined in this module. This works
//!   similar to std's [`RefCell`](std::cell::RefCell) type.
//!
//! # When *not* to use PyCell
//!
//! Usually you can use `&mut` references as method and function receivers and arguments, and you
//! won't need to use `PyCell` directly:
//!
//! ```rust,no_run
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
//! using `PyCell` under the hood:
//!
//! ```rust,ignore
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
//! #       #[allow(deprecated)]
//!         let _cell = py
//!             .from_borrowed_ptr::<_pyo3::PyAny>(_slf)
//!             .cast::<_pyo3::PyCell<Number>>()?;
//!         let mut _ref = _cell.try_borrow_mut()?;
//!         let _slf: &mut Number = &mut *_ref;
//!         _pyo3::impl_::callback::convert(py, Number::increment(_slf))
//!     })
//! }
//! ```
//!
//! # When to use PyCell
//! ## Using pyclasses from Rust
//!
//! However, we *do* need `PyCell` if we want to call its methods from Rust:
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
//! Python::attach(|py| {
//!     let n = Py::new(py, Number { inner: 0 })?;
//!
//!     // We borrow the guard and then dereference
//!     // it to get a mutable reference to Number
//!     let mut guard: PyRefMut<'_, Number> = n.bind(py).borrow_mut();
//!     let n_mutable: &mut Number = &mut *guard;
//!
//!     n_mutable.increment();
//!
//!     // To avoid panics we must dispose of the
//!     // `PyRefMut` before borrowing again.
//!     drop(guard);
//!
//!     let n_immutable: &Number = &n.bind(py).borrow();
//!     assert_eq!(n_immutable.inner, 1);
//!
//!     Ok(())
//! })
//! # }
//! ```
//! ## Dealing with possibly overlapping mutable references
//!
//! It is also necessary to use `PyCell` if you can receive mutable arguments that may overlap.
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
//! #     Python::attach(|py| {
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
//! ```rust,ignore
//! # #![allow(deprecated)]
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
//! #     Python::attach(|py| {
//! #         let n = Py::new(py, Number{inner: 35}).unwrap();
//! #         let n2 = n.clone_ref(py);
//! #         assert!(n.is(&n2));
//! #         let fun = pyo3::wrap_pyfunction!(swap_numbers, py).unwrap();
//! #         fun.call1((n, n2)).unwrap();
//! #     });
//! #
//! #     // With two different numbers
//! #     Python::attach(|py| {
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

use crate::conversion::IntoPyObject;
use crate::exceptions::PyRuntimeError;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::internal_tricks::{ptr_from_mut, ptr_from_ref};
use crate::pyclass::{boolean_struct::False, PyClass};
use crate::{ffi, Borrowed, Bound, PyErr, Python};
use std::convert::Infallible;
use std::fmt;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub(crate) mod impl_;
use impl_::{PyClassBorrowChecker, PyClassObjectLayout};

/// A wrapper type for an immutably borrowed value from a [`Bound<'py, T>`].
///
/// See the [`Bound`] documentation for more information.
///
/// # Examples
///
/// You can use [`PyRef`] as an alternative to a `&self` receiver when
/// - you need to access the pointer of the [`Bound`], or
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
/// # Python::attach(|py| {
/// #     let sub = Py::new(py, Child::new()).unwrap();
/// #     pyo3::py_run!(py, sub, "assert sub.format() == 'Caterpillar(base: Butterfly, cnt: 4)', sub.format()");
/// # });
/// ```
///
/// See the [module-level documentation](self) for more information.
#[repr(transparent)]
pub struct PyRef<'p, T: PyClass> {
    // TODO: once the GIL Ref API is removed, consider adding a lifetime parameter to `PyRef` to
    // store `Borrowed` here instead, avoiding reference counting overhead.
    inner: Bound<'p, T>,
}

impl<'p, T: PyClass> PyRef<'p, T> {
    /// Returns a `Python` token that is bound to the lifetime of the `PyRef`.
    pub fn py(&self) -> Python<'p> {
        self.inner.py()
    }
}

impl<T, U> AsRef<U> for PyRef<'_, T>
where
    T: PyClass<BaseType = U>,
    U: PyClass,
{
    fn as_ref(&self) -> &T::BaseType {
        self.as_super()
    }
}

impl<'py, T: PyClass> PyRef<'py, T> {
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
        self.inner.clone().into_ptr()
    }

    #[track_caller]
    pub(crate) fn borrow(obj: &Bound<'py, T>) -> Self {
        Self::try_borrow(obj).expect("Already mutably borrowed")
    }

    pub(crate) fn try_borrow(obj: &Bound<'py, T>) -> Result<Self, PyBorrowError> {
        let cell = obj.get_class_object();
        cell.ensure_threadsafe();
        cell.borrow_checker()
            .try_borrow()
            .map(|_| Self { inner: obj.clone() })
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
    /// # Python::attach(|py| {
    /// #     let sub = Py::new(py, Sub::new()).unwrap();
    /// #     pyo3::py_run!(py, sub, "assert sub.name() == 'base1 base2 sub'")
    /// # });
    /// ```
    pub fn into_super(self) -> PyRef<'p, U> {
        let py = self.py();
        let t_not_frozen = !<T::Frozen as crate::pyclass::boolean_struct::private::Boolean>::VALUE;
        let u_frozen = <U::Frozen as crate::pyclass::boolean_struct::private::Boolean>::VALUE;
        if t_not_frozen && u_frozen {
            // If `T` is mutable subclass of `U` differ, then it is possible that we need to
            // release the borrow count now. (e.g. `U` may have a noop borrow checker so
            // dropping the `PyRef<U>` later would noop and leak the borrow we currently hold.)
            //
            // However it's nontrivial, if `U` itself has a mutable base class `V`,
            // then the borrow checker of both `T` and `U` is the shared borrow checker of `V`.
            //
            // But it's really hard to prove that in the type system, the soundest thing we
            // can do is just add a borrow to `U` now and then release the borrow of `T`.

            self.inner
                .as_super()
                .get_class_object()
                .borrow_checker()
                .try_borrow()
                .expect("this object is already borrowed");

            self.inner
                .get_class_object()
                .borrow_checker()
                .release_borrow()
        };
        PyRef {
            inner: unsafe {
                ManuallyDrop::new(self)
                    .as_ptr()
                    .assume_owned_unchecked(py)
                    .cast_into_unchecked()
            },
        }
    }

    /// Borrows a shared reference to `PyRef<T::BaseType>`.
    ///
    /// With the help of this method, you can access attributes and call methods
    /// on the superclass without consuming the `PyRef<T>`. This method can also
    /// be chained to access the super-superclass (and so on).
    ///
    /// # Examples
    /// ```
    /// # use pyo3::prelude::*;
    /// #[pyclass(subclass)]
    /// struct Base {
    ///     base_name: &'static str,
    /// }
    /// #[pymethods]
    /// impl Base {
    ///     fn base_name_len(&self) -> usize {
    ///         self.base_name.len()
    ///     }
    /// }
    ///
    /// #[pyclass(extends=Base)]
    /// struct Sub {
    ///     sub_name: &'static str,
    /// }
    ///
    /// #[pymethods]
    /// impl Sub {
    ///     #[new]
    ///     fn new() -> (Self, Base) {
    ///         (Self { sub_name: "sub_name" }, Base { base_name: "base_name" })
    ///     }
    ///     fn sub_name_len(&self) -> usize {
    ///         self.sub_name.len()
    ///     }
    ///     fn format_name_lengths(slf: PyRef<'_, Self>) -> String {
    ///         format!("{} {}", slf.as_super().base_name_len(), slf.sub_name_len())
    ///     }
    /// }
    /// # Python::attach(|py| {
    /// #     let sub = Py::new(py, Sub::new()).unwrap();
    /// #     pyo3::py_run!(py, sub, "assert sub.format_name_lengths() == '9 8'")
    /// # });
    /// ```
    pub fn as_super(&self) -> &PyRef<'p, U> {
        let ptr = ptr_from_ref::<Bound<'p, T>>(&self.inner)
            // `Bound<T>` has the same layout as `Bound<T::BaseType>`
            .cast::<Bound<'p, T::BaseType>>()
            // `Bound<T::BaseType>` has the same layout as `PyRef<T::BaseType>`
            .cast::<PyRef<'p, T::BaseType>>();
        unsafe { &*ptr }
    }
}

impl<T: PyClass> Deref for PyRef<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.inner.get_class_object().get_ptr() }
    }
}

impl<T: PyClass> Drop for PyRef<'_, T> {
    fn drop(&mut self) {
        self.inner
            .get_class_object()
            .borrow_checker()
            .release_borrow()
    }
}

impl<'py, T: PyClass> IntoPyObject<'py> for PyRef<'py, T> {
    type Target = T;
    type Output = Bound<'py, T>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = T::PYTHON_TYPE;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.inner.clone())
    }
}

impl<'a, 'py, T: PyClass> IntoPyObject<'py> for &'a PyRef<'py, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, T>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = T::PYTHON_TYPE;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.inner.as_borrowed())
    }
}

impl<T: PyClass + fmt::Debug> fmt::Debug for PyRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

/// A wrapper type for a mutably borrowed value from a [`Bound<'py, T>`].
///
/// See the [module-level documentation](self) for more information.
#[repr(transparent)]
pub struct PyRefMut<'p, T: PyClass<Frozen = False>> {
    // TODO: once the GIL Ref API is removed, consider adding a lifetime parameter to `PyRef` to
    // store `Borrowed` here instead, avoiding reference counting overhead.
    inner: Bound<'p, T>,
}

impl<'p, T: PyClass<Frozen = False>> PyRefMut<'p, T> {
    /// Returns a `Python` token that is bound to the lifetime of the `PyRefMut`.
    pub fn py(&self) -> Python<'p> {
        self.inner.py()
    }
}

impl<T, U> AsRef<U> for PyRefMut<'_, T>
where
    T: PyClass<BaseType = U, Frozen = False>,
    U: PyClass<Frozen = False>,
{
    fn as_ref(&self) -> &T::BaseType {
        PyRefMut::downgrade(self).as_super()
    }
}

impl<T, U> AsMut<U> for PyRefMut<'_, T>
where
    T: PyClass<BaseType = U, Frozen = False>,
    U: PyClass<Frozen = False>,
{
    fn as_mut(&mut self) -> &mut T::BaseType {
        self.as_super()
    }
}

impl<'py, T: PyClass<Frozen = False>> PyRefMut<'py, T> {
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
        self.inner.clone().into_ptr()
    }

    #[inline]
    #[track_caller]
    pub(crate) fn borrow(obj: &Bound<'py, T>) -> Self {
        Self::try_borrow(obj).expect("Already borrowed")
    }

    pub(crate) fn try_borrow(obj: &Bound<'py, T>) -> Result<Self, PyBorrowMutError> {
        let cell = obj.get_class_object();
        cell.ensure_threadsafe();
        cell.borrow_checker()
            .try_borrow_mut()
            .map(|_| Self { inner: obj.clone() })
    }

    pub(crate) fn downgrade(slf: &Self) -> &PyRef<'py, T> {
        // `PyRefMut<T>` and `PyRef<T>` have the same layout
        unsafe { &*ptr_from_ref(slf).cast() }
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
        let py = self.py();
        PyRefMut {
            inner: unsafe {
                ManuallyDrop::new(self)
                    .as_ptr()
                    .assume_owned_unchecked(py)
                    .cast_into_unchecked()
            },
        }
    }

    /// Borrows a mutable reference to `PyRefMut<T::BaseType>`.
    ///
    /// With the help of this method, you can mutate attributes and call mutating
    /// methods on the superclass without consuming the `PyRefMut<T>`. This method
    /// can also be chained to access the super-superclass (and so on).
    ///
    /// See [`PyRef::as_super`] for more.
    pub fn as_super(&mut self) -> &mut PyRefMut<'p, U> {
        let ptr = ptr_from_mut::<Bound<'p, T>>(&mut self.inner)
            // `Bound<T>` has the same layout as `Bound<T::BaseType>`
            .cast::<Bound<'p, T::BaseType>>()
            // `Bound<T::BaseType>` has the same layout as `PyRefMut<T::BaseType>`,
            // and the mutable borrow on `self` prevents aliasing
            .cast::<PyRefMut<'p, T::BaseType>>();
        unsafe { &mut *ptr }
    }
}

impl<T: PyClass<Frozen = False>> Deref for PyRefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.inner.get_class_object().get_ptr() }
    }
}

impl<T: PyClass<Frozen = False>> DerefMut for PyRefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.get_class_object().get_ptr() }
    }
}

impl<T: PyClass<Frozen = False>> Drop for PyRefMut<'_, T> {
    fn drop(&mut self) {
        self.inner
            .get_class_object()
            .borrow_checker()
            .release_borrow_mut()
    }
}

impl<'py, T: PyClass<Frozen = False>> IntoPyObject<'py> for PyRefMut<'py, T> {
    type Target = T;
    type Output = Bound<'py, T>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = T::PYTHON_TYPE;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.inner.clone())
    }
}

impl<'a, 'py, T: PyClass<Frozen = False>> IntoPyObject<'py> for &'a PyRefMut<'py, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, T>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = T::PYTHON_TYPE;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.inner.as_borrowed())
    }
}

impl<T: PyClass<Frozen = False> + fmt::Debug> fmt::Debug for PyRefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

/// An error type returned by [`Bound::try_borrow`].
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

/// An error type returned by [`Bound::try_borrow_mut`].
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

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {

    use super::*;

    #[crate::pyclass]
    #[pyo3(crate = "crate")]
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    struct SomeClass(i32);

    #[test]
    fn test_as_ptr() {
        Python::attach(|py| {
            let cell = Bound::new(py, SomeClass(0)).unwrap();
            let ptr = cell.as_ptr();

            assert_eq!(cell.borrow().as_ptr(), ptr);
            assert_eq!(cell.borrow_mut().as_ptr(), ptr);
        })
    }

    #[test]
    fn test_into_ptr() {
        Python::attach(|py| {
            let cell = Bound::new(py, SomeClass(0)).unwrap();
            let ptr = cell.as_ptr();

            assert_eq!(cell.borrow().into_ptr(), ptr);
            unsafe { ffi::Py_DECREF(ptr) };

            assert_eq!(cell.borrow_mut().into_ptr(), ptr);
            unsafe { ffi::Py_DECREF(ptr) };
        })
    }

    #[crate::pyclass]
    #[pyo3(crate = "crate", subclass)]
    struct BaseClass {
        val1: usize,
    }

    #[crate::pyclass]
    #[pyo3(crate = "crate", extends=BaseClass, subclass)]
    struct SubClass {
        val2: usize,
    }

    #[crate::pyclass]
    #[pyo3(crate = "crate", extends=SubClass)]
    struct SubSubClass {
        val3: usize,
    }

    #[crate::pymethods]
    #[pyo3(crate = "crate")]
    impl SubSubClass {
        #[new]
        fn new(py: Python<'_>) -> crate::Py<SubSubClass> {
            let init = crate::PyClassInitializer::from(BaseClass { val1: 10 })
                .add_subclass(SubClass { val2: 15 })
                .add_subclass(SubSubClass { val3: 20 });
            crate::Py::new(py, init).expect("allocation error")
        }

        fn get_values(self_: PyRef<'_, Self>) -> (usize, usize, usize) {
            let val1 = self_.as_super().as_super().val1;
            let val2 = self_.as_super().val2;
            (val1, val2, self_.val3)
        }

        fn double_values(mut self_: PyRefMut<'_, Self>) {
            self_.as_super().as_super().val1 *= 2;
            self_.as_super().val2 *= 2;
            self_.val3 *= 2;
        }
    }

    #[test]
    fn test_pyref_as_super() {
        Python::attach(|py| {
            let obj = SubSubClass::new(py).into_bound(py);
            let pyref = obj.borrow();
            assert_eq!(pyref.as_super().as_super().val1, 10);
            assert_eq!(pyref.as_super().val2, 15);
            assert_eq!(pyref.as_ref().val2, 15); // `as_ref` also works
            assert_eq!(pyref.val3, 20);
            assert_eq!(SubSubClass::get_values(pyref), (10, 15, 20));
        });
    }

    #[test]
    fn test_pyrefmut_as_super() {
        Python::attach(|py| {
            let obj = SubSubClass::new(py).into_bound(py);
            assert_eq!(SubSubClass::get_values(obj.borrow()), (10, 15, 20));
            {
                let mut pyrefmut = obj.borrow_mut();
                assert_eq!(pyrefmut.as_super().as_ref().val1, 10);
                pyrefmut.as_super().as_super().val1 -= 5;
                pyrefmut.as_super().val2 -= 3;
                pyrefmut.as_mut().val2 -= 2; // `as_mut` also works
                pyrefmut.val3 -= 5;
            }
            assert_eq!(SubSubClass::get_values(obj.borrow()), (5, 10, 15));
            SubSubClass::double_values(obj.borrow_mut());
            assert_eq!(SubSubClass::get_values(obj.borrow()), (10, 20, 30));
        });
    }

    #[test]
    fn test_pyrefs_in_python() {
        Python::attach(|py| {
            let obj = SubSubClass::new(py);
            crate::py_run!(py, obj, "assert obj.get_values() == (10, 15, 20)");
            crate::py_run!(py, obj, "assert obj.double_values() is None");
            crate::py_run!(py, obj, "assert obj.get_values() == (20, 30, 40)");
        });
    }

    #[test]
    fn test_into_frozen_super_released_borrow() {
        #[crate::pyclass]
        #[pyo3(crate = "crate", subclass, frozen)]
        struct BaseClass {}

        #[crate::pyclass]
        #[pyo3(crate = "crate", extends=BaseClass, subclass)]
        struct SubClass {}

        #[crate::pymethods]
        #[pyo3(crate = "crate")]
        impl SubClass {
            #[new]
            fn new(py: Python<'_>) -> Bound<'_, SubClass> {
                let init = crate::PyClassInitializer::from(BaseClass {}).add_subclass(SubClass {});
                Bound::new(py, init).expect("allocation error")
            }
        }

        Python::attach(|py| {
            let obj = SubClass::new(py);
            drop(obj.borrow().into_super());
            assert!(obj.try_borrow_mut().is_ok());
        })
    }

    #[test]
    fn test_into_frozen_super_mutable_base_holds_borrow() {
        #[crate::pyclass]
        #[pyo3(crate = "crate", subclass)]
        struct BaseClass {}

        #[crate::pyclass]
        #[pyo3(crate = "crate", extends=BaseClass, subclass, frozen)]
        struct SubClass {}

        #[crate::pyclass]
        #[pyo3(crate = "crate", extends=SubClass, subclass)]
        struct SubSubClass {}

        #[crate::pymethods]
        #[pyo3(crate = "crate")]
        impl SubSubClass {
            #[new]
            fn new(py: Python<'_>) -> Bound<'_, SubSubClass> {
                let init = crate::PyClassInitializer::from(BaseClass {})
                    .add_subclass(SubClass {})
                    .add_subclass(SubSubClass {});
                Bound::new(py, init).expect("allocation error")
            }
        }

        Python::attach(|py| {
            let obj = SubSubClass::new(py);
            let _super_borrow = obj.borrow().into_super();
            // the whole object still has an immutable borrow, so we cannot
            // borrow any part mutably (the borrowflag is shared)
            assert!(obj.try_borrow_mut().is_err());
        })
    }
}
