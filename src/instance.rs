use crate::err::{self, PyDowncastError, PyErr, PyResult};
use crate::pycell::{PyBorrowError, PyBorrowMutError, PyCell};
use crate::pyclass::boolean_struct::{False, True};
use crate::type_object::HasPyGilRef;
use crate::types::any::PyAnyMethods;
use crate::types::string::PyStringMethods;
use crate::types::{PyDict, PyString, PyTuple};
use crate::{
    ffi, AsPyPointer, FromPyObject, IntoPy, PyAny, PyClass, PyClassInitializer, PyRef, PyRefMut,
    PyTypeInfo, Python, ToPyObject,
};
use crate::{gil, PyTypeCheck};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::NonNull;

/// Types that are built into the Python interpreter.
///
/// PyO3 is designed in a way that all references to those types are bound
/// to the GIL, which is why you can get a token from all references of those
/// types.
///
/// # Safety
///
/// This trait must only be implemented for types which cannot be accessed without the GIL.
pub unsafe trait PyNativeType: Sized {
    /// The form of this which is stored inside a `Py<T>` smart pointer.
    type AsRefSource: HasPyGilRef<AsRefTarget = Self>;

    /// Cast `&self` to a `Borrowed` smart pointer.
    ///
    /// `Borrowed<T>` implements `Deref<Target=Bound<T>>`, so can also be used in locations
    /// where `Bound<T>` is expected.
    ///
    /// This is available as a migration tool to adjust code from the deprecated "GIL Refs"
    /// API to the `Bound` smart pointer API.
    #[inline]
    fn as_borrowed(&self) -> Borrowed<'_, '_, Self::AsRefSource> {
        // Safety: &'py Self is expected to be a Python pointer,
        // so has the same layout as Borrowed<'py, 'py, T>
        Borrowed(
            unsafe { NonNull::new_unchecked(self as *const Self as *mut _) },
            PhantomData,
            self.py(),
        )
    }

    /// Returns a GIL marker constrained to the lifetime of this type.
    #[inline]
    fn py(&self) -> Python<'_> {
        unsafe { Python::assume_gil_acquired() }
    }
    /// Cast `&PyAny` to `&Self` without no type checking.
    ///
    /// # Safety
    ///
    /// `obj` must have the same layout as `*const ffi::PyObject` and must be
    /// an instance of a type corresponding to `Self`.
    unsafe fn unchecked_downcast(obj: &PyAny) -> &Self {
        &*(obj.as_ptr() as *const Self)
    }
}

/// A GIL-attached equivalent to `Py`.
#[repr(transparent)]
pub struct Bound<'py, T>(Python<'py>, ManuallyDrop<Py<T>>);

impl<'py> Bound<'py, PyAny> {
    /// Constructs a new Bound from a pointer. Panics if ptr is null.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a Python object.
    pub(crate) unsafe fn from_owned_ptr(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        Self(py, ManuallyDrop::new(Py::from_owned_ptr(py, ptr)))
    }

    /// Constructs a new Bound from a pointer. Returns None if ptr is null.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a Python object, or NULL.
    pub(crate) unsafe fn from_owned_ptr_or_opt(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Option<Self> {
        Py::from_owned_ptr_or_opt(py, ptr).map(|obj| Self(py, ManuallyDrop::new(obj)))
    }

    /// Constructs a new Bound from a pointer. Returns error if ptr is null.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a Python object, or NULL.
    pub(crate) unsafe fn from_owned_ptr_or_err(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        Py::from_owned_ptr_or_err(py, ptr).map(|obj| Self(py, ManuallyDrop::new(obj)))
    }
}

impl<'py, T> std::fmt::Debug for Bound<'py, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let any = self.as_any();
        python_format(any, any.repr(), f)
    }
}

impl<'py, T> std::fmt::Display for Bound<'py, T> {
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
        Result::Err(err) => err.write_unraisable_bound(any.py(), Some(any)),
    }

    match any.get_type().name() {
        Result::Ok(name) => std::write!(f, "<unprintable {} object>", name),
        Result::Err(_err) => f.write_str("<unprintable object>"),
    }
}

impl<'py, T> Deref for Bound<'py, T>
where
    T: AsRef<PyAny>,
{
    type Target = Bound<'py, PyAny>;

    #[inline]
    fn deref(&self) -> &Bound<'py, PyAny> {
        self.as_any()
    }
}

