use crate::conversion::FromPyObjectBound;
use crate::impl_::pycell::{PyClassObject, PyClassObjectLayout as _};
use crate::pycell::PyBorrowMutError;
use crate::pycell::{impl_::PyClassBorrowChecker, PyBorrowError};
use crate::pyclass::boolean_struct::False;
use crate::{ffi, Borrowed, IntoPyObject, Py, PyClass};
use std::convert::Infallible;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

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
/// # use pyo3::PyClassGuard;
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
///         // We can get &Self::BaseType by as_super
///         let basename = slf.as_super().basename;
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
    ptr: NonNull<ffi::PyObject>,
    marker: PhantomData<&'a Py<T>>,
}

impl<'a, T: PyClass> PyClassGuard<'a, T> {
    pub(crate) fn try_borrow(obj: &'a Py<T>) -> Result<Self, PyBorrowError> {
        Self::try_from_class_object(obj.get_class_object())
    }

    fn try_from_class_object(obj: &'a PyClassObject<T>) -> Result<Self, PyBorrowError> {
        obj.ensure_threadsafe();
        obj.borrow_checker().try_borrow().map(|_| Self {
            ptr: NonNull::from(obj).cast(),
            marker: PhantomData,
        })
    }

    pub(crate) fn as_class_object(&self) -> &'a PyClassObject<T> {
        // SAFETY: `ptr` by construction points to a `PyClassObject<T>` and is
        // valid for at least 'a
        unsafe { self.ptr.cast().as_ref() }
    }

    /// Consumes the [`PyClassGuard`] and returns a [`PyClassGuardMap`] for a component of the
    /// borrowed data
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::PyClassGuard;
    ///
    /// #[pyclass]
    /// pub struct MyClass {
    ///     msg: String,
    /// }
    ///
    /// # Python::attach(|py| {
    /// let obj = Bound::new(py, MyClass { msg: String::from("hello") })?;
    /// let msg = obj.extract::<PyClassGuard<'_, MyClass>>()?.map(|c| &c.msg);
    /// assert_eq!(&*msg, "hello");
    /// # Ok::<_, PyErr>(())
    /// # }).unwrap();
    /// ```
    pub fn map<F, U: ?Sized>(self, f: F) -> PyClassGuardMap<'a, U, false>
    where
        F: FnOnce(&T) -> &U,
    {
        let slf = std::mem::ManuallyDrop::new(self); // the borrow is released when dropping the `PyClassGuardMap`
        PyClassGuardMap {
            ptr: NonNull::from(f(&slf)),
            checker: slf.as_class_object().borrow_checker(),
        }
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
    /// # use pyo3::PyClassGuard;
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
        unsafe { NonNull::from(self).cast().as_ref() }
    }

    /// Gets a `PyClassGuard<T::BaseType>`.
    ///
    /// With the help of this method, you can get hold of instances of the
    /// super-superclass when needed.
    ///
    /// # Examples
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::PyClassGuard;
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
    ///         format!("{} {} {}", super_.as_super().name1, super_.name2, subname)
    ///     }
    /// }
    /// # Python::attach(|py| {
    /// #     let sub = Py::new(py, Sub::new()).unwrap();
    /// #     pyo3::py_run!(py, sub, "assert sub.name() == 'base1 base2 sub'")
    /// # });
    /// ```
    pub fn into_super(self) -> PyClassGuard<'a, U> {
        let t_not_frozen = !<T::Frozen as crate::pyclass::boolean_struct::private::Boolean>::VALUE;
        let u_frozen = <U::Frozen as crate::pyclass::boolean_struct::private::Boolean>::VALUE;
        if t_not_frozen && u_frozen {
            // If `T` is a mutable subclass of a frozen `U` base, then it is possible that we need
            // to release the borrow count now. (e.g. `U` may have a noop borrow checker so dropping
            // the `PyRef<U>` later would noop and leak the borrow we currently hold.)
            //
            // However it's nontrivial, if `U` is frozen but itself has a mutable base class `V`,
            // then the borrow checker of both `T` and `U` is the shared borrow checker of `V`.
            //
            // But it's really hard to prove that in the type system, the soundest thing we can do
            // is just add a borrow to `U` now and then release the borrow of `T`.

            self.as_super()
                .as_class_object()
                .borrow_checker()
                .try_borrow()
                .expect("this object is already borrowed");

            self.as_class_object().borrow_checker().release_borrow()
        };
        PyClassGuard {
            ptr: std::mem::ManuallyDrop::new(self).ptr,
            marker: PhantomData,
        }
    }
}

