use core::cell::{Cell, UnsafeCell};
use core::ffi::c_void;
use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicU8, Ordering};

use once_cell::sync::{Lazy, OnceCell};
#[cfg(not(Py_LIMITED_API))]
use pyo3_ffi::Py_tss_NEEDS_INIT;

use crate::ffi::{
    PyThread_tss_alloc, PyThread_tss_create, PyThread_tss_delete, PyThread_tss_free,
    PyThread_tss_get, PyThread_tss_set,
};
use crate::platform::prelude::*;

pub struct LocalKey<T: 'static> {
    #[cfg(Py_LIMITED_API)]
    inner: OnceCell<NonNull<crate::ffi::Py_tss_t>>,

    #[cfg(not(Py_LIMITED_API))]
    inner: OnceCell<UnsafeCell<crate::ffi::Py_tss_t>>,

    destroyed: AtomicBool,
    init: fn() -> T,
}

// SAFETY: the unsafecell is only accessed by python tss functions which are thread safe
unsafe impl<T: 'static> Sync for LocalKey<T> {}

impl<T: 'static> Debug for LocalKey<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LocalKey").finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct AccessError;

impl Display for AccessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt("already destroyed", f)
    }
}

impl core::error::Error for AccessError {}

// This ensures the panicking code is outlined from `with` for `LocalKey`.
#[cfg_attr(not(panic = "immediate-abort"), inline(never))]
#[track_caller]
#[cold]
fn panic_access_error(err: AccessError) -> ! {
    panic!("cannot access a Thread Local Storage value during or after destruction: {err:?}")
}

impl<T: 'static> LocalKey<T> {
    pub const unsafe fn new(init: fn() -> T) -> LocalKey<T> {
        LocalKey {
            inner: OnceCell::new(),
            destroyed: AtomicBool::new(false),
            init,
        }
    }

    #[track_caller]
    #[inline]
    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        match self.try_with(f) {
            Ok(r) => r,
            Err(err) => panic_access_error(err),
        }
    }

    #[inline]
    #[track_caller]
    pub fn try_with<F, R>(&'static self, f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&T) -> R,
    {
        if self.destroyed.load(Ordering::SeqCst) {
            return Err(AccessError);
        }
        let val = self.get_val();
        Ok(f(val))
    }

    #[track_caller]
    fn get_val<'a>(&'static self) -> &'a T {
        let inner = self.get_raw();
        // SAFETY: inner is a valid tss key
        let val: *mut T = unsafe { PyThread_tss_get(inner.as_ptr()) }.cast();
        match NonNull::new(val) {
            // SAFETY: no mut ref is ever created from this pointer
            Some(val) => unsafe { val.as_ref() },
            None => {
                let val = Box::new((self.init)());
                let val = Box::into_raw(val);
                // SAFETY: inner is a valid tss key
                let result = unsafe { PyThread_tss_set(inner.as_ptr(), val.cast()) };
                assert_eq!(result, 0, "failed to set thread specific value");
                // SAFETY: val was just allocated above
                unsafe { NonNull::new_unchecked(val).as_ref() }
            }
        }
    }

    fn get_raw(&self) -> NonNull<crate::ffi::Py_tss_t> {
        cfg_select! {
            Py_LIMITED_API => self.inner.get_or_init(initialize_tss),
            _ => NonNull::new(self.inner.get_or_init(initialize_tss).get()).unwrap(),
        }
    }
}

#[cfg(Py_LIMITED_API)]
fn initialize_tss() -> NonNull<crate::ffi::Py_tss_t> {
    // SAFETY: no requirements
    let tss = unsafe { PyThread_tss_alloc() };
    let tss = NonNull::new(tss).unwrap();
    // SAFETY: ptr obtained by calling PyThread_tss_alloc
    let result = unsafe { PyThread_tss_create(tss.as_ptr()) };
    assert_eq!(result, 0, "failed to created thread specific storage");
    tss
}

#[cfg(not(Py_LIMITED_API))]
fn initialize_tss() -> UnsafeCell<crate::ffi::Py_tss_t> {
    let mut tss = Py_tss_NEEDS_INIT;
    // SAFETY: tss is initialized with Py_tss_NEEDS_INIT
    let result = unsafe { PyThread_tss_create(&raw mut tss) };
    assert_eq!(result, 0, "failed to created thread specific storage");
    UnsafeCell::new(tss)
}

impl<T: 'static> Drop for LocalKey<T> {
    fn drop(&mut self) {
        self.destroyed.store(true, Ordering::SeqCst);
        let inner = self.get_raw();
        cfg_select! {
            Py_LIMITED_API => {
                // SAFETY: inner is returned by PyThread_tss_alloc and is not used after this call
                unsafe { PyThread_tss_free(inner.as_ptr()) };
            },
            _ => {
                // SAFETY: inner is not used again after this call
                unsafe { PyThread_tss_delete(inner.as_ptr()) };
            },
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! thread_local {
    ($($(#[$attr:meta])* $vis:vis static $name:ident : $ty:ty = $(const)? $init:expr;)+) => {
        $(
            $(#[$attr])*
            #[allow(unused_braces)]
            // SAFETY: correctly initializes a LocalKey
            $vis static $name: $crate::platform::thread::LocalKey<$ty> = unsafe {
                $crate::platform::thread::LocalKey::new({
                    fn init() -> $ty {
                        $init
                    }
                    init
                })
            };
        )+
    };
}

#[cfg(test)]
mod tests {
    use super::LocalKey;

    use core::cell::Cell;

    #[test]
    fn create_and_set_thread_local() {
        crate::thread_local! {
            static NUM: Cell<u32> = Cell::new(42);
        };

        assert_eq!(NUM.with(|val| val.get()), 42);

        NUM.with(|val| val.set(18));
        assert_eq!(NUM.with(|val| val.get()), 18);
    }
}
