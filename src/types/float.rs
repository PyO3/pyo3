use super::any::PyAnyMethods;
use crate::conversion::IntoPyObject;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    ffi, ffi_ptr_ext::FfiPtrExt, instance::Bound, Borrowed, FromPyObject, PyAny, PyErr, PyResult,
    Python,
};
use std::convert::Infallible;
use std::ffi::c_double;

/// Represents a Python `float` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFloat>`][crate::Py] or [`Bound<'py, PyFloat>`][Bound].
///
/// For APIs available on `float` objects, see the [`PyFloatMethods`] trait which is implemented for
/// [`Bound<'py, PyFloat>`][Bound].
///
/// You can usually avoid directly working with this type
/// by using [`IntoPyObject`] and [`extract`][PyAnyMethods::extract]
/// with [`f32`]/[`f64`].
#[repr(transparent)]
pub struct PyFloat(PyAny);

pyobject_subclassable_native_type!(PyFloat, crate::ffi::PyFloatObject);

pyobject_native_type!(
    PyFloat,
    ffi::PyFloatObject,
    pyobject_native_static_type_object!(ffi::PyFloat_Type),
    #checkfunction=ffi::PyFloat_Check
);

impl PyFloat {
    /// Creates a new Python `float` object.
    pub fn new(py: Python<'_>, val: c_double) -> Bound<'_, PyFloat> {
        unsafe {
            ffi::PyFloat_FromDouble(val)
                .assume_owned(py)
                .cast_into_unchecked()
        }
    }
}

/// Implementation of functionality for [`PyFloat`].
///
/// These methods are defined for the `Bound<'py, PyFloat>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyFloat")]
pub trait PyFloatMethods<'py>: crate::sealed::Sealed {
    /// Gets the value of this float.
    fn value(&self) -> c_double;
}

impl<'py> PyFloatMethods<'py> for Bound<'py, PyFloat> {
    fn value(&self) -> c_double {
        #[cfg(not(Py_LIMITED_API))]
        unsafe {
            // Safety: self is PyFloat object
            ffi::PyFloat_AS_DOUBLE(self.as_ptr())
        }

        #[cfg(Py_LIMITED_API)]
        unsafe {
            ffi::PyFloat_AsDouble(self.as_ptr())
        }
    }
}

impl<'py> IntoPyObject<'py> for f64 {
    type Target = PyFloat;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = "float";

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyFloat::new(py, self))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("float")
    }
}

impl<'py> IntoPyObject<'py> for &f64 {
    type Target = PyFloat;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = f64::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("float")
    }
}

impl<'py> FromPyObject<'py> for f64 {
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "float";

    // PyFloat_AsDouble returns -1.0 upon failure
    #[allow(clippy::float_cmp)]
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        // On non-limited API, .value() uses PyFloat_AS_DOUBLE which
        // allows us to have an optimized fast path for the case when
        // we have exactly a `float` object (it's not worth going through
        // `isinstance` machinery for subclasses).
        #[cfg(not(Py_LIMITED_API))]
        if let Ok(float) = obj.cast_exact::<PyFloat>() {
            return Ok(float.value());
        }

        let v = unsafe { ffi::PyFloat_AsDouble(obj.as_ptr()) };

        if v == -1.0 {
            if let Some(err) = PyErr::take(obj.py()) {
                return Err(err);
            }
        }

        Ok(v)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

impl<'py> IntoPyObject<'py> for f32 {
    type Target = PyFloat;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = "float";

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyFloat::new(py, self.into()))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("float")
    }
}

impl<'py> IntoPyObject<'py> for &f32 {
    type Target = PyFloat;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = f32::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("float")
    }
}

impl<'py> FromPyObject<'py> for f32 {
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "float";

    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        Ok(obj.extract::<f64>()? as f32)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

macro_rules! impl_partial_eq_for_float {
    ($float_type: ty) => {
        impl PartialEq<$float_type> for Bound<'_, PyFloat> {
            #[inline]
            fn eq(&self, other: &$float_type) -> bool {
                self.value() as $float_type == *other
            }
        }

        impl PartialEq<$float_type> for &Bound<'_, PyFloat> {
            #[inline]
            fn eq(&self, other: &$float_type) -> bool {
                self.value() as $float_type == *other
            }
        }

