// TODO compile_error if parking_lot and std are both disabled

#[cfg(not(cfg_select))]
use crate::internal::macros::cfg_select;
use crate::sealed;
use crate::sync::OnceExt;

cfg_select! {
    wip_feature_std => {
        #[allow(clippy::disallowed_types)]
        type OnceInner = std::sync::Once;
    },
    feature = "parking_lot" => {
        type OnceInner = parking_lot::Once;
    },
    _ => {
        compile_error!("Please enable at least one of the following features: std, parking_lot");
    },
}

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

    #[inline(always)]
    pub fn is_completed(&self) -> bool {
        cfg_select! {
            wip_feature_std => {
                self.0.is_completed()
            },
            feature = "parking_lot" => {
                matches!(self.0.state(), parking_lot::OnceState::Done)
            },
            _ => {
                compile_error!("Please enable at least one of the following features: std, parking_lot")
            }
        }
    }
}

impl sealed::Sealed for Once {}
impl OnceExt for Once {
    type OnceState = ();

    fn call_once_py_attached(&self, py: crate::prelude::Python<'_>, f: impl FnOnce()) {
        self.0.call_once_force_py_attached(py, move |_| f());
    }

    fn call_once_force_py_attached(
        &self,
        py: crate::prelude::Python<'_>,
        f: impl FnOnce(&Self::OnceState),
    ) {
        self.0.call_once_force_py_attached(py, |_| f(&()));
    }
}

pub mod non_poison {
    cfg_select! {
        wip_feature_std => {
            pub use std::sync::MutexGuard;
        },
        feature = "parking_lot" => {
            pub use parking_lot::{Mutex, MutexGuard};
        },
        _ => {
            compile_error!("Please enable at least one of the following features: std, parking_lot");
        },
    }

    #[cfg(wip_feature_std)]
    #[derive(Default, Debug)]
    pub struct Mutex<T: ?Sized> {
        #[allow(clippy::disallowed_types)]
        inner: std::sync::Mutex<T>,
    }

    #[cfg(wip_feature_std)]
    impl<T> Mutex<T> {
        #[inline(always)]
        pub const fn new(t: T) -> Mutex<T> {
            Mutex {
                #[allow(clippy::disallowed_types)]
                inner: std::sync::Mutex::new(t),
            }
        }

        pub fn into_inner(self) -> T {
            self.inner.into_inner().unwrap_or_else(|e| e.into_inner())
        }
    }

    #[cfg(wip_feature_std)]
    impl<T: ?Sized> Mutex<T> {
        #[inline(always)]
        pub fn lock(&self) -> MutexGuard<'_, T> {
            self.inner.lock().unwrap_or_else(|e| e.into_inner())
        }

        // TODO try_lock
    }

    #[cfg(wip_feature_std)]
    impl<T> From<T> for Mutex<T> {
        #[inline(always)]
        fn from(t: T) -> Self {
            Mutex {
                #[allow(clippy::disallowed_types)]
                inner: std::sync::Mutex::new(t),
            }
        }
    }

    #[cfg(wip_feature_std)]
    impl<T> crate::sealed::Sealed for Mutex<T> {}

    #[cfg(wip_feature_std)]
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
