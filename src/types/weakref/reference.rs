use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::type_object::{PyTypeCheck, PyTypeInfo};
use crate::types::any::PyAnyMethods;
use crate::{ffi, AsPyPointer, Borrowed, Bound, PyAny, PyNativeType, ToPyObject};

/// Represents a Python `weakref.ReferenceType`.
///
/// In Python this is created by calling `weakref.ref`.
#[repr(transparent)]
pub struct PyWeakRef(PyAny);

pyobject_native_type!(
    PyWeakRef,
    ffi::PyWeakReference,
    pyobject_native_static_type_object!(ffi::_PyWeakref_RefType),
    #module=Some("weakref"),
    #checkfunction=ffi::PyWeakref_CheckRefExact
);

impl PyWeakRef {
    /// Deprecated form of [`PyWeakRef::new_bound`].
    #[inline]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyWeakRef::new` will be replaced by `PyWeakRef::new_bound` in a future PyO3 version"
        )
    )]
    pub fn new<T>(object: &T) -> PyResult<&PyWeakRef>
    where
        T: PyNativeType,
    {
        Self::new_bound(&object.as_borrowed()).map(Bound::into_gil_ref)
    }

    /// Constructs a new Weak Reference (`weakref.ref`/`weakref.ReferenceType`) for the given object.
    ///
    /// Returns a `TypeError` if `object` is not weak referenceable (Most native types and PyClasses without `weakref` flag).
    ///
    /// # Examples
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let foo = Bound::new(py, Foo {})?;
    ///     let weakref = PyWeakRef::new_bound(&foo)?;
    ///     assert!(
    ///         // In normal situations where a direct `Bound<'py, Foo>` is required use `upgrade::<Foo>`
    ///         weakref.get_object()
    ///             .map_or(false, |obj| obj.is(&foo))
    ///     );
    ///
    ///     let weakref2 = PyWeakRef::new_bound(&foo)?;
    ///     assert!(weakref.is(&weakref2));
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.get_object().is_none());
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn new_bound<'py, T>(object: &Bound<'py, T>) -> PyResult<Bound<'py, PyWeakRef>> {
        // TODO: Is this inner pattern still necessary Here?
        fn inner<'py>(object: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyWeakRef>> {
            unsafe {
                Bound::from_owned_ptr_or_err(
                    object.py(),
                    ffi::PyWeakref_NewRef(object.as_ptr(), ffi::Py_None()),
                )
                .downcast_into_unchecked()
            }
        }

        inner(object.as_any())
    }

    /// Deprecated form of [`PyWeakRef::new_bound_with`].
    #[inline]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyWeakRef::new_with` will be replaced by `PyWeakRef::new_bound_with` in a future PyO3 version"
        )
    )]
    pub fn new_with<T, C>(object: &T, callback: C) -> PyResult<&PyWeakRef>
    where
        T: PyNativeType,
        C: ToPyObject,
    {
        Self::new_bound_with(&object.as_borrowed(), callback).map(Bound::into_gil_ref)
    }

    /// Constructs a new Weak Reference (`weakref.ref`/`weakref.ReferenceType`) for the given object with a callback.
    ///
    /// Returns a `TypeError` if `object` is not weak referenceable (Most native types and PyClasses without `weakref` flag) or if the `callback` is not callable or None.
    ///
    /// # Examples
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pyfunction]
    /// fn callback(wref: Bound<'_, PyWeakRef>) -> PyResult<()> {
    ///         let py = wref.py();
    ///         assert!(wref.upgrade::<Foo>()?.is_none());
    ///         py.run_bound("counter = 1", None, None)
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     py.run_bound("counter = 0", None, None)?;
    ///     assert_eq!(py.eval_bound("counter", None, None)?.extract::<u32>()?, 0);
    ///     let foo = Bound::new(py, Foo{})?;
    ///
    ///     // This is fine.
    ///     let weakref = PyWeakRef::new_bound_with(&foo, py.None())?;
    ///     assert!(weakref.upgrade::<Foo>()?.is_some());
    ///     assert!(
    ///         // In normal situations where a direct `Bound<'py, Foo>` is required use `upgrade::<Foo>`
    ///         weakref.get_object()
    ///             .map_or(false, |obj| obj.is(&foo))
    ///     );
    ///     assert_eq!(py.eval_bound("counter", None, None)?.extract::<u32>()?, 0);
    ///
    ///     let weakref2 = PyWeakRef::new_bound_with(&foo, wrap_pyfunction!(callback, py)?)?;
    ///     assert!(!weakref.is(&weakref2)); // Not the same weakref
    ///     assert!(weakref.eq(&weakref2)?);  // But Equal, since they point to the same object
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.upgrade::<Foo>()?.is_none());
    ///     assert_eq!(py.eval_bound("counter", None, None)?.extract::<u32>()?, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn new_bound_with<'py, T, C>(
        object: &Bound<'py, T>,
        callback: C,
    ) -> PyResult<Bound<'py, PyWeakRef>>
    where
        Bound<'py, T>: AsPyPointer,
        C: ToPyObject,
    {
        fn inner<'py>(
            object: &Bound<'py, PyAny>,
            callback: Bound<'py, PyAny>,
        ) -> PyResult<Bound<'py, PyWeakRef>> {
            unsafe {
                Bound::from_owned_ptr_or_err(
                    object.py(),
                    ffi::PyWeakref_NewRef(object.as_ptr(), callback.as_ptr()),
                )
                .downcast_into_unchecked()
            }
        }

        let py = object.py();
        inner(object.as_any(), callback.to_object(py).into_bound(py))
    }

    /// Upgrade the weakref to a direct object reference.
    ///
    /// It is named `upgrade` to be inline with [rust's `Weak::upgrade`](std::rc::Weak::upgrade).
    /// In Python it would be equivalent to [`PyWeakref_GetObject`] or calling the [`weakref.ReferenceType`] (result of calling [`weakref.ref`]).
    ///
    /// # Example
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     fn get_data(&self) -> (&str, u32) {
    ///         ("Dave", 10)
    ///     }
    /// }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakRef>) -> PyResult<String> {
    ///     if let Some(data_src) = reference.upgrade::<Foo>()? {
    ///         let data = data_src.borrow();
    ///         let (name, score) = data.get_data();
    ///         Ok(format!("Processing '{}': score = {}", name, score))
    ///     } else {
    ///         Ok("The supplied data reference is nolonger relavent.".to_owned())
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let data = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakRef::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "Processing 'Dave': score = 10"
    ///     );
    ///
    ///     drop(data);
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The supplied data reference is nolonger relavent."
    ///     );
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics is the current object is invalid.
    /// If used propperly this is never the case. (NonNull and actually a weakref type)
    ///
    /// [`PyWeakref_GetObject`]: https://docs.python.org/3/c-api/weakref.html#c.PyWeakref_GetObject
    /// [`weakref.ReferenceType`]: https://docs.python.org/3/library/weakref.html#weakref.ReferenceType
    /// [`weakref.ref`]: https://docs.python.org/3/library/weakref.html#weakref.ref
    pub fn upgrade<T>(&self) -> PyResult<Option<&T::AsRefTarget>>
    where
        T: PyTypeCheck,
    {
        Ok(self.as_borrowed().upgrade::<T>()?.map(Bound::into_gil_ref))
    }

    /// Upgrade the weakref to an exact direct object reference.
    ///
    /// It is named `upgrade` to be inline with [rust's `Weak::upgrade`](std::rc::Weak::upgrade).
    /// In Python it would be equivalent to [`PyWeakref_GetObject`] or calling the [`weakref.ReferenceType`] (result of calling [`weakref.ref`]).
    ///
    /// # Example
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     fn get_data(&self) -> (&str, u32) {
    ///         ("Dave", 10)
    ///     }
    /// }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakRef>) -> PyResult<String> {
    ///     if let Some(data_src) = reference.upgrade_exact::<Foo>()? {
    ///         let data = data_src.borrow();
    ///         let (name, score) = data.get_data();
    ///         Ok(format!("Processing '{}': score = {}", name, score))
    ///     } else {
    ///         Ok("The supplied data reference is nolonger relavent.".to_owned())
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let data = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakRef::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "Processing 'Dave': score = 10"
    ///     );
    ///
    ///     drop(data);
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The supplied data reference is nolonger relavent."
    ///     );
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics is the current object is invalid.
    /// If used propperly this is never the case. (NonNull and actually a weakref type)
    ///
    /// [`PyWeakref_GetObject`]: https://docs.python.org/3/c-api/weakref.html#c.PyWeakref_GetObject
    /// [`weakref.ReferenceType`]: https://docs.python.org/3/library/weakref.html#weakref.ReferenceType
    /// [`weakref.ref`]: https://docs.python.org/3/library/weakref.html#weakref.ref
    pub fn upgrade_exact<T>(&self) -> PyResult<Option<&T::AsRefTarget>>
    where
        T: PyTypeInfo,
    {
        Ok(self
            .as_borrowed()
            .upgrade_exact::<T>()?
            .map(Bound::into_gil_ref))
    }

    /// Upgrade the weakref to a [`PyAny`] reference to the target if possible.
    ///
    /// This function returns `Some(&'py PyAny)` if the reference still exists, otherwise `None` will be returned.
    ///
    /// This function gets the optional target of this [`weakref.ReferenceType`] (result of calling [`weakref.ref`]).
    /// It produces similair results to calling the `weakref.ReferenceType` or using [`PyWeakref_GetObject`] in the C api.
    ///
    /// # Example
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakRef>) -> PyResult<String> {
    ///     if let Some(object) = reference.get_object() {
    ///         Ok(format!("The object '{}' refered by this reference still exists.", object.getattr("__class__")?.getattr("__qualname__")?))
    ///     } else {
    ///         Ok("The object, which this reference refered to, no longer exists".to_owned())
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let data = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakRef::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The object 'Foo' refered by this reference still exists."
    ///     );
    ///
    ///     drop(data);
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The object, which this reference refered to, no longer exists"
    ///     );
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics is the current object is invalid.
    /// If used propperly this is never the case. (NonNull and actually a weakref type)
    ///
    /// [`PyWeakref_GetObject`]: https://docs.python.org/3/c-api/weakref.html#c.PyWeakref_GetObject
    /// [`weakref.ReferenceType`]: https://docs.python.org/3/library/weakref.html#weakref.ReferenceType
    /// [`weakref.ref`]: https://docs.python.org/3/library/weakref.html#weakref.ref
    pub fn get_object(&self) -> Option<&'_ PyAny> {
        self.as_borrowed().get_object().map(Bound::into_gil_ref)
    }

    /// Retrieve to a object pointed to by the weakref.
    ///
    /// This function returns `&'py PyAny`, which is either the object if it still exists, otherwise it will refer to [`PyNone`](crate::types::none::PyNone).
    ///
    /// This function gets the optional target of this [`weakref.ReferenceType`] (result of calling [`weakref.ref`]).
    /// It produces similair results to calling the `weakref.ReferenceType` or using [`PyWeakref_GetObject`] in the C api.
    ///
    /// # Example
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// fn get_class(reference: Borrowed<'_, '_, PyWeakRef>) -> PyResult<String> {
    ///     reference
    ///         .get_object_raw()
    ///         .getattr("__class__")?
    ///         .repr()?
    ///         .to_str()
    ///         .map(ToOwned::to_owned)
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let object = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakRef::new_bound(&object)?;
    ///
    ///     assert_eq!(
    ///         get_class(reference.as_borrowed())?,
    ///         "<class 'builtins.Foo'>"
    ///     );
    ///
    ///     drop(object);
    ///
    ///     assert_eq!(get_class(reference.as_borrowed())?, "<class 'NoneType'>");
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics is the current object is invalid.
    /// If used propperly this is never the case. (NonNull and actually a weakref type)
    ///
    /// [`PyWeakref_GetObject`]: https://docs.python.org/3/c-api/weakref.html#c.PyWeakref_GetObject
    /// [`weakref.ReferenceType`]: https://docs.python.org/3/library/weakref.html#weakref.ReferenceType
    /// [`weakref.ref`]: https://docs.python.org/3/library/weakref.html#weakref.ref
    pub fn get_object_raw(&self) -> &'_ PyAny {
        self.as_borrowed().get_object_raw().into_gil_ref()
    }
}

