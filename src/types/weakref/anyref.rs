use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::type_object::{PyTypeCheck, PyTypeInfo};
use crate::types::any::PyAny;
use crate::{ffi, Borrowed, Bound, PyNativeType};

use super::PyWeakRefMethods;

/// Represents any Python `weakref` reference.
///
/// In Python this is created by calling `weakref.ref` or `weakref.proxy`.
#[repr(transparent)]
pub struct PyWeakref(PyAny);

pyobject_native_type_named!(PyWeakref);
pyobject_native_type_extract!(PyWeakref);

// TODO: We known the layout but this cannot be implemented, due to the lack of public typeobject pointers
// #[cfg(not(Py_LIMITED_API))]
// pyobject_native_type_sized!(PyWeakref, ffi::PyWeakReference);

impl PyTypeCheck for PyWeakref {
    const NAME: &'static str = "weakref";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        unsafe { ffi::PyWeakref_Check(object.as_ptr()) > 0 }
    }
}

impl PyWeakref {
    // TODO: MAYBE ADD CREATION METHODS OR EASY CASTING?;

    /// Upgrade the weakref to a direct object reference.
    ///
    /// It is named `upgrade` to be inline with [rust's `Weak::upgrade`](std::rc::Weak::upgrade).
    /// In Python it would be equivalent to [`PyWeakref_GetObject`] or retrieving the Object from Python.
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
    /// use pyo3::types::{PyWeakref, PyWeakrefProxy};
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
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakref>) -> PyResult<String> {
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
    ///     let proxy = PyWeakrefProxy::new_bound(&data)?; // Retrieve this as an PyMethods argument.
    ///     let reference = proxy.downcast::<PyWeakref>()?;
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

    /// Upgrade the weakref to a direct object reference unchecked. The type of the recovered object is not checked before downcasting, this could lead to unexpected behavior. Use only when absolutely certain the type can be guaranteed. The `weakref` may still return `None`.
    ///
    /// It is named `upgrade` to be inline with [rust's `Weak::upgrade`](std::rc::Weak::upgrade).
    /// In Python it would be equivalent to [`PyWeakref_GetObject`] or retrieving the Object from Python.
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
    /// use pyo3::types::{PyWeakref, PyWeakrefProxy};
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
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakref>) -> String {
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
    ///     let proxy = PyWeakrefProxy::new_bound(&data)?; // Retrieve this as an PyMethods argument.
    ///     let reference = proxy.downcast::<PyWeakref>()?;
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
    #[track_caller]
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
    /// In Python it would be equivalent to [`PyWeakref_GetObject`] or retrieving the Object from Python.
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
    /// use pyo3::types::{PyWeakref, PyWeakrefProxy};
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
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakref>) -> PyResult<String> {
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
    ///     let proxy = PyWeakrefProxy::new_bound(&data)?; // Retrieve this as an PyMethods argument.  
    ///     let reference = proxy.downcast::<PyWeakref>()?;
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
    /// This function gets the optional target of this [`PyWeakref`] (Any Python `weakref` weakreference).
    /// It produces similair results as using [`PyWeakref_GetObject`] in the C api or retrieving the Object from Python.
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
    /// use pyo3::types::{PyWeakref, PyWeakrefProxy};
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// fn parse_data(reference: Borrowed<'_, '_, PyWeakref>) -> PyResult<String> {
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
    ///     let proxy = PyWeakrefProxy::new_bound(&data)?; // Retrieve this as an PyMethods argument.
    ///     let reference = proxy.downcast::<PyWeakref>()?;
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
    #[track_caller]
    pub fn upgrade(&self) -> Option<&'_ PyAny> {
        self.as_borrowed().upgrade().map(Bound::into_gil_ref)
    }

    /// Retrieve to a object pointed to by the weakref.
    ///
    /// This function returns `&'py PyAny`, which is either the object if it still exists, otherwise it will refer to [`PyNone`](crate::types::none::PyNone).
    ///
    /// This function gets the optional target of this [`PyWeakref`] (Any Python `weakref` weakreference).
    /// It produces similair results as using [`PyWeakref_GetObject`] in the C api or retrieving the Object from Python.
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
    /// use pyo3::types::{PyWeakref, PyWeakrefProxy};
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// fn get_class(reference: Borrowed<'_, '_, PyWeakref>) -> PyResult<String> {
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
    ///     let proxy = PyWeakrefProxy::new_bound(&object)?; // Retrieve this as an PyMethods argument.
    ///     let reference = proxy.downcast::<PyWeakref>()?;
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
    #[track_caller]
    pub fn get_object(&self) -> &'_ PyAny {
        self.as_borrowed().get_object().into_gil_ref()
    }
}

