use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::type_object::{PyTypeCheck, PyTypeInfo};
use crate::types::any::{PyAny, PyAnyMethods};
use crate::{ffi, AsPyPointer, Borrowed, Bound, PyNativeType, ToPyObject};

use super::PyWeakRefMethods;

/// Represents a Python `weakref.ProxyType`.
///
/// In Python this is created by calling `weakref.proxy`.
#[repr(transparent)]
pub struct PyWeakCallableProxy(PyAny);

pyobject_native_type!(
    PyWeakCallableProxy,
    ffi::PyWeakReference,
    pyobject_native_static_type_object!(ffi::_PyWeakref_CallableProxyType),
    #module=Some("weakref"),
    #checkfunction=ffi::PyWeakref_CheckProxy
);

impl PyWeakCallableProxy {
    /// Deprecated form of [`PyWeakCallableProxy::new_bound`].
    #[inline]
    #[track_caller]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyWeakCallableProxy::new` will be replaced by `PyWeakCallableProxy::new_bound` in a future PyO3 version"
        )
    )]
    pub fn new<T>(object: &T) -> PyResult<&PyWeakCallableProxy>
    where
        T: PyNativeType,
    {
        Self::new_bound(object.as_borrowed().as_any()).map(Bound::into_gil_ref)
    }

    /// Constructs a new Weak callable Reference (`weakref.proxy`/`weakref.CallableProxyType`) for the given object.
    ///
    /// Returns a `TypeError` if `object` is not weak referenceable (Most native types and PyClasses without `weakref` flag).
    /// The object should also be callable. For a non-callable weakref proxy see [`PyWeakProxy`](crate::types::weakref::PyWeakProxy).
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
    /// use pyo3::types::PyWeakCallableProxy;
    /// use pyo3::exceptions::PyReferenceError;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     fn __call__(&self) -> &str {
    ///         "This class is callable"
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let foo = Bound::new(py, Foo {})?;
    ///     let weakref = PyWeakCallableProxy::new_bound(&foo)?;
    ///     assert!(
    ///         // In normal situations where a direct `Bound<'py, Foo>` is required use `upgrade::<Foo>`
    ///         weakref.upgrade()
    ///             .map_or(false, |obj| obj.is(&foo))
    ///     );
    ///
    ///     let weakref2 = PyWeakCallableProxy::new_bound(&foo)?;
    ///     assert!(weakref.is(&weakref2));
    ///
    ///     assert_eq!(weakref.call0()?.to_string(), "This class is callable");
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.upgrade().is_none());
    ///     assert!(weakref.call0()
    ///                 .err()
    ///                 .map_or(false, |err| err.is_instance_of::<PyReferenceError>(py))
    ///     );
    ///
    ///     Ok(())
    /// })
    /// # }
    /// ```
    ///
    /// # Panics
    /// This function panics if the provided object is not callable.
    ///
    #[inline]
    #[track_caller]
    pub fn new_bound<'py>(object: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyWeakCallableProxy>> {
        // TODO: Must track caller be used here?
        #[track_caller]
        fn inner<'py>(object: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyWeakCallableProxy>> {
            assert!(
                object.is_callable(),
                "An object to be referenced by a PyWeakCallableProxy should be callable. Use PyWeakProxy instead."
            );

            unsafe {
                Bound::from_owned_ptr_or_err(
                    object.py(),
                    ffi::PyWeakref_NewProxy(object.as_ptr(), ffi::Py_None()),
                )
                .downcast_into_unchecked()
            }
        }

        inner(object)
    }

    /// Deprecated form of [`PyWeakCallableProxy::new_bound_with`].
    #[inline]
    #[track_caller]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyWeakCallableProxy::new_with` will be replaced by `PyWeakCallableProxy::new_bound_with` in a future PyO3 version"
        )
    )]
    pub fn new_with<T, C>(object: &T, callback: C) -> PyResult<&PyWeakCallableProxy>
    where
        T: PyNativeType,
        C: ToPyObject,
    {
        Self::new_bound_with(object.as_borrowed().as_any(), callback).map(Bound::into_gil_ref)
    }

    /// Constructs a new Weak Reference (`weakref.proxy`/`weakref.CallableProxyType`) for the given object with a callback.
    ///
    /// Returns a `TypeError` if `object` is not weak referenceable (Most native types and PyClasses without `weakref` flag) or if the `callback` is not callable or None.
    /// The object should also be callable. For a non-callable weakref proxy see [`PyWeakProxy`](crate::types::weakref::PyWeakProxy).
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
    /// use pyo3::types::PyWeakCallableProxy;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     fn __call__(&self) -> &str {
    ///         "This class is callable"
    ///     }
    /// }
    ///
    /// #[pyfunction]
    /// fn callback(wref: Bound<'_, PyWeakCallableProxy>) -> PyResult<()> {
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
    ///     let weakref = PyWeakCallableProxy::new_bound_with(&foo, py.None())?;
    ///     assert!(weakref.upgrade_as::<Foo>()?.is_some());
    ///     assert!(
    ///         // In normal situations where a direct `Bound<'py, Foo>` is required use `upgrade::<Foo>`
    ///         weakref.upgrade()
    ///             .map_or(false, |obj| obj.is(&foo))
    ///     );
    ///     assert_eq!(py.eval_bound("counter", None, None)?.extract::<u32>()?, 0);
    ///
    ///     let weakref2 = PyWeakCallableProxy::new_bound_with(&foo, wrap_pyfunction!(callback, py)?)?;
    ///     assert!(!weakref.is(&weakref2)); // Not the same weakref
    ///     assert!(weakref.eq(&weakref2)?);  // But Equal, since they point to the same object
    ///
    ///     assert_eq!(weakref.call0()?.to_string(), "This class is callable");
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.upgrade_as::<Foo>()?.is_none());
    ///     assert_eq!(py.eval_bound("counter", None, None)?.extract::<u32>()?, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    /// # Panics
    /// This function panics if the provided object is not callable.
    #[track_caller]
    pub fn new_bound_with<'py, C>(
        object: &Bound<'py, PyAny>,
        callback: C,
    ) -> PyResult<Bound<'py, PyWeakCallableProxy>>
    where
        C: ToPyObject,
    {
        // TODO: Must track caller be used here?
        #[track_caller]
        fn inner<'py>(
            object: &Bound<'py, PyAny>,
            callback: Bound<'py, PyAny>,
        ) -> PyResult<Bound<'py, PyWeakCallableProxy>> {
            assert!(
                object.is_callable(),
                "An object to be referenced by a PyWeakCallableProxy should be callable. Use PyWeakProxy instead."
            );

            unsafe {
                Bound::from_owned_ptr_or_err(
                    object.py(),
                    ffi::PyWeakref_NewProxy(object.as_ptr(), callback.as_ptr()),
                )
                .downcast_into_unchecked()
            }
        }

        let py = object.py();
        inner(object, callback.to_object(py).into_bound(py))
    }

    /// Upgrade the weakref to a direct object reference.
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
    /// use pyo3::types::PyWeakCallableProxy;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     fn __call__(&self) -> &str {
    ///         "This class is callable"
    ///     }
    ///
    ///     fn get_data(&self) -> (&str, u32) {
    ///         ("Dave", 10)
    ///     }
    /// }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakCallableProxy>) -> PyResult<String> {
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
    ///     let reference = PyWeakCallableProxy::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "Processing 'Dave': score = 10"
    ///     );
    ///
    ///     assert_eq!(reference.call0()?.to_string(), "This class is callable");
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
    /// [`weakref.CallableProxyType`]: https://docs.python.org/3/library/weakref.html#weakref.CallableProxyType
    /// [`weakref.proxy`]: https://docs.python.org/3/library/weakref.html#weakref.proxy
    #[track_caller]
    pub fn upgrade_as<T>(&self) -> PyResult<Option<&T::AsRefTarget>>
    where
        T: PyTypeCheck,
    {
        Ok(self
            .as_borrowed()
            .upgrade_as::<T>()?
            .map(Bound::into_gil_ref))
    }

    /// Upgrade the weakref to an exact direct object reference.
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
    /// use pyo3::types::PyWeakCallableProxy;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     fn __call__(&self) -> &str {
    ///         "This class is callable"
    ///     }
    ///
    ///     fn get_data(&self) -> (&str, u32) {
    ///         ("Dave", 10)
    ///     }
    /// }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakCallableProxy>) -> PyResult<String> {
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
    ///     let reference = PyWeakCallableProxy::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "Processing 'Dave': score = 10"
    ///     );
    ///
    ///     assert_eq!(reference.call0()?.to_string(), "This class is callable");
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
    /// [`weakref.CallableProxyType`]: https://docs.python.org/3/library/weakref.html#weakref.CallableProxyType
    /// [`weakref.proxy`]: https://docs.python.org/3/library/weakref.html#weakref.proxy
    #[track_caller]
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
    /// This function gets the optional target of this [`weakref.CallableProxyType`] (result of calling [`weakref.proxy`]).
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
    /// use pyo3::types::PyWeakCallableProxy;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     fn __call__(&self) -> &str {
    ///         "This class is callable"
    ///     }
    /// }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakCallableProxy>) -> PyResult<String> {
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
    ///     let reference = PyWeakCallableProxy::new_bound(&data)?;
    ///
    ///     assert_eq!(
    ///         parse_data(reference.as_borrowed())?,
    ///         "The object 'Foo' refered by this reference still exists."
    ///     );
    ///
    ///     assert_eq!(reference.call0()?.to_string(), "This class is callable");
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
    /// [`weakref.ProxyCallableType`]: https://docs.python.org/3/library/weakref.html#weakref.CallableProxyType
    /// [`weakref.proxy`]: https://docs.python.org/3/library/weakref.html#weakref.proxy
    #[track_caller]
    pub fn upgrade(&self) -> Option<&'_ PyAny> {
        self.as_borrowed().upgrade().map(Bound::into_gil_ref)
    }

    /// Retrieve to a object pointed to by the weakref.
    ///
    /// This function returns `&'py PyAny`, which is either the object if it still exists, otherwise it will refer to [`PyNone`](crate::types::none::PyNone).
    ///
    /// This function gets the optional target of this [`weakref.CallableProxyType`] (result of calling [`weakref.proxy`]).
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
    /// use pyo3::types::PyWeakCallableProxy;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     fn __call__(&self) -> &str {
    ///         "This class is callable"
    ///     }
    /// }
    ///
    /// fn get_class(reference: Borrowed<'_, '_, PyWeakCallableProxy>) -> PyResult<String> {
    ///     reference
    ///         .get_object()
    ///         .getattr("__class__")?
    ///         .repr()?
    ///         .to_str()
    ///         .map(ToOwned::to_owned)
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let object = Bound::new(py, Foo{})?;
    ///     let reference = PyWeakCallableProxy::new_bound(&object)?;
    ///
    ///     assert_eq!(
    ///         get_class(reference.as_borrowed())?,
    ///         "<class 'builtins.Foo'>"
    ///     );
    ///
    ///     assert_eq!(reference.call0()?.to_string(), "This class is callable");
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
    /// [`weakref.CallableProxyType`]: https://docs.python.org/3/library/weakref.html#weakref.CallableProxyType
    /// [`weakref.proxy`]: https://docs.python.org/3/library/weakref.html#weakref.proxy
    #[track_caller]
    pub fn get_object(&self) -> &'_ PyAny {
        self.as_borrowed().get_object().into_gil_ref()
    }
}

