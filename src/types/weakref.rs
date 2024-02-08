use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::type_object::PyTypeCheck;
use crate::types::any::PyAnyMethods;
use crate::{ffi, Borrowed, Bound, PyAny, PyNativeType, Python, ToPyObject};

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
    #checkfunction=ffi::PyWeakref_CheckRef
);

impl PyWeakRef {
    /// Deprecated form of [`PyWeakRef::new_bound`].
    #[inline]
    #[track_caller]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyWeakRef::new` will be replaced by `PyWeakRef::new_bound` in a future PyO3 version"
        )
    )]
    pub fn new<T>(py: Python<'_>, object: T) -> PyResult<&'_ PyWeakRef>
    where
        T: ToPyObject,
    {
        Self::new_bound(py, object).map(Bound::into_gil_ref)
    }

    /// Constructs a new Weak Reference (weakref.ref) for the given object.
    ///
    /// Returns a `TypeError` if `object` is not subclassable (Most native types and PyClasses without `weakref` flag).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let foo = Py::new(py, Foo {})?;
    ///     let weakref = PyWeakRef::new_bound(py, foo.clone_ref(py))?;
    ///     assert!(weakref.call0()?.is(&foo));
    ///     
    ///     let weakref2 = PyWeakRef::new_bound(py, foo.clone_ref(py))?;
    ///     assert!(weakref.is(&weakref2));
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.call0()?.is(&py.None()));
    ///     Ok(())
    /// })
    /// # }
    /// ````
    #[track_caller]
    pub fn new_bound<T>(py: Python<'_>, object: T) -> PyResult<Bound<'_, PyWeakRef>>
    where
        T: ToPyObject,
    {
        unsafe {
            Bound::from_owned_ptr_or_err(
                py,
                ffi::PyWeakref_NewRef(object.to_object(py).as_ptr(), ffi::Py_None()),
            )
            .map(|obj| obj.downcast_into_unchecked())
        }
    }

    #[inline]
    #[track_caller]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyWeakRef::new_with` will be replaced by `PyWeakRef::new_bound_with` in a future PyO3 version"
        )
    )]
    pub fn new_with<'py, T, C>(py: Python<'py>, object: T, callback: C) -> PyResult<&'py PyWeakRef>
    where
        T: ToPyObject,
        C: ToPyObject,
    {
        Self::new_bound_with(py, object, callback).map(Bound::into_gil_ref)
    }

    /// Constructs a new Weak Reference (weakref.ref) for the given object with a callback.
    ///
    /// Returns a `TypeError` if `object` is not subclassable (Most native types and PyClasses without `weakref` flag) or if the `callback` is not callable or None.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyWeakRef;
    ///
    /// #[pyclass(weakref)]
    /// struct Foo { /* fields omitted */ }
    ///
    /// #[pymethods]
    /// impl Foo {
    ///     #[staticmethod]
    ///     fn finalizer(slf_ref: Bound<'_, PyWeakRef>) -> PyResult<()> {
    ///         let py = slf_ref.py();
    ///         assert!(slf_ref.call0()?.is(&py.None()));
    ///         py.run("counter = 1", None, None)?;
    ///         Ok(())
    ///     }
    /// }
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     py.run("counter = 0", None, None)?;
    ///     assert_eq!(py.eval("counter", None, None)?.extract::<u32>()?, 0);
    ///     let foo = Bound::new(py, Foo { } )?;
    ///
    ///     // This is fine.
    ///     let weakref = PyWeakRef::new_bound_with(py, foo.clone(), py.None())?;
    ///     assert!(weakref.call0()?.is(&foo));
    ///     assert_eq!(py.eval("counter", None, None)?.extract::<u32>()?, 0);
    ///     
    ///     let weakref2 = PyWeakRef::new_bound_with(py, foo.clone(), py.get_type::<Foo>().getattr("finalizer")?)?;
    ///     assert!(!weakref.is(&weakref2)); // Not the same weakref
    ///     assert!(weakref.eq(&weakref2)?);  // But Equal, since they point to the same object
    ///
    ///     drop(foo);
    ///
    ///     assert!(weakref.call0()?.is(&py.None()));
    ///     assert_eq!(py.eval("counter", None, None)?.extract::<u32>()?, 1);
    ///     Ok(())
    /// })
    /// # }
    /// ````
    #[track_caller]
    pub fn new_bound_with<'py, T, C>(
        py: Python<'py>,
        object: T,
        callback: C,
    ) -> PyResult<Bound<'py, PyWeakRef>>
    where
        T: ToPyObject,
        C: ToPyObject,
    {
        unsafe {
            Bound::from_owned_ptr_or_err(
                py,
                ffi::PyWeakref_NewRef(
                    object.to_object(py).as_ptr(),
                    callback.to_object(py).as_ptr(),
                ),
            )
            .map(|obj| obj.downcast_into_unchecked())
        }
    }

    pub fn upgrade<T>(&self) -> PyResult<Option<&T::AsRefTarget>>
    where
        T: PyTypeCheck,
    {
        Ok(self.as_borrowed().upgrade::<T>()?.map(Bound::into_gil_ref))
    }

    pub fn get_object(&self) -> PyResult<Option<&'_ PyAny>> {
        Ok(self.as_borrowed().get_object()?.map(Bound::into_gil_ref))
    }

    pub fn get_object_raw(&self) -> PyResult<&'_ PyAny> {
        self.as_borrowed().get_object_raw().map(Bound::into_gil_ref)
    }
}

