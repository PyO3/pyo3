//! C API Compatibility Shims
//!
//! Some CPython C API functions added in recent versions of Python are
//! inherently safer to use than older C API constructs. This module
//! exposes functions available on all Python versions that wrap the
//! old C API on old Python versions and wrap the function directly
//! on newer Python versions.

// Unless otherwise noted, the compatibility shims are adapted from
// the pythoncapi-compat project: https://github.com/python/pythoncapi-compat

/// Internal helper macro which defines compatibility shims for C API functions, deferring to a
/// re-export when that's available.
macro_rules! compat_function {
    (
        originally_defined_for($cfg:meta);

        $(#[$attrs:meta])*
        pub unsafe fn $name:ident($($arg_names:ident: $arg_types:ty),* $(,)?) -> $ret:ty $body:block
    ) => {
        // Define as a standalone function under docsrs cfg so that this shows as a unique function in the docs,
        // not a re-export (the re-export has the wrong visibility)
        #[cfg(any(docsrs, not($cfg)))]
        #[cfg_attr(docsrs, doc(cfg(all())))]
        $(#[$attrs])*
        pub unsafe fn $name(
            $($arg_names: $arg_types,)*
        ) -> $ret $body

        #[cfg(all($cfg, not(docsrs)))]
        pub use $crate::$name;
    };
}

mod py_3_10;
mod py_3_13;

pub use self::py_3_10::*;
pub use self::py_3_13::*;
