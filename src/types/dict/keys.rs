#[cfg(not(any(PyPy, GraalPy, RustPython)))]
use crate::{ffi, PyAny};

/// Represents a Python `dict_keys`.
#[cfg(not(any(PyPy, GraalPy, RustPython)))]
#[repr(transparent)]
pub struct PyDictKeys(PyAny);

#[cfg(not(any(PyPy, GraalPy, RustPython)))]
pyobject_native_type_core!(
    PyDictKeys,
    pyobject_native_static_type_object!(ffi::PyDictKeys_Type),
    "builtins",
    "dict_keys",
    #checkfunction=ffi::PyDictKeys_Check
);
