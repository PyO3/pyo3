use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::types::any::PyAny;
use crate::{ffi, Borrowed, Bound, ToPyObject};

#[cfg(any(any(PyPy, GraalPy, Py_LIMITED_API), feature = "gil-refs"))]
use crate::type_object::PyTypeCheck;
#[cfg(feature = "gil-refs")]
use crate::{type_object::PyTypeInfo, PyNativeType};

use super::PyWeakrefMethods;

/// Represents a Python `weakref.ReferenceType`.
///
/// In Python this is created by calling `weakref.ref`.
#[repr(transparent)]
pub struct PyWeakrefReference(PyAny);

#[cfg(not(any(PyPy, GraalPy, Py_LIMITED_API)))]
pyobject_native_type!(
    PyWeakrefReference,
    ffi::PyWeakReference,
    pyobject_native_static_type_object!(ffi::_PyWeakref_RefType),
    #module=Some("weakref"),
    #checkfunction=ffi::PyWeakref_CheckRefExact
);

// When targetting alternative or multiple interpreters, it is better to not use the internal API.
#[cfg(any(PyPy, GraalPy, Py_LIMITED_API))]
pyobject_native_type_named!(PyWeakrefReference);
#[cfg(any(PyPy, GraalPy, Py_LIMITED_API))]
pyobject_native_type_extract!(PyWeakrefReference);

#[cfg(any(PyPy, GraalPy, Py_LIMITED_API))]
impl PyTypeCheck for PyWeakrefReference {
    const NAME: &'static str = "weakref.ReferenceType";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        unsafe { ffi::PyWeakref_CheckRef(object.as_ptr()) > 0 }
    }
}

impl PyWeakrefReference {
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
    /// use pyo3::types::PyWeakrefReference;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let foo = Bound::new(py, Foo {})?;
    ///     let weakref = PyWeakrefReference::new_bound(&foo)?;
    ///     assert!(
    ///         // In normal situations where a direct `Bound<'py, Foo>` is required use `upgrade::<Foo>`
    ///         weakref.upgrade()
    ///             .map_or(false, |obj| obj.is(&foo))
    ///     );
    ///
    ///     let weakref2 = PyWeakrefReference::new_bound(&foo)?;
    ///     assert!(weakref.is(&weakref2));
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.upgrade().is_none());
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn new_bound<'py>(object: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyWeakrefReference>> {
        // TODO: Is this inner pattern still necessary Here?
        fn inner<'py>(object: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyWeakrefReference>> {
            unsafe {
                Bound::from_owned_ptr_or_err(
                    object.py(),
                    ffi::PyWeakref_NewRef(object.as_ptr(), ffi::Py_None()),
                )
                .downcast_into_unchecked()
            }
        }

        inner(object)
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
    /// use pyo3::types::PyWeakrefReference;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pyfunction]
    /// fn callback(wref: Bound<'_, PyWeakrefReference>) -> PyResult<()> {
    ///         let py = wref.py();
    ///         assert!(wref.upgrade_as::<Foo>()?.is_none());
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
    ///     let weakref = PyWeakrefReference::new_bound_with(&foo, py.None())?;
    ///     assert!(weakref.upgrade_as::<Foo>()?.is_some());
    ///     assert!(
    ///         // In normal situations where a direct `Bound<'py, Foo>` is required use `upgrade::<Foo>`
    ///         weakref.upgrade()
    ///             .map_or(false, |obj| obj.is(&foo))
    ///     );
    ///     assert_eq!(py.eval_bound("counter", None, None)?.extract::<u32>()?, 0);
    ///
    ///     let weakref2 = PyWeakrefReference::new_bound_with(&foo, wrap_pyfunction_bound!(callback, py)?)?;
    ///     assert!(!weakref.is(&weakref2)); // Not the same weakref
    ///     assert!(weakref.eq(&weakref2)?);  // But Equal, since they point to the same object
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.upgrade_as::<Foo>()?.is_none());
    ///     assert_eq!(py.eval_bound("counter", None, None)?.extract::<u32>()?, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn new_bound_with<'py, C>(
        object: &Bound<'py, PyAny>,
        callback: C,
    ) -> PyResult<Bound<'py, PyWeakrefReference>>
    where
        C: ToPyObject,
    {
        fn inner<'py>(
            object: &Bound<'py, PyAny>,
            callback: Bound<'py, PyAny>,
        ) -> PyResult<Bound<'py, PyWeakrefReference>> {
            unsafe {
                Bound::from_owned_ptr_or_err(
                    object.py(),
                    ffi::PyWeakref_NewRef(object.as_ptr(), callback.as_ptr()),
                )
                .downcast_into_unchecked()
            }
        }

        let py = object.py();
        inner(object, callback.to_object(py).into_bound(py))
    }
}

