// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::conversion::{PyTryFrom, ToBorrowedObject};
use crate::err::{self, PyDowncastError, PyErr, PyResult};
use crate::gil;
use crate::pycell::{PyBorrowError, PyBorrowMutError, PyCell};
use crate::types::{PyDict, PyTuple};
use crate::{
    ffi, AsPyPointer, FromPyObject, IntoPy, IntoPyPointer, PyAny, PyClass, PyClassInitializer,
    PyRef, PyRefMut, PyTypeInfo, Python, ToPyObject,
};
use std::marker::PhantomData;
use std::mem;
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
/// #         let foo: &PyCell<Foo> = pyo3::PyTryFrom::try_from(m.getattr("Foo")?.call0()?)?;
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
/// #         let foo: &PyCell<Foo> = pyo3::PyTryFrom::try_from(m.getattr("Foo")?.call0()?)?;
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
    T: PyTypeInfo,
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
    ///     let list: Py<PyList> = PyList::empty(py).into();
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

impl<T> Py<T>
where
    T: PyClass,
{
    /// Immutably borrows the value `T`.
    ///
    /// This borrow lasts while the returned [`PyRef`] exists.
    /// Multiple immutable borrows can be taken out at the same time.
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
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    pub fn borrow_mut<'py>(&'py self, py: Python<'py>) -> PyRefMut<'py, T> {
        self.as_ref(py).borrow_mut()
    }

    /// Attempts to immutably borrow the value `T`, returning an error if the value is currently mutably borrowed.
    ///
    /// The borrow lasts while the returned [`PyRef`] exists.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
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
    ) -> Result<PyRefMut<'py, T>, PyBorrowMutError> {
        self.as_ref(py).try_borrow_mut()
    }
}

impl<T> Py<T> {
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

