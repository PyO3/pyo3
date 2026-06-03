#[cfg(Py_GIL_DISABLED)]
mod atomic_c_ulong {
    pub struct GetAtomicCULong<const WIDTH: usize>();

    pub trait AtomicCULongType {
        type Type;
    }
    impl AtomicCULongType for GetAtomicCULong<32> {
        type Type = core::sync::atomic::AtomicU32;
    }
    impl AtomicCULongType for GetAtomicCULong<64> {
        type Type = core::sync::atomic::AtomicU64;
    }

    pub type TYPE =
        <GetAtomicCULong<{ core::mem::size_of::<core::ffi::c_ulong>() * 8 }> as AtomicCULongType>::Type;
}

/// Typedef for an atomic integer to match the platform-dependent c_ulong type.
#[cfg(Py_GIL_DISABLED)]
#[doc(hidden)]
pub type AtomicCULong = atomic_c_ulong::TYPE;

/// Guard to hang the current thread indefinitely when dropped.
#[cfg(not(any(Py_3_14, target_arch = "wasm32")))]
pub struct HangThread;

#[cfg(not(any(Py_3_14, target_arch = "wasm32")))]
impl Drop for HangThread {
    fn drop(&mut self) {
        loop {
            std::thread::park(); // Block forever.
        }
    }
}