/// Implementation of functionality for [`PyWeakRef`].
///
/// These methods are defined for the `Bound<'py, PyWeakRef>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyWeakRef")]
pub trait PyWeakRefMethods<'py> {
    // Named it upgrade to allign with rust's Weak::upgrade.
    fn upgrade<T>(&self) -> PyResult<Option<Bound<'py, T>>>
    where
        T: PyTypeCheck;
    // fn borrowed_upgrade<T: PyTypeCheck>(&self) -> PyResult<Option<Borrowed<'_, 'py, T>>>;

    // TODO: NAMING
    // maybe upgrade_any
    fn get_object(&self) -> PyResult<Option<Bound<'py, PyAny>>>;
    // maybe upgrade_any_borrowed
    fn borrow_object(&self) -> PyResult<Option<Borrowed<'_, 'py, PyAny>>>;

    // TODO: NAMING
    // get_any
    fn get_object_raw(&self) -> PyResult<Bound<'py, PyAny>>;
    // get_any_borrowed
    fn borrow_object_raw(&self) -> PyResult<Borrowed<'_, 'py, PyAny>>;
}

impl<'py> PyWeakRefMethods<'py> for Bound<'py, PyWeakRef> {
    fn upgrade<T>(&self) -> PyResult<Option<Bound<'py, T>>>
    where
        T: PyTypeCheck,
    {
        Ok(self.get_object()?.map(|obj| obj.downcast_into::<T>().expect(
                    format!("The `weakref.ReferenceType` (`PyWeakRef`) does not refer to the requested type `{}`", T::NAME).as_str(),
                )))
    }

    /*
    fn borrowed_upgrade<T: PyTypeCheck>(&self) -> PyResult<Option<Borrowed<'_, 'py, T>>> {
        Ok(self.borrow_object()?.map(|obj| obj.downcast_into::<T>().expect(
                    format!("The `weakref.ReferenceType` (`PyWeakRef`) does not refer to the requested type `{}`", T::NAME).as_str(),
                )))
    }
    */

    fn get_object(&self) -> PyResult<Option<Bound<'py, PyAny>>> {
        let object = self.get_object_raw()?;

        Ok(if object.is_none() { None } else { Some(object) })
    }

    fn borrow_object(&self) -> PyResult<Option<Borrowed<'_, 'py, PyAny>>> {
        let object = self.borrow_object_raw()?;

        Ok(if object.is_none() { None } else { Some(object) })
    }

    fn get_object_raw(&self) -> PyResult<Bound<'py, PyAny>> {
        // Bound<'_, PyAny>::call0 could also be used in situations where ffi::PyWeakref_GetObject is not available.
        self.borrow_object_raw().map(Borrowed::to_owned)
    }

    fn borrow_object_raw(&self) -> PyResult<Borrowed<'_, 'py, PyAny>> {
        // &PyAny::call0 could also be used in situations where ffi::PyWeakref_GetObject is not available.
        unsafe { ffi::PyWeakref_GetObject(self.as_ptr()).assume_borrowed_or_err(self.py()) }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::{pyclass, Python};
    use crate::types::any::PyAnyMethods;
    use crate::types::weakref::{PyWeakRef, PyWeakRefMethods};
    use crate::Py;

    #[pyclass(weakref, crate = "crate")]
    struct WeakrefablePyClass {}

    #[test]
    fn test_reference_upgrade() {
        Python::with_gil(|py| {
            let foo = Py::new(py, WeakrefablePyClass {}).unwrap();
            let reference = PyWeakRef::new_bound(py, foo.clone_ref(py)).unwrap();

            {
                let obj = reference.upgrade::<WeakrefablePyClass>();

                assert!(obj.is_ok());
                let obj = obj.unwrap();

                assert!(obj.is_some());
                assert!(obj.is_some_and(|obj| obj.as_ptr() == foo.as_ptr()));
            }

            drop(foo);

            {
                let obj = reference.upgrade::<WeakrefablePyClass>();

                assert!(obj.is_ok());
                let obj = obj.unwrap();

                assert!(obj.is_none());
            }
        })
    }

    #[test]
    fn test_reference_get_object() {
        Python::with_gil(|py| {
            let foo = Py::new(py, WeakrefablePyClass {}).unwrap();
            let reference = PyWeakRef::new_bound(py, foo.clone_ref(py)).ok().unwrap();

            assert!(reference.call0().unwrap().is(&foo));
            assert!(reference.get_object().unwrap().is_some());
            assert!(reference
                .get_object()
                .unwrap()
                .is_some_and(|obj| obj.is(&foo)));

            drop(foo);

            assert!(reference.call0().unwrap().is_none());
            assert!(reference.get_object().unwrap().is_none());
        })
    }

    #[test]
    fn test_reference_borrrow_object() {
        Python::with_gil(|py| {
            let foo = Py::new(py, WeakrefablePyClass {}).unwrap();
            let reference = PyWeakRef::new_bound(py, foo.clone_ref(py)).ok().unwrap();

            assert!(reference.call0().unwrap().is(&foo));
            assert!(reference.borrow_object().unwrap().is_some());
            assert!(reference
                .borrow_object()
                .unwrap()
                .is_some_and(|obj| obj.is(&foo)));

            drop(foo);

            assert!(reference.call0().unwrap().is_none());
            assert!(reference.borrow_object().unwrap().is_none());
        })
    }

    #[test]
    fn test_reference_get_object_raw() {
        Python::with_gil(|py| {
            let foo = Py::new(py, WeakrefablePyClass {}).unwrap();
            let reference = PyWeakRef::new_bound(py, foo.clone_ref(py)).ok().unwrap();

            assert!(reference.call0().unwrap().is(&foo));
            assert!(reference.get_object_raw().unwrap().is(&foo));

            drop(foo);

            assert!(reference
                .call0()
                .unwrap()
                .is(&reference.get_object_raw().unwrap()));
            assert!(reference.call0().unwrap().is_none());
            assert!(reference.get_object_raw().unwrap().is_none());
        });
    }

    #[test]
    fn test_reference_borrow_object_raw() {
        Python::with_gil(|py| {
            let foo = Py::new(py, WeakrefablePyClass {}).unwrap();
            let reference = PyWeakRef::new_bound(py, foo.clone_ref(py)).ok().unwrap();

            assert!(reference.call0().unwrap().is(&foo));
            assert!(reference.borrow_object_raw().unwrap().is(&foo));

            drop(foo);

            assert!(reference.call0().unwrap().is_none());
            assert!(reference.borrow_object_raw().unwrap().is_none());
        });
    }
}