impl<'py, T> AsRef<Bound<'py, PyAny>> for Bound<'py, T>
where
    T: AsRef<PyAny>,
{
    #[inline]
    fn as_ref(&self) -> &Bound<'py, PyAny> {
        self.as_any()
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
        unsafe { &*(self as *const Self).cast::<Bound<'py, PyAny>>() }
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

    /// Casts this `Bound<T>` as the corresponding "GIL Ref" type.
    ///
    /// This is a helper to be used for migration from the deprecated "GIL Refs" API.
    #[inline]
    pub fn as_gil_ref(&'py self) -> &'py T::AsRefTarget
    where
        T: HasPyGilRef,
    {
        unsafe { self.py().from_borrowed_ptr(self.as_ptr()) }
    }

    /// Casts this `Bound<T>` as the corresponding "GIL Ref" type, registering the pointer on the
    /// [release pool](Python::from_owned_ptr).
    ///
    /// This is a helper to be used for migration from the deprecated "GIL Refs" API.
    #[inline]
    pub fn into_gil_ref(self) -> &'py T::AsRefTarget
    where
        T: HasPyGilRef,
    {
        unsafe { self.py().from_owned_ptr(self.into_ptr()) }
    }
}

unsafe impl<T> AsPyPointer for Bound<'_, T> {
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.1.as_ptr()
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

impl<'py, T> Borrowed<'_, 'py, T> {
    /// Creates a new owned `Bound` from this borrowed reference by increasing the reference count.
    pub(crate) fn to_owned(self) -> Bound<'py, T> {
        (*self).clone()
    }
}

impl<'a, 'py> Borrowed<'a, 'py, PyAny> {
    /// # Safety
    /// This is similar to `std::slice::from_raw_parts`, the lifetime `'a` is completely defined by
    /// the caller and it's the caller's responsibility to ensure that the reference this is
    /// derived from is valid for the lifetime `'a`.
    pub(crate) unsafe fn from_ptr_or_err(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<Self> {
        NonNull::new(ptr).map_or_else(
            || Err(PyErr::fetch(py)),
            |ptr| Ok(Self(ptr, PhantomData, py)),
        )
    }

    /// # Safety
    /// This is similar to `std::slice::from_raw_parts`, the lifetime `'a` is completely defined by
    /// the caller and it's the caller's responsibility to ensure that the reference this is
    /// derived from is valid for the lifetime `'a`.
    pub(crate) unsafe fn from_ptr_or_opt(py: Python<'py>, ptr: *mut ffi::PyObject) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self(ptr, PhantomData, py))
    }

    /// # Safety
    /// This is similar to `std::slice::from_raw_parts`, the lifetime `'a` is completely defined by
    /// the caller and it's the caller's responsibility to ensure that the reference this is
    /// derived from is valid for the lifetime `'a`.
    pub(crate) unsafe fn from_ptr(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        Self(
            NonNull::new(ptr).unwrap_or_else(|| crate::err::panic_after_error(py)),
            PhantomData,
            py,
        )
    }

    /// # Safety
    /// This is similar to `std::slice::from_raw_parts`, the lifetime `'a` is completely defined by
    /// the caller and it's the caller's responsibility to ensure that the reference this is
    /// derived from is valid for the lifetime `'a`.
    pub(crate) unsafe fn from_ptr_unchecked(py: Python<'py>, ptr: *mut ffi::PyObject) -> Self {
        Self(NonNull::new_unchecked(ptr), PhantomData, py)
    }
}

impl<'a, 'py, T> From<&'a Bound<'py, T>> for Borrowed<'a, 'py, T> {
    /// Create borrow on a Bound
    fn from(instance: &'a Bound<'py, T>) -> Self {
        instance.as_borrowed()
    }
}

impl<'py, T> Borrowed<'py, 'py, T>
where
    T: HasPyGilRef,
{
    pub(crate) fn into_gil_ref(self) -> &'py T::AsRefTarget {
        // Safety: self is a borrow over `'py`.
        unsafe { self.py().from_borrowed_ptr(self.0.as_ptr()) }
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
        unsafe { &*(&self.0 as *const _ as *const Bound<'py, T>) }
    }
}

impl<T> Clone for Borrowed<'_, '_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Borrowed<'_, '_, T> {}

impl<T> ToPyObject for Borrowed<'_, '_, T> {
    /// Converts `Py` instance -> PyObject.
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        (*self).into_py(py)
    }
}

impl<T> IntoPy<PyObject> for Borrowed<'_, '_, T> {
    /// Converts `Py` instance -> PyObject.
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_owned().into_py(py)
    }
}

