use super::any::PyAnyMethods;
use crate::{ffi, instance::Bound, IntoPyObject, PyAny, Python};
use std::convert::Infallible;

/// Represents a Python `int` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyInt>`][crate::Py] or [`Bound<'py, PyInt>`][crate::Bound].
///
/// You can usually avoid directly working with this type by using
/// [`IntoPyObject`] and [`extract`](super::PyAnyMethods::extract)
/// with the primitive Rust integer types.
#[repr(transparent)]
pub struct PyInt(PyAny);

pyobject_native_type_core!(PyInt, pyobject_native_static_type_object!(ffi::PyLong_Type), #checkfunction=ffi::PyLong_Check);

impl PyInt {
    /// Creates a new Python int object.
    ///
    /// Panics if out of memory.
    pub fn new<'a, T>(py: Python<'a>, i: T) -> Bound<'a, PyInt>
    where
        T: IntoPyObject<'a, Target = PyInt, Output = Bound<'a, PyInt>, Error = Infallible>,
    {
        match T::into_pyobject(i, py) {
            Ok(v) => v,
            Err(never) => match never {},
        }
    }
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
        Python::attach(|py| {
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
        Python::attach(|py| {
            let s = PyInt::new(py, 42u8);
            assert_eq!(format!("{s}"), "42");

            let s = PyInt::new(py, 43i32);
            assert_eq!(format!("{s}"), "43");

            let s = PyInt::new(py, 44usize);
            assert_eq!(format!("{s}"), "44");
        })
    }
}