/// Implementation of functionality for [`PyWeakRef`].
///
/// These methods are defined for the `Bound<'py, PyWeakRef>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyWeakRef")]
pub trait PyWeakRefMethods<'py> {
    /// Upgrade the weakref to a direct Bound object reference.
    ///
    /// It is named `upgrade` to be inline with [rust's `Weak::upgrade`](std::rc::Weak::upgrade).
    /// In Python it would be equivalent to [`PyWeakref_GetObject`].
    ///
    /// # Example
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     fn get_data(&self) -> (&str, u32) {
    ///         ("Dave", 10)
    ///     }
    /// }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakRef>) -> PyResult<String> {
    ///     if let Some(data_src) = reference.upgrade::<Foo>()? {
    ///         let data = data_src.borrow();
    ///         let (name, score) = data.get_data();
    ///         Ok(format!("Processing '{}': score = {}", name, score))
    ///     } else {
    ///         Ok("The supplied data reference is nolonger relavent.".to_owned())
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let data = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakRef::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "Processing 'Dave': score = 10"
    ///     );
    ///
    ///     drop(data);
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The supplied data reference is nolonger relavent."
    ///     );
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics is the current object is invalid.
    /// If used propperly this is never the case. (NonNull and actually a weakref type)
    ///
    /// [`PyWeakref_GetObject`]: https://docs.python.org/3/c-api/weakref.html#c.PyWeakref_GetObject
    /// [`weakref.ReferenceType`]: https://docs.python.org/3/library/weakref.html#weakref.ReferenceType
    /// [`weakref.ref`]: https://docs.python.org/3/library/weakref.html#weakref.ref
    fn upgrade<T>(&self) -> PyResult<Option<Bound<'py, T>>>
    where
        T: PyTypeCheck,
    {
        Ok(self
            .get_object()
            .map(|obj| obj.downcast_into::<T>())
            .transpose()?)
    }

    // TODO: Is this even possible?
    // fn borrowed_upgrade<T: PyTypeCheck>(&self) -> PyResult<Option<Borrowed<'_, 'py, T>>>;

    /// Upgrade the weakref to a exact direct Bound object reference.
    ///
    /// It is named `upgrade` to be inline with [rust's `Weak::upgrade`](std::rc::Weak::upgrade).
    /// In Python it would be equivalent to [`PyWeakref_GetObject`].
    ///
    /// # Example
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     fn get_data(&self) -> (&str, u32) {
    ///         ("Dave", 10)
    ///     }
    /// }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakRef>) -> PyResult<String> {
    ///     if let Some(data_src) = reference.upgrade_exact::<Foo>()? {
    ///         let data = data_src.borrow();
    ///         let (name, score) = data.get_data();
    ///         Ok(format!("Processing '{}': score = {}", name, score))
    ///     } else {
    ///         Ok("The supplied data reference is nolonger relavent.".to_owned())
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let data = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakRef::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "Processing 'Dave': score = 10"
    ///     );
    ///
    ///     drop(data);
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The supplied data reference is nolonger relavent."
    ///     );
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics is the current object is invalid.
    /// If used propperly this is never the case. (NonNull and actually a weakref type)
    ///
    /// [`PyWeakref_GetObject`]: https://docs.python.org/3/c-api/weakref.html#c.PyWeakref_GetObject
    /// [`weakref.ReferenceType`]: https://docs.python.org/3/library/weakref.html#weakref.ReferenceType
    /// [`weakref.ref`]: https://docs.python.org/3/library/weakref.html#weakref.ref
    fn upgrade_exact<T>(&self) -> PyResult<Option<Bound<'py, T>>>
    where
        T: PyTypeInfo,
    {
        Ok(self
            .get_object()
            .map(|obj| obj.downcast_into_exact::<T>())
            .transpose()?)
    }

    // TODO: NAMING-ALTERNATIVE: upgrade_any
    /// Upgrade the weakref to a Bound [`PyAny`] reference to the target object if possible.
    ///
    /// This function returns `Some(Bound<'py, PyAny>)` if the reference still exists, otherwise `None` will be returned.
    ///
    /// This function gets the optional target of this [`weakref.ReferenceType`] (result of calling [`weakref.ref`]).
    /// It produces similair results to using [`PyWeakref_GetObject`] in the C api.
    ///
    /// # Example
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakRef>) -> PyResult<String> {
    ///     if let Some(object) = reference.get_object() {
    ///         Ok(format!("The object '{}' refered by this reference still exists.", object.getattr("__class__")?.getattr("__qualname__")?))
    ///     } else {
    ///         Ok("The object, which this reference refered to, no longer exists".to_owned())
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let data = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakRef::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The object 'Foo' refered by this reference still exists."
    ///     );
    ///
    ///     drop(data);
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The object, which this reference refered to, no longer exists"
    ///     );
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics is the current object is invalid.
    /// If used propperly this is never the case. (NonNull and actually a weakref type)
    ///
    /// [`PyWeakref_GetObject`]: https://docs.python.org/3/c-api/weakref.html#c.PyWeakref_GetObject
    /// [`weakref.ReferenceType`]: https://docs.python.org/3/library/weakref.html#weakref.ReferenceType
    /// [`weakref.ref`]: https://docs.python.org/3/library/weakref.html#weakref.ref
    fn get_object(&self) -> Option<Bound<'py, PyAny>> {
        let object = self.get_object_raw();

        if object.is_none() {
            None
        } else {
            Some(object)
        }
    }

    // TODO: NAMING-ALTERNATIVE: upgrade_any_borrowed
    /// Upgrade the weakref to a Borrowed [`PyAny`] reference to the target object if possible.
    ///
    /// This function returns `Some(Borrowed<'_, 'py, PyAny>)` if the reference still exists, otherwise `None` will be returned.
    ///
    /// This function gets the optional target of this [`weakref.ReferenceType`] (result of calling [`weakref.ref`]).
    /// It produces similair results to using [`PyWeakref_GetObject`] in the C api.
    ///
    /// # Example
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakRef>) -> PyResult<String> {
    ///     if let Some(object) = reference.borrow_object() {
    ///         Ok(format!("The object '{}' refered by this reference still exists.", object.getattr("__class__")?.getattr("__qualname__")?))
    ///     } else {
    ///         Ok("The object, which this reference refered to, no longer exists".to_owned())
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let data = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakRef::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The object 'Foo' refered by this reference still exists."
    ///     );
    ///
    ///     drop(data);
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The object, which this reference refered to, no longer exists"
    ///     );
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics is the current object is invalid.
    /// If used propperly this is never the case. (NonNull and actually a weakref type)
    ///
    /// [`PyWeakref_GetObject`]: https://docs.python.org/3/c-api/weakref.html#c.PyWeakref_GetObject
    /// [`weakref.ReferenceType`]: https://docs.python.org/3/library/weakref.html#weakref.ReferenceType
    /// [`weakref.ref`]: https://docs.python.org/3/library/weakref.html#weakref.ref
    fn borrow_object<'a>(&'a self) -> Option<Borrowed<'a, 'py, PyAny>>
    where
        'py: 'a,
    {
        let object = self.borrow_object_raw();

        if object.is_none() {
            None
        } else {
            Some(object)
        }
    }

    // TODO: NAMING-ALTERNATIVE: get_any
    /// Retrieve to a Bound object pointed to by the weakref.
    ///
    /// This function returns `Bound<'py, PyAny>`, which is either the object if it still exists, otherwise it will refer to [`PyNone`](crate::types::none::PyNone).
    ///
    /// This function gets the optional target of this [`weakref.ReferenceType`] (result of calling [`weakref.ref`]).
    /// It produces similair results to using [`PyWeakref_GetObject`] in the C api.
    ///
    /// # Example
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// fn get_class(reference: Borrowed<'_, '_, PyWeakRef>) -> PyResult<String> {
    ///     reference
    ///         .get_object_raw()
    ///         .getattr("__class__")?
    ///         .repr()?
    ///         .to_str()
    ///         .map(ToOwned::to_owned)
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let object = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakRef::new_bound(&object)?;
    ///
    ///     assert_eq!(
    ///         get_class(reference.as_borrowed())?,
    ///         "<class 'builtins.Foo'>"
    ///     );
    ///
    ///     drop(object);
    ///
    ///     assert_eq!(get_class(reference.as_borrowed())?, "<class 'NoneType'>");
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics is the current object is invalid.
    /// If used propperly this is never the case. (NonNull and actually a weakref type)
    ///
    /// [`PyWeakref_GetObject`]: https://docs.python.org/3/c-api/weakref.html#c.PyWeakref_GetObject
    /// [`weakref.ReferenceType`]: https://docs.python.org/3/library/weakref.html#weakref.ReferenceType
    /// [`weakref.ref`]: https://docs.python.org/3/library/weakref.html#weakref.ref
    fn get_object_raw(&self) -> Bound<'py, PyAny> {
        // PyWeakref_GetObject does some error checking, however we ensure the passed object is Non-Null and a Weakref type.
        self.borrow_object_raw().to_owned()
    }

    // TODO: NAMING-ALTERNATIVE: get_any_borrowed
    /// Retrieve to a Borrowed object pointed to by the weakref.
    ///
    /// This function returns `Borrowed<'py, PyAny>`, which is either the object if it still exists, otherwise it will refer to [`PyNone`](crate::types::none::PyNone).
    ///
    /// This function gets the optional target of this [`weakref.ReferenceType`] (result of calling [`weakref.ref`]).
    /// It produces similair results to  using [`PyWeakref_GetObject`] in the C api.
    ///
    /// # Example
    #[cfg_attr(
        not(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9))))),
        doc = "```rust,ignore"
    )]
    #[cfg_attr(
        all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))),
        doc = "```rust"
    )]
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// fn get_class(reference: Borrowed<'_, '_, PyWeakRef>) -> PyResult<String> {
    ///     reference
    ///         .borrow_object_raw()
    ///         .getattr("__class__")?
    ///         .repr()?
    ///         .to_str()
    ///         .map(ToOwned::to_owned)
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let object = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakRef::new_bound(&object)?;
    ///
    ///     assert_eq!(
    ///         get_class(reference.as_borrowed())?,
    ///         "<class 'builtins.Foo'>"
    ///     );
    ///
    ///     drop(object);
    ///
    ///     assert_eq!(get_class(reference.as_borrowed())?, "<class 'NoneType'>");
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics is the current object is invalid.
    /// If used propperly this is never the case. (NonNull and actually a weakref type)
    ///
    /// [`PyWeakref_GetObject`]: https://docs.python.org/3/c-api/weakref.html#c.PyWeakref_GetObject
    /// [`weakref.ReferenceType`]: https://docs.python.org/3/library/weakref.html#weakref.ReferenceType
    /// [`weakref.ref`]: https://docs.python.org/3/library/weakref.html#weakref.ref
    fn borrow_object_raw(&self) -> Borrowed<'_, 'py, PyAny>;
}

