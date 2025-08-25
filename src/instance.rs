use crate::call::PyCallArgs;
use crate::conversion::IntoPyObject;
use crate::err::{self, PyErr, PyResult};
use crate::impl_::pycell::PyClassObject;
use crate::internal_tricks::ptr_from_ref;
use crate::pycell::{PyBorrowError, PyBorrowMutError};
use crate::pyclass::boolean_struct::{False, True};
use crate::types::{any::PyAnyMethods, string::PyStringMethods, typeobject::PyTypeMethods};
use crate::types::{DerefToPyAny, PyDict, PyString};
use crate::{
    ffi, DowncastError, DowncastIntoError, FromPyObject, PyAny, PyClass, PyClassInitializer, PyRef,
    PyRefMut, PyTypeInfo, Python,
};
use crate::{internal::state, PyTypeCheck};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr;
use std::ptr::NonNull;

/// Owned or borrowed gil-bound Python smart pointer
///
/// This is implemented for [`Bound`] and [`Borrowed`].
pub trait BoundObject<'py, T>: bound_object_sealed::Sealed {
    /// Type erased version of `Self`
    type Any: BoundObject<'py, PyAny>;
    /// Borrow this smart pointer.
    fn as_borrowed(&self) -> Borrowed<'_, 'py, T>;
    /// Turns this smart pointer into an owned [`Bound<'py, T>`]
    fn into_bound(self) -> Bound<'py, T>;
    /// Upcast the target type of this smart pointer
    fn into_any(self) -> Self::Any;
    /// Turn this smart pointer into a strong reference pointer
    fn into_ptr(self) -> *mut ffi::PyObject;
    /// Turn this smart pointer into a borrowed reference pointer
    fn as_ptr(&self) -> *mut ffi::PyObject;
    /// Turn this smart pointer into an owned [`Py<T>`]
    fn unbind(self) -> Py<T>;
}

mod bound_object_sealed {
    /// # Safety
    ///
    /// Type must be layout-compatible with `*mut ffi::PyObject`.
    pub unsafe trait Sealed {}

    // SAFETY: `Bound` is layout-compatible with `*mut ffi::PyObject`.
    unsafe impl<T> Sealed for super::Bound<'_, T> {}
    // SAFETY: `Borrowed` is layout-compatible with `*mut ffi::PyObject`.
    unsafe impl<T> Sealed for super::Borrowed<'_, '_, T> {}
}

/// A GIL-attached equivalent to [`Py<T>`].
///
/// This type can be thought of as equivalent to the tuple `(Py<T>, Python<'py>)`. By having the `'py`
/// lifetime of the [`Python<'py>`] token, this ties the lifetime of the [`Bound<'py, T>`] smart pointer
/// to the lifetime of the GIL and allows PyO3 to call Python APIs at maximum efficiency.
///
/// To access the object in situations where the GIL is not held, convert it to [`Py<T>`]
/// using [`.unbind()`][Bound::unbind]. This includes situations where the GIL is temporarily
/// released, such as [`Python::detach`](crate::Python::detach)'s closure.
///
/// See
#[doc = concat!("[the guide](https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/types.html#boundpy-t)")]
/// for more detail.
#[repr(transparent)]
pub struct Bound<'py, T>(Python<'py>, ManuallyDrop<Py<T>>);

impl<'py, T> Bound<'py, T>
where
    T: PyClass,
{
    /// Creates a new instance `Bound<T>` of a `#[pyclass]` on the Python heap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass]
    /// struct Foo {/* fields omitted */}
    ///
    /// # fn main() -> PyResult<()> {
    /// let foo: Py<Foo> = Python::attach(|py| -> PyResult<_> {
    ///     let foo: Bound<'_, Foo> = Bound::new(py, Foo {})?;
    ///     Ok(foo.into())
    /// })?;
    /// # Python::attach(move |_py| drop(foo));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        py: Python<'py>,
        value: impl Into<PyClassInitializer<T>>,
    ) -> PyResult<Bound<'py, T>> {
        value.into().create_class_object(py)
    }
}

impl<'py, T> Bound<'py, T> {
    /// Cast this to a concrete Python type or pyclass.
    ///
    /// Note that you can often avoid casting yourself by just specifying the desired type in
    /// function or method signatures. However, manual casting is sometimes necessary.
    ///
    /// For extracting a Rust-only type, see [`extract`](PyAnyMethods::extract).
    ///
    /// This performs a runtime type check using the equivalent of Python's
    /// `isinstance(self, U)`.
    ///
    /// # Example: Casting to a specific Python object
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyDict, PyList};
    ///
    /// Python::attach(|py| {
    ///     let dict = PyDict::new(py);
    ///     assert!(dict.is_instance_of::<PyAny>());
    ///     let any = dict.as_any();
    ///
    ///     assert!(any.cast::<PyDict>().is_ok());
    ///     assert!(any.cast::<PyList>().is_err());
    /// });
    /// ```
    ///
    /// # Example: Getting a reference to a pyclass
    ///
    /// This is useful if you want to mutate a `Py<PyAny>` that might actually be a pyclass.
    ///
    /// ```rust
    /// # fn main() -> Result<(), pyo3::PyErr> {
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass]
    /// struct Class {
    ///     i: i32,
    /// }
    ///
    /// Python::attach(|py| {
    ///     let class = Bound::new(py, Class { i: 0 })?.into_any();
    ///
    ///     let class_bound: &Bound<'_, Class> = class.cast()?;
    ///
    ///     class_bound.borrow_mut().i += 1;
    ///
    ///     // Alternatively you can get a `PyRefMut` directly
    ///     let class_ref: PyRefMut<'_, Class> = class.extract()?;
    ///     assert_eq!(class_ref.i, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    #[inline]
    pub fn cast<U>(&self) -> Result<&Bound<'py, U>, DowncastError<'_, 'py>>
    where
        U: PyTypeCheck,
    {
        #[inline]
        fn inner<'a, 'py, U>(
            any: &'a Bound<'py, PyAny>,
        ) -> Result<&'a Bound<'py, U>, DowncastError<'a, 'py>>
        where
            U: PyTypeCheck,
        {
            if U::type_check(any) {
                // Safety: type_check is responsible for ensuring that the type is correct
                Ok(unsafe { any.cast_unchecked() })
            } else {
                Err(DowncastError::new(any, U::NAME))
            }
        }

        inner(self.as_any())
    }

    /// Like [`cast`](Self::cast) but takes ownership of `self`.
    ///
    /// In case of an error, it is possible to retrieve `self` again via
    /// [`DowncastIntoError::into_inner`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyDict, PyList};
    ///
    /// Python::attach(|py| {
    ///     let obj: Bound<'_, PyAny> = PyDict::new(py).into_any();
    ///
    ///     let obj: Bound<'_, PyAny> = match obj.cast_into::<PyList>() {
    ///         Ok(_) => panic!("obj should not be a list"),
    ///         Err(err) => err.into_inner(),
    ///     };
    ///
    ///     // obj is a dictionary
    ///     assert!(obj.cast_into::<PyDict>().is_ok());
    /// })
    /// ```
    #[inline]
    pub fn cast_into<U>(self) -> Result<Bound<'py, U>, DowncastIntoError<'py>>
    where
        U: PyTypeCheck,
    {
        #[inline]
        fn inner<U>(any: Bound<'_, PyAny>) -> Result<Bound<'_, U>, DowncastIntoError<'_>>
        where
            U: PyTypeCheck,
        {
            if U::type_check(&any) {
                // Safety: type_check is responsible for ensuring that the type is correct
                Ok(unsafe { any.cast_into_unchecked() })
            } else {
                Err(DowncastIntoError::new(any, U::NAME))
            }
        }

        inner(self.into_any())
    }

    /// Cast this to a concrete Python type or pyclass (but not a subclass of it).
    ///
    /// It is almost always better to use [`cast`](Self::cast) because it accounts for Python
    /// subtyping. Use this method only when you do not want to allow subtypes.
    ///
    /// The advantage of this method over [`cast`](Self::cast) is that it is faster. The
    /// implementation of `cast_exact` uses the equivalent of the Python expression `type(self) is
    /// U`, whereas `cast` uses `isinstance(self, U)`.
    ///
    /// For extracting a Rust-only type, see [`extract`](PyAnyMethods::extract).
    ///
    /// # Example: Casting to a specific Python object but not a subtype
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyBool, PyInt};
    ///
    /// Python::attach(|py| {
    ///     let b = PyBool::new(py, true);
    ///     assert!(b.is_instance_of::<PyBool>());
    ///     let any: &Bound<'_, PyAny> = b.as_any();
    ///
    ///     // `bool` is a subtype of `int`, so `cast` will accept a `bool` as an `int`
    ///     // but `cast_exact` will not.
    ///     assert!(any.cast::<PyInt>().is_ok());
    ///     assert!(any.cast_exact::<PyInt>().is_err());
    ///
    ///     assert!(any.cast_exact::<PyBool>().is_ok());
    /// });
    /// ```
    #[inline]
    pub fn cast_exact<U>(&self) -> Result<&Bound<'py, U>, DowncastError<'_, 'py>>
    where
        U: PyTypeInfo,
    {
        #[inline]
        fn inner<'a, 'py, U>(
            any: &'a Bound<'py, PyAny>,
        ) -> Result<&'a Bound<'py, U>, DowncastError<'a, 'py>>
        where
            U: PyTypeInfo,
        {
            if any.is_exact_instance_of::<U>() {
                // Safety: is_exact_instance_of is responsible for ensuring that the type is correct
                Ok(unsafe { any.cast_unchecked() })
            } else {
                Err(DowncastError::new(any, U::NAME))
            }
        }

        inner(self.as_any())
    }

    /// Like [`cast_exact`](Self::cast_exact) but takes ownership of `self`.
    #[inline]
    pub fn cast_into_exact<U>(self) -> Result<Bound<'py, U>, DowncastIntoError<'py>>
    where
        U: PyTypeInfo,
    {
        #[inline]
        fn inner<U>(any: Bound<'_, PyAny>) -> Result<Bound<'_, U>, DowncastIntoError<'_>>
        where
            U: PyTypeInfo,
        {
            if any.is_exact_instance_of::<U>() {
                // Safety: is_exact_instance_of is responsible for ensuring that the type is correct
                Ok(unsafe { any.cast_into_unchecked() })
            } else {
                Err(DowncastIntoError::new(any, U::NAME))
            }
        }

        inner(self.into_any())
    }

    /// Converts this to a concrete Python type without checking validity.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    #[inline]
    pub unsafe fn cast_unchecked<U>(&self) -> &Bound<'py, U> {
        unsafe { NonNull::from(self).cast().as_ref() }
    }

    /// Like [`cast_unchecked`](Self::cast_unchecked) but takes ownership of `self`.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    #[inline]
    pub unsafe fn cast_into_unchecked<U>(self) -> Bound<'py, U> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<'py> Bound<'py, PyAny> {
    /// Constructs a new `Bound<'py, PyAny>` from a pointer. Panics if `ptr` is null.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a Python object
    /// - `ptr` must be an owned Python reference, as the `Bound<'py, PyAny>` will assume ownership
    #[inline]
    #[track_caller]
    pub unsafe fn from_owned_ptr(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        Self(
            py,
            ManuallyDrop::new(unsafe { Py::from_owned_ptr(py, ptr) }),
        )
    }

    /// Constructs a new `Bound<'py, PyAny>` from a pointer. Returns `None` if `ptr` is null.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a Python object, or null
    /// - `ptr` must be an owned Python reference, as the `Bound<'py, PyAny>` will assume ownership
    #[inline]
    pub unsafe fn from_owned_ptr_or_opt(py: Python<'py>, ptr: *mut ffi::PyObject) -> Option<Self> {
        unsafe { Py::from_owned_ptr_or_opt(py, ptr) }.map(|obj| Self(py, ManuallyDrop::new(obj)))
    }

    /// Constructs a new `Bound<'py, PyAny>` from a pointer. Returns an `Err` by calling `PyErr::fetch`
    /// if `ptr` is null.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a Python object, or null
    /// - `ptr` must be an owned Python reference, as the `Bound<'py, PyAny>` will assume ownership
    #[inline]
    pub unsafe fn from_owned_ptr_or_err(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        unsafe { Py::from_owned_ptr_or_err(py, ptr) }.map(|obj| Self(py, ManuallyDrop::new(obj)))
    }

    /// Constructs a new `Bound<'py, PyAny>` from a pointer without checking for null.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a Python object
    /// - `ptr` must be a strong/owned reference
    pub(crate) unsafe fn from_owned_ptr_unchecked(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Self {
        Self(
            py,
            ManuallyDrop::new(unsafe { Py::from_owned_ptr_unchecked(ptr) }),
        )
    }

    /// Constructs a new `Bound<'py, PyAny>` from a pointer by creating a new Python reference.
    /// Panics if `ptr` is null.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a Python object
    #[inline]
    #[track_caller]
    pub unsafe fn from_borrowed_ptr(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        unsafe { Self(py, ManuallyDrop::new(Py::from_borrowed_ptr(py, ptr))) }
    }

    /// Constructs a new `Bound<'py, PyAny>` from a pointer by creating a new Python reference.
    /// Returns `None` if `ptr` is null.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a Python object, or null
    #[inline]
    pub unsafe fn from_borrowed_ptr_or_opt(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Option<Self> {
        unsafe { Py::from_borrowed_ptr_or_opt(py, ptr).map(|obj| Self(py, ManuallyDrop::new(obj))) }
    }

    /// Constructs a new `Bound<'py, PyAny>` from a pointer by creating a new Python reference.
    /// Returns an `Err` by calling `PyErr::fetch` if `ptr` is null.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a Python object, or null
    #[inline]
    pub unsafe fn from_borrowed_ptr_or_err(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        unsafe { Py::from_borrowed_ptr_or_err(py, ptr).map(|obj| Self(py, ManuallyDrop::new(obj))) }
    }

    /// This slightly strange method is used to obtain `&Bound<PyAny>` from a pointer in macro code
    /// where we need to constrain the lifetime `'a` safely.
    ///
    /// Note that `'py` is required to outlive `'a` implicitly by the nature of the fact that
    /// `&'a Bound<'py>` means that `Bound<'py>` exists for at least the lifetime `'a`.
    ///
    /// # Safety
    /// - `ptr` must be a valid pointer to a Python object for the lifetime `'a`. The `ptr` can
    ///   be either a borrowed reference or an owned reference, it does not matter, as this is
    ///   just `&Bound` there will never be any ownership transfer.
    #[inline]
    pub(crate) unsafe fn ref_from_ptr<'a>(
        _py: Python<'py>,
        ptr: &'a *mut ffi::PyObject,
    ) -> &'a Self {
        unsafe { &*ptr_from_ref(ptr).cast::<Bound<'py, PyAny>>() }
    }

    /// Variant of the above which returns `None` for null pointers.
    ///
    /// # Safety
    /// - `ptr` must be a valid pointer to a Python object for the lifetime `'a, or null.
    #[inline]
    pub(crate) unsafe fn ref_from_ptr_or_opt<'a>(
        _py: Python<'py>,
        ptr: &'a *mut ffi::PyObject,
    ) -> &'a Option<Self> {
        unsafe { &*ptr_from_ref(ptr).cast::<Option<Bound<'py, PyAny>>>() }
    }

    /// This slightly strange method is used to obtain `&Bound<PyAny>` from a [`NonNull`] in macro
    /// code where we need to constrain the lifetime `'a` safely.
    ///
    /// Note that `'py` is required to outlive `'a` implicitly by the nature of the fact that `&'a
    /// Bound<'py>` means that `Bound<'py>` exists for at least the lifetime `'a`.
    ///
    /// # Safety
    /// - `ptr` must be a valid pointer to a Python object for the lifetime `'a`. The `ptr` can be
    ///   either a borrowed reference or an owned reference, it does not matter, as this is just
    ///   `&Bound` there will never be any ownership transfer.
    pub(crate) unsafe fn ref_from_non_null<'a>(
        _py: Python<'py>,
        ptr: &'a NonNull<ffi::PyObject>,
    ) -> &'a Self {
        unsafe { NonNull::from(ptr).cast().as_ref() }
    }
}

