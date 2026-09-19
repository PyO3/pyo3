#[cfg(not(any(PyPy, GraalPy, RustPython)))]
use crate::{ffi, PyAny};

/// Represents a Python `dict_values`.
#[cfg(not(any(PyPy, GraalPy, RustPython)))]
#[repr(transparent)]
pub struct PyDictValues(PyAny);

#[cfg(not(any(PyPy, GraalPy, RustPython)))]
pyobject_native_type_core!(
    PyDictValues,
    pyobject_native_static_type_object!(ffi::PyDictValues_Type),
    "builtins",
    "dict_values",
    #checkfunction=ffi::PyDictValues_Check
);
