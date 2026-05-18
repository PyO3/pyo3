use core::num::NonZero;

extern_libpython! {
    pub fn PyThread_get_thread_ident() -> NonZero<u64>;
}