impl<'py, T> Bound<'py, T>
where
    T: PyClass,
{
    /// Immutably borrows the value `T`.
    ///
    /// This borrow lasts while the returned [`PyRef`] exists.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// For frozen classes, the simpler [`get`][Self::get] is available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use pyo3::prelude::*;
    /// #
    /// #[pyclass]
    /// struct Foo {
    ///     inner: u8,
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| -> PyResult<()> {
    ///     let foo: Bound<'_, Foo> = Bound::new(py, Foo { inner: 73 })?;
    ///     let inner: &u8 = &foo.borrow().inner;
    ///
    ///     assert_eq!(*inner, 73);
    ///     Ok(())
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow`](#method.try_borrow).
    #[inline]
    #[track_caller]
    pub fn borrow(&self) -> PyRef<'py, T> {
        PyRef::borrow(self)
    }

    /// Mutably borrows the value `T`.
    ///
    /// This borrow lasts while the returned [`PyRefMut`] exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// #
    /// #[pyclass]
    /// struct Foo {
    ///     inner: u8,
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| -> PyResult<()> {
    ///     let foo: Bound<'_, Foo> = Bound::new(py, Foo { inner: 73 })?;
    ///     foo.borrow_mut().inner = 35;
    ///
    ///     assert_eq!(foo.borrow().inner, 35);
    ///     Ok(())
    /// })?;
    /// # Ok(())
    /// # }
    ///  ```
    ///
    /// # Panics
    /// Panics if the value is currently borrowed. For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    #[inline]
    #[track_caller]
    pub fn borrow_mut(&self) -> PyRefMut<'py, T>
    where
        T: PyClass<Frozen = False>,
    {
        PyRefMut::borrow(self)
    }

    /// Attempts to immutably borrow the value `T`, returning an error if the value is currently mutably borrowed.
    ///
    /// The borrow lasts while the returned [`PyRef`] exists.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// For frozen classes, the simpler [`get`][Self::get] is available.
    #[inline]
    pub fn try_borrow(&self) -> Result<PyRef<'py, T>, PyBorrowError> {
        PyRef::try_borrow(self)
    }

    /// Attempts to mutably borrow the value `T`, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts while the returned [`PyRefMut`] exists.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    #[inline]
    pub fn try_borrow_mut(&self) -> Result<PyRefMut<'py, T>, PyBorrowMutError>
    where
        T: PyClass<Frozen = False>,
    {
        PyRefMut::try_borrow(self)
    }

    /// Provide an immutable borrow of the value `T` without acquiring the GIL.
    ///
    /// This is available if the class is [`frozen`][macro@crate::pyclass] and [`Sync`].
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
    /// Python::attach(|py| {
    ///     let counter = FrozenCounter { value: AtomicUsize::new(0) };
    ///
    ///     let py_counter = Bound::new(py, counter).unwrap();
    ///
    ///     py_counter.get().value.fetch_add(1, Ordering::Relaxed);
    /// });
    /// ```
    #[inline]
    pub fn get(&self) -> &T
    where
        T: PyClass<Frozen = True> + Sync,
    {
        self.1.get()
    }

    /// Upcast this `Bound<PyClass>` to its base type by reference.
    ///
    /// If this type defined an explicit base class in its `pyclass` declaration
    /// (e.g. `#[pyclass(extends = BaseType)]`), the returned type will be
    /// `&Bound<BaseType>`. If an explicit base class was _not_ declared, the
    /// return value will be `&Bound<PyAny>` (making this method equivalent
    /// to [`as_any`]).
    ///
    /// This method is particularly useful for calling methods defined in an
    /// extension trait that has been implemented for `Bound<BaseType>`.
    ///
    /// See also the [`into_super`] method to upcast by value, and the
    /// [`PyRef::as_super`]/[`PyRefMut::as_super`] methods for upcasting a pyclass
    /// that has already been [`borrow`]ed.
    ///
    /// # Example: Calling a method defined on the `Bound` base type
    ///
    /// ```rust
    /// # fn main() {
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass(subclass)]
    /// struct BaseClass;
    ///
    /// trait MyClassMethods<'py> {
    ///     fn pyrepr(&self) -> PyResult<String>;
    /// }
    /// impl<'py> MyClassMethods<'py> for Bound<'py, BaseClass> {
    ///     fn pyrepr(&self) -> PyResult<String> {
    ///         self.call_method0("__repr__")?.extract()
    ///     }
    /// }
    ///
    /// #[pyclass(extends = BaseClass)]
    /// struct SubClass;
    ///
    /// Python::attach(|py| {
    ///     let obj = Bound::new(py, (SubClass, BaseClass)).unwrap();
    ///     assert!(obj.as_super().pyrepr().is_ok());
    /// })
    /// # }
    /// ```
    ///
    /// [`as_any`]: Bound::as_any
    /// [`into_super`]: Bound::into_super
    /// [`borrow`]: Bound::borrow
    #[inline]
    pub fn as_super(&self) -> &Bound<'py, T::BaseType> {
        // a pyclass can always be safely "cast" to its base type
        unsafe { self.as_any().cast_unchecked() }
    }

    /// Upcast this `Bound<PyClass>` to its base type by value.
    ///
    /// If this type defined an explicit base class in its `pyclass` declaration
    /// (e.g. `#[pyclass(extends = BaseType)]`), the returned type will be
    /// `Bound<BaseType>`. If an explicit base class was _not_ declared, the
    /// return value will be `Bound<PyAny>` (making this method equivalent
    /// to [`into_any`]).
    ///
    /// This method is particularly useful for calling methods defined in an
    /// extension trait that has been implemented for `Bound<BaseType>`.
    ///
    /// See also the [`as_super`] method to upcast by reference, and the
    /// [`PyRef::into_super`]/[`PyRefMut::into_super`] methods for upcasting a pyclass
    /// that has already been [`borrow`]ed.
    ///
    /// # Example: Calling a method defined on the `Bound` base type
    ///
    /// ```rust
    /// # fn main() {
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass(subclass)]
    /// struct BaseClass;
    ///
    /// trait MyClassMethods<'py> {
    ///     fn pyrepr(self) -> PyResult<String>;
    /// }
    /// impl<'py> MyClassMethods<'py> for Bound<'py, BaseClass> {
    ///     fn pyrepr(self) -> PyResult<String> {
    ///         self.call_method0("__repr__")?.extract()
    ///     }
    /// }
    ///
    /// #[pyclass(extends = BaseClass)]
    /// struct SubClass;
    ///
    /// Python::attach(|py| {
    ///     let obj = Bound::new(py, (SubClass, BaseClass)).unwrap();
    ///     assert!(obj.into_super().pyrepr().is_ok());
    /// })
    /// # }
    /// ```
    ///
    /// [`into_any`]: Bound::into_any
    /// [`as_super`]: Bound::as_super
    /// [`borrow`]: Bound::borrow
    #[inline]
    pub fn into_super(self) -> Bound<'py, T::BaseType> {
        // a pyclass can always be safely "cast" to its base type
        unsafe { self.cast_into_unchecked() }
    }

    #[inline]
    pub(crate) fn get_class_object(&self) -> &PyClassObject<T> {
        self.1.get_class_object()
    }
}