/// A GIL-independent reference to an object allocated on the Python heap.
///
/// This type does not auto-dereference to the inner object because you must prove you hold the GIL to access it.
/// Instead, call one of its methods to access the inner object:
///  - [`Py::as_ref`], to borrow a GIL-bound reference to the contained object.
///  - [`Py::borrow`], [`Py::try_borrow`], [`Py::borrow_mut`], or [`Py::try_borrow_mut`],
/// to get a (mutable) reference to a contained pyclass, using a scheme similar to std's [`RefCell`].
/// See the [`PyCell` guide entry](https://pyo3.rs/latest/class.html#pycell-and-interior-mutability)
/// for more information.
///  - You can call methods directly on `Py` with [`Py::call`], [`Py::call_method`] and friends.
/// These require passing in the [`Python<'py>`](crate::Python) token but are otherwise similar to the corresponding
/// methods on [`PyAny`].
///
/// # Example: Storing Python objects in structs
///
/// As all the native Python objects only appear as references, storing them in structs doesn't work well.
/// For example, this won't compile:
///
/// ```compile_fail
/// # use pyo3::prelude::*;
/// # use pyo3::types::PyDict;
/// #
/// #[pyclass]
/// struct Foo<'py> {
///     inner: &'py PyDict,
/// }
///
/// impl Foo {
///     fn new() -> Foo {
///         let foo = Python::with_gil(|py| {
///             // `py` will only last for this scope.
///
///             // `&PyDict` derives its lifetime from `py` and
///             // so won't be able to outlive this closure.
///             let dict: &PyDict = PyDict::new(py);
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
///         Python::with_gil(|py| {
///             let dict: Py<PyDict> = PyDict::new(py).into();
///             Foo { inner: dict }
///         })
///     }
/// }
/// #
/// # fn main() -> PyResult<()> {
/// #     Python::with_gil(|py| {
/// #         let m = pyo3::types::PyModule::new(py, "test")?;
/// #         m.add_class::<Foo>()?;
/// #
/// #         let foo: &PyCell<Foo> = m.getattr("Foo")?.call0()?.downcast()?;
/// #         let dict = &foo.borrow().inner;
/// #         let dict: &PyDict = dict.as_ref(py);
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
///         Python::with_gil(|py| {
///             let bar: Py<Bar> = Py::new(py, Bar {})?;
///             Ok(Foo { inner: bar })
///         })
///     }
/// }
/// #
/// # fn main() -> PyResult<()> {
/// #     Python::with_gil(|py| {
/// #         let m = pyo3::types::PyModule::new(py, "test")?;
/// #         m.add_class::<Foo>()?;
/// #
/// #         let foo: &PyCell<Foo> = m.getattr("Foo")?.call0()?.downcast()?;
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
/// Python::with_gil(|py| {
///     let first: Py<PyDict> = PyDict::new(py).into();
///
///     // All of these are valid syntax
///     let second = Py::clone_ref(&first, py);
///     let third = first.clone_ref(py);
///     let fourth = Py::clone(&first);
///     let fifth = first.clone();
///
///     // Disposing of our original `Py<PyDict>` just decrements the reference count.
///     drop(first);
///
///     // They all point to the same object
///     assert!(second.is(&third));
///     assert!(fourth.is(&fifth));
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
/// # A note on `Send` and `Sync`
///
/// Accessing this object is threadsafe, since any access to its API requires a [`Python<'py>`](crate::Python) token.
/// As you can only get this by acquiring the GIL, `Py<...>` "implements [`Send`] and [`Sync`].
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
    /// Python::with_gil(|py| -> PyResult<Py<Foo>> {
    ///     let foo: Py<Foo> = Py::new(py, Foo {})?;
    ///     Ok(foo)
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(py: Python<'_>, value: impl Into<PyClassInitializer<T>>) -> PyResult<Py<T>> {
        let initializer = value.into();
        let obj = initializer.create_cell(py)?;
        let ob = unsafe { Py::from_owned_ptr(py, obj as _) };
        Ok(ob)
    }
}

