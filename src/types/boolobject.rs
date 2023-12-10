#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::{
    exceptions::PyTypeError, ffi, instance::Py2, FromPyObject, IntoPy, PyAny, PyObject, PyResult,
    Python, ToPyObject,
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
        Py2::borrowed_from_gil_ref(&self).is_true()
    }
}

/// Implementation of functionality for [`PyBool`].
///
/// These methods are defined for the `Py2<'py, PyBool>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyBool")]
pub trait PyBoolMethods<'py> {
    /// Gets whether this boolean is `true`.
    fn is_true(&self) -> bool;
}

impl<'py> PyBoolMethods<'py> for Py2<'py, PyBool> {
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
        if let Ok(obj) = obj.downcast::<PyBool>() {
            return Ok(obj.is_true());
        }

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

            Err(PyTypeError::new_err("object has no __bool__ magic method"))
        }

        #[cfg(any(Py_LIMITED_API, PyPy))]
        {
            let meth = obj
                .lookup_special(crate::intern!(obj.py(), "__bool__"))?
                .ok_or_else(|| PyTypeError::new_err("object has no __bool__ magic method"))?;

            let obj = meth.call0()?.downcast::<PyBool>()?;
            Ok(obj.is_true())
        }
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        Self::type_output()
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyAny, PyBool, PyModule};
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

    #[test]
    fn test_magic_method() {
        Python::with_gil(|py| {
            let module = PyModule::from_code(
                py,
                r#"
class A:
    def __bool__(self): return True
class B:
    def __bool__(self): return "not a bool"
class C:
    def __len__(self): return 23
class D:
    pass
                "#,
                "test.py",
                "test",
            )
            .unwrap();

            let a = module.getattr("A").unwrap().call0().unwrap();
            assert!(a.extract::<bool>().unwrap());

            let b = module.getattr("B").unwrap().call0().unwrap();
            assert!(matches!(
                &*b.extract::<bool>().unwrap_err().to_string(),
                "TypeError: 'str' object cannot be converted to 'PyBool'"
                    | "TypeError: __bool__ should return bool, returned str"
            ));

            let c = module.getattr("C").unwrap().call0().unwrap();
            assert_eq!(
                c.extract::<bool>().unwrap_err().to_string(),
                "TypeError: object has no __bool__ magic method",
            );

            let d = module.getattr("D").unwrap().call0().unwrap();
            assert_eq!(
                d.extract::<bool>().unwrap_err().to_string(),
                "TypeError: object has no __bool__ magic method",
            );
        });
    }
}
