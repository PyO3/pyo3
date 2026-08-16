#[cfg(not(any(PyPy, GraalPy, RustPython)))]
use crate::{ffi, PyAny};

/// Represents a Python `dict_items`.
#[cfg(not(any(PyPy, GraalPy, RustPython)))]
#[repr(transparent)]
pub struct PyDictItems(PyAny);

#[cfg(not(any(PyPy, GraalPy, RustPython)))]
pyobject_native_type_core!(
    PyDictItems,
    pyobject_native_static_type_object!(ffi::PyDictItems_Type),
    "builtins",
    "dict_items",
    #checkfunction=ffi::PyDictItems_Check
);
