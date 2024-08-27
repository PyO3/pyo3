//! Forward definitions to match CPython's `pytypedefs.h`.

#[cfg(not(feature = "unlimited-api"))]
opaque_struct!(PyCodeObject);

#[cfg(not(feature = "unlimited-api"))]
opaque_struct!(PyFrameObject);
