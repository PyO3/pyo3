use crate::impl_::pycell::PyClassObjectLayout as _;
use crate::internal_tricks::{ptr_from_mut, ptr_from_ref};
use crate::pycell::PyBorrowMutError;
use crate::pycell::{impl_::PyClassBorrowChecker, PyBorrowError};
use crate::pyclass::boolean_struct::False;
use crate::{Py, PyClass};
use std::ops::{Deref, DerefMut};

/// A wrapper type for an immutably borrowed value from a `PyClass`.
///
/// Rust has strict aliasing rules - you can either have any number of immutable
/// (shared) references or one mutable reference. Python's ownership model is
/// the complete opposite of that - any Python object can be referenced any
/// number of times, and mutation is allowed from any reference.
///
/// PyO3 deals with these differences by employing the [Interior Mutability]
/// pattern. This requires that PyO3 enforces the borrowing rules and it has two
/// mechanisms for doing so:
/// - Statically it can enforce thread-safe access with the
///   [`Python<'py>`](crate::Python) token. All Rust code holding that token, or
///   anything derived from it, can assume that they have safe access to the
///   Python interpreter's state. For this reason all the native Python objects
///   can be mutated through shared references.
/// - However, methods and functions in Rust usually *do* need `&mut`
///   references. While PyO3 can use the [`Python<'py>`](crate::Python) token to
///   guarantee thread-safe access to them, it cannot statically guarantee
///   uniqueness of `&mut` references. As such those references have to be
///   tracked dynamically at runtime, using [`PyClassGuard`] and
///   [`PyClassGuardMut`] defined in this module. This works similar to std's
///   [`RefCell`](std::cell::RefCell) type. Especially when building for
///   free-threaded Python it gets harder to track which thread borrows which
///   object at any time. This can lead to method calls failing with
///   [`PyBorrowError`]. In these cases consider using `frozen` classes together
///   with Rust interior mutability primitives like [`Mutex`](std::sync::Mutex)
///   instead of using [`PyClassGuardMut`] to get mutable access.
///
/// # Examples
///
/// You can use [`PyClassGuard`] as an alternative to a `&self` receiver when
/// - you need to access the pointer of the `PyClass`, or
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
///     fn format(slf: PyClassGuard<'_, Self>) -> String {
///         // We can get &Self::BaseType by as_ref
///         let basename = slf.as_ref().basename;
///         format!("{}(base: {})", slf.name, basename)
///     }
/// }
/// # Python::attach(|py| {
/// #     let sub = Py::new(py, Child::new()).unwrap();
/// #     pyo3::py_run!(py, sub, "assert sub.format() == 'Caterpillar(base: Butterfly)', sub.format()");
/// # });
/// ```
///
/// See also [`PyClassGuardMut`] and the [guide] for more information.
///
/// [Interior Mutability]:
///     https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
///     "RefCell<T> and the Interior Mutability Pattern - The Rust Programming
///     Language"
/// [guide]: https://pyo3.rs/latest/class.html#bound-and-interior-mutability
///     "Bound and interior mutability"
#[repr(transparent)]
pub struct PyClassGuard<'a, T: PyClass> {
    inner: &'a Py<T>,
}

impl<'a, T: PyClass> PyClassGuard<'a, T> {
    #[track_caller]
    pub(crate) fn borrow(obj: &'a Py<T>) -> Self {
        Self::try_borrow(obj).expect("Already mutably borrowed")
    }

    pub(crate) fn try_borrow(obj: &'a Py<T>) -> Result<Self, PyBorrowError> {
        let cell = obj.get_class_object();
        cell.ensure_threadsafe();
        cell.borrow_checker()
            .try_borrow()
            .map(|_| Self { inner: obj })
    }
}

