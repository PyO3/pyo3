// Copyright (c) 2017-present PyO3 Project and Contributors

//! Various types defined by the Python interpreter such as `int`, `str` and `tuple`.

use crate::{types::Any, Python, PyResult, PyCell, PyClass, PyNativeType, PyTryFrom, ffi, PyRef, PyRefMut, AsPyPointer};

pub use self::any::PyAny;
// pub use self::boolobject::PyBool;
// pub use self::bytearray::PyByteArray;
pub use self::bytes::PyBytes;
// pub use self::complex::PyComplex;
// pub use self::datetime::PyDeltaAccess;
// pub use self::datetime::{
//     PyDate, PyDateAccess, PyDateTime, PyDelta, PyTime, PyTimeAccess, PyTzInfo,
// };
pub use self::dict::{IntoPyDict, PyDict};
// pub use self::floatob::PyFloat;
// pub use self::function::{PyCFunction, PyFunction};
// pub use self::iterator::PyIterator;
pub use self::list::PyList;
// pub use self::module::PyModule;
// pub use self::num::PyLong;
// pub use self::num::PyLong as PyInt;
// pub use self::sequence::PySequence;
// pub use self::set::{PyFrozenSet, PySet};
// pub use self::slice::{PySlice, PySliceIndices};
// pub(crate) use self::string::with_tmp_string;
pub use self::string::PyStr;
// pub use self::tuple::PyTuple;
// pub use self::typeobject::PyType;

#[macro_export]
#[doc(hidden)]
macro_rules! pyo3_native_object_base {
    ($object:ty, $ty:ty, $py:lifetime) => {
        impl<$py> AsPyPointer for $object {
            fn as_ptr(&self) -> *mut $crate::ffi::PyObject {
                self.0.as_ptr()
            }
        }

        impl<$py> $crate::ToPyObject for $object
        {
            #[inline]
            fn to_object(&self, py: $crate::Python) -> $crate::PyObject {
                use $crate::AsPyPointer;
                unsafe { $crate::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        impl<$py> std::ops::Deref for $crate::owned::PyOwned<$py, $ty> {
            type Target = $object;

            fn deref(&self) -> &Self::Target {
                use $crate::AsPyPointer;
                unsafe { std::mem::transmute(self.as_ptr()) }
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
            fn py(&self) -> $crate::Python<$py> {
                self.1
            }
            fn into_ref(&self) -> &$py Self::NativeType {
                use $crate::IntoPyPointer;
                self.py().from_owned_ptr(self.into_ptr())
            }
        }

        impl<$py> $crate::PyTryFrom<$py> for $object
        {
            fn try_from<V: Into<&$py $crate::PyAny>>(value: V) -> Result<&$py Self, $crate::PyDowncastError<$py>> {
                use $crate::PyTypeInfo;
                let value = value.into();
                unsafe {
                    if <$ty>::is_type_of(value) {
                        Ok(Self::try_from_unchecked(value))
                    } else {
                        Err($crate::PyDowncastError::new(value, <$ty>::NAME))
                    }
                }
            }

            fn try_from_exact<V: Into<&$py $crate::PyAny>>(value: V) -> Result<&$py Self, $crate::PyDowncastError<$py>> {
                use $crate::PyTypeInfo;
                let value = value.into();
                unsafe {
                    if <$ty>::is_exact_type_of(value) {
                        Ok(Self::try_from_unchecked(value))
                    } else {
                        Err($crate::PyDowncastError::new(value, <$ty>::NAME))
                    }
                }
            }

            #[inline]
            unsafe fn try_from_unchecked<V: Into<&$py $crate::PyAny>>(value: V) -> &$py Self {
                use $crate::objects::PyNativeObject;
                Self::unchecked_downcast(value.into())
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

            fn deref(&self) -> &Self::Target {
                unsafe { std::mem::transmute(self) }
            }
        }

        impl From<$crate::owned::PyOwned<'_, $ty>> for $crate::PyObject {
            fn from(owned: $crate::owned::PyOwned<'_, $ty>) -> $crate::PyObject {
                owned.into()
            }
        }
    };
}

pub unsafe trait PyNativeObject<'py>: Sized {
    type NativeType: PyNativeType;
    fn py(&self) -> Python<'py>;
    fn into_ref(&self) -> &'py Self::NativeType;
    unsafe fn unchecked_downcast(any: &'py Any) -> &'py Self {
        &*(any.as_ptr() as *const Self)
    }
}

mod any;
// mod boolobject;
// mod bytearray;
mod bytes;
// mod complex;
// mod datetime;
mod dict;
// mod floatob;
// mod function;
// mod iterator;
mod list;
// mod module;
// mod num;
// mod sequence;
// mod set;
// mod slice;
mod string;
// mod tuple;
// mod typeobject;


/// New variant of conversion::FromPyObject which doesn't create owned references.
pub trait FromPyObject<'py>: Sized {
    /// Extracts `Self` from the source `PyAny`.
    fn extract(ob: &PyAny<'py>) -> PyResult<Self>;
}

impl<'py, T> FromPyObject<'py> for &'py PyCell<T>
where
    T: PyClass,
{
    fn extract(obj: &PyAny<'py>) -> PyResult<Self> {
        PyTryFrom::try_from(obj.into_ref()).map_err(Into::into)
    }
}

impl<'py, T> FromPyObject<'py> for T
where
    T: PyClass + Clone,
{
    fn extract(obj: &PyAny<'py>) -> PyResult<Self> {
        let cell: &PyCell<Self> = PyTryFrom::try_from(obj.into_ref())?;
        Ok(unsafe { cell.try_borrow_unguarded()?.clone() })
    }
}

impl<'py, T> FromPyObject<'py> for PyRef<'py, T>
where
    T: PyClass,
{
    fn extract(obj: &PyAny<'py>) -> PyResult<Self> {
        let cell: &PyCell<T> = PyTryFrom::try_from(obj.into_ref())?;
        cell.try_borrow().map_err(Into::into)
    }
}

impl<'py, T> FromPyObject<'py> for PyRefMut<'py, T>
where
    T: PyClass,
{
    fn extract(obj: &PyAny<'py>) -> PyResult<Self> {
        let cell: &PyCell<T> = PyTryFrom::try_from(obj.into_ref())?;
        cell.try_borrow_mut().map_err(Into::into)
    }
}

impl<'py, T> FromPyObject<'py> for Option<T>
where
    T: FromPyObject<'py>,
{
    fn extract(obj: &PyAny<'py>) -> PyResult<Self> {
        if obj.as_ptr() == unsafe { ffi::Py_None() } {
            Ok(None)
        } else {
            T::extract(obj).map(Some)
        }
    }
}
