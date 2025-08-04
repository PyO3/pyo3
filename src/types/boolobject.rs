#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    exceptions::PyTypeError, ffi, ffi_ptr_ext::FfiPtrExt, instance::Bound,
    types::typeobject::PyTypeMethods, Borrowed, FromPyObject, PyAny, PyResult, Python,
};

use super::any::PyAnyMethods;
use crate::conversion::IntoPyObject;
use std::convert::Infallible;
use std::ptr;

/// Represents a Python `bool`.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyBool>`][crate::Py] or [`Bound<'py, PyBool>`][Bound].
///
/// For APIs available on `bool` objects, see the [`PyBoolMethods`] trait which is implemented for
/// [`Bound<'py, PyBool>`][Bound].
#[repr(transparent)]
pub struct PyBool(PyAny);

pyobject_native_type!(PyBool, ffi::PyObject, pyobject_native_static_type_object!(ffi::PyBool_Type), #checkfunction=ffi::PyBool_Check);

impl PyBool {
    /// Depending on `val`, returns `true` or `false`.
    ///
    /// # Note
    /// This returns a [`Borrowed`] reference to one of Pythons `True` or
    /// `False` singletons
    #[inline]
    pub fn new(py: Python<'_>, val: bool) -> Borrowed<'_, '_, Self> {
        unsafe {
            if val { ffi::Py_True() } else { ffi::Py_False() }
                .assume_borrowed(py)
                .cast_unchecked()
        }
    }
}

/// Implementation of functionality for [`PyBool`].
///
/// These methods are defined for the `Bound<'py, PyBool>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyBool")]
pub trait PyBoolMethods<'py>: crate::sealed::Sealed {
    /// Gets whether this boolean is `true`.
    fn is_true(&self) -> bool;
}

impl<'py> PyBoolMethods<'py> for Bound<'py, PyBool> {
    #[inline]
    fn is_true(&self) -> bool {
        unsafe { ptr::eq(self.as_ptr(), ffi::Py_True()) }
    }
}

/// Compare `Bound<PyBool>` with `bool`.
impl PartialEq<bool> for Bound<'_, PyBool> {
    #[inline]
    fn eq(&self, other: &bool) -> bool {
        self.as_borrowed() == *other
    }
}

/// Compare `&Bound<PyBool>` with `bool`.
impl PartialEq<bool> for &'_ Bound<'_, PyBool> {
    #[inline]
    fn eq(&self, other: &bool) -> bool {
        self.as_borrowed() == *other
    }
}

/// Compare `Bound<PyBool>` with `&bool`.
impl PartialEq<&'_ bool> for Bound<'_, PyBool> {
    #[inline]
    fn eq(&self, other: &&bool) -> bool {
        self.as_borrowed() == **other
    }
}

/// Compare `bool` with `Bound<PyBool>`
impl PartialEq<Bound<'_, PyBool>> for bool {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyBool>) -> bool {
        *self == other.as_borrowed()
    }
}

/// Compare `bool` with `&Bound<PyBool>`
impl PartialEq<&'_ Bound<'_, PyBool>> for bool {
    #[inline]
    fn eq(&self, other: &&'_ Bound<'_, PyBool>) -> bool {
        *self == other.as_borrowed()
    }
}

/// Compare `&bool` with `Bound<PyBool>`
impl PartialEq<Bound<'_, PyBool>> for &'_ bool {
    #[inline]
    fn eq(&self, other: &Bound<'_, PyBool>) -> bool {
        **self == other.as_borrowed()
    }
}

/// Compare `Borrowed<PyBool>` with `bool`
impl PartialEq<bool> for Borrowed<'_, '_, PyBool> {
    #[inline]
    fn eq(&self, other: &bool) -> bool {
        self.is_true() == *other
    }
}

/// Compare `Borrowed<PyBool>` with `&bool`
impl PartialEq<&bool> for Borrowed<'_, '_, PyBool> {
    #[inline]
    fn eq(&self, other: &&bool) -> bool {
        self.is_true() == **other
    }
}

/// Compare `bool` with `Borrowed<PyBool>`
impl PartialEq<Borrowed<'_, '_, PyBool>> for bool {
    #[inline]
    fn eq(&self, other: &Borrowed<'_, '_, PyBool>) -> bool {
        *self == other.is_true()
    }
}