#[cfg(feature = "gil-refs")]
impl PyWeakrefReference {
    /// Deprecated form of [`PyWeakrefReference::new_bound`].
    #[inline]
    #[deprecated(
        since = "0.21.0",
        note = "`PyWeakrefReference::new` will be replaced by `PyWeakrefReference::new_bound` in a future PyO3 version"
    )]
    pub fn new<T>(object: &T) -> PyResult<&PyWeakrefReference>
    where
        T: PyNativeType,
    {
        Self::new_bound(object.as_borrowed().as_any()).map(Bound::into_gil_ref)
    }

    /// Deprecated form of [`PyWeakrefReference::new_bound_with`].
    #[inline]
    #[deprecated(
        since = "0.21.0",
        note = "`PyWeakrefReference::new_with` will be replaced by `PyWeakrefReference::new_bound_with` in a future PyO3 version"
    )]
    pub fn new_with<T, C>(object: &T, callback: C) -> PyResult<&PyWeakrefReference>
    where
        T: PyNativeType,
        C: ToPyObject,
    {
        Self::new_bound_with(object.as_borrowed().as_any(), callback).map(Bound::into_gil_ref)
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
    /// use pyo3::types::PyWeakrefReference;
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
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakrefReference>) -> PyResult<String> {
    ///     if let Some(data_src) = reference.upgrade_as::<Foo>()? {
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
    ///     let reference = PyWeakrefReference::new_bound(&data)?;
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
    pub fn upgrade_as<T>(&self) -> PyResult<Option<&T::AsRefTarget>>
    where
        T: PyTypeCheck,
    {
        Ok(self
            .as_borrowed()
            .upgrade_as::<T>()?
            .map(Bound::into_gil_ref))
    }

    /// Upgrade the weakref to a direct object reference unchecked. The type of the recovered object is not checked before downcasting, this could lead to unexpected behavior. Use only when absolutely certain the type can be guaranteed. The `weakref` may still return `None`.
    ///
    /// It is named `upgrade` to be inline with [rust's `Weak::upgrade`](std::rc::Weak::upgrade).
    /// In Python it would be equivalent to [`PyWeakref_GetObject`] or calling the [`weakref.ReferenceType`] (result of calling [`weakref.ref`]).
    ///
    /// # Safety
    /// Callers must ensure that the type is valid or risk type confusion.
    /// The `weakref` is still allowed to be `None`, if the referenced object has been cleaned up.
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
    /// use pyo3::types::PyWeakrefReference;
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
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakrefReference>) -> String {
    ///     if let Some(data_src) = unsafe { reference.upgrade_as_unchecked::<Foo>() } {
    ///         let data = data_src.borrow();
    ///         let (name, score) = data.get_data();
    ///         format!("Processing '{}': score = {}", name, score)
    ///     } else {
    ///         "The supplied data reference is nolonger relavent.".to_owned()
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let data = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakrefReference::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed()),
    ///         "Processing 'Dave': score = 10"
    ///     );
    ///
    ///     drop(data);
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed()),
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
    pub unsafe fn upgrade_as_unchecked<T>(&self) -> Option<&T::AsRefTarget>
    where
        T: PyTypeCheck,
    {
        self.as_borrowed()
            .upgrade_as_unchecked::<T>()
            .map(Bound::into_gil_ref)
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
    /// use pyo3::types::PyWeakrefReference;
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
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakrefReference>) -> PyResult<String> {
    ///     if let Some(data_src) = reference.upgrade_as_exact::<Foo>()? {
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
    ///     let reference = PyWeakrefReference::new_bound(&data)?;
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
    pub fn upgrade_as_exact<T>(&self) -> PyResult<Option<&T::AsRefTarget>>
    where
        T: PyTypeInfo,
    {
        Ok(self
            .as_borrowed()
            .upgrade_as_exact::<T>()?
            .map(Bound::into_gil_ref))
    }

    /// Upgrade the weakref to a [`PyAny`] reference to the target if possible.
    ///
    /// It is named `upgrade` to be inline with [rust's `Weak::upgrade`](std::rc::Weak::upgrade).
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
    /// use pyo3::types::PyWeakrefReference;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakrefReference>) -> PyResult<String> {
    ///     if let Some(object) = reference.upgrade() {
    ///         Ok(format!("The object '{}' refered by this reference still exists.", object.getattr("__class__")?.getattr("__qualname__")?))
    ///     } else {
    ///         Ok("The object, which this reference refered to, no longer exists".to_owned())
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let data = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakrefReference::new_bound(&data)?;
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
    pub fn upgrade(&self) -> Option<&'_ PyAny> {
        self.as_borrowed().upgrade().map(Bound::into_gil_ref)
    }

    /// Retrieve to a object pointed to by the weakref.
    ///
    /// This function returns `&'py PyAny`, which is either the object if it still exists, otherwise it will refer to [`PyNone`](crate::types::PyNone).
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
    /// use pyo3::types::PyWeakrefReference;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// fn get_class(reference: Borrowed<'_, '_, PyWeakrefReference>) -> PyResult<String> {
    ///     reference
    ///         .get_object()
    ///         .getattr("__class__")?
    ///         .repr()
    ///         .map(|repr| repr.to_string())
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let object = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakrefReference::new_bound(&object)?;
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
    pub fn get_object(&self) -> &'_ PyAny {
        self.as_borrowed().get_object().into_gil_ref()
    }
}