impl<'py> PyWeakRefMethods<'py> for Bound<'py, PyWeakRef> {
    /*
    fn borrowed_upgrade<T>(&self) -> PyResult<Option<Borrowed<'_, 'py, T>>>
    where
        T: PyTypeCheck
    {
        Ok(self.borrow_object()?.map(|obj| obj.downcast_into::<T>().expect(
                    "The `weakref.ReferenceType` (`PyWeakRef`) should refer to an instance of the specified class",
                )))
    }
    */

    fn borrow_object_raw(&self) -> Borrowed<'_, 'py, PyAny> {
        // PyWeakref_GetObject does some error checking, however we ensure the passed object is Non-Null and a Weakref type.
        unsafe { ffi::PyWeakref_GetObject(self.as_ptr()).assume_borrowed_or_err(self.py()) }
            .expect("The 'weakref.ReferenceType' instance should be valid (non-null and actually a weakref reference)")
    }
}

#[cfg(test)]
mod tests {
    use crate::types::any::PyAnyMethods;
    use crate::types::weakref::{PyWeakRef, PyWeakRefMethods};
    use crate::{Bound, PyAny, PyResult, Python};

    #[cfg(all(not(Py_LIMITED_API), Py_3_10))]
    const CLASS_NAME: &str = "<class 'weakref.ReferenceType'>";
    #[cfg(all(not(Py_LIMITED_API), not(Py_3_10)))]
    const CLASS_NAME: &str = "<class 'weakref'>";