    /// Returns whether the object is considered to be true.
    ///
    /// This is equivalent to the Python expression `bool(self)`.
    pub fn is_true(&self, py: Python<'_>) -> PyResult<bool> {
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
    pub fn getattr<N>(&self, py: Python<'_>, attr_name: N) -> PyResult<PyObject>
    where
        N: ToPyObject,
    {
        attr_name.with_borrowed_ptr(py, |attr_name| unsafe {
            PyObject::from_owned_ptr_or_err(py, ffi::PyObject_GetAttr(self.as_ptr(), attr_name))
        })
    }

    /// Sets an attribute value.
    ///
    /// This is equivalent to the Python expression `self.attr_name = value`.
    pub fn setattr<N, V>(&self, py: Python<'_>, attr_name: N, value: V) -> PyResult<()>
    where
        N: ToPyObject,
        V: ToPyObject,
    {
        attr_name.with_borrowed_ptr(py, move |attr_name| {
            value.with_borrowed_ptr(py, |value| unsafe {
                err::error_on_minusone(py, ffi::PyObject_SetAttr(self.as_ptr(), attr_name, value))
            })
        })
    }

    /// Calls the object.
    ///
    /// This is equivalent to the Python expression `self(*args, **kwargs)`.
    pub fn call(
        &self,
        py: Python<'_>,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let args = args.into_py(py).into_ptr();
        let kwargs = kwargs.into_ptr();
        let result = unsafe {
            PyObject::from_owned_ptr_or_err(py, ffi::PyObject_Call(self.as_ptr(), args, kwargs))
        };
        unsafe {
            ffi::Py_XDECREF(args);
            ffi::Py_XDECREF(kwargs);
        }
        result
    }

    /// Calls the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self(*args)`.
    pub fn call1(&self, py: Python<'_>, args: impl IntoPy<Py<PyTuple>>) -> PyResult<PyObject> {
        self.call(py, args, None)
    }

    /// Calls the object without arguments.
    ///
    /// This is equivalent to the Python expression `self()`.
    pub fn call0(&self, py: Python<'_>) -> PyResult<PyObject> {
        cfg_if::cfg_if! {
            if #[cfg(all(Py_3_9, not(PyPy)))] {
                // Optimized path on python 3.9+
                unsafe {
                    PyObject::from_owned_ptr_or_err(py, ffi::PyObject_CallNoArgs(self.as_ptr()))
                }
            } else {
                self.call(py, (), None)
            }
        }
    }

    /// Calls a method on the object.
    ///
    /// This is equivalent to the Python expression `self.name(*args, **kwargs)`.
    pub fn call_method(
        &self,
        py: Python<'_>,
        name: &str,
        args: impl IntoPy<Py<PyTuple>>,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        name.with_borrowed_ptr(py, |name| unsafe {
            let args = args.into_py(py).into_ptr();
            let kwargs = kwargs.into_ptr();
            let ptr = ffi::PyObject_GetAttr(self.as_ptr(), name);
            if ptr.is_null() {
                return Err(PyErr::fetch(py));
            }
            let result = PyObject::from_owned_ptr_or_err(py, ffi::PyObject_Call(ptr, args, kwargs));
            ffi::Py_DECREF(ptr);
            ffi::Py_XDECREF(args);
            ffi::Py_XDECREF(kwargs);
            result
        })
    }

    /// Calls a method on the object with only positional arguments.
    ///
    /// This is equivalent to the Python expression `self.name(*args)`.
    pub fn call_method1(
        &self,
        py: Python<'_>,
        name: &str,
        args: impl IntoPy<Py<PyTuple>>,
    ) -> PyResult<PyObject> {
        self.call_method(py, name, args, None)
    }

    /// Calls a method on the object with no arguments.
    ///
    /// This is equivalent to the Python expression `self.name()`.
    pub fn call_method0(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        cfg_if::cfg_if! {
            if #[cfg(all(Py_3_9, not(any(Py_LIMITED_API, PyPy))))] {
                // Optimized path on python 3.9+
                unsafe {
                    let name = name.into_py(py);
                    PyObject::from_owned_ptr_or_err(py, ffi::PyObject_CallMethodNoArgs(self.as_ptr(), name.as_ptr()))
                }
            } else {
                self.call_method(py, name, (), None)
            }
        }
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
    /// If `ptr` is null then the current Python exception is fetched as a `PyErr`.
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
    #[inline]
    unsafe fn from_non_null(ptr: NonNull<ffi::PyObject>) -> Self {
        Self(ptr, PhantomData)
    }

    /// Returns the inner pointer without decreasing the refcount.
    #[inline]
    fn into_non_null(self) -> NonNull<ffi::PyObject> {
        let pointer = self.0;
        mem::forget(self);
        pointer
    }
}

impl<T> ToPyObject for Py<T> {
    /// Converts `Py` instance -> PyObject.
    fn to_object(&self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

impl<T> IntoPy<PyObject> for Py<T> {
    /// Converts a `Py` instance to `PyObject`.
    /// Consumes `self` without calling `Py_DECREF()`.
    #[inline]
    fn into_py(self, _py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_non_null(self.into_non_null()) }
    }
}

impl<T> AsPyPointer for Py<T> {
    /// Gets the underlying FFI pointer, returns a borrowed pointer.
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        self.0.as_ptr()
    }
}

impl<T> IntoPyPointer for Py<T> {
    /// Gets the underlying FFI pointer, returns a owned pointer.
    #[inline]
    #[must_use]
    fn into_ptr(self) -> *mut ffi::PyObject {
        self.into_non_null().as_ptr()
    }
}

impl<T> std::convert::From<&'_ T> for PyObject
where
    T: AsPyPointer + PyNativeType,
{
    fn from(obj: &T) -> Self {
        unsafe { Py::from_borrowed_ptr(obj.py(), obj.as_ptr()) }
    }
}

impl<T> std::convert::From<Py<T>> for PyObject
where
    T: AsRef<PyAny>,
{
    #[inline]
    fn from(other: Py<T>) -> Self {
        unsafe { Self::from_non_null(other.into_non_null()) }
    }
}

// `&PyCell<T>` can be converted to `Py<T>`
impl<T> std::convert::From<&PyCell<T>> for Py<T>
where
    T: PyClass,
{
    fn from(cell: &PyCell<T>) -> Self {
        unsafe { Py::from_borrowed_ptr(cell.py(), cell.as_ptr()) }
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
    T: PyClass,
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

impl<'a, T> FromPyObject<'a> for Py<T>
where
    T: PyTypeInfo,
    &'a T::AsRefTarget: FromPyObject<'a>,
    T::AsRefTarget: 'a + AsPyPointer,
{
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        unsafe {
            ob.extract::<&T::AsRefTarget>()
                .map(|val| Py::from_borrowed_ptr(ob.py(), val.as_ptr()))
        }
    }
}

/// Py<T> can be used as an error when T is an Error.
///
/// However for GIL lifetime reasons, cause() cannot be implemented for Py<T>.
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
    /// Casts the PyObject to a concrete Python object type.
    ///
    /// This can cast only to native Python types, not types implemented in Rust. For a more
    /// flexible alternative, see [`Py::extract`](struct.Py.html#method.extract).
    pub fn cast_as<'p, D>(&'p self, py: Python<'p>) -> Result<&'p D, PyDowncastError<'_>>
    where
        D: PyTryFrom<'p>,
    {
        <D as PyTryFrom<'_>>::try_from(unsafe { py.from_borrowed_ptr::<PyAny>(self.as_ptr()) })
    }
}

#[cfg(test)]
mod tests {
    use super::{Py, PyObject};
    use crate::types::PyDict;
    use crate::{Python, ToPyObject};

    #[test]
    fn test_call0() {
        Python::with_gil(|py| {
            let obj = py.get_type::<PyDict>().to_object(py);
            assert_eq!(
                obj.call0(py)
                    .unwrap()
                    .as_ref(py)
                    .repr()
                    .unwrap()
                    .to_string_lossy(),
                "{}"
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
}
