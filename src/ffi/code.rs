#[cfg(not(Py_LIMITED_API))]
pub use crate::ffi::PyCodeObject;
#[cfg(Py_LIMITED_API)]
opaque_struct!(PyCodeObject);
