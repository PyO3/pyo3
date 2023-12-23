#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    ffi, instance::Bound, FromPyObject, IntoPy, PyAny, PyErr, PyNativeType, PyObject, PyResult,
    Python, ToPyObject,
};
use std::os::raw::c_double;

/// Represents a Python `float` object.
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`] and [`extract`](PyAny::extract)
/// with `f32`/`f64`.
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
    pub fn new(py: Python<'_>, val: c_double) -> &PyFloat {
        unsafe { py.from_owned_ptr(ffi::PyFloat_FromDouble(val)) }
    }

    /// Gets the value of this float.
    pub fn value(&self) -> c_double {
        self.as_bound().value()
    }
}

/// Implementation of functionality for [`PyFloat`].
///
/// These methods are defined for the `Bound<'py, PyFloat>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyFloat")]
pub trait PyFloatMethods<'py> {
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
        PyFloat::new(py, *self).into()
    }
}

impl IntoPy<PyObject> for f64 {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyFloat::new(py, self).into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("float")
    }
}

impl<'source> FromPyObject<'source> for f64 {
    // PyFloat_AsDouble returns -1.0 upon failure
    #![allow(clippy::float_cmp)]
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
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
        PyFloat::new(py, f64::from(*self)).into()
    }
}

impl IntoPy<PyObject> for f32 {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyFloat::new(py, f64::from(self)).into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("float")
    }
}

impl<'source> FromPyObject<'source> for f32 {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        Ok(obj.extract::<f64>()? as f32)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

#[cfg(test)]
mod tests {
    use crate::{types::PyFloat, Python, ToPyObject};

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
            let obj = PyFloat::new(py, 1.23);
            assert_approx_eq!(v, obj.value());
        });
    }
}