/// Compare `&bool` with `Borrowed<PyBool>`
impl PartialEq<Borrowed<'_, '_, PyBool>> for &'_ bool {
    #[inline]
    fn eq(&self, other: &Borrowed<'_, '_, PyBool>) -> bool {
        **self == other.is_true()
    }
}

impl<'py> IntoPyObject<'py> for bool {
    type Target = PyBool;
    type Output = Borrowed<'py, 'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = "bool";

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBool::new(py, self))
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::builtin("bool")
    }
}

impl<'py> IntoPyObject<'py> for &bool {
    type Target = PyBool;
    type Output = Borrowed<'py, 'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = bool::OUTPUT_TYPE;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
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
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "bool";

    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let err = match obj.cast::<PyBool>() {
            Ok(obj) => return Ok(obj.is_true()),
            Err(err) => err,
        };

        let is_numpy_bool = {
            let ty = obj.get_type();
            ty.module().is_ok_and(|module| module == "numpy")
                && ty
                    .name()
                    .is_ok_and(|name| name == "bool_" || name == "bool")
        };

        if is_numpy_bool {
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

                let obj = meth.call0()?.cast_into::<PyBool>()?;
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
    use crate::IntoPyObject;
    use crate::Python;

    #[test]
    fn test_true() {
        Python::attach(|py| {
            assert!(PyBool::new(py, true).is_true());
            let t = PyBool::new(py, true);
            assert!(t.extract::<bool>().unwrap());
            assert!(true.into_pyobject(py).unwrap().is(&*PyBool::new(py, true)));
        });
    }

    #[test]
    fn test_false() {
        Python::attach(|py| {
            assert!(!PyBool::new(py, false).is_true());
            let t = PyBool::new(py, false);
            assert!(!t.extract::<bool>().unwrap());
            assert!(false
                .into_pyobject(py)
                .unwrap()
                .is(&*PyBool::new(py, false)));
        });
    }

    #[test]
    fn test_pybool_comparisons() {
        Python::attach(|py| {
            let py_bool = PyBool::new(py, true);
            let py_bool_false = PyBool::new(py, false);
            let rust_bool = true;

            // Bound<'_, PyBool> == bool
            assert_eq!(*py_bool, rust_bool);
            assert_ne!(*py_bool_false, rust_bool);

            // Bound<'_, PyBool> == &bool
            assert_eq!(*py_bool, &rust_bool);
            assert_ne!(*py_bool_false, &rust_bool);

            // &Bound<'_, PyBool> == bool
            assert_eq!(&*py_bool, rust_bool);
            assert_ne!(&*py_bool_false, rust_bool);

            // &Bound<'_, PyBool> == &bool
            assert_eq!(&*py_bool, &rust_bool);
            assert_ne!(&*py_bool_false, &rust_bool);

            // bool == Bound<'_, PyBool>
            assert_eq!(rust_bool, *py_bool);
            assert_ne!(rust_bool, *py_bool_false);

            // bool == &Bound<'_, PyBool>
            assert_eq!(rust_bool, &*py_bool);
            assert_ne!(rust_bool, &*py_bool_false);

            // &bool == Bound<'_, PyBool>
            assert_eq!(&rust_bool, *py_bool);
            assert_ne!(&rust_bool, *py_bool_false);

            // &bool == &Bound<'_, PyBool>
            assert_eq!(&rust_bool, &*py_bool);
            assert_ne!(&rust_bool, &*py_bool_false);

            // Borrowed<'_, '_, PyBool> == bool
            assert_eq!(py_bool, rust_bool);
            assert_ne!(py_bool_false, rust_bool);

            // Borrowed<'_, '_, PyBool> == &bool
            assert_eq!(py_bool, &rust_bool);
            assert_ne!(py_bool_false, &rust_bool);

            // bool == Borrowed<'_, '_, PyBool>
            assert_eq!(rust_bool, py_bool);
            assert_ne!(rust_bool, py_bool_false);

            // &bool == Borrowed<'_, '_, PyBool>
            assert_eq!(&rust_bool, py_bool);
            assert_ne!(&rust_bool, py_bool_false);
            assert_eq!(py_bool, rust_bool);
            assert_ne!(py_bool_false, rust_bool);
        })
    }
}
