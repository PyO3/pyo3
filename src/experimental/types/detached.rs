use std::ptr::NonNull;

use crate::experimental::types as attached;
use crate::ffi;

/// FIXME: this design is horrible, instead call this
/// DetachedPyAny and export it from the types module

#[repr(transparent)]
pub struct PyAny(NonNull<ffi::PyObject>);

pub trait PyAttachedType<'py>: 'py + Sized {
    type Detached: From<Self>;
}

impl<'py> PyAttachedType<'py> for attached::PyAny<'py> {
    type Detached = PyAny;
}

impl From<attached::PyAny<'_>> for PyAny {
    fn from(other: attached::PyAny<'_>) -> Self {
        Self(other.into_non_null())
    }
}
