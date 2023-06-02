use crate::{ffi, PyAny};

/// Represents a Python `int` object.
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](crate::conversion::ToPyObject)
/// and [`extract`](PyAny::extract)
/// with the primitive Rust integer types.
#[repr(transparent)]
pub struct PyLong(PyAny);

pyobject_native_type_core!(PyLong, ffi::PyLong_Type, #checkfunction=ffi::PyLong_Check);