impl<'a, T, U> PyClassGuard<'a, T>
where
    T: PyClass<BaseType = U>,
    U: PyClass,
{
    /// Borrows a shared reference to `PyClassGuard<T::BaseType>`.
    ///
    /// With the help of this method, you can access attributes and call methods
    /// on the superclass without consuming the `PyClassGuard<T>`. This method
    /// can also be chained to access the super-superclass (and so on).
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
    ///     fn format_name_lengths(slf: PyClassGuard<'_, Self>) -> String {
    ///         format!("{} {}", slf.as_super().base_name_len(), slf.sub_name_len())
    ///     }
    /// }
    /// # Python::attach(|py| {
    /// #     let sub = Py::new(py, Sub::new()).unwrap();
    /// #     pyo3::py_run!(py, sub, "assert sub.format_name_lengths() == '9 8'")
    /// # });
    /// ```
    pub fn as_super(&self) -> &PyClassGuard<'a, U> {
        // SAFETY: `PyClassGuard<T>` and `PyClassGuard<U>` have the same layout
        unsafe { &*ptr_from_ref(self).cast() }
    }

    /// Gets a `PyClassGuard<T::BaseType>`.
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
    ///     fn name(slf: PyClassGuard<'_, Self>) -> String {
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
    pub fn into_super(self) -> PyClassGuard<'a, U> {
        PyClassGuard {
            // SAFETY: `Py<T>` and `Py<U>` have the same layout
            inner: unsafe { &*ptr_from_ref(std::mem::ManuallyDrop::new(self).inner).cast() },
        }
    }
}

impl<T, U> AsRef<U> for PyClassGuard<'_, T>
where
    T: PyClass<BaseType = U>,
    U: PyClass,
{
    /// See [`PyClassGuard::as_super`]
    fn as_ref(&self) -> &T::BaseType {
        self.as_super()
    }
}

impl<T: PyClass> Deref for PyClassGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.inner.get_class_object().get_ptr() }
    }
}

impl<T: PyClass> Drop for PyClassGuard<'_, T> {
    /// Releases the shared borrow
    fn drop(&mut self) {
        self.inner
            .get_class_object()
            .borrow_checker()
            .release_borrow()
    }
}