impl<T> Py<T>
where
    T: HasPyGilRef,
{
    /// Borrows a GIL-bound reference to the contained `T`.
    ///
    /// By binding to the GIL lifetime, this allows the GIL-bound reference to not require
    /// [`Python<'py>`](crate::Python) for any of its methods, which makes calling methods
    /// on it more ergonomic.
    ///
    /// For native types, this reference is `&T`. For pyclasses, this is `&PyCell<T>`.
    ///
    /// Note that the lifetime of the returned reference is the shortest of `&self` and
    /// [`Python<'py>`](crate::Python).
    /// Consider using [`Py::into_ref`] instead if this poses a problem.
    ///
    /// # Examples
    ///
    /// Get access to `&PyList` from `Py<PyList>`:
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// # use pyo3::types::PyList;
    /// #
    /// Python::with_gil(|py| {
    ///     let list: Py<PyList> = PyList::empty_bound(py).into();
    ///     let list: &PyList = list.as_ref(py);
    ///     assert_eq!(list.len(), 0);
    /// });
    /// ```
    ///
    /// Get access to `&PyCell<MyClass>` from `Py<MyClass>`:
    ///
    /// ```
    /// # use pyo3::prelude::*;
    /// #
    /// #[pyclass]
    /// struct MyClass {}
    ///
    /// Python::with_gil(|py| {
    ///     let my_class: Py<MyClass> = Py::new(py, MyClass {}).unwrap();
    ///     let my_class_cell: &PyCell<MyClass> = my_class.as_ref(py);
    ///     assert!(my_class_cell.try_borrow().is_ok());
    /// });
    /// ```
    pub fn as_ref<'py>(&'py self, _py: Python<'py>) -> &'py T::AsRefTarget {
        let any = self.as_ptr() as *const PyAny;
        unsafe { PyNativeType::unchecked_downcast(&*any) }
    }

    /// Borrows a GIL-bound reference to the contained `T` independently of the lifetime of `T`.
    ///
    /// This method is similar to [`as_ref`](#method.as_ref) but consumes `self` and registers the
    /// Python object reference in PyO3's object storage. The reference count for the Python
    /// object will not be decreased until the GIL lifetime ends.
    ///
    /// You should prefer using [`as_ref`](#method.as_ref) if you can as it'll have less overhead.
    ///
    /// # Examples
    ///
    /// [`Py::as_ref`]'s lifetime limitation forbids creating a function that references a
    /// variable created inside the function.
    ///
    /// ```rust,compile_fail
    /// # use pyo3::prelude::*;
    /// #
    /// fn new_py_any<'py>(py: Python<'py>, value: impl IntoPy<Py<PyAny>>) -> &'py PyAny {
    ///     let obj: Py<PyAny> = value.into_py(py);
    ///
    ///     // The lifetime of the return value of this function is the shortest
    ///     // of `obj` and `py`. As `obj` is owned by the current function,
    ///     // Rust won't let the return value escape this function!
    ///     obj.as_ref(py)
    /// }
    /// ```
    ///
    /// This can be solved by using [`Py::into_ref`] instead, which does not suffer from this issue.
    /// Note that the lifetime of the [`Python<'py>`](crate::Python) token is transferred to
    /// the returned reference.
    ///
    /// ```rust
    /// # use pyo3::prelude::*;
    /// # #[allow(dead_code)] // This is just to show it compiles.
    /// fn new_py_any<'py>(py: Python<'py>, value: impl IntoPy<Py<PyAny>>) -> &'py PyAny {
    ///     let obj: Py<PyAny> = value.into_py(py);
    ///
    ///     // This reference's lifetime is determined by `py`'s lifetime.
    ///     // Because that originates from outside this function,
    ///     // this return value is allowed.
    ///     obj.into_ref(py)
    /// }
    /// ```
    pub fn into_ref(self, py: Python<'_>) -> &T::AsRefTarget {
        unsafe { py.from_owned_ptr(self.into_ptr()) }
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
        unsafe { &*(self as *const Self).cast::<Py<PyAny>>() }
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
    /// Equivalent to `self.as_ref(py).borrow()` -
    /// see [`PyCell::borrow`](crate::pycell::PyCell::borrow).
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
    /// Python::with_gil(|py| -> PyResult<()> {
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
    pub fn borrow<'py>(&'py self, py: Python<'py>) -> PyRef<'py, T> {
        self.as_ref(py).borrow()
    }

    /// Mutably borrows the value `T`.
    ///
    /// This borrow lasts while the returned [`PyRefMut`] exists.
    ///
    /// Equivalent to `self.as_ref(py).borrow_mut()` -
    /// see [`PyCell::borrow_mut`](crate::pycell::PyCell::borrow_mut).
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
    /// Python::with_gil(|py| -> PyResult<()> {
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
    pub fn borrow_mut<'py>(&'py self, py: Python<'py>) -> PyRefMut<'py, T>
    where
        T: PyClass<Frozen = False>,
    {
        self.as_ref(py).borrow_mut()
    }

    /// Attempts to immutably borrow the value `T`, returning an error if the value is currently mutably borrowed.
    ///
    /// The borrow lasts while the returned [`PyRef`] exists.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// For frozen classes, the simpler [`get`][Self::get] is available.
    ///
    /// Equivalent to `self.as_ref(py).borrow_mut()` -
    /// see [`PyCell::try_borrow`](crate::pycell::PyCell::try_borrow).
    pub fn try_borrow<'py>(&'py self, py: Python<'py>) -> Result<PyRef<'py, T>, PyBorrowError> {
        self.as_ref(py).try_borrow()
    }

    /// Attempts to mutably borrow the value `T`, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts while the returned [`PyRefMut`] exists.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    ///
    /// Equivalent to `self.as_ref(py).try_borrow_mut()` -
    /// see [`PyCell::try_borrow_mut`](crate::pycell::PyCell::try_borrow_mut).
    pub fn try_borrow_mut<'py>(
        &'py self,
        py: Python<'py>,
    ) -> Result<PyRefMut<'py, T>, PyBorrowMutError>
    where
        T: PyClass<Frozen = False>,
    {
        self.as_ref(py).try_borrow_mut()
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
    /// let cell  = Python::with_gil(|py| {
    ///     let counter = FrozenCounter { value: AtomicUsize::new(0) };
    ///
    ///     Py::new(py, counter).unwrap()
    /// });
    ///
    /// cell.get().value.fetch_add(1, Ordering::Relaxed);
    /// ```
    pub fn get(&self) -> &T
    where
        T: PyClass<Frozen = True> + Sync,
    {
        let any = self.as_ptr() as *const PyAny;
        // SAFETY: The class itself is frozen and `Sync` and we do not access anything but `cell.contents.value`.
        unsafe {
            let cell: &PyCell<T> = PyNativeType::unchecked_downcast(&*any);
            &*cell.get_ptr()
        }
    }
}

impl<T> Py<T> {
    /// Attaches this `Py` to the given Python context, allowing access to further Python APIs.
    #[inline]
    pub fn bind<'py>(&self, _py: Python<'py>) -> &Bound<'py, T> {
        // Safety: `Bound` has the same layout as `Py`
        unsafe { &*(self as *const Py<T>).cast() }
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
    /// the equality of two objects (the `==` operator), use [`eq`](PyAny::eq).
    ///
    /// This is equivalent to the Python expression `self is other`.
    #[inline]
    pub fn is<U: AsPyPointer>(&self, o: &U) -> bool {
        self.as_ptr() == o.as_ptr()
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
    /// Python::with_gil(|py| {
    ///     let first: Py<PyDict> = PyDict::new(py).into();
    ///     let second = Py::clone_ref(&first, py);
    ///
    ///     // Both point to the same object
    ///     assert!(first.is(&second));
    /// });
    /// # }
    /// ```
    #[inline]
    pub fn clone_ref(&self, py: Python<'_>) -> Py<T> {
        unsafe { Py::from_borrowed_ptr(py, self.0.as_ptr()) }
    }

    /// Returns whether the object is considered to be None.
    ///
    /// This is equivalent to the Python expression `self is None`.
    pub fn is_none(&self, _py: Python<'_>) -> bool {
        unsafe { ffi::Py_None() == self.as_ptr() }
    }

    /// Returns whether the object is Ellipsis, e.g. `...`.
    ///
    /// This is equivalent to the Python expression `self is ...`.
    #[deprecated(since = "0.20.0", note = "use `.is(py.Ellipsis())` instead")]
    pub fn is_ellipsis(&self) -> bool {
        unsafe { ffi::Py_Ellipsis() == self.as_ptr() }
    }

    /// Returns whether the object is considered to be true.
    ///
    /// This is equivalent to the Python expression `bool(self)`.
    #[deprecated(since = "0.21.0", note = "use `.is_truthy()` instead")]
    pub fn is_true(&self, py: Python<'_>) -> PyResult<bool> {
        self.is_truthy(py)
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
    pub fn extract<'p, D>(&'p self, py: Python<'p>) -> PyResult<D>
    where
        D: FromPyObject<'p>,
    {
        FromPyObject::extract(unsafe { py.from_borrowed_ptr(self.as_ptr()) })
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
    /// # use pyo3::{intern, pyfunction, types::PyModule, IntoPy, Py, Python, PyObject, PyResult};
    /// #
    /// #[pyfunction]
    /// fn version(sys: Py<PyModule>, py: Python<'_>) -> PyResult<PyObject> {
    ///     sys.getattr(py, intern!(py, "version"))
    /// }
    /// #
    /// # Python::with_gil(|py| {
    /// #    let sys = py.import("sys").unwrap().into_py(py);
    /// #    version(sys, py).unwrap();
    /// # });
    /// ```
    pub fn getattr<N>(&self, py: Python<'_>, attr_name: N) -> PyResult<PyObject>
    where
        N: IntoPy<Py<PyString>>,
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
    /// # use pyo3::{intern, pyfunction, types::PyModule, IntoPy, PyObject, Python, PyResult};
    /// #
    /// #[pyfunction]
    /// fn set_answer(ob: PyObject, py: Python<'_>) -> PyResult<()> {
    ///     ob.setattr(py, intern!(py, "answer"), 42)
    /// }
    /// #
    /// # Python::with_gil(|py| {
    /// #    let ob = PyModule::new(py, "empty").unwrap().into_py(py);
    /// #    set_answer(ob, py).unwrap();
    /// # });
    /// ```
    pub fn setattr<N, V>(&self, py: Python<'_>, attr_name: N, value: V) -> PyResult<()>
    where
        N: IntoPy<Py<PyString>>,
        V: IntoPy<Py<PyAny>>,
    {
        self.bind(py)
            .as_any()
            .setattr(attr_name, value.into_py(py).into_bound(py))
    }

    /// Deprecated form of [`call_bound`][Py::call_bound].
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`call` will be replaced by `call_bound` in a future PyO3 version"
        )
    )]
    #[inline]
    pub fn call<A>(&self, py: Python<'_>, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject>
    where
        A: IntoPy<Py<PyTuple>>,
    {
        self.call_bound(py, args, kwargs.map(PyDict::as_borrowed).as_deref())
    }

    /// Calls the object.
    ///
    /// This is equivalent to the Python expression `self(*args, **kwargs)`.
    pub fn call_bound(
        &self,
        py: Python<'_>,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyObject> {
        self.bind(py).as_any().call(args, kwargs).map(Bound::unbind)
    }

    /// Calls the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self(*args)`.
    pub fn call1(&self, py: Python<'_>, args: impl IntoPy<Py<PyTuple>>) -> PyResult<PyObject> {
        self.bind(py).as_any().call1(args).map(Bound::unbind)
    }

    /// Calls the object without arguments.
    ///
    /// This is equivalent to the Python expression `self()`.
    pub fn call0(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.bind(py).as_any().call0().map(Bound::unbind)
    }

    /// Deprecated form of [`call_method_bound`][Py::call_method_bound].
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`call_method` will be replaced by `call_method_bound` in a future PyO3 version"
        )
    )]
    #[inline]
    pub fn call_method<N, A>(
        &self,
        py: Python<'_>,
        name: N,
        args: A,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject>
    where
        N: IntoPy<Py<PyString>>,
        A: IntoPy<Py<PyTuple>>,
    {
        self.call_method_bound(py, name, args, kwargs.map(PyDict::as_borrowed).as_deref())
    }

    /// Calls a method on the object.
    ///
    /// This is equivalent to the Python expression `self.name(*args, **kwargs)`.
    ///
    /// To avoid repeated temporary allocations of Python strings, the [`intern!`](crate::intern)
    /// macro can be used to intern `name`.
    pub fn call_method_bound<N, A>(
        &self,
        py: Python<'_>,
        name: N,
        args: A,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyObject>
    where
        N: IntoPy<Py<PyString>>,
        A: IntoPy<Py<PyTuple>>,
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
    pub fn call_method1<N, A>(&self, py: Python<'_>, name: N, args: A) -> PyResult<PyObject>
    where
        N: IntoPy<Py<PyString>>,
        A: IntoPy<Py<PyTuple>>,
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
    pub fn call_method0<N>(&self, py: Python<'_>, name: N) -> PyResult<PyObject>
    where
        N: IntoPy<Py<PyString>>,
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

    /// Create a `Py<T>` instance by creating a new reference from the given FFI pointer.
    ///
    /// # Safety
    /// `ptr` must be a pointer to a Python object of type T.
    ///
    /// # Panics
    /// Panics if `ptr` is null.
    #[inline]
    pub unsafe fn from_borrowed_ptr(py: Python<'_>, ptr: *mut ffi::PyObject) -> Py<T> {
        match Self::from_borrowed_ptr_or_opt(py, ptr) {
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
        Self::from_borrowed_ptr_or_opt(py, ptr).ok_or_else(|| PyErr::fetch(py))
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
        NonNull::new(ptr).map(|nonnull_ptr| {
            ffi::Py_INCREF(ptr);
            Py(nonnull_ptr, PhantomData)
        })
    }

    /// For internal conversions.
    ///
    /// # Safety
    /// `ptr` must point to a Python object of type T.
    unsafe fn from_non_null(ptr: NonNull<ffi::PyObject>) -> Self {
        Self(ptr, PhantomData)
    }
}

impl<T> ToPyObject for Py<T> {
    /// Converts `Py` instance -> PyObject.
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone_ref(py).into_any()
    }
}

impl<T> IntoPy<PyObject> for Py<T> {
    /// Converts a `Py` instance to `PyObject`.
    /// Consumes `self` without calling `Py_DECREF()`.
    #[inline]
    fn into_py(self, _py: Python<'_>) -> PyObject {
        self.into_any()
    }
}

impl<T> IntoPy<PyObject> for &'_ Py<T> {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

impl<T> ToPyObject for Bound<'_, T> {
    /// Converts `&Bound` instance -> PyObject, increasing the reference count.
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone().into_py(py)
    }
}

impl<T> IntoPy<PyObject> for Bound<'_, T> {
    /// Converts a `Bound` instance to `PyObject`.
    #[inline]
    fn into_py(self, _py: Python<'_>) -> PyObject {
        self.into_any().unbind()
    }
}

impl<T> IntoPy<PyObject> for &Bound<'_, T> {
    /// Converts `&Bound` instance -> PyObject, increasing the reference count.
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

unsafe impl<T> crate::AsPyPointer for Py<T> {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0.as_ptr()
    }
}