impl<'py> PyWeakRefMethods<'py> for Bound<'py, PyWeakCallableProxy> {
    #[track_caller]
    fn get_object_borrowed(&self) -> Borrowed<'_, 'py, PyAny> {
        // PyWeakref_GetObject does some error checking, however we ensure the passed object is Non-Null and a Weakref type.
        unsafe { ffi::PyWeakref_GetObject(self.as_ptr()).assume_borrowed_or_err(self.py()) }
            .expect("The 'weakref.CallableProxyType' instance should be valid (non-null and actually a weakref reference)")
    }
}

#[cfg(test)]
mod tests {
    use crate::exceptions::{PyAttributeError, PyReferenceError};
    use crate::types::any::{PyAny, PyAnyMethods};
    use crate::types::weakref::{PyWeakCallableProxy, PyWeakRefMethods};
    use crate::{Bound, PyResult, Python};

    #[cfg(all(not(Py_LIMITED_API), Py_3_10))]
    const CLASS_NAME: &str = "<class 'weakref.CallableProxyType'>";
    #[cfg(all(not(Py_LIMITED_API), not(Py_3_10)))]
    const CLASS_NAME: &str = "<class 'weakcallableproxy'>";

    fn check_repr(
        reference: &Bound<'_, PyWeakCallableProxy>,
        object: &Bound<'_, PyAny>,
        class: &str,
    ) -> PyResult<()> {
        let repr = reference.repr()?.to_string();
        let (first_part, second_part) = repr.split_once(" to ").unwrap();

        {
            let (msg, addr) = first_part.split_once("0x").unwrap();

            assert_eq!(msg, "<weakproxy at ");
            assert!(addr
                .to_lowercase()
                .contains(format!("{:x?}", reference.as_ptr()).split_at(2).1));
        }

        let (msg, addr) = second_part.split_once("0x").unwrap();

        assert_eq!(msg, format!("{} at ", class));
        assert!(addr
            .to_lowercase()
            .contains(format!("{:x?}", object.as_ptr()).split_at(2).1));

        Ok(())
    }

