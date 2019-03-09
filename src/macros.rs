
#[macro_export]
/// Constructs a `&'static CStr` literal.
macro_rules! cstr {
    ($s: tt) => {
        // TODO: verify that $s is a string literal without nuls
        unsafe { ::std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr() as *const _) }
    };
}
