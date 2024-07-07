use super::any::PyAnyMethods;

use crate::{ffi, ffi_ptr_ext::FfiPtrExt, instance::Bound, PyAny, Python};
use std::os::raw::c_long;

/// Represents a Python `int` object.
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](crate::conversion::ToPyObject)
/// and [`extract`](super::PyAnyMethods::extract)
/// with the primitive Rust integer types.
#[repr(transparent)]
pub struct PyLong(PyAny);

pyobject_native_type_core!(PyLong, pyobject_native_static_type_object!(ffi::PyLong_Type), #checkfunction=ffi::PyLong_Check);

impl PyLong {
    /// Creates a new `PyLong` from a C `long` integer.
    /// # Returns
    /// A [`Bound`] reference to a `PyLong` object representing the given `val`.
    #[inline]
    pub fn new_bound_from_c_long(py: Python<'_>, val: c_long) -> Bound<'_, Self> {
        unsafe {
            ffi::PyLong_FromLong(val)
                .assume_owned(py)
                .downcast_into_unchecked()
        }
    }
}

/// Implementation of functionality for [`PyLong`].
///
/// These methods are defined for the `Bound<'py, PyLong>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyLong")]
pub trait PyLongMethods<'py>: crate::sealed::Sealed {
    /// Gets the value of this int.
    fn value(&self) -> c_long;
}

impl<'py> PyLongMethods<'py> for Bound<'py, PyLong> {
    fn value(&self) -> c_long {
        unsafe {
            // Safety: self is PyLong object
            ffi::PyLong_AsLong(self.as_ptr())
        }
    }
}

/// Implement <Bound<'_, PyLong>> == i8 comparisons
impl PartialEq<i8> for Bound<'_, PyLong> {
    #[inline]
    fn eq(&self, other: &i8) -> bool {
        self.value() == *other as c_long
    }
}

/// Implement i8 == <Bound<'_, PyLong>> comparisons
impl PartialEq<Bound<'_, PyLong>> for i8 {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyLong>) -> bool {
        *self as c_long == other.value()
    }
}

/// Implement <Bound<'_, PyLong>> == u8 comparisons
impl PartialEq<u8> for Bound<'_, PyLong> {
    #[inline]
    fn eq(&self, other: &u8) -> bool {
        self.value() == *other as c_long
    }
}

/// Implement u8 == <Bound<'_, PyLong>> comparisons
impl PartialEq<Bound<'_, PyLong>> for u8 {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyLong>) -> bool {
        *self as c_long == other.value()
    }
}

/// Implement <Bound<'_, PyLong>> == i16 comparisons
impl PartialEq<i16> for Bound<'_, PyLong> {
    #[inline]
    fn eq(&self, other: &i16) -> bool {
        self.value() == *other as c_long
    }
}

/// Implement i16 == <Bound<'_, PyLong>> comparisons
impl PartialEq<Bound<'_, PyLong>> for i16 {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyLong>) -> bool {
        *self as c_long == other.value()
    }
}

/// Implement <Bound<'_, PyLong>> == u16 comparisons
impl PartialEq<u16> for Bound<'_, PyLong> {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.value() == *other as c_long
    }
}

/// Implement u16 == <Bound<'_, PyLong>> comparisons
impl PartialEq<Bound<'_, PyLong>> for u16 {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyLong>) -> bool {
        *self as c_long == other.value()
    }
}

/// Implement <Bound<'_, PyLong>> == i32 comparisons
impl PartialEq<i32> for Bound<'_, PyLong> {
    #[inline]
    fn eq(&self, other: &i32) -> bool {
        self.value() == *other as c_long
    }
}

/// Implement i32 == <Bound<'_, PyLong>> comparisons
impl PartialEq<Bound<'_, PyLong>> for i32 {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyLong>) -> bool {
        *self as c_long == other.value()
    }
}

/// Implement <Bound<'_, PyLong>> == u32 comparisons
impl PartialEq<u32> for Bound<'_, PyLong> {
    #[inline]
    fn eq(&self, other: &u32) -> bool {
        self.value() == *other as c_long
    }
}

/// Implement u32 == <Bound<'_, PyLong>> comparisons
impl PartialEq<Bound<'_, PyLong>> for u32 {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyLong>) -> bool {
        *self as c_long == other.value()
    }
}

/// Implement <Bound<'_, PyLong>> == i64 comparisons
impl PartialEq<i64> for Bound<'_, PyLong> {
    #[inline]
    fn eq(&self, other: &i64) -> bool {
        self.value() == *other as c_long
    }
}

/// Implement i64 == <Bound<'_, PyLong>> comparisons
impl PartialEq<Bound<'_, PyLong>> for i64 {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyLong>) -> bool {
        *self as c_long == other.value()
    }
}

/// Implement <Bound<'_, PyLong>> == u64 comparisons
impl PartialEq<u64> for Bound<'_, PyLong> {
    #[inline]
    fn eq(&self, other: &u64) -> bool {
        self.value() == *other as c_long
    }
}

/// Implement u64 == <Bound<'_, PyLong>> comparisons
impl PartialEq<Bound<'_, PyLong>> for u64 {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyLong>) -> bool {
        *self as c_long == other.value()
    }
}

/// Implement <Bound<'_, PyLong>> == isize comparisons
impl PartialEq<isize> for Bound<'_, PyLong> {
    #[inline]
    fn eq(&self, other: &isize) -> bool {
        self.value() == *other as c_long
    }
}

/// Implement isize == <Bound<'_, PyLong>> comparisons
impl PartialEq<Bound<'_, PyLong>> for isize {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyLong>) -> bool {
        *self as c_long == other.value()
    }
}

/// Implement <Bound<'_, PyLong>> == usize comparisons
impl PartialEq<usize> for Bound<'_, PyLong> {
    #[inline]
    fn eq(&self, other: &usize) -> bool {
        self.value() == *other as c_long
    }
}

/// Implement usize == <Bound<'_, PyLong>> comparisons
impl PartialEq<Bound<'_, PyLong>> for usize {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyLong>) -> bool {
        *self as c_long == other.value()
    }
}

#[cfg(test)]
mod tests {
    use super::PyLong;
    use super::PyLongMethods;
    use crate::Python;

    #[test]
    fn test_c_long_value() {
        Python::with_gil(|py| {
            let v = 123;
            let obj = PyLong::new_bound_from_c_long(py, 123);
            assert_eq!(v, obj.value());
        });
    }

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
            let v_isize = 123isize;
            let v_usize = 123usize;
            let obj = PyLong::new_bound_from_c_long(py, 123);
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

            assert_eq!(v_isize, obj);
            assert_eq!(obj, v_isize);

            assert_eq!(v_usize, obj);
            assert_eq!(obj, v_usize);
        });
    }

}