    mod python_class {
        use super::*;
        use crate::{py_result_ext::PyResultExt, types::PyType};

        fn get_type(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
            py.run_bound(
                "class A:\n    def __call__(self):\n        return 'This class is callable!'\n",
                None,
                None,
            )?;
            py.eval_bound("A", None, None).downcast_into::<PyType>()
        }

        #[test]
        fn test_weakref_proxy_behavior() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakCallableProxy::new_bound(&object)?;

                assert!(!reference.is(&object));
                assert!(reference.get_object().is(&object));
                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.get_type().to_string(), CLASS_NAME);

                assert_eq!(
                    reference.getattr("__class__")?.to_string(),
                    "<class '__main__.A'>"
                );
                check_repr(&reference, &object, "A")?;

                assert!(reference
                    .getattr("__callback__")
                    .err()
                    .map_or(false, |err| err.is_instance_of::<PyAttributeError>(py)));

                assert_eq!(reference.call0()?.to_string(), "This class is callable!");

                drop(object);

                assert!(reference.get_object().is_none());
                assert!(reference
                    .getattr("__class__")
                    .err()
                    .map_or(false, |err| err.is_instance_of::<PyReferenceError>(py)));
                check_repr(&reference, py.None().bind(py), "NoneType")?;

                assert!(reference
                    .getattr("__callback__")
                    .err()
                    .map_or(false, |err| err.is_instance_of::<PyReferenceError>(py)));

