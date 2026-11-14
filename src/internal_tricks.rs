use core::ptr::NonNull;

use crate::platform::prelude::*;

#[cfg(wip_feature_std)]
macro_rules! pyo3_exception {
    ($doc: expr, $name: ident, $base: ty) => {
        #[doc = $doc]
        #[repr(transparent)]
        pub struct $name($crate::PyAny);

        $crate::impl_exception_boilerplate!($name);

        $crate::create_exception_type_object!(pyo3_runtime, $name, $base, Some($doc));
    };
}

// TODO: use Box::into_non_null when stabilized
pub(crate) fn box_into_non_null<T>(b: Box<T>) -> NonNull<T> {
    NonNull::from(Box::leak(b))
}

/// Replacement for the unstable `<*mut [T; N]>::as_mut_ptr` method, which avoids
/// possibility of type getting lost from using e.g. `.cast()` to change array
/// type to point to the data.
pub(crate) const fn array_ptr_as_mut<T, const N: usize>(ptr: *mut [T; N]) -> *mut T {
    ptr.cast()
}
