#[cfg(Py_GIL_DISABLED)]
mod atomic_c_ulong {
    pub struct GetAtomicCULong<const WIDTH: usize>();

    pub trait AtomicCULongType {
        type Type;
    }
    impl AtomicCULongType for GetAtomicCULong<32> {
        type Type = std::sync::atomic::AtomicU32;
    }
    impl AtomicCULongType for GetAtomicCULong<64> {
        type Type = std::sync::atomic::AtomicU64;
    }

    pub type TYPE =
        <GetAtomicCULong<{ std::mem::size_of::<std::ffi::c_ulong>() * 8 }> as AtomicCULongType>::Type;
}

/// Typedef for an atomic integer to match the platform-dependent c_ulong type.
#[cfg(Py_GIL_DISABLED)]
#[doc(hidden)]
pub type AtomicCULong = atomic_c_ulong::TYPE;