/// A wrapper type for a mutably borrowed value from a `PyClass`
///
/// # When *not* to use [`PyClassGuardMut`]
///
/// Usually you can use `&mut` references as method and function receivers and
/// arguments, and you won't need to use [`PyClassGuardMut`] directly:
///
/// ```rust,no_run
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
/// ```
///
/// The [`#[pymethods]`](crate::pymethods) proc macro will generate this wrapper
/// function (and more), using [`PyClassGuardMut`] under the hood:
///
/// ```rust,no_run
/// # use pyo3::prelude::*;
/// # #[pyclass]
/// # struct Number {
/// #    inner: u32,
/// # }
/// #
/// # #[pymethods]
/// # impl Number {
/// #    fn increment(&mut self) {
/// #        self.inner += 1;
/// #    }
/// # }
/// #
/// // The function which is exported to Python looks roughly like the following
/// unsafe extern "C" fn __pymethod_increment__(
///     _slf: *mut ::pyo3::ffi::PyObject,
///     _args: *mut ::pyo3::ffi::PyObject,
/// ) -> *mut ::pyo3::ffi::PyObject {
///     unsafe fn inner<'py>(
///         py: ::pyo3::Python<'py>,
///         _slf: *mut ::pyo3::ffi::PyObject,
///     ) -> ::pyo3::PyResult<*mut ::pyo3::ffi::PyObject> {
///         let function = Number::increment;
/// #       #[allow(clippy::let_unit_value)]
///         let mut holder_0 = ::pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT;
///         let result = {
///             let ret = function(::pyo3::impl_::extract_argument::extract_pyclass_ref_mut::<Number>(
///                 unsafe { ::pyo3::impl_::pymethods::BoundRef::ref_from_ptr(py, &_slf) }.0,
///                 &mut holder_0,
///             )?);
///             {
///                 let result = {
///                     let obj = ret;
/// #                   #[allow(clippy::useless_conversion)]
///                     ::pyo3::impl_::wrap::converter(&obj)
///                         .wrap(obj)
///                         .map_err(::core::convert::Into::<::pyo3::PyErr>::into)
///                 };
///                 ::pyo3::impl_::wrap::converter(&result).map_into_ptr(py, result)
///             }
///         };
///         result
///     }
///
///     unsafe {
///         ::pyo3::impl_::trampoline::noargs(
///             _slf,
///             _args,
///             inner,
///         )
///     }
/// }
/// ```
///
/// # When to use [`PyClassGuardMut`]
/// ## Using PyClasses from Rust
///
/// However, we *do* need [`PyClassGuardMut`] if we want to call its methods
/// from Rust:
/// ```rust
/// # use pyo3::prelude::*;
/// #
/// # #[pyclass]
/// # struct Number {
/// #     inner: u32,
/// # }
/// #
/// # #[pymethods]
/// # impl Number {
/// #     fn increment(&mut self) {
/// #         self.inner += 1;
/// #     }
/// # }
/// # fn main() -> PyResult<()> {
/// Python::attach(|py| {
///     let n = Py::new(py, Number { inner: 0 })?;
///
///     // We borrow the guard and then dereference
///     // it to get a mutable reference to Number
///     let mut guard: PyClassGuardMut<'_, Number> = n.borrow_mut(py);
///     let n_mutable: &mut Number = &mut *guard;
///
///     n_mutable.increment();
///
///     // To avoid panics we must dispose of the
///     // `PyClassGuardMut` before borrowing again.
///     drop(guard);
///
///     let n_immutable: &Number = &n.borrow(py);
///     assert_eq!(n_immutable.inner, 1);
///
///     Ok(())
/// })
/// # }
/// ```
/// ## Dealing with possibly overlapping mutable references
///
/// It is also necessary to use [`PyClassGuardMut`] if you can receive mutable
/// arguments that may overlap. Suppose the following function that swaps the
/// values of two `Number`s:
/// ```
/// # use pyo3::prelude::*;
/// # #[pyclass]
/// # pub struct Number {
/// #     inner: u32,
/// # }
/// #[pyfunction]
/// fn swap_numbers(a: &mut Number, b: &mut Number) {
///     std::mem::swap(&mut a.inner, &mut b.inner);
/// }
/// # fn main() {
/// #     Python::attach(|py| {
/// #         let n = Py::new(py, Number{inner: 35}).unwrap();
/// #         let n2 = n.clone_ref(py);
/// #         assert!(n.is(&n2));
/// #         let fun = pyo3::wrap_pyfunction!(swap_numbers, py).unwrap();
/// #         fun.call1((n, n2)).expect_err("Managed to create overlapping mutable references. Note: this is undefined behaviour.");
/// #     });
/// # }
/// ```
/// When users pass in the same `Number` as both arguments, one of the mutable
/// borrows will fail and raise a `RuntimeError`:
/// ```text
/// >>> a = Number()
/// >>> swap_numbers(a, a)
/// Traceback (most recent call last):
///   File "<stdin>", line 1, in <module>
///   RuntimeError: Already borrowed
/// ```
///
/// It is better to write that function like this:
/// ```rust
/// # use pyo3::prelude::*;
/// # #[pyclass]
/// # pub struct Number {
/// #     inner: u32,
/// # }
/// #[pyfunction]
/// fn swap_numbers(a: &Bound<'_, Number>, b: &Bound<'_, Number>) {
///     // Check that the pointers are unequal
///     if !a.is(b) {
///         std::mem::swap(&mut a.borrow_mut().inner, &mut b.borrow_mut().inner);
///     } else {
///         // Do nothing - they are the same object, so don't need swapping.
///     }
/// }
/// # fn main() {
/// #     // With duplicate numbers
/// #     Python::attach(|py| {
/// #         let n = Py::new(py, Number{inner: 35}).unwrap();
/// #         let n2 = n.clone_ref(py);
/// #         assert!(n.is(&n2));
/// #         let fun = pyo3::wrap_pyfunction!(swap_numbers, py).unwrap();
/// #         fun.call1((n, n2)).unwrap();
/// #     });
/// #
/// #     // With two different numbers
/// #     Python::attach(|py| {
/// #         let n = Py::new(py, Number{inner: 35}).unwrap();
/// #         let n2 = Py::new(py, Number{inner: 42}).unwrap();
/// #         assert!(!n.is(&n2));
/// #         let fun = pyo3::wrap_pyfunction!(swap_numbers, py).unwrap();
/// #         fun.call1((&n, &n2)).unwrap();
/// #         let n: u32 = n.borrow(py).inner;
/// #         let n2: u32 = n2.borrow(py).inner;
/// #         assert_eq!(n, 42);
/// #         assert_eq!(n2, 35);
/// #     });
/// # }
/// ```
/// See [`PyClassGuard`] and the [guide] for more information.
///
/// [guide]: https://pyo3.rs/latest/class.html#bound-and-interior-mutability
///     "Bound and interior mutability"
#[repr(transparent)]
pub struct PyClassGuardMut<'a, T: PyClass<Frozen = False>> {
    inner: &'a Py<T>,
}

