//! Utilities around [Python capsules](https://docs.python.org/3/c-api/capsule.html#capsules):
//!
//! > This subtype of PyObject represents an opaque value, useful for C extension
//! > modules who need to pass an opaque value (as a void* pointer) through Python
//! > code to other C code. It is often used to make a C function pointer defined
//! > in one module available to other modules, so the regular import mechanism can
//! > be used to access C APIs defined in dynamically loaded modules.
//!
//! It provides two wrapper types [`PyCapsuleValue`] and [`PyCapsuleValueRef`] allowing to build and easily extract capsules.
//! They implement [`FromPyObject`] and [ÌntoPyObject`].
//! The type they wrap must implement [`PyCapsuleType`].
//!
//! # Example
//! ```
//! use std::ffi::CStr;
//! use pyo3::{prelude::*, capsule::*};
//!
//! #[repr(C)]
//! struct Foo {
//!     pub val: u32,
//! }
//!
//! unsafe impl PyCapsuleType for Foo {
//!     const NAME: &CStr = c"mypackage.foo.1";
//! }
//!
//! #[pyfunction]
//! fn add_one(foo: PyCapsuleValueRef<'_, Foo>) -> PyCapsuleValue<Foo> {
//!     PyCapsuleValue(Foo { val: foo.val })
//! }
//!
//! let r = Python::attach(|py| {
//!     let foo = Foo { val: 123 };
//!     let capsule = PyCapsuleValue(foo).into_pyobject(py)?;
//!     let result = wrap_pyfunction!(add_one, py)?.call1((capsule,))?;
//!     let value = result.extract::<PyCapsuleValueRef<'_, Foo>>()?;
//!     assert_eq!(value.val, 123);
//!     PyResult::Ok(())
//! });
//! # r.unwrap()
//! ```

use crate::types::{PyCapsule, PyCapsuleMethods};
use crate::{Borrowed, Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python};
use std::borrow::Borrow;
use std::ffi::CStr;
use std::ops::Deref;

/// Trait to tag that the type can be stored in a [`PyCapsule`] and state the capsule name using [`NAME`](Self::NAME).
///
/// The capsule name aims at uniquely identify the type. Two different types or variants of the same type with a different ABI MUST not share the same name.
///
/// # Safety
///
/// - The type must have a stable ABI like `#[repr(C)]`
/// - [`NAME`](Self::NAME) must uniquely identify the type and its ABI.
///   Don't use the same name for different types and update the name if the ABI compatibility is broken.
pub unsafe trait PyCapsuleType {
    /// The capsule name that must uniquely identify the type and it's ABI.
    const NAME: &'static CStr;
}

/// Wraps a [`PyCapsuleType`] and implement [`FromPyObject`] and [`IntoPyObject`].
///
/// Note that [`FromPyObject`] requires the type to be [`Clone`], use [`PyCapsuleValueRef`] to avoid that.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct PyCapsuleValue<T>(pub T);

impl<'a, 'py, T: PyCapsuleType + Clone + Sync> FromPyObject<'a, 'py> for PyCapsuleValue<T> {
    type Error = PyErr;

    #[inline]
    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        Ok(Self(PyCapsuleValueRef::<T>::extract(obj)?.0.clone()))
    }
}

impl<'py, T: PyCapsuleType + Send + 'static> IntoPyObject<'py> for PyCapsuleValue<T> {
    type Target = PyCapsule;
    type Output = Bound<'py, PyCapsule>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> PyResult<Bound<'py, PyCapsule>> {
        PyCapsule::new_with_value(py, self, T::NAME)
    }
}

/// Wraps a [`PyCapsuleType`] and implement [`FromPyObject`] and [`IntoPyObject`].
///
/// Note that [`IntoPyObject`] requires the type to be [`Clone`], use [`PyCapsuleValue`] to avoid that.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct PyCapsuleValueRef<'a, T>(pub &'a T);

impl<T> AsRef<T> for PyCapsuleValueRef<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.0
    }
}

impl<T> Deref for PyCapsuleValueRef<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.0
    }
}

impl<T> Borrow<T> for PyCapsuleValueRef<'_, T> {
    #[inline]
    fn borrow(&self) -> &T {
        self.0
    }
}

impl<'a, 'py, T: PyCapsuleType + Sync> FromPyObject<'a, 'py> for PyCapsuleValueRef<'a, T> {
    type Error = PyErr;

    #[inline]
    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let ptr = obj
            .cast::<PyCapsule>()?
            .pointer_checked(Some(T::NAME))?
            .cast::<T>();
        // SAFETY:
        // - the trait implementation pinkie swears that all capsule values with the given name have the same ABI as T
        // - the Sync bound ensures that it's fine to have multiple threads with read-only references to the value
        // - fetching concurrent mutable references requires an other unsafe, the UB might be considered to be there
        // - the 'a lifetime bounds ensure the reference lifetime is smaller than the capsule one
        // - if PyCapsule_SetPointer is called, the old value destructor is not called so the pointer is still valid
        Ok(Self(unsafe { ptr.as_ref() }))
    }
}

impl<'a, 'py, T: PyCapsuleType + Clone + Send + 'static> IntoPyObject<'py>
    for PyCapsuleValueRef<'a, T>
{
    type Target = PyCapsule;
    type Output = Bound<'py, PyCapsule>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> PyResult<Bound<'py, PyCapsule>> {
        PyCapsuleValue(self.0.clone()).into_pyobject(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyAnyMethods;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(C)]
    struct Value(usize);

    unsafe impl PyCapsuleType for Value {
        const NAME: &'static CStr = c"value";
    }

    #[test]
    fn test_capsule_wrapping() -> PyResult<()> {
        Python::attach(|py| {
            let value = Value(1);
            let capsule = PyCapsuleValue(value).into_pyobject(py)?;
            let new_value = capsule.extract::<PyCapsuleValue<Value>>()?.0;
            assert_eq!(value, new_value);
            Ok(())
        })
    }

    #[test]
    fn test_capsule_ref_wrapping() -> PyResult<()> {
        Python::attach(|py| {
            let value = Value(1);
            let capsule = PyCapsuleValueRef(&value).into_pyobject(py)?;
            let new_value = *capsule.extract::<PyCapsuleValueRef<'_, Value>>()?.0;
            assert_eq!(value, new_value);
            Ok(())
        })
    }
}
