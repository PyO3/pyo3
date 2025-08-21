use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::type_object::PyTypeCheck;
use crate::types::any::PyAny;
use crate::{ffi, Borrowed, Bound, BoundObject, IntoPyObject, IntoPyObjectExt};

use super::PyWeakrefMethods;

/// Represents any Python `weakref` Proxy type.
///
/// In Python this is created by calling `weakref.proxy`.
/// This is either a `weakref.ProxyType` or a `weakref.CallableProxyType` (`weakref.ProxyTypes`).
#[repr(transparent)]
pub struct PyWeakrefProxy(PyAny);

pyobject_native_type_named!(PyWeakrefProxy);

// TODO: We known the layout but this cannot be implemented, due to the lack of public typeobject pointers. And it is 2 distinct types
// #[cfg(not(Py_LIMITED_API))]
// pyobject_native_type_sized!(PyWeakrefProxy, ffi::PyWeakReference);

impl PyTypeCheck for PyWeakrefProxy {
    const NAME: &'static str = "weakref.ProxyTypes";
    #[cfg(feature = "experimental-inspect")]
    const PYTHON_TYPE: &'static str = "weakref.ProxyType | weakref.CallableProxyType";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        unsafe { ffi::PyWeakref_CheckProxy(object.as_ptr()) > 0 }
    }
}

/// TODO: UPDATE DOCS
impl PyWeakrefProxy {
    /// Constructs a new Weak Reference (`weakref.proxy`/`weakref.ProxyType`/`weakref.CallableProxyType`) for the given object.
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
    /// use pyo3::types::PyWeakrefProxy;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     let foo = Bound::new(py, Foo {})?;
    ///     let weakref = PyWeakrefProxy::new(&foo)?;
    ///     assert!(
    ///         // In normal situations where a direct `Bound<'py, Foo>` is required use `upgrade::<Foo>`
    ///         weakref.upgrade().is_some_and(|obj| obj.is(&foo))
    ///     );
    ///
    ///     let weakref2 = PyWeakrefProxy::new(&foo)?;
    ///     assert!(weakref.is(&weakref2));
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.upgrade().is_none());
    ///     Ok(())
    /// })
    /// # }
    /// ```
    #[inline]
    pub fn new<'py>(object: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyWeakrefProxy>> {
        unsafe {
            Bound::from_owned_ptr_or_err(
                object.py(),
                ffi::PyWeakref_NewProxy(object.as_ptr(), ffi::Py_None()),
            )
            .cast_into_unchecked()
        }
    }

    /// Constructs a new Weak Reference (`weakref.proxy`/`weakref.ProxyType`/`weakref.CallableProxyType`) for the given object with a callback.
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
    /// use pyo3::types::PyWeakrefProxy;
    /// use pyo3::ffi::c_str;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pyfunction]
    /// fn callback(wref: Bound<'_, PyWeakrefProxy>) -> PyResult<()> {
    ///         let py = wref.py();
    ///         assert!(wref.upgrade_as::<Foo>()?.is_none());
    ///         py.run(c_str!("counter = 1"), None, None)
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::attach(|py| {
    ///     py.run(c_str!("counter = 0"), None, None)?;
    ///     assert_eq!(py.eval(c_str!("counter"), None, None)?.extract::<u32>()?, 0);
    ///     let foo = Bound::new(py, Foo{})?;
    ///
    ///     // This is fine.
    ///     let weakref = PyWeakrefProxy::new_with(&foo, py.None())?;
    ///     assert!(weakref.upgrade_as::<Foo>()?.is_some());
    ///     assert!(
    ///         // In normal situations where a direct `Bound<'py, Foo>` is required use `upgrade::<Foo>`
    ///         weakref.upgrade().is_some_and(|obj| obj.is(&foo))
    ///     );
    ///     assert_eq!(py.eval(c_str!("counter"), None, None)?.extract::<u32>()?, 0);
    ///
    ///     let weakref2 = PyWeakrefProxy::new_with(&foo, wrap_pyfunction!(callback, py)?)?;
    ///     assert!(!weakref.is(&weakref2)); // Not the same weakref
    ///     assert!(weakref.eq(&weakref2)?);  // But Equal, since they point to the same object
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.upgrade_as::<Foo>()?.is_none());
    ///     assert_eq!(py.eval(c_str!("counter"), None, None)?.extract::<u32>()?, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ```
    #[inline]
    pub fn new_with<'py, C>(
        object: &Bound<'py, PyAny>,
        callback: C,
    ) -> PyResult<Bound<'py, PyWeakrefProxy>>
    where
        C: IntoPyObject<'py>,
    {
        fn inner<'py>(
            object: &Bound<'py, PyAny>,
            callback: Borrowed<'_, 'py, PyAny>,
        ) -> PyResult<Bound<'py, PyWeakrefProxy>> {
            unsafe {
                Bound::from_owned_ptr_or_err(
                    object.py(),
                    ffi::PyWeakref_NewProxy(object.as_ptr(), callback.as_ptr()),
                )
                .cast_into_unchecked()
            }
        }

        let py = object.py();
        inner(
            object,
            callback
                .into_pyobject_or_pyerr(py)?
                .into_any()
                .as_borrowed(),
        )
    }
}

