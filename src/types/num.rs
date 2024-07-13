use crate::{ffi, PyAny};

/// Represents a Python `int` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyInt>`][crate::Py] or [`Bound<'py, PyInt>`][crate::Bound].
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](crate::conversion::ToPyObject)
/// and [`extract`](super::PyAnyMethods::extract)
/// with the primitive Rust integer types.
#[repr(transparent)]
pub struct PyInt(PyAny);

pyobject_native_type_core!(PyInt, pyobject_native_static_type_object!(ffi::PyLong_Type), #checkfunction=ffi::PyLong_Check);