impl<T> std::fmt::Debug for Bound<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let any = self.as_any();
        python_format(any, any.repr(), f)
    }
}

impl<T> std::fmt::Display for Bound<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let any = self.as_any();
        python_format(any, any.str(), f)
    }
}

fn python_format(
    any: &Bound<'_, PyAny>,
    format_result: PyResult<Bound<'_, PyString>>,
    f: &mut std::fmt::Formatter<'_>,
) -> Result<(), std::fmt::Error> {
    match format_result {
        Result::Ok(s) => return f.write_str(&s.to_string_lossy()),
        Result::Err(err) => err.write_unraisable(any.py(), Some(any)),
    }

    match any.get_type().name() {
        Result::Ok(name) => std::write!(f, "<unprintable {name} object>"),
        Result::Err(_err) => f.write_str("<unprintable object>"),
    }
}

// The trait bound is needed to avoid running into the auto-deref recursion
// limit (error[E0055]), because `Bound<PyAny>` would deref into itself. See:
// https://github.com/rust-lang/rust/issues/19509
impl<'py, T> Deref for Bound<'py, T>
where
    T: DerefToPyAny,
{
    type Target = Bound<'py, PyAny>;

    #[inline]
    fn deref(&self) -> &Bound<'py, PyAny> {
        self.as_any()
    }
}

impl<'py, T> AsRef<Bound<'py, PyAny>> for Bound<'py, T> {
    #[inline]
    fn as_ref(&self) -> &Bound<'py, PyAny> {
        self.as_any()
    }
}

impl<T> AsRef<Py<PyAny>> for Bound<'_, T> {
    #[inline]
    fn as_ref(&self) -> &Py<PyAny> {
        self.as_any().as_unbound()
    }
}

impl<T> Clone for Bound<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0, ManuallyDrop::new(self.1.clone_ref(self.0)))
    }
}

impl<T> Drop for Bound<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::Py_DECREF(self.as_ptr()) }
    }
}

impl<'py, T> Bound<'py, T> {
    /// Returns the GIL token associated with this object.
    #[inline]
    pub fn py(&self) -> Python<'py> {
        self.0
    }

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
        self.1.as_ptr()
    }

    /// Returns an owned raw FFI pointer represented by self.
    ///
    /// # Safety
    ///
    /// The reference is owned; when finished the caller should either transfer ownership
    /// of the pointer or decrease the reference count (e.g. with [`pyo3::ffi::Py_DecRef`](crate::ffi::Py_DecRef)).
    #[inline]
    pub fn into_ptr(self) -> *mut ffi::PyObject {
        ManuallyDrop::new(self).as_ptr()
    }

    /// Helper to cast to `Bound<'py, PyAny>`.
    #[inline]
    pub fn as_any(&self) -> &Bound<'py, PyAny> {
        // Safety: all Bound<T> have the same memory layout, and all Bound<T> are valid
        // Bound<PyAny>, so pointer casting is valid.
        unsafe { &*ptr_from_ref(self).cast::<Bound<'py, PyAny>>() }
    }

    /// Helper to cast to `Bound<'py, PyAny>`, transferring ownership.
    #[inline]
    pub fn into_any(self) -> Bound<'py, PyAny> {
        // Safety: all Bound<T> are valid Bound<PyAny>
        Bound(self.0, ManuallyDrop::new(self.unbind().into_any()))
    }

    /// Casts this `Bound<T>` to a `Borrowed<T>` smart pointer.
    #[inline]
    pub fn as_borrowed<'a>(&'a self) -> Borrowed<'a, 'py, T> {
        Borrowed(
            unsafe { NonNull::new_unchecked(self.as_ptr()) },
            PhantomData,
            self.py(),
        )
    }

    /// Removes the connection for this `Bound<T>` from the GIL, allowing
    /// it to cross thread boundaries.
    #[inline]
    pub fn unbind(self) -> Py<T> {
        // Safety: the type T is known to be correct and the ownership of the
        // pointer is transferred to the new Py<T> instance.
        let non_null = (ManuallyDrop::new(self).1).0;
        unsafe { Py::from_non_null(non_null) }
    }

    /// Removes the connection for this `Bound<T>` from the GIL, allowing
    /// it to cross thread boundaries, without transferring ownership.
    #[inline]
    pub fn as_unbound(&self) -> &Py<T> {
        &self.1
    }
}

impl<'py, T> BoundObject<'py, T> for Bound<'py, T> {
    type Any = Bound<'py, PyAny>;

    fn as_borrowed(&self) -> Borrowed<'_, 'py, T> {
        Bound::as_borrowed(self)
    }

    fn into_bound(self) -> Bound<'py, T> {
        self
    }

    fn into_any(self) -> Self::Any {
        self.into_any()
    }

    fn into_ptr(self) -> *mut ffi::PyObject {
        self.into_ptr()
    }

    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.as_ptr()
    }

    fn unbind(self) -> Py<T> {
        self.unbind()
    }
}

/// A borrowed equivalent to `Bound`.
///
/// The advantage of this over `&Bound` is that it avoids the need to have a pointer-to-pointer, as Bound
/// is already a pointer to an `ffi::PyObject``.
///
/// Similarly, this type is `Copy` and `Clone`, like a shared reference (`&T`).
#[repr(transparent)]
pub struct Borrowed<'a, 'py, T>(NonNull<ffi::PyObject>, PhantomData<&'a Py<T>>, Python<'py>);

impl<'a, 'py, T> Borrowed<'a, 'py, T> {
    /// Creates a new owned [`Bound<T>`] from this borrowed reference by
    /// increasing the reference count.
    ///
    /// # Example
    /// ```
    /// use pyo3::{prelude::*, types::PyTuple};
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| -> PyResult<()> {
    ///     let tuple = PyTuple::new(py, [1, 2, 3])?;
    ///
    ///     // borrows from `tuple`, so can only be
    ///     // used while `tuple` stays alive
    ///     let borrowed = tuple.get_borrowed_item(0)?;
    ///
    ///     // creates a new owned reference, which
    ///     // can be used indendently of `tuple`
    ///     let bound = borrowed.to_owned();
    ///     drop(tuple);
    ///
    ///     assert_eq!(bound.extract::<i32>().unwrap(), 1);
    ///     Ok(())
    /// })
    /// # }
    pub fn to_owned(self) -> Bound<'py, T> {
        (*self).clone()
    }

    /// Returns the raw FFI pointer represented by self.
    ///
    /// # Safety
    ///
    /// Callers are responsible for ensuring that the pointer does not outlive self.
    ///
    /// The reference is borrowed; callers should not decrease the reference count
    /// when they are finished with the pointer.
    #[inline]
    pub fn as_ptr(self) -> *mut ffi::PyObject {
        self.0.as_ptr()
    }

    pub(crate) fn to_any(self) -> Borrowed<'a, 'py, PyAny> {
        Borrowed(self.0, PhantomData, self.2)
    }
}

impl<'a, T: PyClass> Borrowed<'a, '_, T> {
    /// Get a view on the underlying `PyClass` contents.
    #[inline]
    pub(crate) fn get_class_object(self) -> &'a PyClassObject<T> {
        // Safety: Borrowed<'a, '_, T: PyClass> is known to contain an object
        // which is laid out in memory as a PyClassObject<T> and lives for at
        // least 'a.
        unsafe { &*self.as_ptr().cast::<PyClassObject<T>>() }
    }
}

impl<'a, 'py> Borrowed<'a, 'py, PyAny> {
    /// Constructs a new `Borrowed<'a, 'py, PyAny>` from a pointer. Panics if `ptr` is null.
    ///
    /// Prefer to use [`Bound::from_borrowed_ptr`], as that avoids the major safety risk
    /// of needing to precisely define the lifetime `'a` for which the borrow is valid.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a Python object
    /// - similar to `std::slice::from_raw_parts`, the lifetime `'a` is completely defined by
    ///   the caller and it is the caller's responsibility to ensure that the reference this is
    ///   derived from is valid for the lifetime `'a`.
    #[inline]
    #[track_caller]
    pub unsafe fn from_ptr(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        Self(
            NonNull::new(ptr).unwrap_or_else(|| crate::err::panic_after_error(py)),
            PhantomData,
            py,
        )
    }

