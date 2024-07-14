use super::any::PyAnyMethods;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{ffi, ffi_ptr_ext::FfiPtrExt, instance::Bound, FromPyObject, IntoPy, PyAny, PyErr, PyObject, PyResult, Python, ToPyObject, Borrowed};
use std::os::raw::c_double;

/// Represents a Python `float` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFloat>`][crate::Py] or [`Bound<'py, PyFloat>`][Bound].
///
/// For APIs available on `float` objects, see the [`PyFloatMethods`] trait which is implemented for
/// [`Bound<'py, PyFloat>`][Bound].
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`] and [`extract`][PyAnyMethods::extract]
/// with [`f32`]/[`f64`].
#[repr(transparent)]
pub struct PyFloat(PyAny);

pyobject_native_type!(
    PyFloat,
    ffi::PyFloatObject,
    pyobject_native_static_type_object!(ffi::PyFloat_Type),
    #checkfunction=ffi::PyFloat_Check
);

impl PyFloat {
    /// Creates a new Python `float` object.
    pub fn new_bound(py: Python<'_>, val: c_double) -> Bound<'_, PyFloat> {
        unsafe {
            ffi::PyFloat_FromDouble(val)
                .assume_owned(py)
                .downcast_into_unchecked()
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

impl ToPyObject for f64 {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyFloat::new_bound(py, *self).into()
    }
}

impl IntoPy<PyObject> for f64 {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyFloat::new_bound(py, self).into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("float")
    }
}

impl<'py> FromPyObject<'py> for f64 {
    // PyFloat_AsDouble returns -1.0 upon failure
    #![allow(clippy::float_cmp)]
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        // On non-limited API, .value() uses PyFloat_AS_DOUBLE which
        // allows us to have an optimized fast path for the case when
        // we have exactly a `float` object (it's not worth going through
        // `isinstance` machinery for subclasses).
        #[cfg(not(Py_LIMITED_API))]
        if let Ok(float) = obj.downcast_exact::<PyFloat>() {
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

impl ToPyObject for f32 {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyFloat::new_bound(py, f64::from(*self)).into()
    }
}

impl IntoPy<PyObject> for f32 {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyFloat::new_bound(py, f64::from(self)).into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("float")
    }
}

impl<'py> FromPyObject<'py> for f32 {
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
            if let Ok(val) = self.extract::<$float_type>(){
                val == *other
            } else {
                false
            }
        }
    }

    impl PartialEq<$float_type> for &Bound<'_, PyFloat> {
        #[inline]
        fn eq(&self, other: &$float_type) -> bool {
            if let Ok(val) = self.extract::<$float_type>(){
                val == *other
            } else {
                false
            }
        }
    }

    impl PartialEq<&$float_type> for Bound<'_, PyFloat> {
        #[inline]
        fn eq(&self, other: &&$float_type) -> bool {
            if let Ok(val) = self.extract::<$float_type>(){
                val == **other
            } else {
                false
            }
        }
    }

    impl PartialEq<Bound<'_, PyFloat>> for $float_type {
        #[inline]
        fn eq(&self, other: &Bound<'_, PyFloat>) -> bool {
            if let Ok(val) = other.extract::<$float_type>() {
                val == *self
            } else {
                false
            }
        }
    }

    impl PartialEq<&'_ Bound<'_, PyFloat>> for $float_type {
        #[inline]
        fn eq(&self, other: &&'_ Bound<'_, PyFloat>) -> bool {
            if let Ok(val) = other.extract::<$float_type>() {
                val == *self
            } else {
                false
            }
        }
    }

    impl PartialEq<Bound<'_, PyFloat>> for &'_ $float_type {
        #[inline]
        fn eq(&self, other: &Bound<'_, PyFloat>) -> bool {
            if let Ok(val) = other.extract::<$float_type>() {
                val == **self
            } else {
                false
            }
        }
    }

    impl PartialEq<$float_type> for Borrowed<'_, '_, PyFloat> {
        #[inline]
        fn eq(&self, other: &$float_type) -> bool {
            if let Ok(val) = self.extract::<$float_type>(){
                val == *other
            } else {
                false
            }
        }
    }

    impl PartialEq<&$float_type> for Borrowed<'_, '_, PyFloat> {
        #[inline]
        fn eq(&self, other: &&$float_type) -> bool {
            if let Ok(val) = self.extract::<$float_type>(){
                val == **other
            } else {
                false
            }
        }
    }

    impl PartialEq<Borrowed<'_, '_, PyFloat>> for $float_type {
        #[inline]
        fn eq(&self, other: &Borrowed<'_, '_, PyFloat>) -> bool {
            if let Ok(val) = other.extract::<$float_type>() {
                val == *self
            } else {
                false
            }
        }
    }

    impl PartialEq<Borrowed<'_, '_, PyFloat>> for &$float_type {
        #[inline]
        fn eq(&self, other: &Borrowed<'_, '_, PyFloat>) -> bool {
            if let Ok(val) = other.extract::<$float_type>() {
                val == **self
            } else {
                false
            }
        }
    }
}}

impl_partial_eq_for_float!(f64);
impl_partial_eq_for_float!(f32);

#[cfg(test)]
mod tests {
    use crate::{
        types::{PyFloat, PyFloatMethods},
        Python, ToPyObject,
    };

    macro_rules! num_to_py_object_and_back (
        ($func_name:ident, $t1:ty, $t2:ty) => (
            #[test]
            fn $func_name() {
                use assert_approx_eq::assert_approx_eq;

                Python::with_gil(|py| {

                let val = 123 as $t1;
                let obj = val.to_object(py);
                assert_approx_eq!(obj.extract::<$t2>(py).unwrap(), val as $t2);
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

        Python::with_gil(|py| {
            let v = 1.23f64;
            let obj = PyFloat::new_bound(py, 1.23);
            assert_approx_eq!(v, obj.value());
        });
    }

    #[test]
    fn test_pyfloat_comparisons() {
        Python::with_gil(|py| {
            let f_64 = 1.01f64;
            let py_f64 = PyFloat::new_bound(py, 1.01);
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
            let py_f32 = PyFloat::new_bound(py, 2.02);
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
