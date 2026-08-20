use core::hash::Hash;

pub use ahash::RandomState;
use pyo3_ffi::Py_hash_t;

pub trait PyHashable: Hash {
    const RANDOM_STATE: RandomState;

    fn py_hash(&self) -> Py_hash_t {
        Self::RANDOM_STATE.hash_one(self) as _
    }
}
