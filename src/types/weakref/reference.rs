use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::types::any::PyAny;
use crate::{ffi, Borrowed, Bound, BoundObject, IntoPyObject, IntoPyObjectExt};

#[cfg(any(PyPy, GraalPy, Py_LIMITED_API))]
use crate::type_object::PyTypeCheck;

use super::PyWeakrefMethods;

/// Represents a Python `weakref.ReferenceType`.
///
/// In Python this is created by calling `weakref.ref`.
#[repr(transparent)]
pub struct PyWeakrefReference(PyAny);

#[cfg(not(any(PyPy, GraalPy, Py_LIMITED_API)))]
pyobject_subclassable_native_type!(PyWeakrefReference, crate::ffi::PyWeakReference);

#[cfg(not(any(PyPy, GraalPy, Py_LIMITED_API)))]
pyobject_native_type!(
    PyWeakrefReference,
    ffi::PyWeakReference,
    // TODO: should not be depending on a private symbol here!
    pyobject_native_static_type_object!(ffi::_PyWeakref_RefType),
    #module=Some("weakref"),
    #checkfunction=ffi::PyWeakref_CheckRefExact
);

// When targetting alternative or multiple interpreters, it is better to not use the internal API.
#[cfg(any(PyPy, GraalPy, Py_LIMITED_API))]
pyobject_native_type_named!(PyWeakrefReference);