    /// Constructs a new `Borrowed<'a, 'py, PyAny>` from a pointer. Returns `None` if `ptr` is null.
    ///
    /// Prefer to use [`Bound::from_borrowed_ptr_or_opt`], as that avoids the major safety risk
    /// of needing to precisely define the lifetime `'a` for which the borrow is valid.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a Python object, or null
    /// - similar to `std::slice::from_raw_parts`, the lifetime `'a` is completely defined by
    ///   the caller and it is the caller's responsibility to ensure that the reference this is
    ///   derived from is valid for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_ptr_or_opt(py: Python<'py>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self(ptr, PhantomData, py))
    }

    /// Constructs a new `Borrowed<'a, 'py, PyAny>` from a pointer. Returns an `Err` by calling `PyErr::fetch`
    /// if `ptr` is null.
    ///
    /// Prefer to use [`Bound::from_borrowed_ptr_or_err`], as that avoids the major safety risk
    /// of needing to precisely define the lifetime `'a` for which the borrow is valid.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a Python object, or null
    /// - similar to `std::slice::from_raw_parts`, the lifetime `'a` is completely defined by
    ///   the caller and it is the caller's responsibility to ensure that the reference this is
    ///   derived from is valid for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_ptr_or_err(py: Python<'py>, ptr: *mut ffi::PyObject) -> PyResult<Self> {
        NonNull::new(ptr).map_or_else(
            || Err(PyErr::fetch(py)),
            |ptr| Ok(Self(ptr, PhantomData, py)),
        )
    }

    /// # Safety
    /// This is similar to `std::slice::from_raw_parts`, the lifetime `'a` is completely defined by
    /// the caller and it's the caller's responsibility to ensure that the reference this is
    /// derived from is valid for the lifetime `'a`.
    #[inline]
    pub(crate) unsafe fn from_ptr_unchecked(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        Self(unsafe { NonNull::new_unchecked(ptr) }, PhantomData, py)
    }

    /// # Safety
    /// This similar to `std::slice::from_raw_parts`, the lifetime `'a` is
    /// completely defined by the caller and it is the caller's responsibility
    /// to ensure that the reference this is derived from is valid for the
    /// lifetime `'a`.
    pub(crate) unsafe fn from_non_null(py: Python<'py>, ptr: NonNull<ffi::PyObject>) -> Self {
        Self(ptr, PhantomData, py)
    }

    #[inline]
    pub(crate) fn cast<T>(self) -> Result<Borrowed<'a, 'py, T>, DowncastError<'a, 'py>>
    where
        T: PyTypeCheck,
    {
        if T::type_check(&self) {
            // Safety: type_check is responsible for ensuring that the type is correct
            Ok(unsafe { self.cast_unchecked() })
        } else {
            Err(DowncastError::new_from_borrowed(self, T::NAME))
        }
    }

    /// Converts this `PyAny` to a concrete Python type without checking validity.
    ///
    /// # Safety
    /// Callers must ensure that the type is valid or risk type confusion.
    #[inline]
    pub(crate) unsafe fn cast_unchecked<T>(self) -> Borrowed<'a, 'py, T> {
        Borrowed(self.0, PhantomData, self.2)
    }
}

impl<'a, 'py, T> From<&'a Bound<'py, T>> for Borrowed<'a, 'py, T> {
    /// Create borrow on a Bound
    #[inline]
    fn from(instance: &'a Bound<'py, T>) -> Self {
        instance.as_borrowed()
    }
}

impl<T> AsRef<Py<PyAny>> for Borrowed<'_, '_, T> {
    #[inline]
    fn as_ref(&self) -> &Py<PyAny> {
        self.as_any().as_unbound()
    }
}

impl<T> std::fmt::Debug for Borrowed<'_, '_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Bound::fmt(self, f)
    }
}

impl<'py, T> Deref for Borrowed<'_, 'py, T> {
    type Target = Bound<'py, T>;

    #[inline]
    fn deref(&self) -> &Bound<'py, T> {
        // safety: Bound has the same layout as NonNull<ffi::PyObject>
        unsafe { &*ptr_from_ref(&self.0).cast() }
    }
}

impl<T> Clone for Borrowed<'_, '_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Borrowed<'_, '_, T> {}

impl<'a, 'py, T> BoundObject<'py, T> for Borrowed<'a, 'py, T> {
    type Any = Borrowed<'a, 'py, PyAny>;

    fn as_borrowed(&self) -> Borrowed<'a, 'py, T> {
        *self
    }

    fn into_bound(self) -> Bound<'py, T> {
        (*self).to_owned()
    }

    fn into_any(self) -> Self::Any {
        self.to_any()
    }

    fn into_ptr(self) -> *mut ffi::PyObject {
        (*self).to_owned().into_ptr()
    }

    fn as_ptr(&self) -> *mut ffi::PyObject {
        (*self).as_ptr()
    }

    fn unbind(self) -> Py<T> {
        (*self).to_owned().unbind()
    }
}

/// A GIL-independent reference to an object allocated on the Python heap.
///
/// This type does not auto-dereference to the inner object because you must prove you hold the GIL to access it.
/// Instead, call one of its methods to access the inner object:
///  - [`Py::bind`] or [`Py::into_bound`], to borrow a GIL-bound reference to the contained object.
///  - [`Py::borrow`], [`Py::try_borrow`], [`Py::borrow_mut`], or [`Py::try_borrow_mut`],
///
/// to get a (mutable) reference to a contained pyclass, using a scheme similar to std's [`RefCell`].
/// See the
#[doc = concat!("[guide entry](https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/class.html#bound-and-interior-mutability)")]
/// for more information.
///  - You can call methods directly on `Py` with [`Py::call`], [`Py::call_method`] and friends.
///
/// These require passing in the [`Python<'py>`](crate::Python) token but are otherwise similar to the corresponding
/// methods on [`PyAny`].
///
/// # Example: Storing Python objects in `#[pyclass]` structs
///
/// Usually `Bound<'py, T>` is recommended for interacting with Python objects as its lifetime `'py`
/// is an association to the GIL and that enables many operations to be done as efficiently as possible.
///
/// However, `#[pyclass]` structs cannot carry a lifetime, so `Py<T>` is the only way to store
/// a Python object in a `#[pyclass]` struct.
///
/// For example, this won't compile:
///
/// ```compile_fail
/// # use pyo3::prelude::*;
/// # use pyo3::types::PyDict;
/// #
/// #[pyclass]
/// struct Foo<'py> {
///     inner: Bound<'py, PyDict>,
/// }
///
/// impl Foo {
///     fn new() -> Foo {
///         let foo = Python::attach(|py| {
///             // `py` will only last for this scope.
///
///             // `Bound<'py, PyDict>` inherits the GIL lifetime from `py` and
///             // so won't be able to outlive this closure.
///             let dict: Bound<'_, PyDict> = PyDict::new(py);
///
///             // because `Foo` contains `dict` its lifetime
///             // is now also tied to `py`.
///             Foo { inner: dict }
///         });
///         // Foo is no longer valid.
///         // Returning it from this function is a 💥 compiler error 💥
///         foo
///     }
/// }
/// ```
///
/// [`Py`]`<T>` can be used to get around this by converting `dict` into a GIL-independent reference:
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::types::PyDict;
///
/// #[pyclass]
/// struct Foo {
///     inner: Py<PyDict>,
/// }
///
/// #[pymethods]
/// impl Foo {
///     #[new]
///     fn __new__() -> Foo {
///         Python::attach(|py| {
///             let dict: Py<PyDict> = PyDict::new(py).unbind();
///             Foo { inner: dict }
///         })
///     }
/// }
/// #
/// # fn main() -> PyResult<()> {
/// #     Python::attach(|py| {
/// #         let m = pyo3::types::PyModule::new(py, "test")?;
/// #         m.add_class::<Foo>()?;
/// #
/// #         let foo: Bound<'_, Foo> = m.getattr("Foo")?.call0()?.cast_into()?;
/// #         let dict = &foo.borrow().inner;
/// #         let dict: &Bound<'_, PyDict> = dict.bind(py);
/// #
/// #         Ok(())
/// #     })
/// # }
/// ```
///
/// This can also be done with other pyclasses:
/// ```rust
/// use pyo3::prelude::*;
///
/// #[pyclass]
/// struct Bar {/* ... */}
///
/// #[pyclass]
/// struct Foo {
///     inner: Py<Bar>,
/// }
///
/// #[pymethods]
/// impl Foo {
///     #[new]
///     fn __new__() -> PyResult<Foo> {
///         Python::attach(|py| {
///             let bar: Py<Bar> = Py::new(py, Bar {})?;
///             Ok(Foo { inner: bar })
///         })
///     }
/// }
/// #
/// # fn main() -> PyResult<()> {
/// #     Python::attach(|py| {
/// #         let m = pyo3::types::PyModule::new(py, "test")?;
/// #         m.add_class::<Foo>()?;
/// #
/// #         let foo: Bound<'_, Foo> = m.getattr("Foo")?.call0()?.cast_into()?;
/// #         let bar = &foo.borrow().inner;
/// #         let bar: &Bar = &*bar.borrow(py);
/// #
/// #         Ok(())
/// #     })
/// # }
/// ```
///
/// # Example: Shared ownership of Python objects
///
/// `Py<T>` can be used to share ownership of a Python object, similar to std's [`Rc`]`<T>`.
/// As with [`Rc`]`<T>`, cloning it increases its reference count rather than duplicating
/// the underlying object.
///
/// This can be done using either [`Py::clone_ref`] or [`Py`]`<T>`'s [`Clone`] trait implementation.
/// [`Py::clone_ref`] will be faster if you happen to be already holding the GIL.
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::types::PyDict;
///
/// # fn main() {
/// Python::attach(|py| {
///     let first: Py<PyDict> = PyDict::new(py).unbind();
///
///     // All of these are valid syntax
///     let second = Py::clone_ref(&first, py);
///     let third = first.clone_ref(py);
///     #[cfg(feature = "py-clone")]
///     let fourth = Py::clone(&first);
///     #[cfg(feature = "py-clone")]
///     let fifth = first.clone();
///
///     // Disposing of our original `Py<PyDict>` just decrements the reference count.
///     drop(first);
///
///     // They all point to the same object
///     assert!(second.is(&third));
///     #[cfg(feature = "py-clone")]
///     assert!(fourth.is(&fifth));
///     #[cfg(feature = "py-clone")]
///     assert!(second.is(&fourth));
/// });
/// # }
/// ```
///
/// # Preventing reference cycles
///
/// It is easy to accidentally create reference cycles using [`Py`]`<T>`.
/// The Python interpreter can break these reference cycles within pyclasses if they
/// [integrate with the garbage collector][gc]. If your pyclass contains other Python
/// objects you should implement it to avoid leaking memory.
///
/// # A note on Python reference counts
///
/// Dropping a [`Py`]`<T>` will eventually decrease Python's reference count
/// of the pointed-to variable, allowing Python's garbage collector to free
/// the associated memory, but this may not happen immediately.  This is
/// because a [`Py`]`<T>` can be dropped at any time, but the Python reference
/// count can only be modified when the GIL is held.
///
/// If a [`Py`]`<T>` is dropped while its thread happens to be holding the
/// GIL then the Python reference count will be decreased immediately.
/// Otherwise, the reference count will be decreased the next time the GIL is
/// reacquired.
///
/// If you happen to be already holding the GIL, [`Py::drop_ref`] will decrease
/// the Python reference count immediately and will execute slightly faster than
/// relying on implicit [`Drop`]s.
///
/// # A note on `Send` and `Sync`
///
/// Accessing this object is thread-safe, since any access to its API requires a [`Python<'py>`](crate::Python) token.
/// As you can only get this by acquiring the GIL, `Py<...>` implements [`Send`] and [`Sync`].
///
/// [`Rc`]: std::rc::Rc
/// [`RefCell`]: std::cell::RefCell
/// [gc]: https://pyo3.rs/main/class/protocols.html#garbage-collector-integration
#[repr(transparent)]
pub struct Py<T>(NonNull<ffi::PyObject>, PhantomData<T>);