impl<'py> PyWeakrefMethods<'py> for Bound<'py, PyWeakrefProxy> {
    fn upgrade(&self) -> Option<Bound<'py, PyAny>> {
        let mut obj: *mut ffi::PyObject = std::ptr::null_mut();
        match unsafe { ffi::compat::PyWeakref_GetRef(self.as_ptr(), &mut obj) } {
            std::ffi::c_int::MIN..=-1 => panic!("The 'weakref.ProxyType' (or `weakref.CallableProxyType`) instance should be valid (non-null and actually a weakref reference)"),
            0 => None,
            1..=std::ffi::c_int::MAX => Some(unsafe { obj.assume_owned_unchecked(self.py()) }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::exceptions::{PyAttributeError, PyReferenceError, PyTypeError};
    use crate::types::any::{PyAny, PyAnyMethods};
    use crate::types::weakref::{PyWeakrefMethods, PyWeakrefProxy};
    use crate::{Bound, PyResult, Python};

    #[cfg(all(Py_3_13, not(Py_LIMITED_API)))]
    const DEADREF_FIX: Option<&str> = None;
    #[cfg(all(not(Py_3_13), not(Py_LIMITED_API)))]
    const DEADREF_FIX: Option<&str> = Some("NoneType");

    #[cfg(not(Py_LIMITED_API))]
    fn check_repr(
        reference: &Bound<'_, PyWeakrefProxy>,
        object: &Bound<'_, PyAny>,
        class: Option<&str>,
    ) -> PyResult<()> {
        let repr = reference.repr()?.to_string();

        #[cfg(Py_3_13)]
        let (first_part, second_part) = repr.split_once(';').unwrap();
        #[cfg(not(Py_3_13))]
        let (first_part, second_part) = repr.split_once(" to ").unwrap();

        {
            let (msg, addr) = first_part.split_once("0x").unwrap();

            assert_eq!(msg, "<weakproxy at ");
            assert!(addr
                .to_lowercase()
                .contains(format!("{:x?}", reference.as_ptr()).split_at(2).1));
        }

        if let Some(class) = class.or(DEADREF_FIX) {
            let (msg, addr) = second_part.split_once("0x").unwrap();

            // Avoids not succeeding at unreliable quotation (Python 3.13-dev adds ' around classname without documenting)
            #[cfg(Py_3_13)]
            assert!(msg.starts_with(" to '"));
            assert!(msg.contains(class));
            assert!(msg.ends_with(" at "));

            assert!(addr
                .to_lowercase()
                .contains(format!("{:x?}", object.as_ptr()).split_at(2).1));
        } else {
            assert!(second_part.contains("dead"));
        }

        Ok(())
    }

    mod proxy {
        use super::*;

        #[cfg(all(not(Py_LIMITED_API), Py_3_10))]
        const CLASS_NAME: &str = "'weakref.ProxyType'";
        #[cfg(all(not(Py_LIMITED_API), not(Py_3_10)))]
        const CLASS_NAME: &str = "'weakproxy'";

        mod python_class {
            use super::*;
            use crate::ffi;
            use crate::{py_result_ext::PyResultExt, types::PyDict, types::PyType};
            use std::ptr;

            fn get_type(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
                let globals = PyDict::new(py);
                py.run(ffi::c_str!("class A:\n    pass\n"), Some(&globals), None)?;
                py.eval(ffi::c_str!("A"), Some(&globals), None)
                    .cast_into::<PyType>()
            }

            #[test]
            fn test_weakref_proxy_behavior() -> PyResult<()> {
                Python::attach(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    assert!(!reference.is(&object));
                    assert!(reference.upgrade().unwrap().is(&object));

                    #[cfg(not(Py_LIMITED_API))]
                    assert_eq!(
                        reference.get_type().to_string(),
                        format!("<class {CLASS_NAME}>")
                    );

                    assert_eq!(reference.getattr("__class__")?.to_string(), "<class 'A'>");
                    #[cfg(not(Py_LIMITED_API))]
                    check_repr(&reference, &object, Some("A"))?;

                    assert!(reference
                        .getattr("__callback__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyAttributeError>(py)));

                    assert!(reference.call0().err().is_some_and(|err| {
                        let result = err.is_instance_of::<PyTypeError>(py);
                        #[cfg(not(Py_LIMITED_API))]
                        let result = result
                            & (err.value(py).to_string()
                                == format!("{CLASS_NAME} object is not callable"));
                        result
                    }));

                    drop(object);

                    assert!(reference.upgrade().is_none());
                    assert!(reference
                        .getattr("__class__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyReferenceError>(py)));
                    #[cfg(not(Py_LIMITED_API))]
                    check_repr(&reference, py.None().bind(py), None)?;

                    assert!(reference
                        .getattr("__callback__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyReferenceError>(py)));

                    assert!(reference.call0().err().is_some_and(|err| {
                        let result = err.is_instance_of::<PyTypeError>(py);
                        #[cfg(not(Py_LIMITED_API))]
                        let result = result
                            & (err.value(py).to_string()
                                == format!("{CLASS_NAME} object is not callable"));
                        result
                    }));

                    Ok(())
                })
            }

            #[test]
            fn test_weakref_upgrade_as() -> PyResult<()> {
                Python::attach(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    {
                        // This test is a bit weird but ok.
                        let obj = reference.upgrade_as::<PyAny>();

                        assert!(obj.is_ok());
                        let obj = obj.unwrap();

                        assert!(obj.is_some());
                        assert!(obj.is_some_and(|obj| ptr::eq(obj.as_ptr(), object.as_ptr())
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
            fn test_weakref_upgrade_as_unchecked() -> PyResult<()> {
                Python::attach(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    {
                        // This test is a bit weird but ok.
                        let obj = unsafe { reference.upgrade_as_unchecked::<PyAny>() };

                        assert!(obj.is_some());
                        assert!(obj.is_some_and(|obj| ptr::eq(obj.as_ptr(), object.as_ptr())
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
            fn test_weakref_upgrade() -> PyResult<()> {
                Python::attach(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    assert!(reference.upgrade().is_some());
                    assert!(reference.upgrade().is_some_and(|obj| obj.is(&object)));

                    drop(object);

                    assert!(reference.upgrade().is_none());

                    Ok(())
                })
            }

            #[test]
            fn test_weakref_get_object() -> PyResult<()> {
                Python::attach(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    assert!(reference.upgrade().unwrap().is(&object));

                    drop(object);

                    assert!(reference.upgrade().is_none());

                    Ok(())
                })
            }
        }

        // under 'abi3-py37' and 'abi3-py38' PyClass cannot be weakreferencable.
        #[cfg(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))))]
        mod pyo3_pyclass {
            use super::*;
            use crate::{pyclass, Py};
            use std::ptr;

            #[pyclass(weakref, crate = "crate")]
            struct WeakrefablePyClass {}

            #[test]
            fn test_weakref_proxy_behavior() -> PyResult<()> {
                Python::attach(|py| {
                    let object: Bound<'_, WeakrefablePyClass> =
                        Bound::new(py, WeakrefablePyClass {})?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    assert!(!reference.is(&object));
                    assert!(reference.upgrade().unwrap().is(&object));
                    #[cfg(not(Py_LIMITED_API))]
                    assert_eq!(
                        reference.get_type().to_string(),
                        format!("<class {CLASS_NAME}>")
                    );

                    assert_eq!(
                        reference.getattr("__class__")?.to_string(),
                        "<class 'builtins.WeakrefablePyClass'>"
                    );
                    #[cfg(not(Py_LIMITED_API))]
                    check_repr(&reference, object.as_any(), Some("WeakrefablePyClass"))?;

                    assert!(reference
                        .getattr("__callback__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyAttributeError>(py)));

                    assert!(reference.call0().err().is_some_and(|err| {
                        let result = err.is_instance_of::<PyTypeError>(py);
                        #[cfg(not(Py_LIMITED_API))]
                        let result = result
                            & (err.value(py).to_string()
                                == format!("{CLASS_NAME} object is not callable"));
                        result
                    }));

                    drop(object);

                    assert!(reference.upgrade().is_none());
                    assert!(reference
                        .getattr("__class__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyReferenceError>(py)));
                    #[cfg(not(Py_LIMITED_API))]
                    check_repr(&reference, py.None().bind(py), None)?;

                    assert!(reference
                        .getattr("__callback__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyReferenceError>(py)));

                    assert!(reference.call0().err().is_some_and(|err| {
                        let result = err.is_instance_of::<PyTypeError>(py);
                        #[cfg(not(Py_LIMITED_API))]
                        let result = result
                            & (err.value(py).to_string()
                                == format!("{CLASS_NAME} object is not callable"));
                        result
                    }));

                    Ok(())
                })
            }

            #[test]
            fn test_weakref_upgrade_as() -> PyResult<()> {
                Python::attach(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = PyWeakrefProxy::new(object.bind(py))?;

                    {
                        let obj = reference.upgrade_as::<WeakrefablePyClass>();

                        assert!(obj.is_ok());
                        let obj = obj.unwrap();

                        assert!(obj.is_some());
                        assert!(obj.is_some_and(|obj| ptr::eq(obj.as_ptr(), object.as_ptr())));
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
            fn test_weakref_upgrade_as_unchecked() -> PyResult<()> {
                Python::attach(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = PyWeakrefProxy::new(object.bind(py))?;

                    {
                        let obj = unsafe { reference.upgrade_as_unchecked::<WeakrefablePyClass>() };

                        assert!(obj.is_some());
                        assert!(obj.is_some_and(|obj| ptr::eq(obj.as_ptr(), object.as_ptr())));
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
            fn test_weakref_upgrade() -> PyResult<()> {
                Python::attach(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = PyWeakrefProxy::new(object.bind(py))?;

                    assert!(reference.upgrade().is_some());
                    assert!(reference.upgrade().is_some_and(|obj| obj.is(&object)));

                    drop(object);

                    assert!(reference.upgrade().is_none());

                    Ok(())
                })
            }
        }
    }

    mod callable_proxy {
        use super::*;

        #[cfg(all(not(Py_LIMITED_API), Py_3_10))]
        const CLASS_NAME: &str = "<class 'weakref.CallableProxyType'>";
        #[cfg(all(not(Py_LIMITED_API), not(Py_3_10)))]
        const CLASS_NAME: &str = "<class 'weakcallableproxy'>";

        mod python_class {
            use super::*;
            use crate::ffi;
            use crate::{py_result_ext::PyResultExt, types::PyDict, types::PyType};
            use std::ptr;

            fn get_type(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
                let globals = PyDict::new(py);
                py.run(
                    ffi::c_str!("class A:\n    def __call__(self):\n        return 'This class is callable!'\n"),
                    Some(&globals),
                    None,
                )?;
                py.eval(ffi::c_str!("A"), Some(&globals), None)
                    .cast_into::<PyType>()
            }

            #[test]
            fn test_weakref_proxy_behavior() -> PyResult<()> {
                Python::attach(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    assert!(!reference.is(&object));
                    assert!(reference.upgrade().unwrap().is(&object));
                    #[cfg(not(Py_LIMITED_API))]
                    assert_eq!(reference.get_type().to_string(), CLASS_NAME);

                    assert_eq!(reference.getattr("__class__")?.to_string(), "<class 'A'>");
                    #[cfg(not(Py_LIMITED_API))]
                    check_repr(&reference, &object, Some("A"))?;

                    assert!(reference
                        .getattr("__callback__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyAttributeError>(py)));

                    assert_eq!(reference.call0()?.to_string(), "This class is callable!");

                    drop(object);

                    assert!(reference.upgrade().is_none());
                    assert!(reference
                        .getattr("__class__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyReferenceError>(py)));
                    #[cfg(not(Py_LIMITED_API))]
                    check_repr(&reference, py.None().bind(py), None)?;

                    assert!(reference
                        .getattr("__callback__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyReferenceError>(py)));

                    assert!(reference
                        .call0()
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyReferenceError>(py)
                            & (err.value(py).to_string()
                                == "weakly-referenced object no longer exists")));

                    Ok(())
                })
            }

            #[test]
            fn test_weakref_upgrade_as() -> PyResult<()> {
                Python::attach(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    {
                        // This test is a bit weird but ok.
                        let obj = reference.upgrade_as::<PyAny>();

                        assert!(obj.is_ok());
                        let obj = obj.unwrap();

                        assert!(obj.is_some());
                        assert!(obj.is_some_and(|obj| ptr::eq(obj.as_ptr(), object.as_ptr())
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
            fn test_weakref_upgrade_as_unchecked() -> PyResult<()> {
                Python::attach(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    {
                        // This test is a bit weird but ok.
                        let obj = unsafe { reference.upgrade_as_unchecked::<PyAny>() };

                        assert!(obj.is_some());
                        assert!(obj.is_some_and(|obj| ptr::eq(obj.as_ptr(), object.as_ptr())
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
            fn test_weakref_upgrade() -> PyResult<()> {
                Python::attach(|py| {
                    let class = get_type(py)?;
                    let object = class.call0()?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    assert!(reference.upgrade().is_some());
                    assert!(reference.upgrade().is_some_and(|obj| obj.is(&object)));

                    drop(object);

                    assert!(reference.upgrade().is_none());

                    Ok(())
                })
            }
        }

        // under 'abi3-py37' and 'abi3-py38' PyClass cannot be weakreferencable.
        #[cfg(all(feature = "macros", not(all(Py_LIMITED_API, not(Py_3_9)))))]
        mod pyo3_pyclass {
            use super::*;
            use crate::{pyclass, pymethods, Py};
            use std::ptr;

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
                Python::attach(|py| {
                    let object: Bound<'_, WeakrefablePyClass> =
                        Bound::new(py, WeakrefablePyClass {})?;
                    let reference = PyWeakrefProxy::new(&object)?;

                    assert!(!reference.is(&object));
                    assert!(reference.upgrade().unwrap().is(&object));
                    #[cfg(not(Py_LIMITED_API))]
                    assert_eq!(reference.get_type().to_string(), CLASS_NAME);

                    assert_eq!(
                        reference.getattr("__class__")?.to_string(),
                        "<class 'builtins.WeakrefablePyClass'>"
                    );
                    #[cfg(not(Py_LIMITED_API))]
                    check_repr(&reference, object.as_any(), Some("WeakrefablePyClass"))?;

                    assert!(reference
                        .getattr("__callback__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyAttributeError>(py)));

                    assert_eq!(reference.call0()?.to_string(), "This class is callable!");

                    drop(object);

                    assert!(reference.upgrade().is_none());
                    assert!(reference
                        .getattr("__class__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyReferenceError>(py)));
                    #[cfg(not(Py_LIMITED_API))]
                    check_repr(&reference, py.None().bind(py), None)?;

                    assert!(reference
                        .getattr("__callback__")
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyReferenceError>(py)));

                    assert!(reference
                        .call0()
                        .err()
                        .is_some_and(|err| err.is_instance_of::<PyReferenceError>(py)
                            & (err.value(py).to_string()
                                == "weakly-referenced object no longer exists")));

                    Ok(())
                })
            }

            #[test]
            fn test_weakref_upgrade_as() -> PyResult<()> {
                Python::attach(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = PyWeakrefProxy::new(object.bind(py))?;

                    {
                        let obj = reference.upgrade_as::<WeakrefablePyClass>();

                        assert!(obj.is_ok());
                        let obj = obj.unwrap();

                        assert!(obj.is_some());
                        assert!(obj.is_some_and(|obj| ptr::eq(obj.as_ptr(), object.as_ptr())));
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
            fn test_weakref_upgrade_as_unchecked() -> PyResult<()> {
                Python::attach(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = PyWeakrefProxy::new(object.bind(py))?;

                    {
                        let obj = unsafe { reference.upgrade_as_unchecked::<WeakrefablePyClass>() };

                        assert!(obj.is_some());
                        assert!(obj.is_some_and(|obj| ptr::eq(obj.as_ptr(), object.as_ptr())));
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
            fn test_weakref_upgrade() -> PyResult<()> {
                Python::attach(|py| {
                    let object = Py::new(py, WeakrefablePyClass {})?;
                    let reference = PyWeakrefProxy::new(object.bind(py))?;

                    assert!(reference.upgrade().is_some());
                    assert!(reference.upgrade().is_some_and(|obj| obj.is(&object)));

                    drop(object);

                    assert!(reference.upgrade().is_none());

                    Ok(())
                })
            }
        }
    }
}
