// Copyright (c) 2017-present PyO3 Project and Contributors

//! Various types defined by the Python interpreter such as `int`, `str` and `tuple`.

use crate::{
    ffi, AsPyPointer, PyCell, PyClass, PyDowncastError, PyNativeType, PyRef, PyRefMut, PyResult,
    PyTypeInfo, Python,
};

pub use self::any::PyAny;
pub use self::boolobject::PyBool;
pub use self::bytearray::PyByteArray;
pub use self::bytes::PyBytes;
pub use self::complex::PyComplex;
pub use self::datetime::PyDeltaAccess;
pub use self::datetime::{
    PyDate, PyDateAccess, PyDateTime, PyTime, PyTimeAccess, PyTimeDelta, PyTzInfo,
};
pub use self::dict::{IntoPyDict, PyDict};
pub use self::floatob::PyFloat;
pub use self::function::{PyCFunction, PyFunction};
pub use self::iterator::PyIterator;
pub use self::list::PyList;
pub use self::module::PyModule;
pub use self::num::PyInt;
pub use self::sequence::PySequence;
pub use self::set::{PyFrozenSet, PySet};
pub use self::slice::{PySlice, PySliceIndices};
pub use self::str::PyStr;
pub use self::tuple::PyTuple;
pub use self::typeobject::PyType;

// For easing the transition
pub use self::num::PyInt as PyLong;
pub use self::str::PyStr as PyString;

