use core::hash::Hash;

pub use ahash::RandomState;
use pyo3_ffi::Py_hash_t;

pub trait PyHashable: Hash {
    const BUILD_HASHER: RandomState;

    fn py_hash(&self) -> Py_hash_t {
        Self::BUILD_HASHER.hash_one(self) as _
    }
}