    fn check_repr(
        reference: &Bound<'_, PyWeakRef>,
        object: Option<(&Bound<'_, PyAny>, &str)>,
    ) -> PyResult<()> {
        let repr = reference.repr()?.to_string();
        let (first_part, second_part) = repr.split_once("; ").unwrap();

        {
            let (msg, addr) = first_part.split_once("0x").unwrap();

            assert_eq!(msg, "<weakref at ");
            assert!(addr
                .to_lowercase()
                .contains(format!("{:x?}", reference.as_ptr()).split_at(2).1));
        }

        match object {
            Some((object, class)) => {
                let (msg, addr) = second_part.split_once("0x").unwrap();

                assert_eq!(msg, format!("to '{}' at ", class));
                assert!(addr
                    .to_lowercase()
                    .contains(format!("{:x?}", object.as_ptr()).split_at(2).1));
            }
            None => {
                assert_eq!(second_part, "dead>")
            }
        }

        Ok(())
    }

    mod python_class {
        use super::*;
        use crate::{py_result_ext::PyResultExt, types::PyType};

        fn get_type(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
            py.run_bound("class A:\n    pass\n", None, None)?;
            py.eval_bound("A", None, None).downcast_into::<PyType>()
        }

        #[test]
        fn test_weakref_refence_behavior() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakRef::new_bound(&object)?;

                assert!(!reference.is(&object));
                assert!(reference.get_object_raw().is(&object));

                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.get_type().to_string(), CLASS_NAME);

                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.getattr("__class__")?.to_string(), CLASS_NAME);