impl<T: PyClass> Deref for PyClassGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: `PyClassObject<T>` constains a valid `T`, by construction no
        // mutable alias is enforced
        unsafe { &*self.as_class_object().get_ptr().cast_const() }
    }
}

impl<'a, 'py, T: PyClass> FromPyObjectBound<'a, 'py> for PyClassGuard<'a, T> {
    fn from_py_object_bound(obj: Borrowed<'a, 'py, crate::PyAny>) -> crate::PyResult<Self> {
        Self::try_from_class_object(obj.cast()?.get_class_object()).map_err(Into::into)
    }
}

impl<'a, 'py, T: PyClass> IntoPyObject<'py> for PyClassGuard<'a, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, T>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: crate::Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'a, 'py, T: PyClass> IntoPyObject<'py> for &PyClassGuard<'a, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, T>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = T::PYTHON_TYPE;

    #[inline]
    fn into_pyobject(self, py: crate::Python<'py>) -> Result<Self::Output, Self::Error> {
        // SAFETY: `ptr` is guaranteed to be valid for 'a and points to an
        // object of type T
        unsafe { Ok(Borrowed::from_non_null(py, self.ptr).cast_unchecked()) }
    }
}

impl<T: PyClass> Drop for PyClassGuard<'_, T> {
    /// Releases the shared borrow
    fn drop(&mut self) {
        self.as_class_object().borrow_checker().release_borrow()
    }
}

// SAFETY: `PyClassGuard` only provides access to the inner `T` (and no other
// Python APIs) which does not require a Python thread state
#[cfg(feature = "nightly")]
unsafe impl<T: PyClass> crate::marker::Ungil for PyClassGuard<'_, T> {}
// SAFETY: we provide access to
// - `&T`, which requires `T: Sync` to be Send and `T: Sync` to be Sync
unsafe impl<T: PyClass + Sync> Send for PyClassGuard<'_, T> {}
unsafe impl<T: PyClass + Sync> Sync for PyClassGuard<'_, T> {}

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
/// # use pyo3::{PyClassGuard, PyClassGuardMut};
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
///     let mut guard: PyClassGuardMut<'_, Number> = n.extract(py)?;
///     let n_mutable: &mut Number = &mut *guard;
///
///     n_mutable.increment();
///
///     // To avoid panics we must dispose of the
///     // `PyClassGuardMut` before borrowing again.
///     drop(guard);
///
///     let n_immutable: &Number = &*n.extract::<PyClassGuard<'_, Number>>(py)?;
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
/// # use pyo3::{PyClassGuard, PyClassGuardMut};
/// # #[pyclass]
/// # pub struct Number {
/// #     inner: u32,
/// # }
/// #[pyfunction]
/// fn swap_numbers(a: &Bound<'_, Number>, b: &Bound<'_, Number>) -> PyResult<()> {
///     // Check that the pointers are unequal
///     if !a.is(b) {
///         let mut a: PyClassGuardMut<'_, Number> = a.extract()?;
///         let mut b: PyClassGuardMut<'_, Number> = b.extract()?;
///         std::mem::swap(&mut a.inner, &mut b.inner);
///     } else {
///         // Do nothing - they are the same object, so don't need swapping.
///     }
///     Ok(())
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
/// #         let n: u32 = n.extract::<PyClassGuard<'_, Number>>(py).unwrap().inner;
/// #         let n2: u32 = n2.extract::<PyClassGuard<'_, Number>>(py).unwrap().inner;
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
    ptr: NonNull<ffi::PyObject>,
    marker: PhantomData<&'a Py<T>>,
}

impl<'a, T: PyClass<Frozen = False>> PyClassGuardMut<'a, T> {
    pub(crate) fn try_borrow_mut(obj: &'a Py<T>) -> Result<Self, PyBorrowMutError> {
        Self::try_from_class_object(obj.get_class_object())
    }

    fn try_from_class_object(obj: &'a PyClassObject<T>) -> Result<Self, PyBorrowMutError> {
        obj.ensure_threadsafe();
        obj.borrow_checker().try_borrow_mut().map(|_| Self {
            ptr: NonNull::from(obj).cast(),
            marker: PhantomData,
        })
    }

