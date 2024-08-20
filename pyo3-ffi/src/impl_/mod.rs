#[cfg(Py_GIL_DISABLED)]
mod atomic_c_ulong {
    pub(crate) struct GetAtomicCULong<const WIDTH: usize>();

    pub(crate) trait AtomicCULongType {
        type Type;
    }
    impl AtomicCULongType for GetAtomicCULong<32> {
        type Type = std::sync::atomic::AtomicU32;
    }
    impl AtomicCULongType for GetAtomicCULong<64> {
        type Type = std::sync::atomic::AtomicU64;
    }

    pub(crate) type TYPE = GetAtomicCULong<{ std::mem::size_of::<std::os::raw::c_ulong>() * 8 }>;
}

/// Typedef for an atomic integer to match the platform-dependent c_ulong type.
#[cfg(Py_GIL_DISABLED)]
pub(crate) type AtomicCULong = atomic_c_ulong::TYPE;