impl<T> std::convert::From<&'_ T> for PyObject
where
    T: PyNativeType,
{
    #[inline]
    fn from(obj: &T) -> Self {
        obj.as_borrowed().to_owned().into_any().unbind()
    }
}

impl<T> std::convert::From<Py<T>> for PyObject
where
    T: AsRef<PyAny>,
{
    #[inline]
    fn from(other: Py<T>) -> Self {
        other.into_any()
    }
}

impl<T> std::convert::From<Bound<'_, T>> for PyObject
where
    T: AsRef<PyAny>,
{
    #[inline]
    fn from(other: Bound<'_, T>) -> Self {
        let py = other.py();
        other.into_py(py)
    }
}

impl<T> std::convert::From<Bound<'_, T>> for Py<T> {
    #[inline]
    fn from(other: Bound<'_, T>) -> Self {
        other.unbind()
    }
}

// `&PyCell<T>` can be converted to `Py<T>`
impl<T> std::convert::From<&PyCell<T>> for Py<T>
where
    T: PyClass,
{
    fn from(cell: &PyCell<T>) -> Self {
        cell.as_borrowed().to_owned().unbind()
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
/// Otherwise this registers the [`Py`]`<T>` instance to have its reference count
/// incremented the next time PyO3 acquires the GIL.
impl<T> Clone for Py<T> {
    fn clone(&self) -> Self {
        unsafe {
            gil::register_incref(self.0);
        }
        Self(self.0, PhantomData)
    }
}

/// Dropping a `Py` instance decrements the reference count on the object by 1.
impl<T> Drop for Py<T> {
    fn drop(&mut self) {
        unsafe {
            gil::register_decref(self.0);
        }
    }
}

impl<T> FromPyObject<'_> for Py<T>
where
    T: PyTypeCheck,
{
    /// Extracts `Self` from the source `PyObject`.
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        ob.extract::<Bound<'_, T>>().map(Bound::unbind)
    }
}

impl<'py, T> FromPyObject<'py> for Bound<'py, T>
where
    T: PyTypeCheck,
{
    /// Extracts `Self` from the source `PyObject`.
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        ob.downcast().map(Clone::clone).map_err(Into::into)
    }
}

