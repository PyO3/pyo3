#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::{ffi, types::any::PyAnyMethods, Bound, PyAny, Python};

/// Represents a Python [`types.GenericAlias`](https://docs.python.org/3/library/types.html#types.GenericAlias) object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyGenericAlias>`][crate::Py] or [`Bound<'py, PyGenericAlias>`][Bound].
#[repr(transparent)]
pub struct PyGenericAlias(PyAny);

pyobject_native_type!(
    PyGenericAlias,
    ffi::PyDictObject,
    pyobject_native_static_type_object!(ffi::Py_GenericAliasType),
    #checkfunction=ffi::PyGenericAlias_Check
);

impl PyGenericAlias {
    /// Creates a new Python GenericAlias object.
    ///
    /// origin should be a non-parameterized generic class.
    /// args should be a tuple (possibly of length 1) of types which parameterize origin.
    pub fn new<'py>(
        py: Python<'py>,
        origin: &Bound<'py, PyAny>,
        args: &Bound<'py, PyAny>,
    ) -> Bound<'py, PyGenericAlias> {
        unsafe {
            ffi::Py_GenericAlias(origin.as_ptr(), args.as_ptr())
                .assume_owned(py)
                .downcast_into_unchecked()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instance::BoundObject;
    use crate::types::any::PyAnyMethods;
    use crate::{ffi, Python};

    use super::PyGenericAlias;

    // Tests that PyGenericAlias::new is identical to types.GenericAlias
    // created from Python.
    #[test]
    fn equivalency_test() {
        Python::with_gil(|py| {
            let list_int = py
                .eval(ffi::c_str!("list[int]"), None, None)
                .unwrap()
                .into_bound();

            let cls = py
                .eval(ffi::c_str!("list"), None, None)
                .unwrap()
                .into_bound();
            let key = py
                .eval(ffi::c_str!("(int,)"), None, None)
                .unwrap()
                .into_bound();
            let generic_alias = PyGenericAlias::new(py, &cls, &key);

            assert!(generic_alias.eq(list_int).unwrap());
        })
    }
}