    pub(crate) fn as_class_object(&self) -> &'a PyClassObject<T> {
        // SAFETY: `ptr` by construction points to a `PyClassObject<T>` and is
        // valid for at least 'a
        unsafe { self.ptr.cast().as_ref() }
    }

    /// Consumes the [`PyClassGuardMut`] and returns a [`PyClassGuardMap`] for a component of the
    /// borrowed data
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::PyClassGuardMut;
    ///
    /// #[pyclass]
    /// pub struct MyClass {
    ///     data: [i32; 100],
    /// }
    ///
    /// # Python::attach(|py| {
    /// let obj = Bound::new(py, MyClass { data: [0; 100] })?;
    /// let mut data = obj.extract::<PyClassGuardMut<'_, MyClass>>()?.map(|c| c.data.as_mut_slice());
    /// data[0] = 42;
    /// # Ok::<_, PyErr>(())
    /// # }).unwrap();
    /// ```
    pub fn map<F, U: ?Sized>(self, f: F) -> PyClassGuardMap<'a, U, true>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        let mut slf = std::mem::ManuallyDrop::new(self); // the borrow is released when dropping the `PyClassGuardMap`
        PyClassGuardMap {
            ptr: NonNull::from(f(&mut slf)),
            checker: slf.as_class_object().borrow_checker(),
        }
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
        unsafe { NonNull::from(self).cast().as_mut() }
    }

    /// Gets a `PyClassGuardMut<T::BaseType>`.
    ///
    /// See [`PyClassGuard::into_super`] for more.
    pub fn into_super(self) -> PyClassGuardMut<'a, U> {
        // `PyClassGuardMut` is only available for non-frozen classes, so there
        // is no possibility of leaking borrows like `PyClassGuard`
        PyClassGuardMut {
            ptr: std::mem::ManuallyDrop::new(self).ptr,
            marker: PhantomData,
        }
    }
}

impl<T: PyClass<Frozen = False>> Deref for PyClassGuardMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: `PyClassObject<T>` constains a valid `T`, by construction no
        // alias is enforced
        unsafe { &*self.as_class_object().get_ptr().cast_const() }
    }
}
impl<T: PyClass<Frozen = False>> DerefMut for PyClassGuardMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: `PyClassObject<T>` constains a valid `T`, by construction no
        // alias is enforced
        unsafe { &mut *self.as_class_object().get_ptr() }
    }
}

impl<'a, 'py, T: PyClass<Frozen = False>> FromPyObjectBound<'a, 'py> for PyClassGuardMut<'a, T> {
    fn from_py_object_bound(obj: Borrowed<'a, 'py, crate::PyAny>) -> crate::PyResult<Self> {
        Self::try_from_class_object(obj.cast()?.get_class_object()).map_err(Into::into)
    }
}

impl<'a, 'py, T: PyClass<Frozen = False>> IntoPyObject<'py> for PyClassGuardMut<'a, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, T>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: crate::Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

impl<'a, 'py, T: PyClass<Frozen = False>> IntoPyObject<'py> for &PyClassGuardMut<'a, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, T>;
    type Error = Infallible;

    #[inline]
    fn into_pyobject(self, py: crate::Python<'py>) -> Result<Self::Output, Self::Error> {
        // SAFETY: `ptr` is guaranteed to be valid for 'a and points to an
        // object of type T
        unsafe { Ok(Borrowed::from_non_null(py, self.ptr).cast_unchecked()) }
    }
}

impl<T: PyClass<Frozen = False>> Drop for PyClassGuardMut<'_, T> {
    /// Releases the mutable borrow
    fn drop(&mut self) {
        self.as_class_object().borrow_checker().release_borrow_mut()
    }
}

// SAFETY: `PyClassGuardMut` only provides access to the inner `T` (and no other
// Python APIs) which does not require a Python thread state
#[cfg(feature = "nightly")]
unsafe impl<T: PyClass<Frozen = False>> crate::marker::Ungil for PyClassGuardMut<'_, T> {}
// SAFETY: we provide access to
// - `&T`, which requires `T: Sync` to be Send and `T: Sync` to be Sync
// - `&mut T`, which requires `T: Send` to be Send and `T: Sync` to be Sync
unsafe impl<T: PyClass<Frozen = False> + Send + Sync> Send for PyClassGuardMut<'_, T> {}
unsafe impl<T: PyClass<Frozen = False> + Sync> Sync for PyClassGuardMut<'_, T> {}

/// Wraps a borrowed reference `U` to a value stored inside of a pyclass `T`
///
/// See [`PyClassGuard::map`] and [`PyClassGuardMut::map`]
pub struct PyClassGuardMap<'a, U: ?Sized, const MUT: bool> {
    ptr: NonNull<U>,
    checker: &'a dyn PyClassBorrowChecker,
}

