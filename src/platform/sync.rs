// TODO compile_error if parking_lot and std are both disabled

#[cfg(feature = "parking_lot")]
type OnceInner = parking_lot::Once;

#[cfg(not(feature = "parking_lot"))]
type OnceInner = std::sync::Once;

pub struct Once(OnceInner);

impl Default for Once {
    fn default() -> Self {
        Self::new()
    }
}

// #[cfg(feature = "parking_lot")]
impl Once {
    /// Creates a new `Once` value.
    #[inline(always)]
    #[must_use]
    pub const fn new() -> Once {
        Once(OnceInner::new())
    }

    #[inline]
    pub fn call_once(&self, f: impl FnOnce()) {
        self.0.call_once(f);
    }

    #[inline]
    pub fn call_once_force(&self, f: impl FnOnce()) {
        self.0.call_once_force(move |_| f());
    }

    #[cfg(feature = "parking_lot")]
    pub fn is_completed(&self) -> bool {
        matches!(self.0.state(), parking_lot::OnceState::Done)
    }

    #[cfg(not(feature = "parking_lot"))]
    #[inline(always)]
    pub fn is_completed(&self) -> bool {
        self.0.is_completed()
    }
}

pub mod non_poison {
    #[cfg(feature = "parking_lot")]
    pub use parking_lot::{Mutex, MutexGuard};

    #[cfg(not(feature = "parking_lot"))]
    pub use std::sync::MutexGuard;

    #[cfg(not(feature = "parking_lot"))]
    #[derive(Default, Debug)]
    pub struct Mutex<T: ?Sized> {
        #[allow(clippy::disallowed_types)]
        inner: std::sync::Mutex<T>,
    }

    #[cfg(not(feature = "parking_lot"))]
    impl<T> Mutex<T> {
        #[inline(always)]
        pub const fn new(t: T) -> Mutex<T> {
            Mutex {
                #[allow(clippy::disallowed_types)]
                inner: std::sync::Mutex::new(t),
            }
        }
    }

    #[cfg(not(feature = "parking_lot"))]
    impl<T: ?Sized> Mutex<T> {
        #[inline(always)]
        pub fn lock(&self) -> MutexGuard<'_, T> {
            self.inner.lock().unwrap_or_else(|e| e.into_inner())
        }

        // TODO try_lock
    }

    #[cfg(not(feature = "parking_lot"))]
    impl<T> From<T> for Mutex<T> {
        #[inline(always)]
        fn from(t: T) -> Self {
            Mutex {
                #[allow(clippy::disallowed_types)]
                inner: std::sync::Mutex::new(t),
            }
        }
    }

    #[cfg(not(feature = "parking_lot"))]
    impl<T> crate::sealed::Sealed for Mutex<T> {}

    #[cfg(not(feature = "parking_lot"))]
    impl<T> crate::sync::MutexExt<T> for Mutex<T> {
        type LockResult<'a>
            = MutexGuard<'a, T>
        where
            T: 'a;

        fn lock_py_attached(&self, py: crate::prelude::Python<'_>) -> Self::LockResult<'_> {
            self.inner
                .lock_py_attached(py)
                .unwrap_or_else(|e| e.into_inner())
        }
    }
}