#[cfg(any(PyPy, GraalPy, Py_LIMITED_API))]
impl PyTypeCheck for PyWeakrefReference {
    const NAME: &'static str = "weakref.ReferenceType";
    #[cfg(feature = "experimental-inspect")]
    const PYTHON_TYPE: &'static str = "weakref.ReferenceType";

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
    /// Python::attach(|py| {
    ///     let foo = Bound::new(py, Foo {})?;
    ///     let weakref = PyWeakrefReference::new(&foo)?;
    ///     assert!(
    ///         // In normal situations where a direct `Bound<'py, Foo>` is required use `upgrade::<Foo>`
    ///         weakref.upgrade().is_some_and(|obj| obj.is(&foo))
    ///     );
    ///
    ///     let weakref2 = PyWeakrefReference::new(&foo)?;
    ///     assert!(weakref.is(&weakref2));
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.upgrade().is_none());
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn new<'py>(object: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyWeakrefReference>> {
        unsafe {
            Bound::from_owned_ptr_or_err(
                object.py(),
                ffi::PyWeakref_NewRef(object.as_ptr(), ffi::Py_None()),
            )
            .cast_into_unchecked()
        }
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
    /// use pyo3::ffi::c_str;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pyfunction]
    /// fn callback(wref: Bound<'_, PyWeakrefReference>) -> PyResult<()> {
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
    ///     let weakref = PyWeakrefReference::new_with(&foo, py.None())?;
    ///     assert!(weakref.upgrade_as::<Foo>()?.is_some());
    ///     assert!(
    ///         // In normal situations where a direct `Bound<'py, Foo>` is required use `upgrade::<Foo>`
    ///         weakref.upgrade().is_some_and(|obj| obj.is(&foo))
    ///     );
    ///     assert_eq!(py.eval(c_str!("counter"), None, None)?.extract::<u32>()?, 0);
    ///
    ///     let weakref2 = PyWeakrefReference::new_with(&foo, wrap_pyfunction!(callback, py)?)?;
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
    pub fn new_with<'py, C>(
        object: &Bound<'py, PyAny>,
        callback: C,
    ) -> PyResult<Bound<'py, PyWeakrefReference>>
    where
        C: IntoPyObject<'py>,
    {
        fn inner<'py>(
            object: &Bound<'py, PyAny>,
            callback: Borrowed<'_, 'py, PyAny>,
        ) -> PyResult<Bound<'py, PyWeakrefReference>> {
            unsafe {
                Bound::from_owned_ptr_or_err(
                    object.py(),
                    ffi::PyWeakref_NewRef(object.as_ptr(), callback.as_ptr()),
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

impl<'py> PyWeakrefMethods<'py> for Bound<'py, PyWeakrefReference> {
    fn upgrade(&self) -> Option<Bound<'py, PyAny>> {
        let mut obj: *mut ffi::PyObject = std::ptr::null_mut();
        match unsafe { ffi::compat::PyWeakref_GetRef(self.as_ptr(), &mut obj) } {
            std::ffi::c_int::MIN..=-1 => panic!("The 'weakref.ReferenceType' instance should be valid (non-null and actually a weakref reference)"),
            0 => None,
            1..=std::ffi::c_int::MAX => Some(unsafe { obj.assume_owned_unchecked(self.py()) }),
        }
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

                // Avoid testing on reprs directly since they the quoting and full path vs class name tends to be changedi undocumented.
                assert!(msg.starts_with("to '"));
                assert!(msg.contains(class));
                assert!(msg.ends_with("' at "));

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
        use crate::ffi;
        use crate::{py_result_ext::PyResultExt, types::PyType};
        use std::ptr;

        fn get_type(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
            py.run(ffi::c_str!("class A:\n    pass\n"), None, None)?;
            py.eval(ffi::c_str!("A"), None, None).cast_into::<PyType>()
        }

        #[test]
        fn test_weakref_reference_behavior() -> PyResult<()> {
            Python::attach(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakrefReference::new(&object)?;

                assert!(!reference.is(&object));
                assert!(reference.upgrade().unwrap().is(&object));

                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.get_type().to_string(), CLASS_NAME);

                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.getattr("__class__")?.to_string(), CLASS_NAME);

                #[cfg(not(Py_LIMITED_API))]
                check_repr(&reference, Some((object.as_any(), "A")))?;

                assert!(reference
                    .getattr("__callback__")
                    .is_ok_and(|result| result.is_none()));

                assert!(reference.call0()?.is(&object));

                drop(object);

                assert!(reference.upgrade().is_none());
                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.getattr("__class__")?.to_string(), CLASS_NAME);
                check_repr(&reference, None)?;

                assert!(reference
                    .getattr("__callback__")
                    .is_ok_and(|result| result.is_none()));

                assert!(reference.call0()?.is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_as() -> PyResult<()> {
            Python::attach(|py| {
                let class = get_type(py)?;
                let object = class.call0()?;
                let reference = PyWeakrefReference::new(&object)?;

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
                let reference = PyWeakrefReference::new(&object)?;

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
                let reference = PyWeakrefReference::new(&object)?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.upgrade().is_some());
                assert!(reference.upgrade().is_some_and(|obj| obj.is(&object)));

                drop(object);

                assert!(reference.call0()?.is_none());
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
        fn test_weakref_reference_behavior() -> PyResult<()> {
            Python::attach(|py| {
                let object: Bound<'_, WeakrefablePyClass> = Bound::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new(&object)?;

                assert!(!reference.is(&object));
                assert!(reference.upgrade().unwrap().is(&object));
                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.get_type().to_string(), CLASS_NAME);

                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.getattr("__class__")?.to_string(), CLASS_NAME);
                #[cfg(not(Py_LIMITED_API))]
                check_repr(&reference, Some((object.as_any(), "WeakrefablePyClass")))?;

                assert!(reference
                    .getattr("__callback__")
                    .is_ok_and(|result| result.is_none()));

                assert!(reference.call0()?.is(&object));

                drop(object);

                assert!(reference.upgrade().is_none());
                #[cfg(not(Py_LIMITED_API))]
                assert_eq!(reference.getattr("__class__")?.to_string(), CLASS_NAME);
                check_repr(&reference, None)?;

                assert!(reference
                    .getattr("__callback__")
                    .is_ok_and(|result| result.is_none()));

                assert!(reference.call0()?.is_none());

                Ok(())
            })
        }

        #[test]
        fn test_weakref_upgrade_as() -> PyResult<()> {
            Python::attach(|py| {
                let object = Py::new(py, WeakrefablePyClass {})?;
                let reference = PyWeakrefReference::new(object.bind(py))?;

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
                let reference = PyWeakrefReference::new(object.bind(py))?;

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
                let reference = PyWeakrefReference::new(object.bind(py))?;

                assert!(reference.call0()?.is(&object));
                assert!(reference.upgrade().is_some());
                assert!(reference.upgrade().is_some_and(|obj| obj.is(&object)));

                drop(object);

                assert!(reference.call0()?.is_none());
                assert!(reference.upgrade().is_none());

                Ok(())
            })
        }
    }
}