// The inner value is only accessed through ways that require proving the gil is held
#[cfg(feature = "nightly")]
unsafe impl<T> crate::marker::Ungil for Py<T> {}
unsafe impl<T> Send for Py<T> {}
unsafe impl<T> Sync for Py<T> {}

impl<T> Py<T>
where
    T: PyClass,
{
    /// Creates a new instance `Py<T>` of a `#[pyclass]` on the Python heap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass]
    /// struct Foo {/* fields omitted */}
    ///
    /// # fn main() -> PyResult<()> {
    /// let foo = Python::attach(|py| -> PyResult<_> {
    ///     let foo: Py<Foo> = Py::new(py, Foo {})?;
    ///     Ok(foo)
    /// })?;
    /// # Python::attach(move |_py| drop(foo));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(py: Python<'_>, value: impl Into<PyClassInitializer<T>>) -> PyResult<Py<T>> {
        Bound::new(py, value).map(Bound::unbind)
    }
}

impl<T> Py<T> {
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
        self.0.as_ptr()
    }

    /// Returns an owned raw FFI pointer represented by self.
    ///
    /// # Safety
    ///
    /// The reference is owned; when finished the caller should either transfer ownership
    /// of the pointer or decrease the reference count (e.g. with [`pyo3::ffi::Py_DecRef`](crate::ffi::Py_DecRef)).
    #[inline]
    pub fn into_ptr(self) -> *mut ffi::PyObject {
        ManuallyDrop::new(self).0.as_ptr()
    }

    /// Helper to cast to `Py<PyAny>`.
    #[inline]
    pub fn as_any(&self) -> &Py<PyAny> {
        // Safety: all Py<T> have the same memory layout, and all Py<T> are valid
        // Py<PyAny>, so pointer casting is valid.
        unsafe { &*ptr_from_ref(self).cast::<Py<PyAny>>() }
    }

    /// Helper to cast to `Py<PyAny>`, transferring ownership.
    #[inline]
    pub fn into_any(self) -> Py<PyAny> {
        // Safety: all Py<T> are valid Py<PyAny>
        unsafe { Py::from_non_null(ManuallyDrop::new(self).0) }
    }
}

impl<T> Py<T>
where
    T: PyClass,
{
    /// Immutably borrows the value `T`.
    ///
    /// This borrow lasts while the returned [`PyRef`] exists.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// For frozen classes, the simpler [`get`][Self::get] is available.
    ///
    /// Equivalent to `self.bind(py).borrow()` - see [`Bound::borrow`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use pyo3::prelude::*;
    /// #
    /// #[pyclass]
    /// struct Foo {
    ///     inner: u8,
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| -> PyResult<()> {
    ///     let foo: Py<Foo> = Py::new(py, Foo { inner: 73 })?;
    ///     let inner: &u8 = &foo.borrow(py).inner;
    ///
    ///     assert_eq!(*inner, 73);
    ///     Ok(())
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow`](#method.try_borrow).
    #[inline]
    #[track_caller]
    pub fn borrow<'py>(&'py self, py: Python<'py>) -> PyRef<'py, T> {
        self.bind(py).borrow()
    }

    /// Mutably borrows the value `T`.
    ///
    /// This borrow lasts while the returned [`PyRefMut`] exists.
    ///
    /// Equivalent to `self.bind(py).borrow_mut()` - see [`Bound::borrow_mut`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// #
    /// #[pyclass]
    /// struct Foo {
    ///     inner: u8,
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| -> PyResult<()> {
    ///     let foo: Py<Foo> = Py::new(py, Foo { inner: 73 })?;
    ///     foo.borrow_mut(py).inner = 35;
    ///
    ///     assert_eq!(foo.borrow(py).inner, 35);
    ///     Ok(())
    /// })?;
    /// # Ok(())
    /// # }
    ///  ```
    ///
    /// # Panics
    /// Panics if the value is currently borrowed. For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    #[inline]
    #[track_caller]
    pub fn borrow_mut<'py>(&'py self, py: Python<'py>) -> PyRefMut<'py, T>
    where
        T: PyClass<Frozen = False>,
    {
        self.bind(py).borrow_mut()
    }

    /// Attempts to immutably borrow the value `T`, returning an error if the value is currently mutably borrowed.
    ///
    /// The borrow lasts while the returned [`PyRef`] exists.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// For frozen classes, the simpler [`get`][Self::get] is available.
    ///
    /// Equivalent to `self.bind(py).try_borrow()` - see [`Bound::try_borrow`].
    #[inline]
    pub fn try_borrow<'py>(&'py self, py: Python<'py>) -> Result<PyRef<'py, T>, PyBorrowError> {
        self.bind(py).try_borrow()
    }

    /// Attempts to mutably borrow the value `T`, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts while the returned [`PyRefMut`] exists.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    ///
    /// Equivalent to `self.bind(py).try_borrow_mut()` - see [`Bound::try_borrow_mut`].
    #[inline]
    pub fn try_borrow_mut<'py>(
        &'py self,
        py: Python<'py>,
    ) -> Result<PyRefMut<'py, T>, PyBorrowMutError>
    where
        T: PyClass<Frozen = False>,
    {
        self.bind(py).try_borrow_mut()
    }

    /// Provide an immutable borrow of the value `T` without acquiring the GIL.
    ///
    /// This is available if the class is [`frozen`][macro@crate::pyclass] and [`Sync`].
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
    /// let cell  = Python::attach(|py| {
    ///     let counter = FrozenCounter { value: AtomicUsize::new(0) };
    ///
    ///     Py::new(py, counter).unwrap()
    /// });
    ///
    /// cell.get().value.fetch_add(1, Ordering::Relaxed);
    /// # Python::attach(move |_py| drop(cell));
    /// ```
    #[inline]
    pub fn get(&self) -> &T
    where
        T: PyClass<Frozen = True> + Sync,
    {
        // Safety: The class itself is frozen and `Sync`
        unsafe { &*self.get_class_object().get_ptr() }
    }

    /// Get a view on the underlying `PyClass` contents.
    #[inline]
    pub(crate) fn get_class_object(&self) -> &PyClassObject<T> {
        let class_object = self.as_ptr().cast::<PyClassObject<T>>();
        // Safety: Bound<T: PyClass> is known to contain an object which is laid out in memory as a
        // PyClassObject<T>.
        unsafe { &*class_object }
    }
}