impl<U: ?Sized, const MUT: bool> Deref for PyClassGuardMap<'_, U, MUT> {
    type Target = U;

    fn deref(&self) -> &U {
        // SAFETY: `checker` guards our access to the `T` that `U` points into
        unsafe { self.ptr.as_ref() }
    }
}

impl<U: ?Sized> DerefMut for PyClassGuardMap<'_, U, true> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `checker` guards our access to the `T` that `U` points into
        unsafe { self.ptr.as_mut() }
    }
}

impl<U: ?Sized, const MUT: bool> Drop for PyClassGuardMap<'_, U, MUT> {
    fn drop(&mut self) {
        if MUT {
            self.checker.release_borrow_mut();
        } else {
            self.checker.release_borrow();
        }
    }
}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use super::{PyClassGuard, PyClassGuardMut};
    use crate::{types::PyAnyMethods as _, Bound, IntoPyObject as _, Py, PyErr, Python};

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
            fn new(py: Python<'_>) -> Py<SubClass> {
                let init = crate::PyClassInitializer::from(BaseClass {}).add_subclass(SubClass {});
                Py::new(py, init).expect("allocation error")
            }
        }

        Python::attach(|py| {
            let obj = SubClass::new(py);
            drop(PyClassGuard::try_borrow(&obj).unwrap().into_super());
            assert!(PyClassGuardMut::try_borrow_mut(&obj).is_ok());
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
            fn new(py: Python<'_>) -> Py<SubSubClass> {
                let init = crate::PyClassInitializer::from(BaseClass {})
                    .add_subclass(SubClass {})
                    .add_subclass(SubSubClass {});
                Py::new(py, init).expect("allocation error")
            }
        }

        Python::attach(|py| {
            let obj = SubSubClass::new(py);
            let _super_borrow = PyClassGuard::try_borrow(&obj).unwrap().into_super();
            // the whole object still has an immutable borrow, so we cannot
            // borrow any part mutably (the borrowflag is shared)
            assert!(PyClassGuardMut::try_borrow_mut(&obj).is_err());
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
        #[pyo3(get)]
        val3: usize,
    }

    #[crate::pymethods]
    #[pyo3(crate = "crate")]
    impl SubSubClass {
        #[new]
        fn new(py: Python<'_>) -> Py<SubSubClass> {
            let init = crate::PyClassInitializer::from(BaseClass { val1: 10 })
                .add_subclass(SubClass { val2: 15 })
                .add_subclass(SubSubClass { val3: 20 });
            Py::new(py, init).expect("allocation error")
        }

        fn get_values(self_: PyClassGuard<'_, Self>) -> (usize, usize, usize) {
            let val1 = self_.as_super().as_super().val1;
            let val2 = self_.as_super().val2;
            (val1, val2, self_.val3)
        }

        fn double_values(mut self_: PyClassGuardMut<'_, Self>) {
            self_.as_super().as_super().val1 *= 2;
            self_.as_super().val2 *= 2;
            self_.val3 *= 2;
        }

        fn __add__<'a>(
            mut slf: PyClassGuardMut<'a, Self>,
            other: PyClassGuard<'a, Self>,
        ) -> PyClassGuardMut<'a, Self> {
            slf.val3 += other.val3;
            slf
        }

        fn __rsub__<'a>(
            slf: PyClassGuard<'a, Self>,
            mut other: PyClassGuardMut<'a, Self>,
        ) -> PyClassGuardMut<'a, Self> {
            other.val3 -= slf.val3;
            other
        }
    }

    #[test]
    fn test_pyclassguard_into_pyobject() {
        Python::attach(|py| {
            let class = Py::new(py, BaseClass { val1: 42 })?;
            let guard = PyClassGuard::try_borrow(&class).unwrap();
            let new_ref = (&guard).into_pyobject(py)?;
            assert!(new_ref.is(&class));
            let new = guard.into_pyobject(py)?;
            assert!(new.is(&class));
            Ok::<_, PyErr>(())
        })
        .unwrap();
    }

    #[test]
    fn test_pyclassguardmut_into_pyobject() {
        Python::attach(|py| {
            let class = Py::new(py, BaseClass { val1: 42 })?;
            let guard = PyClassGuardMut::try_borrow_mut(&class).unwrap();
            let new_ref = (&guard).into_pyobject(py)?;
            assert!(new_ref.is(&class));
            let new = guard.into_pyobject(py)?;
            assert!(new.is(&class));
            Ok::<_, PyErr>(())
        })
        .unwrap();
    }
    #[test]
    fn test_pyclassguard_as_super() {
        Python::attach(|py| {
            let obj = SubSubClass::new(py).into_bound(py);
            let pyref = PyClassGuard::try_borrow(obj.as_unbound()).unwrap();
            assert_eq!(pyref.as_super().as_super().val1, 10);
            assert_eq!(pyref.as_super().val2, 15);
            assert_eq!(pyref.val3, 20);
            assert_eq!(SubSubClass::get_values(pyref), (10, 15, 20));
        });
    }

    #[test]
    fn test_pyclassguardmut_as_super() {
        Python::attach(|py| {
            let obj = SubSubClass::new(py).into_bound(py);
            assert_eq!(
                SubSubClass::get_values(PyClassGuard::try_borrow(obj.as_unbound()).unwrap()),
                (10, 15, 20)
            );
            {
                let mut pyrefmut = PyClassGuardMut::try_borrow_mut(obj.as_unbound()).unwrap();
                assert_eq!(pyrefmut.as_super().as_super().val1, 10);
                pyrefmut.as_super().as_super().val1 -= 5;
                pyrefmut.as_super().val2 -= 5;
                pyrefmut.val3 -= 5;
            }
            assert_eq!(
                SubSubClass::get_values(PyClassGuard::try_borrow(obj.as_unbound()).unwrap()),
                (5, 10, 15)
            );
            SubSubClass::double_values(PyClassGuardMut::try_borrow_mut(obj.as_unbound()).unwrap());
            assert_eq!(
                SubSubClass::get_values(PyClassGuard::try_borrow(obj.as_unbound()).unwrap()),
                (10, 20, 30)
            );
        });
    }

    #[test]
    fn test_extract_guard() {
        Python::attach(|py| {
            let obj1 = SubSubClass::new(py);
            let obj2 = SubSubClass::new(py);
            crate::py_run!(py, obj1 obj2, "assert ((obj1 + obj2) - obj2).val3 == obj1.val3");
        });
    }

    #[test]
    fn test_pyclassguards_in_python() {
        Python::attach(|py| {
            let obj = SubSubClass::new(py);
            crate::py_run!(py, obj, "assert obj.get_values() == (10, 15, 20)");
            crate::py_run!(py, obj, "assert obj.double_values() is None");
            crate::py_run!(py, obj, "assert obj.get_values() == (20, 30, 40)");
        });
    }

    #[crate::pyclass]
    #[pyo3(crate = "crate")]
    pub struct MyClass {
        data: [i32; 100],
    }

    #[test]
    fn test_pyclassguard_map() {
        Python::attach(|py| {
            let obj = Bound::new(py, MyClass { data: [0; 100] })?;
            let data = PyClassGuard::try_borrow(obj.as_unbound())?.map(|c| &c.data);
            assert_eq!(data[0], 0);
            assert!(obj.try_borrow_mut().is_err()); // obj is still protected
            drop(data);
            assert!(obj.try_borrow_mut().is_ok()); // drop released shared borrow
            Ok::<_, PyErr>(())
        })
        .unwrap()
    }

    #[test]
    fn test_pyclassguardmut_map() {
        Python::attach(|py| {
            let obj = Bound::new(py, MyClass { data: [0; 100] })?;
            let mut data =
                PyClassGuardMut::try_borrow_mut(obj.as_unbound())?.map(|c| c.data.as_mut_slice());
            assert_eq!(data[0], 0);
            data[0] = 5;
            assert_eq!(data[0], 5);
            assert!(obj.try_borrow_mut().is_err()); // obj is still protected
            drop(data);
            assert!(obj.try_borrow_mut().is_ok()); // drop released mutable borrow
            Ok::<_, PyErr>(())
        })
        .unwrap()
    }

    #[test]
    fn test_pyclassguard_map_unrelated() {
        use crate::types::{PyString, PyStringMethods};
        Python::attach(|py| {
            let obj = Bound::new(py, MyClass { data: [0; 100] })?;
            let string = PyString::new(py, "pyo3");
            // It is possible to return something not borrowing from the guard, but that shouldn't
            // matter. `RefCell` has the same behaviour
            let refmap = PyClassGuard::try_borrow(obj.as_unbound())?.map(|_| &string);
            assert_eq!(refmap.to_cow()?, "pyo3");
            Ok::<_, PyErr>(())
        })
        .unwrap()
    }
}