impl<'py> PyWeakrefMethods<'py> for Bound<'py, PyWeakrefReference> {
    fn get_object_borrowed(&self) -> Borrowed<'_, 'py, PyAny> {
        // PyWeakref_GetObject does some error checking, however we ensure the passed object is Non-Null and a Weakref type.
        unsafe { ffi::PyWeakref_GetObject(self.as_ptr()).assume_borrowed_or_err(self.py()) }
            .expect("The 'weakref.ReferenceType' instance should be valid (non-null and actually a weakref reference)")
    }
}

#[cfg(test)]
mod tests {
    use crate::types::any::{PyAny, PyAnyMethods};
    use crate::types::weakref::{PyWeakrefMethods, PyWeakrefReference};
    use crate::{Bound, PyResult, Python};

    #[cfg(all(not(Py_LIMITED_API), Py_3_10))]
    const CLASS_NAME: &str = "<class 'weakref.ReferenceType'>";
    #[cfg(all(not(Py_LIMITED_API), not(Py_3_10)))]
    const CLASS_NAME: &str = "<class 'weakref'>";

    fn check_repr(
        reference: &Bound<'_, PyWeakrefReference>,
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
                let reference = PyWeakrefReference::new_bound(&object)?;

                assert!(!reference.is(&object));
                assert!(reference.get_object().is(&object));

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

                assert!(reference.get_object().is_none());
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
        fn test_weakref_upgrade_as() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakrefReference::new_bound(&object)?;

                {
                    // This test is a bit weird but ok.
                    let obj = reference.upgrade_as::<PyAny>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_some());
                    assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()
                        && obj.is_exact_instance(&class)));
                }

                drop(object);

                {
                    // This test is a bit weird but ok.
                    let obj = reference.upgrade_as::<PyAny>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_none());
                }

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_borrowed_as() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakrefReference::new_bound(&object)?;

                {
                    // This test is a bit weird but ok.
                    let obj = reference.upgrade_borrowed_as::<PyAny>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_some());
                    assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()
                        && obj.is_exact_instance(&class)));
                }

                drop(object);

                {
                    // This test is a bit weird but ok.
                    let obj = reference.upgrade_borrowed_as::<PyAny>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_none());
                }

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_as_unchecked() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakrefReference::new_bound(&object)?;

                {
                    // This test is a bit weird but ok.
                    let obj = unsafe { reference.upgrade_as_unchecked::<PyAny>() };

                    assert!(obj.is_some());
                    assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()
                        && obj.is_exact_instance(&class)));
                }

                drop(object);

                {
                    // This test is a bit weird but ok.
                    let obj = unsafe { reference.upgrade_as_unchecked::<PyAny>() };

                    assert!(obj.is_none());
                }

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_borrowed_as_unchecked() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakrefReference::new_bound(&object)?;

                {
                    // This test is a bit weird but ok.
                    let obj = unsafe { reference.upgrade_borrowed_as_unchecked::<PyAny>() };

                    assert!(obj.is_some());
                    assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()
                        && obj.is_exact_instance(&class)));
                }

                drop(object);

                {
                    // This test is a bit weird but ok.
                    let obj = unsafe { reference.upgrade_borrowed_as_unchecked::<PyAny>() };

                    assert!(obj.is_none());
                }

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakrefReference::new_bound(&object)?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.upgrade().is_some());
                assert!(reference.upgrade().map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.upgrade().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_borrowed() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakrefReference::new_bound(&object)?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.upgrade_borrowed().is_some());
                assert!(reference
                    .upgrade_borrowed()
                    .map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.upgrade_borrowed().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakrefReference::new_bound(&object)?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.get_object().is(&object));

                drop(object);

                assert!(reference.call0()?.is(&reference.get_object()));
                assert!(reference.call0()?.is_none());
                assert!(reference.get_object().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object_borrowed() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakrefReference::new_bound(&object)?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.get_object_borrowed().is(&object));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.get_object_borrowed().is_none());

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
                let object: Bound<'_, WeakrefablePyClass> = Bound::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new_bound(&object)?;

                assert!(!reference.is(&object));
                assert!(reference.get_object().is(&object));
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

                assert!(reference.get_object().is_none());
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
        fn test_weakref_upgrade_as() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new_bound(object.bind(py))?;

                {
                    let obj = reference.upgrade_as::<WeakrefablePyClass>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_some());
                    assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()));
                }

                drop(object);

                {
                    let obj = reference.upgrade_as::<WeakrefablePyClass>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_none());
                }

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_borrowed_as() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new_bound(object.bind(py))?;

                {
                    let obj = reference.upgrade_borrowed_as::<WeakrefablePyClass>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_some());
                    assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()));
                }

                drop(object);

                {
                    let obj = reference.upgrade_borrowed_as::<WeakrefablePyClass>();

                    assert!(obj.is_ok());
                    let obj = obj.unwrap();

                    assert!(obj.is_none());
                }

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_as_unchecked() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new_bound(object.bind(py))?;

                {
                    let obj = unsafe { reference.upgrade_as_unchecked::<WeakrefablePyClass>() };

                    assert!(obj.is_some());
                    assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()));
                }

                drop(object);

                {
                    let obj = unsafe { reference.upgrade_as_unchecked::<WeakrefablePyClass>() };

                    assert!(obj.is_none());
                }

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_borrowed_as_unchecked() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new_bound(object.bind(py))?;

                {
                    let obj =
                        unsafe { reference.upgrade_borrowed_as_unchecked::<WeakrefablePyClass>() };

                    assert!(obj.is_some());
                    assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()));
                }

                drop(object);

                {
                    let obj =
                        unsafe { reference.upgrade_borrowed_as_unchecked::<WeakrefablePyClass>() };

                    assert!(obj.is_none());
                }

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new_bound(object.bind(py))?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.upgrade().is_some());
                assert!(reference.upgrade().map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.upgrade().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_borrowed() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new_bound(object.bind(py))?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.upgrade_borrowed().is_some());
                assert!(reference
                    .upgrade_borrowed()
                    .map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.upgrade_borrowed().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new_bound(object.bind(py))?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.get_object().is(&object));

                drop(object);

                assert!(reference.call0()?.is(&reference.get_object()));
                assert!(reference.call0()?.is_none());
                assert!(reference.get_object().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object_borrowed() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new_bound(object.bind(py))?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.get_object_borrowed().is(&object));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.get_object_borrowed().is_none());

                Ok(())
            })
        }
    }
}
