#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::types::any::PyAnyMethods;
use crate::types::typeobject::PyTypeMethods;
use crate::{
    exceptions::PyTypeError, ffi, ffi_ptr_ext::FfiPtrExt, instance::Bound, Borrowed, FromPyObject,
    IntoPy, PyAny, PyNativeType, PyObject, PyResult, Python, ToPyObject,
};

/// Represents a Python `bool`.
#[repr(transparent)]
pub struct PyBool(PyAny);

pyobject_native_type!(PyBool, ffi::PyObject, pyobject_native_static_type_object!(ffi::PyBool_Type), #checkfunction=ffi::PyBool_Check);

impl PyBool {
    /// Deprecated form of [`PyBool::new_bound`]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyBool::new` will be replaced by `PyBool::new_bound` in a future PyO3 version"
        )
    )]
    #[inline]
    pub fn new(py: Python<'_>, val: bool) -> &PyBool {
        unsafe { py.from_borrowed_ptr(if val { ffi::Py_True() } else { ffi::Py_False() }) }
    }

    /// Depending on `val`, returns `true` or `false`.
    ///
    /// # Note
    /// This returns a [`Borrowed`] reference to one of Pythons `True` or
    /// `False` singletons
    #[inline]
    pub fn new_bound(py: Python<'_>, val: bool) -> Borrowed<'_, '_, Self> {
        unsafe {
            if val { ffi::Py_True() } else { ffi::Py_False() }
                .assume_borrowed(py)
                .downcast_unchecked()
        }
    }

    /// Gets whether this boolean is `true`.
    #[inline]
    pub fn is_true(&self) -> bool {
        self.as_borrowed().is_true()
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
        PyBool::new_bound(py, self).into_py(py)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("bool")
    }
}

/// Converts a Python `bool` to a Rust `bool`.
///
/// Fails with `TypeError` if the input is not a Python `bool`.
impl FromPyObject<'_> for bool {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let err = match obj.downcast::<PyBool>() {
            Ok(obj) => return Ok(obj.is_true()),
            Err(err) => err,
        };

        if obj
            .get_type()
            .name()
            .map_or(false, |name| name == "numpy.bool_")
        {
            let missing_conversion = |obj: &Bound<'_, PyAny>| {
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

                let obj = meth.call0()?.downcast_into::<PyBool>()?;
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
    use crate::types::any::PyAnyMethods;
    use crate::types::boolobject::PyBoolMethods;
    use crate::types::PyBool;
    use crate::Python;
    use crate::ToPyObject;

    #[test]
    fn test_true() {
        Python::with_gil(|py| {
            assert!(PyBool::new_bound(py, true).is_true());
            let t = PyBool::new_bound(py, true);
            assert!(t.as_any().extract::<bool>().unwrap());
            assert!(true.to_object(py).is(&*PyBool::new_bound(py, true)));
        });
    }

    #[test]
    fn test_false() {
        Python::with_gil(|py| {
            assert!(!PyBool::new_bound(py, false).is_true());
            let t = PyBool::new_bound(py, false);
            assert!(!t.as_any().extract::<bool>().unwrap());
            assert!(false.to_object(py).is(&*PyBool::new_bound(py, false)));
        });
    }
}