impl<T> Py<T> {
    /// Attaches this `Py` to the given Python context, allowing access to further Python APIs.
    #[inline]
    pub fn bind<'py>(&self, _py: Python<'py>) -> &Bound<'py, T> {
        // Safety: `Bound` has the same layout as `Py`
        unsafe { &*ptr_from_ref(self).cast() }
    }

    /// Same as `bind` but takes ownership of `self`.
    #[inline]
    pub fn into_bound(self, py: Python<'_>) -> Bound<'_, T> {
        Bound(py, ManuallyDrop::new(self))
    }

    /// Same as `bind` but produces a `Borrowed<T>` instead of a `Bound<T>`.
    #[inline]
    pub fn bind_borrowed<'a, 'py>(&'a self, py: Python<'py>) -> Borrowed<'a, 'py, T> {
        Borrowed(self.0, PhantomData, py)
    }

    /// Returns whether `self` and `other` point to the same object. To compare
    /// the equality of two objects (the `==` operator), use [`eq`](PyAnyMethods::eq).
    ///
    /// This is equivalent to the Python expression `self is other`.
    #[inline]
    pub fn is<U: AsRef<Py<PyAny>>>(&self, o: U) -> bool {
        ptr::eq(self.as_ptr(), o.as_ref().as_ptr())
    }

    /// Gets the reference count of the `ffi::PyObject` pointer.
    #[inline]
    pub fn get_refcnt(&self, _py: Python<'_>) -> isize {
        unsafe { ffi::Py_REFCNT(self.0.as_ptr()) }
    }

    /// Makes a clone of `self`.
    ///
    /// This creates another pointer to the same object, increasing its reference count.
    ///
    /// You should prefer using this method over [`Clone`] if you happen to be holding the GIL already.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyDict;
    ///
    /// # fn main() {
    /// Python::attach(|py| {
    ///     let first: Py<PyDict> = PyDict::new(py).unbind();
    ///     let second = Py::clone_ref(&first, py);
    ///
    ///     // Both point to the same object
    ///     assert!(first.is(&second));
    /// });
    /// # }
    /// ```
    #[inline]
    pub fn clone_ref(&self, _py: Python<'_>) -> Py<T> {
        unsafe {
            ffi::Py_INCREF(self.as_ptr());
            Self::from_non_null(self.0)
        }
    }

    /// Drops `self` and immediately decreases its reference count.
    ///
    /// This method is a micro-optimisation over [`Drop`] if you happen to be holding the GIL
    /// already.
    ///
    /// Note that if you are using [`Bound`], you do not need to use [`Self::drop_ref`] since
    /// [`Bound`] guarantees that the GIL is held.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyDict;
    ///
    /// # fn main() {
    /// Python::attach(|py| {
    ///     let object: Py<PyDict> = PyDict::new(py).unbind();
    ///
    ///     // some usage of object
    ///
    ///     object.drop_ref(py);
    /// });
    /// # }
    /// ```
    #[inline]
    pub fn drop_ref(self, py: Python<'_>) {
        let _ = self.into_bound(py);
    }

    /// Returns whether the object is considered to be None.
    ///
    /// This is equivalent to the Python expression `self is None`.
    pub fn is_none(&self, _py: Python<'_>) -> bool {
        unsafe { ptr::eq(ffi::Py_None(), self.as_ptr()) }
    }

    /// Returns whether the object is considered to be true.
    ///
    /// This applies truth value testing equivalent to the Python expression `bool(self)`.
    pub fn is_truthy(&self, py: Python<'_>) -> PyResult<bool> {
        let v = unsafe { ffi::PyObject_IsTrue(self.as_ptr()) };
        err::error_on_minusone(py, v)?;
        Ok(v != 0)
    }

    /// Extracts some type from the Python object.
    ///
    /// This is a wrapper function around `FromPyObject::extract()`.
    pub fn extract<'a, 'py, D>(&'a self, py: Python<'py>) -> PyResult<D>
    where
        D: crate::conversion::FromPyObjectBound<'a, 'py>,
        // TODO it might be possible to relax this bound in future, to allow
        // e.g. `.extract::<&str>(py)` where `py` is short-lived.
        'py: 'a,
    {
        self.bind(py).as_any().extract()
    }

    /// Retrieves an attribute value.
    ///
    /// This is equivalent to the Python expression `self.attr_name`.
    ///
    /// If calling this method becomes performance-critical, the [`intern!`](crate::intern) macro
    /// can be used to intern `attr_name`, thereby avoiding repeated temporary allocations of
    /// Python strings.
    ///
    /// # Example: `intern!`ing the attribute name
    ///
    /// ```
    /// # use pyo3::{prelude::*, intern};
    /// #
    /// #[pyfunction]
    /// fn version(sys: Py<PyModule>, py: Python<'_>) -> PyResult<Py<PyAny>> {
    ///     sys.getattr(py, intern!(py, "version"))
    /// }
    /// #
    /// # Python::attach(|py| {
    /// #    let sys = py.import("sys").unwrap().unbind();
    /// #    version(sys, py).unwrap();
    /// # });
    /// ```
    pub fn getattr<'py, N>(&self, py: Python<'py>, attr_name: N) -> PyResult<Py<PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>,
    {
        self.bind(py).as_any().getattr(attr_name).map(Bound::unbind)
    }

    /// Sets an attribute value.
    ///
    /// This is equivalent to the Python expression `self.attr_name = value`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`](crate::intern)
    /// macro can be used to intern `attr_name`.
    ///
    /// # Example: `intern!`ing the attribute name
    ///
    /// ```
    /// # use pyo3::{intern, pyfunction, types::PyModule, IntoPyObjectExt, Py, PyAny, Python, PyResult};
    /// #
    /// #[pyfunction]
    /// fn set_answer(ob: Py<PyAny>, py: Python<'_>) -> PyResult<()> {
    ///     ob.setattr(py, intern!(py, "answer"), 42)
    /// }
    /// #
    /// # Python::attach(|py| {
    /// #    let ob = PyModule::new(py, "empty").unwrap().into_py_any(py).unwrap();
    /// #    set_answer(ob, py).unwrap();
    /// # });
    /// ```
    pub fn setattr<'py, N, V>(&self, py: Python<'py>, attr_name: N, value: V) -> PyResult<()>
    where
        N: IntoPyObject<'py, Target = PyString>,
        V: IntoPyObject<'py>,
    {
        self.bind(py).as_any().setattr(attr_name, value)
    }

    /// Calls the object.
    ///
    /// This is equivalent to the Python expression `self(*args, **kwargs)`.
    pub fn call<'py, A>(
        &self,
        py: Python<'py>,
        args: A,
        kwargs: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Py<PyAny>>
    where
        A: PyCallArgs<'py>,
    {
        self.bind(py).as_any().call(args, kwargs).map(Bound::unbind)
    }

    /// Calls the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self(*args)`.
    pub fn call1<'py, A>(&self, py: Python<'py>, args: A) -> PyResult<Py<PyAny>>
    where
        A: PyCallArgs<'py>,
    {
        self.bind(py).as_any().call1(args).map(Bound::unbind)
    }

    /// Calls the object without arguments.
    ///
    /// This is equivalent to the Python expression `self()`.
    pub fn call0(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.bind(py).as_any().call0().map(Bound::unbind)
    }

    /// Calls a method on the object.
    ///
    /// This is equivalent to the Python expression `self.name(*args, **kwargs)`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`](crate::intern)
    /// macro can be used to intern `name`.
    pub fn call_method<'py, N, A>(
        &self,
        py: Python<'py>,
        name: N,
        args: A,
        kwargs: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Py<PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>,
        A: PyCallArgs<'py>,
    {
        self.bind(py)
            .as_any()
            .call_method(name, args, kwargs)
            .map(Bound::unbind)
    }

    /// Calls a method on the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self.name(*args)`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`](crate::intern)
    /// macro can be used to intern `name`.
    pub fn call_method1<'py, N, A>(&self, py: Python<'py>, name: N, args: A) -> PyResult<Py<PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>,
        A: PyCallArgs<'py>,
    {
        self.bind(py)
            .as_any()
            .call_method1(name, args)
            .map(Bound::unbind)
    }

    /// Calls a method on the object with no arguments.
    ///
    /// This is equivalent to the Python expression `self.name()`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`](crate::intern)
    /// macro can be used to intern `name`.
    pub fn call_method0<'py, N>(&self, py: Python<'py>, name: N) -> PyResult<Py<PyAny>>
    where
        N: IntoPyObject<'py, Target = PyString>,
    {
        self.bind(py).as_any().call_method0(name).map(Bound::unbind)
    }

    /// Create a `Py<T>` instance by taking ownership of the given FFI pointer.
    ///
    /// # Safety
    /// `ptr` must be a pointer to a Python object of type T.
    ///
    /// Callers must own the object referred to by `ptr`, as this function
    /// implicitly takes ownership of that object.
    ///
    /// # Panics
    /// Panics if `ptr` is null.
    #[inline]
    #[track_caller]
    pub unsafe fn from_owned_ptr(py: Python<'_>, ptr: *mut ffi::PyObject) -> Py<T> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Py(nonnull_ptr, PhantomData),
            None => crate::err::panic_after_error(py),
        }
    }

    /// Create a `Py<T>` instance by taking ownership of the given FFI pointer.
    ///
    /// If `ptr` is null then the current Python exception is fetched as a [`PyErr`].
    ///
    /// # Safety
    /// If non-null, `ptr` must be a pointer to a Python object of type T.
    #[inline]
    pub unsafe fn from_owned_ptr_or_err(
        py: Python<'_>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Py<T>> {
        match NonNull::new(ptr) {
            Some(nonnull_ptr) => Ok(Py(nonnull_ptr, PhantomData)),
            None => Err(PyErr::fetch(py)),
        }
    }

    /// Create a `Py<T>` instance by taking ownership of the given FFI pointer.
    ///
    /// If `ptr` is null then `None` is returned.
    ///
    /// # Safety
    /// If non-null, `ptr` must be a pointer to a Python object of type T.
    #[inline]
    pub unsafe fn from_owned_ptr_or_opt(_py: Python<'_>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|nonnull_ptr| Py(nonnull_ptr, PhantomData))
    }

    /// Constructs a new `Py<T>` instance by taking ownership of the given FFI pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a non-null pointer to a Python object or type `T`.
    pub(crate) unsafe fn from_owned_ptr_unchecked(ptr: *mut ffi::PyObject) -> Self {
        Py(unsafe { NonNull::new_unchecked(ptr) }, PhantomData)
    }

    /// Create a `Py<T>` instance by creating a new reference from the given FFI pointer.
    ///
    /// # Safety
    /// `ptr` must be a pointer to a Python object of type T.
    ///
    /// # Panics
    /// Panics if `ptr` is null.
    #[inline]
    #[track_caller]
    pub unsafe fn from_borrowed_ptr(py: Python<'_>, ptr: *mut ffi::PyObject) -> Py<T> {
        match unsafe { Self::from_borrowed_ptr_or_opt(py, ptr) } {
            Some(slf) => slf,
            None => crate::err::panic_after_error(py),
        }
    }

    /// Create a `Py<T>` instance by creating a new reference from the given FFI pointer.
    ///
    /// If `ptr` is null then the current Python exception is fetched as a `PyErr`.
    ///
    /// # Safety
    /// `ptr` must be a pointer to a Python object of type T.
    #[inline]
    pub unsafe fn from_borrowed_ptr_or_err(
        py: Python<'_>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        unsafe { Self::from_borrowed_ptr_or_opt(py, ptr).ok_or_else(|| PyErr::fetch(py)) }
    }

    /// Create a `Py<T>` instance by creating a new reference from the given FFI pointer.
    ///
    /// If `ptr` is null then `None` is returned.
    ///
    /// # Safety
    /// `ptr` must be a pointer to a Python object of type T.
    #[inline]
    pub unsafe fn from_borrowed_ptr_or_opt(
        _py: Python<'_>,
        ptr: *mut ffi::PyObject,
    ) -> Option<Self> {
        unsafe {
            NonNull::new(ptr).map(|nonnull_ptr| {
                ffi::Py_INCREF(ptr);
                Py(nonnull_ptr, PhantomData)
            })
        }
    }

    /// For internal conversions.
    ///
    /// # Safety
    /// `ptr` must point to a Python object of type T.
    unsafe fn from_non_null(ptr: NonNull<ffi::PyObject>) -> Self {
        Self(ptr, PhantomData)
    }
}

impl<T> AsRef<Py<PyAny>> for Py<T> {
    #[inline]
    fn as_ref(&self) -> &Py<PyAny> {
        self.as_any()
    }
}

impl<T> std::convert::From<Py<T>> for Py<PyAny>
where
    T: DerefToPyAny,
{
    #[inline]
    fn from(other: Py<T>) -> Self {
        other.into_any()
    }
}

impl<T> std::convert::From<Bound<'_, T>> for Py<PyAny>
where
    T: DerefToPyAny,
{
    #[inline]
    fn from(other: Bound<'_, T>) -> Self {
        other.into_any().unbind()
    }
}

impl<T> std::convert::From<Bound<'_, T>> for Py<T> {
    #[inline]
    fn from(other: Bound<'_, T>) -> Self {
        other.unbind()
    }
}

impl<T> std::convert::From<Borrowed<'_, '_, T>> for Py<T> {
    fn from(value: Borrowed<'_, '_, T>) -> Self {
        value.unbind()
    }
}

impl<'a, T> std::convert::From<PyRef<'a, T>> for Py<T>
where
    T: PyClass,
{
    fn from(pyref: PyRef<'a, T>) -> Self {
        unsafe { Py::from_borrowed_ptr(pyref.py(), pyref.as_ptr()) }
    }
}

impl<'a, T> std::convert::From<PyRefMut<'a, T>> for Py<T>
where
    T: PyClass<Frozen = False>,
{
    fn from(pyref: PyRefMut<'a, T>) -> Self {
        unsafe { Py::from_borrowed_ptr(pyref.py(), pyref.as_ptr()) }
    }
}

/// If the GIL is held this increments `self`'s reference count.
/// Otherwise, it will panic.
///
/// Only available if the `py-clone` feature is enabled.
#[cfg(feature = "py-clone")]
impl<T> Clone for Py<T> {
    #[track_caller]
    fn clone(&self) -> Self {
        unsafe {
            state::register_incref(self.0);
        }
        Self(self.0, PhantomData)
    }
}

