use crate::err::PyResult;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::py_result_ext::PyResultExt;
use crate::{ffi, Bound, PyAny, Python};

/// Represents a Python [`types.GenericAlias`](https://docs.python.org/3/library/types.html#types.GenericAlias) object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyGenericAlias>`][crate::Py] or [`Bound<'py, PyGenericAlias>`][Bound].
///
/// This type is particularly convenient for users implementing
/// [`__class_getitem__`](https://docs.python.org/3/reference/datamodel.html#object.__class_getitem__)
/// for PyO3 classes to allow runtime parameterization.
#[repr(transparent)]
pub struct PyGenericAlias(PyAny);

pyobject_native_type!(
    PyGenericAlias,
    ffi::PyDictObject,
    pyobject_native_static_type_object!(ffi::Py_GenericAliasType)
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
    ) -> PyResult<Bound<'py, PyGenericAlias>> {
        unsafe {
            ffi::Py_GenericAlias(origin.as_ptr(), args.as_ptr())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
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
        Python::attach(|py| {
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
            let generic_alias = PyGenericAlias::new(py, &cls, &key).unwrap();

            assert!(generic_alias.eq(list_int).unwrap());
        })
    }
}