                assert!(reference
                    .call0()
                    .err()
                    .map_or(false, |err| err.is_instance_of::<PyReferenceError>(py)
                        & (err.value_bound(py).to_string()
                            == "weakly-referenced object no longer exists")));

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_as() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakCallableProxy::new_bound(&object)?;

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
                let reference = PyWeakCallableProxy::new_bound(&object)?;

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
        fn test_weakref_upgrade() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakCallableProxy::new_bound(&object)?;

                assert!(reference.upgrade().is_some());
                assert!(reference.upgrade().map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.upgrade().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_borrowed() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakCallableProxy::new_bound(&object)?;

                assert!(reference.upgrade_borrowed().is_some());
                assert!(reference
                    .upgrade_borrowed()
                    .map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.upgrade_borrowed().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakCallableProxy::new_bound(&object)?;

                assert!(reference.get_object().is(&object));

                drop(object);

                assert!(reference.get_object().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object_borrowed() -> PyResult<()> {
            Python::with_gil(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakCallableProxy::new_bound(&object)?;

                assert!(reference.get_object_borrowed().is(&object));

                drop(object);

                assert!(reference.get_object_borrowed().is_none());

                Ok(())
            })
        }
    }

    // under 'abi3-py37' and 'abi3-py38' PyClass cannot be weakreferencable.
    #[cfg(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))))]
    mod pyo3_pyclass {
        use super::*;
        use crate::{pyclass, pymethods, Py};

        #[pyclass(weakref, crate = "crate")]
        struct WeakrefablePyClass {}

        #[pymethods(crate = "crate")]
        impl WeakrefablePyClass {
            fn __call__(&self) -> &str {
                "This class is callable!"
            }
        }

        #[test]
        fn test_weakref_proxy_behavior() -> PyResult<()> {
            Python::with_gil(|py| {
                let object: Bound<'_, WeakrefablePyClass> = Bound::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakCallableProxy::new_bound(&object)?;

                assert!(!reference.is(&object));
                assert!(reference.get_object().is(&object));
                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.get_type().to_string(), CLASS_NAME);

                assert_eq!(
                    reference.getattr("__class__")?.to_string(),
                    "<class 'builtins.WeakrefablePyClass'>"
                );
                check_repr(&reference, object.as_any(), "builtins.WeakrefablePyClass")?;

                assert!(reference
                    .getattr("__callback__")
                    .err()
                    .map_or(false, |err| err.is_instance_of::<PyAttributeError>(py)));

                assert_eq!(reference.call0()?.to_string(), "This class is callable!");

                drop(object);

                assert!(reference.get_object().is_none());
                assert!(reference
                    .getattr("__class__")
                    .err()
                    .map_or(false, |err| err.is_instance_of::<PyReferenceError>(py)));
                check_repr(&reference, py.None().bind(py), "NoneType")?;

                assert!(reference
                    .getattr("__callback__")
                    .err()
                    .map_or(false, |err| err.is_instance_of::<PyReferenceError>(py)));

                assert!(reference
                    .call0()
                    .err()
                    .map_or(false, |err| err.is_instance_of::<PyReferenceError>(py)
                        & (err.value_bound(py).to_string()
                            == "weakly-referenced object no longer exists")));

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_as() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakCallableProxy::new_bound(object.bind(py))?;

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
                let reference = PyWeakCallableProxy::new_bound(object.bind(py))?;

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
        fn test_weakref_upgrade() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakCallableProxy::new_bound(object.bind(py))?;

                assert!(reference.upgrade().is_some());
                assert!(reference.upgrade().map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.upgrade().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_borrowed() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakCallableProxy::new_bound(object.bind(py))?;

                assert!(reference.upgrade_borrowed().is_some());
                assert!(reference
                    .upgrade_borrowed()
                    .map_or(false, |obj| obj.is(&object)));

                drop(object);

                assert!(reference.upgrade_borrowed().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakCallableProxy::new_bound(object.bind(py))?;

                assert!(reference.get_object().is(&object));

                drop(object);

                assert!(reference.get_object().is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_get_object_borrowed() -> PyResult<()> {
            Python::with_gil(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakCallableProxy::new_bound(object.bind(py))?;

                assert!(reference.get_object_borrowed().is(&object));

                drop(object);

                assert!(reference.get_object_borrowed().is_none());

                Ok(())
            })
        }
    }
}
