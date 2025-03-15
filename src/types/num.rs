use super::any::PyAnyMethods;

use crate::{ffi, instance::Bound, PyAny, Python};

/// Represents a Python `int` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyInt>`][crate::Py] or [`Bound<'py, PyInt>`][crate::Bound].
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](crate::conversion::ToPyObject)
/// and [`extract`](super::PyAnyMethods::extract)
/// with the primitive Rust integer types.
#[repr(transparent)]
pub struct PyInt(PyAny);

pyobject_native_type_core!(PyInt, pyobject_native_static_type_object!(ffi::PyLong_Type), #checkfunction=ffi::PyLong_Check);

/// Deprecated alias for [`PyInt`].
#[deprecated(since = "0.23.0", note = "use `PyInt` instead")]
pub type PyLong = PyInt;

impl PyInt {}

impl PyInt {
    /// Creates a new Python int object.
    ///
    /// Panics if out of memory.
    pub fn new<T: ToPyInt>(py: Python<'_>, i: T) -> Bound<'_, PyInt> {
        T::to_pyint(py, i)
    }
}

/// Trait for the conversion to [`PyInt`]`.
pub trait ToPyInt {
    /// Creates a new Python int object.
    ///
    /// Panics if out of memory.
    fn to_pyint(py: Python<'_>, i: Self) -> Bound<'_, PyInt>;
}

/// Macro to invoke the corresponding PyLong_From variant.
macro_rules! int_from {
    ($rust_type: ty, $from_function: ident) => {
        impl crate::types::num::ToPyInt for $rust_type {
            fn to_pyint(py: crate::Python<'_>, i: Self) -> crate::Bound<'_, crate::types::PyInt> {
                unsafe {
                    let t = crate::ffi::$from_function(i);
                    let owned = crate::ffi_ptr_ext::FfiPtrExt::assume_owned(t, py);
                    crate::types::any::PyAnyMethods::downcast_into_unchecked(owned)
                }
            }
        }
    };
}

/// Macro to invoke the corresponding PyLong_From variant, upcasting the value if required.
#[cfg(not(target_family = "windows"))]
macro_rules! int_from_upcasting {
    ($rust_type: ty, $from_function: ident) => {
        impl crate::types::num::ToPyInt for $rust_type {
            fn to_pyint(py: crate::Python<'_>, i: Self) -> crate::Bound<'_, crate::types::PyInt> {
                unsafe {
                    let t = crate::ffi::$from_function(i.into());
                    let owned = crate::ffi_ptr_ext::FfiPtrExt::assume_owned(t, py);
                    crate::types::any::PyAnyMethods::downcast_into_unchecked(owned)
                }
            }
        }
    };
}

#[cfg(target_family = "windows")]
mod windows {
    int_from!(i32, PyLong_FromLong);
    int_from!(u32, PyLong_FromUnsignedLong);
    int_from!(i64, PyLong_FromLongLong);
    int_from!(u64, PyLong_FromUnsignedLongLong);
    int_from!(isize, PyLong_FromSsize_t);
    int_from!(usize, PyLong_FromSize_t);
    int_from!(f64, PyLong_FromDouble);
}
#[cfg(not(target_family = "windows"))]
mod linux {
    int_from_upcasting!(i32, PyLong_FromLong);
    int_from_upcasting!(u32, PyLong_FromUnsignedLong);
    int_from!(i64, PyLong_FromLongLong);
    int_from!(u64, PyLong_FromUnsignedLongLong);
    int_from!(isize, PyLong_FromSsize_t);
    int_from!(usize, PyLong_FromSize_t);
    int_from!(f64, PyLong_FromDouble);
}

macro_rules! int_compare {
    ($rust_type: ty) => {
        impl PartialEq<$rust_type> for Bound<'_, PyInt> {
            #[inline]
            fn eq(&self, other: &$rust_type) -> bool {
                if let Ok(value) = self.extract::<$rust_type>() {
                    value == *other
                } else {
                    false
                }
            }
        }
        impl PartialEq<Bound<'_, PyInt>> for $rust_type {
            #[inline]
            fn eq(&self, other: &Bound<'_, PyInt>) -> bool {
                if let Ok(value) = other.extract::<$rust_type>() {
                    value == *self
                } else {
                    false
                }
            }
        }
    };
}

int_compare!(i8);
int_compare!(u8);
int_compare!(i16);
int_compare!(u16);
int_compare!(i32);
int_compare!(u32);
int_compare!(i64);
int_compare!(u64);
int_compare!(i128);
int_compare!(u128);
int_compare!(isize);
int_compare!(usize);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IntoPyObject, Python};

    #[test]
    fn test_partial_eq() {
        Python::with_gil(|py| {
            let v_i8 = 123i8;
            let v_u8 = 123i8;
            let v_i16 = 123i16;
            let v_u16 = 123u16;
            let v_i32 = 123i32;
            let v_u32 = 123u32;
            let v_i64 = 123i64;
            let v_u64 = 123u64;
            let v_i128 = 123i128;
            let v_u128 = 123u128;
            let v_isize = 123isize;
            let v_usize = 123usize;
            let obj = 123_i64.into_pyobject(py).unwrap();
            assert_eq!(v_i8, obj);
            assert_eq!(obj, v_i8);

            assert_eq!(v_u8, obj);
            assert_eq!(obj, v_u8);

            assert_eq!(v_i16, obj);
            assert_eq!(obj, v_i16);

            assert_eq!(v_u16, obj);
            assert_eq!(obj, v_u16);

            assert_eq!(v_i32, obj);
            assert_eq!(obj, v_i32);

            assert_eq!(v_u32, obj);
            assert_eq!(obj, v_u32);

            assert_eq!(v_i64, obj);
            assert_eq!(obj, v_i64);

            assert_eq!(v_u64, obj);
            assert_eq!(obj, v_u64);

            assert_eq!(v_i128, obj);
            assert_eq!(obj, v_i128);

            assert_eq!(v_u128, obj);
            assert_eq!(obj, v_u128);

            assert_eq!(v_isize, obj);
            assert_eq!(obj, v_isize);

            assert_eq!(v_usize, obj);
            assert_eq!(obj, v_usize);

            let big_num = (u8::MAX as u16) + 1;
            let big_obj = big_num.into_pyobject(py).unwrap();

            for x in 0u8..=u8::MAX {
                assert_ne!(x, big_obj);
                assert_ne!(big_obj, x);
            }
        });
    }

    #[test]
    fn test_display_int() {
        Python::with_gil(|py| {
            let s = PyInt::new(py, 42);
            assert_eq!(format!("{}", s), "42");

            let s = PyInt::new(py, 69.420);
            assert_eq!(format!("{}", s), "69");
        })
    }
}