/// `Py<T>` can be used as an error when T is an Error.
///
/// However for GIL lifetime reasons, cause() cannot be implemented for `Py<T>`.
/// Use .as_ref() to get the GIL-scoped error if you need to inspect the cause.
impl<T> std::error::Error for Py<T>
where
    T: std::error::Error + PyTypeInfo,
    T::AsRefTarget: std::fmt::Display,
{
}

impl<T> std::fmt::Display for Py<T>
where
    T: PyTypeInfo,
    T::AsRefTarget: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Python::with_gil(|py| std::fmt::Display::fmt(self.as_ref(py), f))
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
pub type PyObject = Py<PyAny>;

impl PyObject {
    /// Downcast this `PyObject` to a concrete Python type or pyclass.
    ///
    /// Note that you can often avoid downcasting yourself by just specifying
    /// the desired type in function or method signatures.
    /// However, manual downcasting is sometimes necessary.
    ///
    /// For extracting a Rust-only type, see [`Py::extract`](struct.Py.html#method.extract).
    ///
    /// # Example: Downcasting to a specific Python object
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::{PyDict, PyList};
    ///
    /// Python::with_gil(|py| {
    ///     let any: PyObject = PyDict::new(py).into();
    ///
    ///     assert!(any.downcast::<PyDict>(py).is_ok());
    ///     assert!(any.downcast::<PyList>(py).is_err());
    /// });
    /// ```
    ///
    /// # Example: Getting a reference to a pyclass
    ///
    /// This is useful if you want to mutate a `PyObject` that
    /// might actually be a pyclass.
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
    /// Python::with_gil(|py| {
    ///     let class: PyObject = Py::new(py, Class { i: 0 }).unwrap().into_py(py);
    ///
    ///     let class_cell: &PyCell<Class> = class.downcast(py)?;
    ///
    ///     class_cell.borrow_mut().i += 1;
    ///
    ///     // Alternatively you can get a `PyRefMut` directly
    ///     let class_ref: PyRefMut<'_, Class> = class.extract(py)?;
    ///     assert_eq!(class_ref.i, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    #[inline]
    pub fn downcast<'py, T>(&'py self, py: Python<'py>) -> Result<&'py T, PyDowncastError<'py>>
    where
        T: PyTypeCheck<AsRefTarget = T>,
    {
        self.as_ref(py).downcast()
    }