        impl PartialEq<&$float_type> for Bound<'_, PyFloat> {
            #[inline]
            fn eq(&self, other: &&$float_type) -> bool {
                self.value() as $float_type == **other
            }
        }

        impl PartialEq<Bound<'_, PyFloat>> for $float_type {
            #[inline]
            fn eq(&self, other: &Bound<'_, PyFloat>) -> bool {
                other.value() as $float_type == *self
            }
        }

        impl PartialEq<&'_ Bound<'_, PyFloat>> for $float_type {
            #[inline]
            fn eq(&self, other: &&'_ Bound<'_, PyFloat>) -> bool {
                other.value() as $float_type == *self
            }
        }

        impl PartialEq<Bound<'_, PyFloat>> for &'_ $float_type {
            #[inline]
            fn eq(&self, other: &Bound<'_, PyFloat>) -> bool {
                other.value() as $float_type == **self
            }
        }

        impl PartialEq<$float_type> for Borrowed<'_, '_, PyFloat> {
            #[inline]
            fn eq(&self, other: &$float_type) -> bool {
                self.value() as $float_type == *other
            }
        }

        impl PartialEq<&$float_type> for Borrowed<'_, '_, PyFloat> {
            #[inline]
            fn eq(&self, other: &&$float_type) -> bool {
                self.value() as $float_type == **other
            }
        }

        impl PartialEq<Borrowed<'_, '_, PyFloat>> for $float_type {
            #[inline]
            fn eq(&self, other: &Borrowed<'_, '_, PyFloat>) -> bool {
                other.value() as $float_type == *self
            }
        }

        impl PartialEq<Borrowed<'_, '_, PyFloat>> for &$float_type {
            #[inline]
            fn eq(&self, other: &Borrowed<'_, '_, PyFloat>) -> bool {
                other.value() as $float_type == **self
            }
        }
    };
}

impl_partial_eq_for_float!(f64);
impl_partial_eq_for_float!(f32);

#[cfg(test)]
mod tests {
    use crate::{
        conversion::IntoPyObject,
        types::{PyAnyMethods, PyFloat, PyFloatMethods},
        Python,
    };

    macro_rules! num_to_py_object_and_back (
        ($func_name:ident, $t1:ty, $t2:ty) => (
            #[test]
            fn $func_name() {
                use assert_approx_eq::assert_approx_eq;

                Python::attach(|py| {

                let val = 123 as $t1;
                let obj = val.into_pyobject(py).unwrap();
                assert_approx_eq!(obj.extract::<$t2>().unwrap(), val as $t2);
                });
            }
        )
    );

    num_to_py_object_and_back!(to_from_f64, f64, f64);
    num_to_py_object_and_back!(to_from_f32, f32, f32);
    num_to_py_object_and_back!(int_to_float, i32, f64);

    #[test]
    fn test_float_value() {
        use assert_approx_eq::assert_approx_eq;

        Python::attach(|py| {
            let v = 1.23f64;
            let obj = PyFloat::new(py, 1.23);
            assert_approx_eq!(v, obj.value());
        });
    }

    #[test]
    fn test_pyfloat_comparisons() {
        Python::attach(|py| {
            let f_64 = 1.01f64;
            let py_f64 = PyFloat::new(py, 1.01);
            let py_f64_ref = &py_f64;
            let py_f64_borrowed = py_f64.as_borrowed();

            // Bound<'_, PyFloat> == f64 and vice versa
            assert_eq!(py_f64, f_64);
            assert_eq!(f_64, py_f64);

            // Bound<'_, PyFloat> == &f64 and vice versa
            assert_eq!(py_f64, &f_64);
            assert_eq!(&f_64, py_f64);

            // &Bound<'_, PyFloat> == &f64 and vice versa
            assert_eq!(py_f64_ref, f_64);
            assert_eq!(f_64, py_f64_ref);

            // &Bound<'_, PyFloat> == &f64 and vice versa
            assert_eq!(py_f64_ref, &f_64);
            assert_eq!(&f_64, py_f64_ref);

            // Borrowed<'_, '_, PyFloat> == f64 and vice versa
            assert_eq!(py_f64_borrowed, f_64);
            assert_eq!(f_64, py_f64_borrowed);

            // Borrowed<'_, '_, PyFloat> == &f64 and vice versa
            assert_eq!(py_f64_borrowed, &f_64);
            assert_eq!(&f_64, py_f64_borrowed);

            let f_32 = 2.02f32;
            let py_f32 = PyFloat::new(py, 2.02);
            let py_f32_ref = &py_f32;
            let py_f32_borrowed = py_f32.as_borrowed();

            // Bound<'_, PyFloat> == f32 and vice versa
            assert_eq!(py_f32, f_32);
            assert_eq!(f_32, py_f32);

            // Bound<'_, PyFloat> == &f32 and vice versa
            assert_eq!(py_f32, &f_32);
            assert_eq!(&f_32, py_f32);

            // &Bound<'_, PyFloat> == &f32 and vice versa
            assert_eq!(py_f32_ref, f_32);
            assert_eq!(f_32, py_f32_ref);

            // &Bound<'_, PyFloat> == &f32 and vice versa
            assert_eq!(py_f32_ref, &f_32);
            assert_eq!(&f_32, py_f32_ref);

            // Borrowed<'_, '_, PyFloat> == f32 and vice versa
            assert_eq!(py_f32_borrowed, f_32);
            assert_eq!(f_32, py_f32_borrowed);

            // Borrowed<'_, '_, PyFloat> == &f32 and vice versa
            assert_eq!(py_f32_borrowed, &f_32);
            assert_eq!(&f_32, py_f32_borrowed);
        });
    }
}