impl<'a, T: PyClass<Frozen = False>> PyClassGuardMut<'a, T> {
    #[track_caller]
    pub(crate) fn borrow_mut(obj: &'a Py<T>) -> Self {
        Self::try_borrow_mut(obj).expect("Already borrowed")
    }

    pub(crate) fn try_borrow_mut(obj: &'a Py<T>) -> Result<Self, PyBorrowMutError> {
        let cell = obj.get_class_object();
        cell.ensure_threadsafe();
        cell.borrow_checker()
            .try_borrow_mut()
            .map(|_| Self { inner: obj })
    }

    pub(crate) fn downgrade(slf: &Self) -> &PyClassGuard<'a, T> {
        // SAFETY: `PyClassGuardMut<T>` and `PyClassGuard<T>` have the same layout
        unsafe { &*ptr_from_ref(slf).cast() }
    }
}

impl<'a, T, U> PyClassGuardMut<'a, T>
where
    T: PyClass<BaseType = U, Frozen = False>,
    U: PyClass<Frozen = False>,
{
    /// Borrows a mutable reference to `PyClassGuardMut<T::BaseType>`.
    ///
    /// With the help of this method, you can mutate attributes and call
    /// mutating methods on the superclass without consuming the
    /// `PyClassGuardMut<T>`. This method can also be chained to access the
    /// super-superclass (and so on).
    ///
    /// See [`PyClassGuard::as_super`] for more.
    pub fn as_super(&mut self) -> &mut PyClassGuardMut<'a, U> {
        // SAFETY: `PyClassGuardMut<T>` and `PyClassGuardMut<U>` have the same layout
        unsafe { &mut *ptr_from_mut(self).cast() }
    }

    /// Gets a `PyClassGuardMut<T::BaseType>`.
    ///
    /// See [`PyClassGuard::into_super`] for more.
    pub fn into_super(self) -> PyClassGuardMut<'a, U> {
        PyClassGuardMut {
            // SAFETY: `Py<T>` and `Py<U>` have the same layout
            inner: unsafe { &*ptr_from_ref(std::mem::ManuallyDrop::new(self).inner).cast() },
        }
    }
}

impl<T, U> AsRef<U> for PyClassGuardMut<'_, T>
where
    T: PyClass<BaseType = U, Frozen = False>,
    U: PyClass<Frozen = False>,
{
    /// See [PyClassGuard::as_super]
    fn as_ref(&self) -> &T::BaseType {
        PyClassGuardMut::downgrade(self).as_super()
    }
}

impl<T, U> AsMut<U> for PyClassGuardMut<'_, T>
where
    T: PyClass<BaseType = U, Frozen = False>,
    U: PyClass<Frozen = False>,
{
    /// See [PyClassGuardMut::as_super]
    fn as_mut(&mut self) -> &mut T::BaseType {
        self.as_super()
    }
}

impl<T: PyClass<Frozen = False>> Deref for PyClassGuardMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.inner.get_class_object().get_ptr() }
    }
}
impl<T: PyClass<Frozen = False>> DerefMut for PyClassGuardMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.get_class_object().get_ptr() }
    }
}

impl<T: PyClass<Frozen = False>> Drop for PyClassGuardMut<'_, T> {
    /// Releases the mutable borrow
    fn drop(&mut self) {
        self.inner
            .get_class_object()
            .borrow_checker()
            .release_borrow_mut()
    }
}