    /// Casts the PyObject to a concrete Python object type without checking validity.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    #[inline]
    pub unsafe fn downcast_unchecked<'p, T>(&'p self, py: Python<'p>) -> &T
    where
        T: HasPyGilRef<AsRefTarget = T>,
    {
        self.as_ref(py).downcast_unchecked()
    }
}

#[cfg(test)]
#[cfg_attr(not(feature = "gil-refs"), allow(deprecated))]
mod tests {
    use super::{Bound, Py, PyObject};
    use crate::types::{dict::IntoPyDict, PyDict, PyString};
    use crate::{PyAny, PyNativeType, PyResult, Python, ToPyObject};

    #[test]
    fn test_call() {
        Python::with_gil(|py| {
            let obj = py.get_type::<PyDict>().to_object(py);

            let assert_repr = |obj: &PyAny, expected: &str| {
                assert_eq!(obj.repr().unwrap().to_str().unwrap(), expected);
            };

            assert_repr(obj.call0(py).unwrap().as_ref(py), "{}");
            assert_repr(obj.call1(py, ()).unwrap().as_ref(py), "{}");
            assert_repr(obj.call(py, (), None).unwrap().as_ref(py), "{}");

            assert_repr(
                obj.call1(py, ((('x', 1),),)).unwrap().as_ref(py),
                "{'x': 1}",
            );
            assert_repr(
                obj.call(py, (), Some([('x', 1)].into_py_dict(py)))
                    .unwrap()
                    .as_ref(py),
                "{'x': 1}",
            );
        })
    }

