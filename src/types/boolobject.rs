#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    exceptions::PyTypeError, ffi, instance::Bound, FromPyObject, IntoPy, PyAny, PyNativeType,
    PyObject, PyResult, Python, ToPyObject,
};

/// Represents a Python `bool`.
#[repr(transparent)]
pub struct PyBool(PyAny);

pyobject_native_type!(PyBool, ffi::PyObject, pyobject_native_static_type_object!(ffi::PyBool_Type), #checkfunction=ffi::PyBool_Check);

impl PyBool {
    /// Depending on `val`, returns `true` or `false`.
    #[inline]
    pub fn new(py: Python<'_>, val: bool) -> &PyBool {
        unsafe { py.from_borrowed_ptr(if val { ffi::Py_True() } else { ffi::Py_False() }) }
    }

    /// Gets whether this boolean is `true`.
    #[inline]
    pub fn is_true(&self) -> bool {
        self.as_bound().is_true()
    }
}

/// Implementation of functionality for [`PyBool`].
///
/// These methods are defined for the `Bound<'py, PyBool>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyBool")]
pub trait PyBoolMethods<'py> {
    /// Gets whether this boolean is `true`.
    fn is_true(&self) -> bool;
}

impl<'py> PyBoolMethods<'py> for Bound<'py, PyBool> {
    #[inline]
    fn is_true(&self) -> bool {
        self.as_ptr() == unsafe { crate::ffi::Py_True() }
    }
}

/// Converts a Rust `bool` to a Python `bool`.
impl ToPyObject for bool {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        unsafe {
            PyObject::from_borrowed_ptr(
                py,
                if *self {
                    ffi::Py_True()
                } else {
                    ffi::Py_False()
                },
            )
        }
    }
}

impl IntoPy<PyObject> for bool {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyBool::new(py, self).into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("bool")
    }
}

/// Converts a Python `bool` to a Rust `bool`.
///
/// Fails with `TypeError` if the input is not a Python `bool`.
impl<'source> FromPyObject<'source> for bool {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        let err = match obj.downcast::<PyBool>() {
            Ok(obj) => return Ok(obj.is_true()),
            Err(err) => err,
        };

        if obj
            .get_type()
            .name()
            .map_or(false, |name| name == "numpy.bool_")
        {
            let missing_conversion = |obj: &PyAny| {
                PyTypeError::new_err(format!(
                    "object of type '{}' does not define a '__bool__' conversion",
                    obj.get_type()
                ))
            };

            #[cfg(not(any(Py_LIMITED_API, PyPy)))]
            unsafe {
                let ptr = obj.as_ptr();

                if let Some(tp_as_number) = (*(*ptr).ob_type).tp_as_number.as_ref() {
                    if let Some(nb_bool) = tp_as_number.nb_bool {
                        match (nb_bool)(ptr) {
                            0 => return Ok(false),
                            1 => return Ok(true),
                            _ => return Err(crate::PyErr::fetch(obj.py())),
                        }
                    }
                }

                return Err(missing_conversion(obj));
            }

            #[cfg(any(Py_LIMITED_API, PyPy))]
            {
                let meth = obj
                    .lookup_special(crate::intern!(obj.py(), "__bool__"))?
                    .ok_or_else(|| missing_conversion(obj))?;

                let obj = meth.call0()?.downcast::<PyBool>()?;
                return Ok(obj.is_true());
            }
        }

        Err(err.into())
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyAny, PyBool};
    use crate::Python;
    use crate::ToPyObject;

    #[test]
    fn test_true() {
        Python::with_gil(|py| {
            assert!(PyBool::new(py, true).is_true());
            let t: &PyAny = PyBool::new(py, true).into();
            assert!(t.extract::<bool>().unwrap());
            assert!(true.to_object(py).is(PyBool::new(py, true)));
        });
    }

    #[test]
    fn test_false() {
        Python::with_gil(|py| {
            assert!(!PyBool::new(py, false).is_true());
            let t: &PyAny = PyBool::new(py, false).into();
            assert!(!t.extract::<bool>().unwrap());
            assert!(false.to_object(py).is(PyBool::new(py, false)));
        });
    }
}