/// Dropping a `Py` instance decrements the reference count
/// on the object by one if the GIL is held.
///
/// Otherwise and by default, this registers the underlying pointer to have its reference count
/// decremented the next time PyO3 acquires the GIL.
///
/// However, if the `pyo3_disable_reference_pool` conditional compilation flag
/// is enabled, it will abort the process.
impl<T> Drop for Py<T> {
    #[track_caller]
    fn drop(&mut self) {
        unsafe {
            state::register_decref(self.0);
        }
    }
}

impl<T> FromPyObject<'_> for Py<T>
where
    T: PyTypeCheck,
{
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = T::PYTHON_TYPE;

    /// Extracts `Self` from the source `PyObject`.
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        ob.extract::<Bound<'_, T>>().map(Bound::unbind)
    }
}

impl<'py, T> FromPyObject<'py> for Bound<'py, T>
where
    T: PyTypeCheck,
{
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = T::PYTHON_TYPE;

    /// Extracts `Self` from the source `PyObject`.
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.cast().cloned().map_err(Into::into)
    }
}

impl<T> std::fmt::Display for Py<T>
where
    T: PyTypeInfo,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Python::attach(|py| std::fmt::Display::fmt(self.bind(py), f))
    }
}

impl<T> std::fmt::Debug for Py<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Py").field(&self.0.as_ptr()).finish()
    }
}

/// A commonly-used alias for `Py<PyAny>`.
///
/// This is an owned reference a Python object without any type information. This value can also be
/// safely sent between threads.
///
/// See the documentation for [`Py`](struct.Py.html).
#[deprecated(since = "0.26.0", note = "use `Py<PyAny>` instead")]
pub type PyObject = Py<PyAny>;

impl Py<PyAny> {
    /// Downcast this `Py<PyAny>` to a concrete Python type or pyclass.
    ///
    /// Note that you can often avoid casting yourself by just specifying the desired type in
    /// function or method signatures. However, manual casting is sometimes necessary.
    ///
    /// For extracting a Rust-only type, see [`Py::extract`].
    ///
    ///  # Example: Downcasting to a specific Python object
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyDict, PyList};
    ///
    /// Python::attach(|py| {
    ///     let any = PyDict::new(py).into_any().unbind();
    ///
    ///     assert!(any.downcast_bound::<PyDict>(py).is_ok());
    ///     assert!(any.downcast_bound::<PyList>(py).is_err());
    /// });
    /// ```
    ///
    /// # Example: Getting a reference to a pyclass
    ///
    /// This is useful if you want to mutate a `Py<PyAny>` that might actually be a pyclass.
    ///
    /// ```rust
    /// # fn main() -> Result<(), pyo3::PyErr> {
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass]
    /// struct Class {
    ///     i: i32,
    /// }
    ///
    /// Python::attach(|py| {
    ///     let class = Py::new(py, Class { i: 0 })?.into_any();
    ///
    ///     let class_bound = class.downcast_bound::<Class>(py)?;
    ///
    ///     class_bound.borrow_mut().i += 1;
    ///
    ///     // Alternatively you can get a `PyRefMut` directly
    ///     let class_ref: PyRefMut<'_, Class> = class.extract(py)?;
    ///     assert_eq!(class_ref.i, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    // FIXME(icxolu) deprecate in favor of `Py::cast_bound`
    #[inline]
    pub fn downcast_bound<'py, T>(
        &self,
        py: Python<'py>,
    ) -> Result<&Bound<'py, T>, DowncastError<'_, 'py>>
    where
        T: PyTypeCheck,
    {
        self.cast_bound(py)
    }

    /// Casts the `Py<PyAny>` to a concrete Python object type without checking validity.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    // FIXME(icxolu) deprecate in favor of `Py::cast_bound_unchecked`
    #[inline]
    pub unsafe fn downcast_bound_unchecked<'py, T>(&self, py: Python<'py>) -> &Bound<'py, T> {
        unsafe { self.cast_bound_unchecked(py) }
    }
}

