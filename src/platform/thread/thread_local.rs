use core::cell::{Cell, UnsafeCell};
use core::ffi::c_void;
use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicPtr, AtomicU8, Ordering};

#[cfg(not(Py_LIMITED_API))]
use pyo3_ffi::Py_tss_NEEDS_INIT;

use crate::ffi::{
    PyThread_tss_alloc, PyThread_tss_create, PyThread_tss_delete, PyThread_tss_free,
    PyThread_tss_get, PyThread_tss_set,
};
use crate::platform::prelude::*;

pub struct LocalKey<T: 'static> {
    #[cfg(Py_LIMITED_API)]
    inner: AtomicPtr<crate::ffi::Py_tss_t>,

    #[cfg(not(Py_LIMITED_API))]
    inner: UnsafeCell<crate::ffi::Py_tss_t>,

    state: AtomicU8,
    init: fn() -> T,
}

// SAFETY: the unsafecell is only accessed by python tss functions which are thread safe
#[cfg(not(Py_LIMITED_API))]
unsafe impl<T: 'static> Sync for LocalKey<T> {}

const LOCAL_KEY_UNINIT: u8 = 0;
const LOCAL_KEY_INITIALIZING: u8 = 1;
const LOCAL_KEY_CREATED: u8 = 2;
const LOCAL_KEY_DESTROYED: u8 = 3;

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
            inner: cfg_select! {
                Py_LIMITED_API => AtomicPtr::new(core::ptr::null_mut()),
                _ => UnsafeCell::new(Py_tss_NEEDS_INIT),
            },
            state: AtomicU8::new(LOCAL_KEY_UNINIT),
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
        loop {
            match self.state.load(Ordering::SeqCst) {
                LOCAL_KEY_UNINIT => {
                    if self
                        .state
                        .compare_exchange(
                            LOCAL_KEY_UNINIT,
                            LOCAL_KEY_INITIALIZING,
                            Ordering::SeqCst,
                            Ordering::SeqCst,
                        )
                        .is_ok()
                    {
                        self.initialize();
                    }
                }
                LOCAL_KEY_INITIALIZING => {
                    while self.state.load(Ordering::SeqCst) == LOCAL_KEY_INITIALIZING {
                        core::hint::spin_loop();
                    }
                }
                LOCAL_KEY_CREATED => break,
                LOCAL_KEY_DESTROYED => return Err(AccessError),
                _ => unreachable!(),
            }
        }
        // SAFETY: the match statement above ensures we only reach this point if tss is created
        let val = unsafe { self.get_val() };
        Ok(f(val))
    }

    #[cfg(Py_LIMITED_API)]
    #[track_caller]
    fn initialize(&'static self) {
        // SAFETY: no requirements
        let inner = unsafe { PyThread_tss_alloc() };
        let inner = NonNull::new(inner).unwrap();
        // SAFETY: ptr obtained by calling PyThread_tss_alloc
        let result = unsafe { PyThread_tss_create(inner.as_ptr()) };
        assert_eq!(result, 0, "failed to created thread specific storage");

        self.inner.store(inner.as_ptr(), Ordering::SeqCst);
        self.state.store(LOCAL_KEY_CREATED, Ordering::SeqCst);
    }

    #[cfg(not(Py_LIMITED_API))]
    #[track_caller]
    fn initialize(&'static self) {
        // SAFETY: inner is initialized with Py_tss_NEEDS_INIT
        let result = unsafe { PyThread_tss_create(self.inner.get()) };
        assert_eq!(result, 0, "failed to created thread specific storage");
        self.state.store(LOCAL_KEY_CREATED, Ordering::SeqCst);
    }

    /// # Safety
    /// Can only be called if tss is created
    #[track_caller]
    unsafe fn get_val<'a>(&'static self) -> &'a T {
        let inner = cfg_select! {
            Py_LIMITED_API => self.inner.load(Ordering::SeqCst),
            _ => self.inner.get(),
        };
        // SAFETY: inner is a valid tss key (upheld by caller)
        let val: *mut T = unsafe { PyThread_tss_get(inner) }.cast();
        match NonNull::new(val) {
            // SAFETY: no mut ref is ever created from this pointer
            Some(val) => unsafe { val.as_ref() },
            None => {
                let val = Box::new((self.init)());
                let val = Box::into_raw(val);
                // SAFETY: inner is a valid tss key
                let result = unsafe { PyThread_tss_set(inner, val.cast()) };
                assert_eq!(result, 0, "failed to set thread specific value");
                // SAFETY: val was just allocated above
                unsafe { NonNull::new_unchecked(val).as_ref() }
            }
        }
    }
}

impl<T: 'static> Drop for LocalKey<T> {
    fn drop(&mut self) {
        self.state.store(LOCAL_KEY_DESTROYED, Ordering::SeqCst);
        cfg_select! {
            Py_LIMITED_API => {
                // SAFETY: inner is returned by PyThread_tss_alloc and is not used after this call
                unsafe { PyThread_tss_free(self.inner.load(Ordering::SeqCst)) };
            },
            _ => {
                // SAFETY: inner is not used again after this call
                unsafe { PyThread_tss_delete(self.inner.get()) };
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