#[macro_export]
#[doc(hidden)]
macro_rules! pyo3_native_object_base {
    ($object:ty, $ty:ty, $py:lifetime) => {
        impl<$py> $crate::ToPyObject for $object
        {
            #[inline]
            fn to_object(&self, py: $crate::Python) -> $crate::PyObject {
                use $crate::AsPyPointer;
                unsafe { $crate::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        impl<$py> From<&'_ $object> for $crate::PyObject
        {
            #[inline]
            fn from(object: & $object) -> $crate::PyObject {
                use $crate::{AsPyPointer, objects::PyNativeObject};
                unsafe { $crate::PyObject::from_borrowed_ptr(object.py(), object.as_ptr()) }
            }
        }

        impl<$py> From<$object> for $crate::Py<$ty>
        {
            #[inline]
            fn from(object: $object) -> $crate::Py<$ty> {
                use $crate::{IntoPyPointer, objects::PyNativeObject};
                unsafe { $crate::Py::from_owned_ptr(object.py(), object.into_ptr()) }
            }
        }

        impl<'a, $py> $crate::objects::FromPyObject<'a, $py> for &'a $object {
            fn extract(any: &'a $crate::objects::PyAny<$py>) -> $crate::PyResult<Self> {
                any.downcast::<$object>().map_err(Into::into)
            }
        }

        impl<$py> ::std::fmt::Debug for $object {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> Result<(), ::std::fmt::Error>
            {
                let s = self.repr().map_err(|_| ::std::fmt::Error)?;
                f.write_str(&s.to_string_lossy())
            }
        }

        impl<$py> ::std::fmt::Display for $object {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> Result<(), ::std::fmt::Error>
            {
                let s = self.str().map_err(|_| ::std::fmt::Error)?;
                f.write_str(&s.to_string_lossy())
            }
        }

        unsafe impl<$py> $crate::objects::PyNativeObject<$py> for $object {
            type NativeType = $ty;
            #[inline]
            fn py(&self) -> $crate::Python<$py> {
                self.1
            }
            #[inline]
            fn as_owned_ref(&self) -> &Self::NativeType {
                unsafe { self.py().from_borrowed_ptr(self.as_ptr()) }
            }
            #[inline]
            fn into_ty_ref(&self) -> &$py Self::NativeType {
                use $crate::IntoPyPointer;
                unsafe { self.py().from_owned_ptr(self.into_ptr()) }
            }
        }

        impl $ty {
            pub fn to_owned<$py>(&$py self) -> $object {
                use $crate::{PyNativeType, IntoPyPointer};
                unsafe { $crate::objects::PyAny::from_raw_or_panic(self.py(), self.into_ptr()).extract().unwrap() }
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! pyo3_native_object {
    ($object:ty, $ty:ty, $py:lifetime) => {
        $crate::pyo3_native_object_base!($object, $ty, $py);

        impl<$py> std::ops::Deref for $object {
            type Target = $crate::objects::PyAny<$py>;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<$py> From<$object> for $crate::PyObject {
            #[inline]
            fn from(obj: $object) -> $crate::PyObject {
                $crate::Py::<$ty>::from(obj).into()
            }
        }

        impl<$py> $crate::objects::FromPyObject<'_, $py> for $object {
            fn extract(any: &$crate::objects::PyAny<$py>) -> $crate::PyResult<Self> {
                Ok(Self(any.clone()))
            }
        }

        impl<$py> AsPyPointer for $object {
            #[inline]
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }
    };
}

mod any;
mod boolobject;
mod bytearray;
mod bytes;
mod complex;
mod datetime;
mod dict;
mod floatob;
mod function;
mod iterator;
mod list;
mod module;
mod num;
mod sequence;
mod set;
mod slice;
mod str;
mod tuple;
mod typeobject;

/// To implement this trait, &Self *must* be equivalent to *mut ffi::PyObject
pub unsafe trait PyNativeObject<'py>: Sized + 'py {
    type NativeType: PyNativeType;
    fn py(&self) -> Python<'py>;
    fn as_owned_ref(&self) -> &Self::NativeType;
    fn into_ty_ref(&self) -> &'py Self::NativeType;
    #[inline]
    unsafe fn unchecked_downcast<'a>(any: &'a PyAny<'py>) -> &'a Self {
        &*(any as *const PyAny as *const Self)
    }

    #[inline]
    unsafe fn from_borrowed_ptr<'a>(_py: Python<'py>, ptr: &*mut ffi::PyObject) -> &'a Self {
        &*(ptr as *const *mut ffi::PyObject as *const Self)
    }
}

pub trait PyNativeObjectInfo<'py>: PyNativeObject<'py>
where
    Self::NativeType: PyTypeInfo,
{
}

impl<'py, T> PyNativeObjectInfo<'py> for T
where
    T: PyNativeObject<'py>,
    T::NativeType: PyTypeInfo,
{
}

/// New variant of conversion::FromPyObject which doesn't create owned references.
pub trait FromPyObject<'a, 'py>: Sized {
    /// Extracts `Self` from the source `PyAny`.
    fn extract(ob: &'a PyAny<'py>) -> PyResult<Self>;
}

impl<'a, T> FromPyObject<'a, 'a> for &'a PyCell<T>
where
    T: PyClass,
{
    fn extract(obj: &'a PyAny<'a>) -> PyResult<Self> {
        PyTryFrom::try_from(obj).map_err(Into::into)
    }
}

impl<T> FromPyObject<'_, '_> for T
where
    T: PyClass + Clone,
{
    fn extract(obj: &PyAny) -> PyResult<Self> {
        let cell: &PyCell<Self> = PyTryFrom::try_from(obj)?;
        Ok(unsafe { cell.try_borrow_unguarded()?.clone() })
    }
}

impl<'py, T> FromPyObject<'py, 'py> for PyRef<'py, T>
where
    T: PyClass,
{
    fn extract(obj: &'py PyAny<'py>) -> PyResult<Self> {
        let cell: &PyCell<T> = PyTryFrom::try_from(obj)?;
        cell.try_borrow().map_err(Into::into)
    }
}

impl<'py, T> FromPyObject<'py, 'py> for PyRefMut<'py, T>
where
    T: PyClass,
{
    fn extract(obj: &'py PyAny<'py>) -> PyResult<Self> {
        let cell: &PyCell<T> = PyTryFrom::try_from(obj)?;
        cell.try_borrow_mut().map_err(Into::into)
    }
}

impl<'a, 'py, T> FromPyObject<'a, 'py> for Option<T>
where
    T: FromPyObject<'a, 'py>,
{
    fn extract(obj: &'a PyAny<'py>) -> PyResult<Self> {
        if obj.as_ptr() == unsafe { ffi::Py_None() } {
            Ok(None)
        } else {
            T::extract(obj).map(Some)
        }
    }
}

/// Trait implemented by Python object types that allow a checked downcast.
/// If `T` implements `PyTryFrom`, we can convert `&PyAny` to `&T`.
///
/// This trait is similar to `std::convert::TryFrom`
pub trait PyTryFrom<'a, 'py>: Sized {
    /// Cast from a concrete Python object type to PyObject.
    fn try_from(any: &'a PyAny<'py>) -> Result<&'a Self, crate::PyDowncastError<'py>>;

    /// Cast from a concrete Python object type to PyObject. With exact type check.
    fn try_from_exact(any: &'a PyAny<'py>) -> Result<&'a Self, crate::PyDowncastError<'py>>;

    /// Cast a PyAny to a specific type of PyObject. The caller must
    /// have already verified the reference is for this type.
    unsafe fn try_from_unchecked(any: &'a PyAny<'py>) -> &'a Self;
}

impl<'a, 'py, T> PyTryFrom<'a, 'py> for T
where
    T: PyNativeObject<'py> + PyNativeObjectInfo<'py> + 'py,
    T::NativeType: PyTypeInfo,
{
    fn try_from(any: &'a PyAny<'py>) -> Result<&'a T, PyDowncastError<'py>> {
        unsafe {
            if T::NativeType::is_type_of(any.as_owned_ref()) {
                Ok(T::try_from_unchecked(any))
            } else {
                Err(PyDowncastError::new(any.into_ty_ref(), T::NativeType::NAME))
            }
        }
    }

    fn try_from_exact(any: &'a PyAny<'py>) -> Result<&'a T, PyDowncastError<'py>> {
        unsafe {
            if T::NativeType::is_exact_type_of(any.as_owned_ref()) {
                Ok(T::try_from_unchecked(any))
            } else {
                Err(PyDowncastError::new(any.into_ty_ref(), T::NativeType::NAME))
            }
        }
    }

    #[inline]
    unsafe fn try_from_unchecked(any: &'a PyAny<'py>) -> &'a T {
        T::unchecked_downcast(any)
    }
}

impl<'py, T> PyTryFrom<'py, 'py> for PyCell<T>
where
    T: 'py + PyClass,
{
    fn try_from(any: &'py PyAny<'py>) -> Result<&'py Self, PyDowncastError<'py>> {
        unsafe {
            if T::is_type_of(any.as_owned_ref()) {
                Ok(Self::try_from_unchecked(any))
            } else {
                Err(PyDowncastError::new(any.into_ty_ref(), T::NAME))
            }
        }
    }
    fn try_from_exact(any: &'py PyAny<'py>) -> Result<&'py Self, PyDowncastError<'py>> {
        unsafe {
            if T::is_exact_type_of(any.as_owned_ref()) {
                Ok(Self::try_from_unchecked(any))
            } else {
                Err(PyDowncastError::new(any.into_ty_ref(), T::NAME))
            }
        }
    }
    #[inline]
    unsafe fn try_from_unchecked(any: &'py PyAny<'py>) -> &'py Self {
        PyCell::unchecked_downcast(any.into_ty_ref())
    }
}
