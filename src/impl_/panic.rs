/// Type which will panic if dropped.
///
/// If this is dropped during a panic, this will cause an abort.
///
/// Use this to avoid letting unwinds cross through the FFI boundary, which is UB.
pub struct PanicTrap {
    msg: &'static str,
}

impl PanicTrap {
    #[inline]
    pub const fn new(msg: &'static str) -> Self {
        Self { msg }
    }

    #[inline]
    pub const fn disarm(self) {
        std::mem::forget(self)
    }
}

impl Drop for PanicTrap {
    fn drop(&mut self) {
        // Panic here will abort the process, assuming in an unwind.
        panic!("{}", self.msg)
    }
}
