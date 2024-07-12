use crate::{ffi, PyAny};

/// Represents a Python `int` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyLong>`][crate::Py] or [`Bound<'py, PyLong>`][crate::Bound].
///
/// You can usually avoid directly working with this type
/// by using [`ToPyObject`](crate::conversion::ToPyObject)
/// and [`extract`](super::PyAnyMethods::extract)
/// with the primitive Rust integer types.
#[repr(transparent)]
pub struct PyLong(PyAny);

pyobject_native_type_core!(PyLong, pyobject_native_static_type_object!(ffi::PyLong_Type), #checkfunction=ffi::PyLong_Check);