impl<T> Py<T> {
    /// Cast this `Py<T>` to a concrete Python type or pyclass.
    ///
    /// Note that you can often avoid casting yourself by just specifying the desired type in
    /// function or method signatures. However, manual casting is sometimes necessary.
    ///
    /// For extracting a Rust-only type, see [`Py::extract`].
    ///
    /// # Example: Casting to a specific Python object
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyDict, PyList};
    ///
    /// Python::attach(|py| {
    ///     let any = PyDict::new(py).into_any().unbind();
    ///
    ///     assert!(any.cast_bound::<PyDict>(py).is_ok());
    ///     assert!(any.cast_bound::<PyList>(py).is_err());
    /// });
    /// ```
    ///
    /// # Example: Getting a reference to a pyclass
    ///
    /// This is useful if you want to mutate a `Py<PyAny>` that might actually be a pyclass.
    ///
    /// ```rust
    /// # fn main() -> Result<(), pyo3::PyErr> {
    /// use pyo3::prelude::*;
    ///
    /// #[pyclass]
    /// struct Class {
    ///     i: i32,
    /// }
    ///
    /// Python::attach(|py| {
    ///     let class = Py::new(py, Class { i: 0 })?.into_any();
    ///
    ///     let class_bound = class.cast_bound::<Class>(py)?;
    ///
    ///     class_bound.borrow_mut().i += 1;
    ///
    ///     // Alternatively you can get a `PyRefMut` directly
    ///     let class_ref: PyRefMut<'_, Class> = class.extract(py)?;
    ///     assert_eq!(class_ref.i, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn cast_bound<'py, U>(
        &self,
        py: Python<'py>,
    ) -> Result<&Bound<'py, U>, DowncastError<'_, 'py>>
    where
        U: PyTypeCheck,
    {
        self.bind(py).cast()
    }

    /// Casts the `Py<T>` to a concrete Python object type without checking validity.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    #[inline]
    pub unsafe fn cast_bound_unchecked<'py, U>(&self, py: Python<'py>) -> &Bound<'py, U> {
        unsafe { self.bind(py).cast_unchecked() }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bound, IntoPyObject, Py};
    use crate::test_utils::generate_unique_module_name;
    use crate::types::{dict::IntoPyDict, PyAnyMethods, PyCapsule, PyDict, PyString};
    use crate::{ffi, Borrowed, PyAny, PyResult, Python};
    use pyo3_ffi::c_str;
    use std::ffi::CStr;

    #[test]
    fn test_call() {
        Python::attach(|py| {
            let obj = py.get_type::<PyDict>().into_pyobject(py).unwrap();

            let assert_repr = |obj: Bound<'_, PyAny>, expected: &str| {
                assert_eq!(obj.repr().unwrap(), expected);
            };

            assert_repr(obj.call0().unwrap(), "{}");
            assert_repr(obj.call1(()).unwrap(), "{}");
            assert_repr(obj.call((), None).unwrap(), "{}");

            assert_repr(obj.call1(((('x', 1),),)).unwrap(), "{'x': 1}");
            assert_repr(
                obj.call((), Some(&[('x', 1)].into_py_dict(py).unwrap()))
                    .unwrap(),
                "{'x': 1}",
            );
        })
    }

    #[test]
    fn test_call_tuple_ref() {
        let assert_repr = |obj: &Bound<'_, PyAny>, expected: &str| {
            use crate::prelude::PyStringMethods;
            assert_eq!(
                obj.repr()
                    .unwrap()
                    .to_cow()
                    .unwrap()
                    .trim_matches(|c| c == '{' || c == '}'),
                expected.trim_matches(|c| c == ',' || c == ' ')
            );
        };

        macro_rules! tuple {
            ($py:ident, $($key: literal => $value: literal),+) => {
                let ty_obj = $py.get_type::<PyDict>().into_pyobject($py).unwrap();
                assert!(ty_obj.call1(&(($(($key),)+),)).is_err());
                let obj = ty_obj.call1(&(($(($key, i32::from($value)),)+),)).unwrap();
                assert_repr(&obj, concat!($("'", $key, "'", ": ", stringify!($value), ", ",)+));
                assert!(obj.call_method1("update", &(($(($key),)+),)).is_err());
                obj.call_method1("update", &(($((i32::from($value), $key),)+),)).unwrap();
                assert_repr(&obj, concat!(
                    concat!($("'", $key, "'", ": ", stringify!($value), ", ",)+),
                    concat!($(stringify!($value), ": ", "'", $key, "'", ", ",)+)
                ));
            };
        }

        Python::attach(|py| {
            tuple!(py, "a" => 1);
            tuple!(py, "a" => 1, "b" => 2);
            tuple!(py, "a" => 1, "b" => 2, "c" => 3);
            tuple!(py, "a" => 1, "b" => 2, "c" => 3, "d" => 4);
            tuple!(py, "a" => 1, "b" => 2, "c" => 3, "d" => 4, "e" => 5);
            tuple!(py, "a" => 1, "b" => 2, "c" => 3, "d" => 4, "e" => 5, "f" => 6);
            tuple!(py, "a" => 1, "b" => 2, "c" => 3, "d" => 4, "e" => 5, "f" => 6, "g" => 7);
            tuple!(py, "a" => 1, "b" => 2, "c" => 3, "d" => 4, "e" => 5, "f" => 6, "g" => 7, "h" => 8);
            tuple!(py, "a" => 1, "b" => 2, "c" => 3, "d" => 4, "e" => 5, "f" => 6, "g" => 7, "h" => 8, "i" => 9);
            tuple!(py, "a" => 1, "b" => 2, "c" => 3, "d" => 4, "e" => 5, "f" => 6, "g" => 7, "h" => 8, "i" => 9, "j" => 10, "k" => 11);
            tuple!(py, "a" => 1, "b" => 2, "c" => 3, "d" => 4, "e" => 5, "f" => 6, "g" => 7, "h" => 8, "i" => 9, "j" => 10, "k" => 11, "l" => 12);
        })
    }

    #[test]
    fn test_call_for_non_existing_method() {
        Python::attach(|py| {
            let obj: Py<PyAny> = PyDict::new(py).into();
            assert!(obj.call_method0(py, "asdf").is_err());
            assert!(obj
                .call_method(py, "nonexistent_method", (1,), None)
                .is_err());
            assert!(obj.call_method0(py, "nonexistent_method").is_err());
            assert!(obj.call_method1(py, "nonexistent_method", (1,)).is_err());
        });
    }

    #[test]
    fn py_from_dict() {
        let dict: Py<PyDict> = Python::attach(|py| {
            let native = PyDict::new(py);
            Py::from(native)
        });

        Python::attach(move |py| {
            assert_eq!(dict.get_refcnt(py), 1);
        });
    }

    #[test]
    fn pyobject_from_py() {
        Python::attach(|py| {
            let dict: Py<PyDict> = PyDict::new(py).unbind();
            let cnt = dict.get_refcnt(py);
            let p: Py<PyAny> = dict.into();
            assert_eq!(p.get_refcnt(py), cnt);
        });
    }

    #[test]
    fn attr() -> PyResult<()> {
        use crate::types::PyModule;

        Python::attach(|py| {
            const CODE: &CStr = c_str!(
                r#"
class A:
    pass
a = A()
   "#
            );
            let module =
                PyModule::from_code(py, CODE, c_str!(""), &generate_unique_module_name(""))?;
            let instance: Py<PyAny> = module.getattr("a")?.into();

            instance.getattr(py, "foo").unwrap_err();

            instance.setattr(py, "foo", "bar")?;

            assert!(instance
                .getattr(py, "foo")?
                .bind(py)
                .eq(PyString::new(py, "bar"))?);

            instance.getattr(py, "foo")?;
            Ok(())
        })
    }

    #[test]
    fn pystring_attr() -> PyResult<()> {
        use crate::types::PyModule;

        Python::attach(|py| {
            const CODE: &CStr = c_str!(
                r#"
class A:
    pass
a = A()
   "#
            );
            let module =
                PyModule::from_code(py, CODE, c_str!(""), &generate_unique_module_name(""))?;
            let instance: Py<PyAny> = module.getattr("a")?.into();

            let foo = crate::intern!(py, "foo");
            let bar = crate::intern!(py, "bar");

            instance.getattr(py, foo).unwrap_err();
            instance.setattr(py, foo, bar)?;
            assert!(instance.getattr(py, foo)?.bind(py).eq(bar)?);
            Ok(())
        })
    }

    #[test]
    fn invalid_attr() -> PyResult<()> {
        Python::attach(|py| {
            let instance: Py<PyAny> = py.eval(ffi::c_str!("object()"), None, None)?.into();

            instance.getattr(py, "foo").unwrap_err();

            // Cannot assign arbitrary attributes to `object`
            instance.setattr(py, "foo", "bar").unwrap_err();
            Ok(())
        })
    }

    #[test]
    fn test_py2_from_py_object() {
        Python::attach(|py| {
            let instance = py.eval(ffi::c_str!("object()"), None, None).unwrap();
            let ptr = instance.as_ptr();
            let instance: Bound<'_, PyAny> = instance.extract().unwrap();
            assert_eq!(instance.as_ptr(), ptr);
        })
    }

    #[test]
    fn test_py2_into_py_object() {
        Python::attach(|py| {
            let instance = py.eval(ffi::c_str!("object()"), None, None).unwrap();
            let ptr = instance.as_ptr();
            let instance: Py<PyAny> = instance.clone().unbind();
            assert_eq!(instance.as_ptr(), ptr);
        })
    }

    #[test]
    fn test_debug_fmt() {
        Python::attach(|py| {
            let obj = "hello world".into_pyobject(py).unwrap();
            assert_eq!(format!("{obj:?}"), "'hello world'");
        });
    }

    #[test]
    fn test_display_fmt() {
        Python::attach(|py| {
            let obj = "hello world".into_pyobject(py).unwrap();
            assert_eq!(format!("{obj}"), "hello world");
        });
    }

    #[test]
    fn test_bound_as_any() {
        Python::attach(|py| {
            let obj = PyString::new(py, "hello world");
            let any = obj.as_any();
            assert_eq!(any.as_ptr(), obj.as_ptr());
        });
    }

    #[test]
    fn test_bound_into_any() {
        Python::attach(|py| {
            let obj = PyString::new(py, "hello world");
            let any = obj.clone().into_any();
            assert_eq!(any.as_ptr(), obj.as_ptr());
        });
    }

    #[test]
    fn test_bound_py_conversions() {
        Python::attach(|py| {
            let obj: Bound<'_, PyString> = PyString::new(py, "hello world");
            let obj_unbound: &Py<PyString> = obj.as_unbound();
            let _: &Bound<'_, PyString> = obj_unbound.bind(py);

            let obj_unbound: Py<PyString> = obj.unbind();
            let obj: Bound<'_, PyString> = obj_unbound.into_bound(py);

            assert_eq!(obj, "hello world");
        });
    }

    #[test]
    fn test_borrowed_identity() {
        Python::attach(|py| {
            let yes = true.into_pyobject(py).unwrap();
            let no = false.into_pyobject(py).unwrap();

            assert!(yes.is(yes));
            assert!(!yes.is(no));
        });
    }

    #[test]
    fn bound_from_borrowed_ptr_constructors() {
        // More detailed tests of the underlying semantics in pycell.rs
        Python::attach(|py| {
            fn check_drop<'py>(
                py: Python<'py>,
                method: impl FnOnce(*mut ffi::PyObject) -> Bound<'py, PyAny>,
            ) {
                let mut dropped = false;
                let capsule = PyCapsule::new_with_destructor(
                    py,
                    (&mut dropped) as *mut _ as usize,
                    None,
                    |ptr, _| unsafe { std::ptr::write(ptr as *mut bool, true) },
                )
                .unwrap();

                let bound = method(capsule.as_ptr());
                assert!(!dropped);

                // creating the bound should have increased the refcount
                drop(capsule);
                assert!(!dropped);

                // dropping the bound should now also decrease the refcount and free the object
                drop(bound);
                assert!(dropped);
            }

            check_drop(py, |ptr| unsafe { Bound::from_borrowed_ptr(py, ptr) });
            check_drop(py, |ptr| unsafe {
                Bound::from_borrowed_ptr_or_opt(py, ptr).unwrap()
            });
            check_drop(py, |ptr| unsafe {
                Bound::from_borrowed_ptr_or_err(py, ptr).unwrap()
            });
        })
    }

    #[test]
    fn borrowed_ptr_constructors() {
        // More detailed tests of the underlying semantics in pycell.rs
        Python::attach(|py| {
            fn check_drop<'py>(
                py: Python<'py>,
                method: impl FnOnce(&*mut ffi::PyObject) -> Borrowed<'_, 'py, PyAny>,
            ) {
                let mut dropped = false;
                let capsule = PyCapsule::new_with_destructor(
                    py,
                    (&mut dropped) as *mut _ as usize,
                    None,
                    |ptr, _| unsafe { std::ptr::write(ptr as *mut bool, true) },
                )
                .unwrap();

                let ptr = &capsule.as_ptr();
                let _borrowed = method(ptr);
                assert!(!dropped);

                // creating the borrow should not have increased the refcount
                drop(capsule);
                assert!(dropped);
            }

            check_drop(py, |&ptr| unsafe { Borrowed::from_ptr(py, ptr) });
            check_drop(py, |&ptr| unsafe {
                Borrowed::from_ptr_or_opt(py, ptr).unwrap()
            });
            check_drop(py, |&ptr| unsafe {
                Borrowed::from_ptr_or_err(py, ptr).unwrap()
            });
        })
    }

    #[test]
    fn explicit_drop_ref() {
        Python::attach(|py| {
            let object: Py<PyDict> = PyDict::new(py).unbind();
            let object2 = object.clone_ref(py);

            assert_eq!(object.as_ptr(), object2.as_ptr());
            assert_eq!(object.get_refcnt(py), 2);

            object.drop_ref(py);

            assert_eq!(object2.get_refcnt(py), 1);

            object2.drop_ref(py);
        });
    }

    #[cfg(feature = "macros")]
    mod using_macros {
        use super::*;

        #[crate::pyclass(crate = "crate")]
        struct SomeClass(i32);

        #[test]
        fn py_borrow_methods() {
            // More detailed tests of the underlying semantics in pycell.rs
            Python::attach(|py| {
                let instance = Py::new(py, SomeClass(0)).unwrap();
                assert_eq!(instance.borrow(py).0, 0);
                assert_eq!(instance.try_borrow(py).unwrap().0, 0);
                assert_eq!(instance.borrow_mut(py).0, 0);
                assert_eq!(instance.try_borrow_mut(py).unwrap().0, 0);

                instance.borrow_mut(py).0 = 123;

                assert_eq!(instance.borrow(py).0, 123);
                assert_eq!(instance.try_borrow(py).unwrap().0, 123);
                assert_eq!(instance.borrow_mut(py).0, 123);
                assert_eq!(instance.try_borrow_mut(py).unwrap().0, 123);
            })
        }

        #[test]
        fn bound_borrow_methods() {
            // More detailed tests of the underlying semantics in pycell.rs
            Python::attach(|py| {
                let instance = Bound::new(py, SomeClass(0)).unwrap();
                assert_eq!(instance.borrow().0, 0);
                assert_eq!(instance.try_borrow().unwrap().0, 0);
                assert_eq!(instance.borrow_mut().0, 0);
                assert_eq!(instance.try_borrow_mut().unwrap().0, 0);

                instance.borrow_mut().0 = 123;

                assert_eq!(instance.borrow().0, 123);
                assert_eq!(instance.try_borrow().unwrap().0, 123);
                assert_eq!(instance.borrow_mut().0, 123);
                assert_eq!(instance.try_borrow_mut().unwrap().0, 123);
            })
        }

        #[crate::pyclass(frozen, crate = "crate")]
        struct FrozenClass(i32);

        #[test]
        fn test_frozen_get() {
            Python::attach(|py| {
                for i in 0..10 {
                    let instance = Py::new(py, FrozenClass(i)).unwrap();
                    assert_eq!(instance.get().0, i);

                    assert_eq!(instance.bind(py).get().0, i);
                }
            })
        }

        #[crate::pyclass(crate = "crate", subclass)]
        struct BaseClass;

        trait MyClassMethods<'py>: Sized {
            fn pyrepr_by_ref(&self) -> PyResult<String>;
            fn pyrepr_by_val(self) -> PyResult<String> {
                self.pyrepr_by_ref()
            }
        }
        impl<'py> MyClassMethods<'py> for Bound<'py, BaseClass> {
            fn pyrepr_by_ref(&self) -> PyResult<String> {
                self.call_method0("__repr__")?.extract()
            }
        }

        #[crate::pyclass(crate = "crate", extends = BaseClass)]
        struct SubClass;

        #[test]
        fn test_as_super() {
            Python::attach(|py| {
                let obj = Bound::new(py, (SubClass, BaseClass)).unwrap();
                let _: &Bound<'_, BaseClass> = obj.as_super();
                let _: &Bound<'_, PyAny> = obj.as_super().as_super();
                assert!(obj.as_super().pyrepr_by_ref().is_ok());
            })
        }

        #[test]
        fn test_into_super() {
            Python::attach(|py| {
                let obj = Bound::new(py, (SubClass, BaseClass)).unwrap();
                let _: Bound<'_, BaseClass> = obj.clone().into_super();
                let _: Bound<'_, PyAny> = obj.clone().into_super().into_super();
                assert!(obj.into_super().pyrepr_by_val().is_ok());
            })
        }
    }
}