impl<'py> PyWeakRefMethods<'py> for Bound<'py, PyWeakref> {
    fn get_object_borrowed(&self) -> Borrowed<'_, 'py, PyAny> {
        // PyWeakref_GetObject does some error checking, however we ensure the passed object is Non-Null and a Weakref type.
        unsafe { ffi::PyWeakref_GetObject(self.as_ptr()).assume_borrowed_or_err(self.py()) }
             .expect("The 'weakref' weak reference instance should be valid (non-null and actually a weakref reference)")
    }
}

#[cfg(test)]
mod tests {
    use crate::types::any::{PyAny, PyAnyMethods};
    use crate::types::weakref::{PyWeakRef, PyWeakRefMethods, PyWeakref, PyWeakrefProxy};
    use crate::{Bound, PyResult, Python};

    fn new_reference<'py>(object: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyWeakref>> {
        let reference = PyWeakRef::new_bound(object)?;
        reference.into_any().downcast_into().map_err(Into::into)
    }

    fn new_proxy<'py>(object: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyWeakref>> {
        let reference = PyWeakrefProxy::new_bound(object)?;
        reference.into_any().downcast_into().map_err(Into::into)
    }

    mod python_class {
        use super::*;
        use crate::{py_result_ext::PyResultExt, types::PyType};

        fn get_type(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
            py.run_bound("class A:\n    pass\n", None, None)?;
            py.eval_bound("A", None, None).downcast_into::<PyType>()
        }

        #[test]
        fn test_weakref_upgrade_as() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
            ) -> PyResult<()> {
                Python::with_gil(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = create_reference(&object)?;

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

            inner(new_reference)?;
            inner(new_proxy)
        }

        #[test]
        fn test_weakref_upgrade_borrowed_as() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
            ) -> PyResult<()> {
                Python::with_gil(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = create_reference(&object)?;

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

            inner(new_reference)?;
            inner(new_proxy)
        }

        #[test]
        fn test_weakref_upgrade_as_unchecked() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
            ) -> PyResult<()> {
                Python::with_gil(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = create_reference(&object)?;

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

            inner(new_reference)?;
            inner(new_proxy)
        }

        #[test]
        fn test_weakref_upgrade_borrowed_as_unchecked() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
            ) -> PyResult<()> {
                Python::with_gil(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = create_reference(&object)?;

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

            inner(new_reference)?;
            inner(new_proxy)
        }

        #[test]
        fn test_weakref_upgrade() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
                call_retrievable: bool,
            ) -> PyResult<()> {
                let not_call_retrievable = !call_retrievable;

                Python::with_gil(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = create_reference(&object)?;

                    assert!(not_call_retrievable || reference.call0()?.is(&object));
                    assert!(reference.upgrade().is_some());
                    assert!(reference.upgrade().map_or(false, |obj| obj.is(&object)));

                    drop(object);

                    assert!(not_call_retrievable || reference.call0()?.is_none());
                    assert!(reference.upgrade().is_none());

                    Ok(())
                })
            }

            inner(new_reference, true)?;
            inner(new_proxy, false)
        }

        #[test]
        fn test_weakref_upgrade_borrowed() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
                call_retrievable: bool,
            ) -> PyResult<()> {
                let not_call_retrievable = !call_retrievable;

                Python::with_gil(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = create_reference(&object)?;

                    assert!(not_call_retrievable || reference.call0()?.is(&object));
                    assert!(reference.upgrade_borrowed().is_some());
                    assert!(reference
                        .upgrade_borrowed()
                        .map_or(false, |obj| obj.is(&object)));

                    drop(object);

                    assert!(not_call_retrievable || reference.call0()?.is_none());
                    assert!(reference.upgrade_borrowed().is_none());

                    Ok(())
                })
            }

            inner(new_reference, true)?;
            inner(new_proxy, false)
        }

        #[test]
        fn test_weakref_get_object() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
                call_retrievable: bool,
            ) -> PyResult<()> {
                let not_call_retrievable = !call_retrievable;

                Python::with_gil(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = create_reference(&object)?;

                    assert!(not_call_retrievable || reference.call0()?.is(&object));
                    assert!(reference.get_object().is(&object));

                    drop(object);

                    assert!(not_call_retrievable || reference.call0()?.is(&reference.get_object()));
                    assert!(not_call_retrievable || reference.call0()?.is_none());
                    assert!(reference.get_object().is_none());

                    Ok(())
                })
            }

            inner(new_reference, true)?;
            inner(new_proxy, false)
        }

        #[test]
        fn test_weakref_get_object_borrowed() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
                call_retrievable: bool,
            ) -> PyResult<()> {
                let not_call_retrievable = !call_retrievable;

                Python::with_gil(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = create_reference(&object)?;

                    assert!(not_call_retrievable || reference.call0()?.is(&object));
                    assert!(reference.get_object_borrowed().is(&object));

                    drop(object);

                    assert!(not_call_retrievable || reference.call0()?.is_none());
                    assert!(reference.get_object_borrowed().is_none());

                    Ok(())
                })
            }

            inner(new_reference, true)?;
            inner(new_proxy, false)
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
        fn test_weakref_upgrade_as() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
            ) -> PyResult<()> {
                Python::with_gil(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = create_reference(object.bind(py))?;

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

            inner(new_reference)?;
            inner(new_proxy)
        }

        #[test]
        fn test_weakref_upgrade_borrowed_as() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
            ) -> PyResult<()> {
                Python::with_gil(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = create_reference(object.bind(py))?;

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

            inner(new_reference)?;
            inner(new_proxy)
        }

        #[test]
        fn test_weakref_upgrade_as_unchecked() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
            ) -> PyResult<()> {
                Python::with_gil(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = create_reference(object.bind(py))?;

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

            inner(new_reference)?;
            inner(new_proxy)
        }

        #[test]
        fn test_weakref_upgrade_borrowed_as_unchecked() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
            ) -> PyResult<()> {
                Python::with_gil(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = create_reference(object.bind(py))?;

                    {
                        let obj = unsafe {
                            reference.upgrade_borrowed_as_unchecked::<WeakrefablePyClass>()
                        };

                        assert!(obj.is_some());
                        assert!(obj.map_or(false, |obj| obj.as_ptr() == object.as_ptr()));
                    }

                    drop(object);

                    {
                        let obj = unsafe {
                            reference.upgrade_borrowed_as_unchecked::<WeakrefablePyClass>()
                        };

                        assert!(obj.is_none());
                    }

                    Ok(())
                })
            }

            inner(new_reference)?;
            inner(new_proxy)
        }

        #[test]
        fn test_weakref_upgrade() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
                call_retrievable: bool,
            ) -> PyResult<()> {
                let not_call_retrievable = !call_retrievable;

                Python::with_gil(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = create_reference(object.bind(py))?;

                    assert!(not_call_retrievable || reference.call0()?.is(&object));
                    assert!(reference.upgrade().is_some());
                    assert!(reference.upgrade().map_or(false, |obj| obj.is(&object)));

                    drop(object);

                    assert!(not_call_retrievable || reference.call0()?.is_none());
                    assert!(reference.upgrade().is_none());

                    Ok(())
                })
            }

            inner(new_reference, true)?;
            inner(new_proxy, false)
        }

        #[test]
        fn test_weakref_upgrade_borrowed() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
                call_retrievable: bool,
            ) -> PyResult<()> {
                let not_call_retrievable = !call_retrievable;

                Python::with_gil(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = create_reference(object.bind(py))?;

                    assert!(not_call_retrievable || reference.call0()?.is(&object));
                    assert!(reference.upgrade_borrowed().is_some());
                    assert!(reference
                        .upgrade_borrowed()
                        .map_or(false, |obj| obj.is(&object)));

                    drop(object);

                    assert!(not_call_retrievable || reference.call0()?.is_none());
                    assert!(reference.upgrade_borrowed().is_none());

                    Ok(())
                })
            }

            inner(new_reference, true)?;
            inner(new_proxy, false)
        }

        #[test]
        fn test_weakref_get_object() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
                call_retrievable: bool,
            ) -> PyResult<()> {
                let not_call_retrievable = !call_retrievable;

                Python::with_gil(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = create_reference(object.bind(py))?;

                    assert!(not_call_retrievable || reference.call0()?.is(&object));
                    assert!(reference.get_object().is(&object));

                    drop(object);

                    assert!(not_call_retrievable || reference.call0()?.is(&reference.get_object()));
                    assert!(not_call_retrievable || reference.call0()?.is_none());
                    assert!(reference.get_object().is_none());

                    Ok(())
                })
            }

            inner(new_reference, true)?;
            inner(new_proxy, false)
        }

        #[test]
        fn test_weakref_get_object_borrowed() -> PyResult<()> {
            fn inner(
                create_reference: impl for<'py> FnOnce(
                    &Bound<'py, PyAny>,
                )
                    -> PyResult<Bound<'py, PyWeakref>>,
                call_retrievable: bool,
            ) -> PyResult<()> {
                let not_call_retrievable = !call_retrievable;

                Python::with_gil(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = create_reference(object.bind(py))?;

                    assert!(not_call_retrievable || reference.call0()?.is(&object));
                    assert!(reference.get_object_borrowed().is(&object));

                    drop(object);

                    assert!(not_call_retrievable || reference.call0()?.is_none());
                    assert!(reference.get_object_borrowed().is_none());

                    Ok(())
                })
            }

            inner(new_reference, true)?;
            inner(new_proxy, false)
        }
    }
}