                check_repr(&reference, Some((object.as_any(), "A")))?;

                assert!(reference
                    .getattr("__callback__")
                    .map_or(false, |result| result.is_none()));

                assert!(reference.call0()?.is(&object));

                drop(object);

                assert!(reference.get_object_raw().is_none());
                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.getattr("__class__")?.to_string(), CLASS_NAME);
                check_repr(&reference, None)?;

                assert!(reference
                    .getattr("__callback__")
                    .map_or(false, |result| result.is_none()));

                assert!(reference.call0()?.is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakRef::new_bound(&object)?;

                {
                    // This test is a bit weird but ok.
                    let obj = reference.upgrade::<PyAny>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_some());
                    assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()
                        && obj.is_exact_instance(&class)));
                }

                drop(object);

                {
                    // This test is a bit weird but ok.
                    let obj = reference.upgrade::<PyAny>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_none());
                }

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakRef::new_bound(&object)?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.get_object().is_some());
                assert!(reference.get_object().map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.get_object().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_borrrow_object() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakRef::new_bound(&object)?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.borrow_object().is_some());
                assert!(reference
                    .borrow_object()
                    .map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.borrow_object().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object_raw() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakRef::new_bound(&object)?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.get_object_raw().is(&object));

                drop(object);

                assert!(reference.call0()?.is(&reference.get_object_raw()));
                assert!(reference.call0()?.is_none());
                assert!(reference.get_object_raw().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_borrow_object_raw() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakRef::new_bound(&object)?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.borrow_object_raw().is(&object));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.borrow_object_raw().is_none());

                Ok(())
            })
        }
    }

    // under 'abi3-py37' and 'abi3-py38' PyClass cannot be weakreferencable.
    #[cfg(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))))]
    mod pyo3_pyclass {
        use super::*;
        use crate::{pyclass, Py};

        #[pyclass(weakref, crate = "crate")]
        struct WeakrefablePyClass {}

        #[test]
        fn test_weakref_refence_behavior() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Bound::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakRef::new_bound(&object)?;

                assert!(!reference.is(&object));
                assert!(reference.get_object_raw().is(&object));
                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.get_type().to_string(), CLASS_NAME);

                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.getattr("__class__")?.to_string(), CLASS_NAME);
                check_repr(
                    &reference,
                    Some((object.as_any(), "builtins.WeakrefablePyClass")),
                )?;

                assert!(reference
                    .getattr("__callback__")
                    .map_or(false, |result| result.is_none()));

                assert!(reference.call0()?.is(&object));

                drop(object);

                assert!(reference.get_object_raw().is_none());
                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.getattr("__class__")?.to_string(), CLASS_NAME);
                check_repr(&reference, None)?;

                assert!(reference
                    .getattr("__callback__")
                    .map_or(false, |result| result.is_none()));

                assert!(reference.call0()?.is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakRef::new_bound(object.bind(py))?;

                {
                    let obj = reference.upgrade::<WeakrefablePyClass>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_some());
                    assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()));
                }

                drop(object);

                {
                    let obj = reference.upgrade::<WeakrefablePyClass>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_none());
                }

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakRef::new_bound(object.bind(py))?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.get_object().is_some());
                assert!(reference.get_object().map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.get_object().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_borrrow_object() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakRef::new_bound(object.bind(py))?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.borrow_object().is_some());
                assert!(reference
                    .borrow_object()
                    .map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.borrow_object().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object_raw() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakRef::new_bound(object.bind(py))?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.get_object_raw().is(&object));

                drop(object);

                assert!(reference.call0()?.is(&reference.get_object_raw()));
                assert!(reference.call0()?.is_none());
                assert!(reference.get_object_raw().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_borrow_object_raw() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakRef::new_bound(object.bind(py))?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.borrow_object_raw().is(&object));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.borrow_object_raw().is_none());

                Ok(())
            })
        }
    }
}