    #[test]
    fn test_call_for_non_existing_method() {
        Python::with_gil(|py| {
            let obj: PyObject = PyDict::new(py).into();
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
        let dict: Py<PyDict> = Python::with_gil(|py| {
            let native = PyDict::new(py);
            Py::from(native)
        });

        assert_eq!(Python::with_gil(|py| dict.get_refcnt(py)), 1);
    }

    #[test]
    fn pyobject_from_py() {
        Python::with_gil(|py| {
            let dict: Py<PyDict> = PyDict::new(py).into();
            let cnt = dict.get_refcnt(py);
            let p: PyObject = dict.into();
            assert_eq!(p.get_refcnt(py), cnt);
        });
    }

    #[test]
    fn attr() -> PyResult<()> {
        use crate::types::PyModule;

        Python::with_gil(|py| {
            const CODE: &str = r#"
class A:
    pass
a = A()
   "#;
            let module = PyModule::from_code(py, CODE, "", "")?;
            let instance: Py<PyAny> = module.getattr("a")?.into();

            instance.getattr(py, "foo").unwrap_err();

            instance.setattr(py, "foo", "bar")?;

            assert!(instance
                .getattr(py, "foo")?
                .as_ref(py)
                .eq(PyString::new_bound(py, "bar"))?);

            instance.getattr(py, "foo")?;
            Ok(())
        })
    }

    #[test]
    fn pystring_attr() -> PyResult<()> {
        use crate::types::PyModule;

        Python::with_gil(|py| {
            const CODE: &str = r#"
class A:
    pass
a = A()
   "#;
            let module = PyModule::from_code(py, CODE, "", "")?;
            let instance: Py<PyAny> = module.getattr("a")?.into();

            let foo = crate::intern!(py, "foo");
            let bar = crate::intern!(py, "bar");

            instance.getattr(py, foo).unwrap_err();
            instance.setattr(py, foo, bar)?;
            assert!(instance.getattr(py, foo)?.as_ref(py).eq(bar)?);
            Ok(())
        })
    }

    #[test]
    fn invalid_attr() -> PyResult<()> {
        Python::with_gil(|py| {
            let instance: Py<PyAny> = py.eval("object()", None, None)?.into();

            instance.getattr(py, "foo").unwrap_err();

            // Cannot assign arbitrary attributes to `object`
            instance.setattr(py, "foo", "bar").unwrap_err();
            Ok(())
        })
    }

    #[test]
    fn test_py2_from_py_object() {
        Python::with_gil(|py| {
            let instance: &PyAny = py.eval("object()", None, None).unwrap();
            let ptr = instance.as_ptr();
            let instance: Bound<'_, PyAny> = instance.extract().unwrap();
            assert_eq!(instance.as_ptr(), ptr);
        })
    }

    #[test]
    fn test_py2_into_py_object() {
        Python::with_gil(|py| {
            let instance = py
                .eval("object()", None, None)
                .unwrap()
                .as_borrowed()
                .to_owned();
            let ptr = instance.as_ptr();
            let instance: PyObject = instance.clone().unbind();
            assert_eq!(instance.as_ptr(), ptr);
        })
    }

    #[test]
    #[allow(deprecated)]
    fn test_is_ellipsis() {
        Python::with_gil(|py| {
            let v = py
                .eval("...", None, None)
                .map_err(|e| e.display(py))
                .unwrap()
                .to_object(py);

            assert!(v.is_ellipsis());

            let not_ellipsis = 5.to_object(py);
            assert!(!not_ellipsis.is_ellipsis());
        });
    }

    #[test]
    fn test_debug_fmt() {
        Python::with_gil(|py| {
            let obj = "hello world".to_object(py).into_bound(py);
            assert_eq!(format!("{:?}", obj), "'hello world'");
        });
    }

    #[test]
    fn test_display_fmt() {
        Python::with_gil(|py| {
            let obj = "hello world".to_object(py).into_bound(py);
            assert_eq!(format!("{}", obj), "hello world");
        });
    }

    #[test]
    fn test_bound_as_any() {
        Python::with_gil(|py| {
            let obj = PyString::new_bound(py, "hello world");
            let any = obj.as_any();
            assert_eq!(any.as_ptr(), obj.as_ptr());
        });
    }

    #[test]
    fn test_bound_into_any() {
        Python::with_gil(|py| {
            let obj = PyString::new_bound(py, "hello world");
            let any = obj.clone().into_any();
            assert_eq!(any.as_ptr(), obj.as_ptr());
        });
    }

    #[cfg(feature = "macros")]
    mod using_macros {
        use crate::PyCell;

        use super::*;

        #[crate::pyclass]
        #[pyo3(crate = "crate")]
        struct SomeClass(i32);

        #[test]
        fn instance_borrow_methods() {
            // More detailed tests of the underlying semantics in pycell.rs
            Python::with_gil(|py| {
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
        #[allow(deprecated)]
        fn cell_tryfrom() {
            use crate::PyTryInto;
            // More detailed tests of the underlying semantics in pycell.rs
            Python::with_gil(|py| {
                let instance: &PyAny = Py::new(py, SomeClass(0)).unwrap().into_ref(py);
                let _: &PyCell<SomeClass> = PyTryInto::try_into(instance).unwrap();
                let _: &PyCell<SomeClass> = PyTryInto::try_into_exact(instance).unwrap();
            })
        }
    }
}
