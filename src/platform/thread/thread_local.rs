use core::cell::{Cell, UnsafeCell};
use core::ffi::c_void;
use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
#[cfg(Py_LIMITED_API)]
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

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
    inner: AtomicPtr<crate::ffi::Py_tss_t>,

    #[cfg(not(Py_LIMITED_API))]
    inner: UnsafeCell<crate::ffi::Py_tss_t>,

    state: AtomicU8,
    init: fn() -> T,
}

// SAFETY: the unsafecell is only accessed by python tss functions which are thread safe
unsafe impl<T: 'static> Sync for LocalKey<T> {}

const LOCAL_KEY_UNINIT: u8 = 0;
const LOCAL_KEY_INITIALIZING: u8 = 1;
const LOCAL_KEY_CREATED: u8 = 2;

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

impl<T: 'static> LocalKey<T> {
    pub const unsafe fn new(init: fn() -> T) -> LocalKey<T> {
        LocalKey {
            inner: cfg_select! {
                Py_LIMITED_API => AtomicPtr::new(ptr::null_mut()),
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
                        // SAFETY: the spin loop ensures it's only called from one thread
                        unsafe { self.initialize() };
                    }
                }
                LOCAL_KEY_INITIALIZING => {
                    while self.state.load(Ordering::SeqCst) != LOCAL_KEY_CREATED {
                        core::hint::spin_loop();
                    }
                }
                LOCAL_KEY_CREATED => break,
                _ => unreachable!(),
            }
        }

        // SAFETY: the match above enusres we only reach this point if tss is created
        let val = unsafe { self.get_val() };
        f(val)
    }

    #[inline]
    #[track_caller]
    pub fn try_with<F, R>(&'static self, f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&T) -> R,
    {
        Ok(self.with(f))
    }

    /// # Safety
    /// Can only be called if tss is created
    #[track_caller]
    unsafe fn get_val<'a>(&'static self) -> &'a T {
        let inner = cfg_select! {
            Py_LIMITED_API => self.inner.load(Ordering::SeqCst),
            _ => self.inner.get(),
        };
        // SAFETY: inner is a valid tss key
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

    /// # Safety
    /// This function must not be called while it is running on another thread
    #[cfg(Py_LIMITED_API)]
    #[track_caller]
    unsafe fn initialize(&'static self) {
        // SAFETY: no requirements
        let inner = unsafe { PyThread_tss_alloc() };
        let inner = NonNull::new(inner).unwrap();
        // SAFETY: ptr obtained by calling PyThread_tss_alloc
        let result = unsafe { PyThread_tss_create(inner.as_ptr()) };
        assert_eq!(result, 0, "failed to created thread specific storage");

        self.inner.store(inner.as_ptr(), Ordering::SeqCst);
        self.state.store(LOCAL_KEY_CREATED, Ordering::SeqCst);
    }

    /// # Safety
    /// This function must not be called while it is running on another thread
    #[cfg(not(Py_LIMITED_API))]
    #[track_caller]
    unsafe fn initialize(&'static self) {
        // SAFETY: inner is initialized with Py_tss_NEEDS_INIT
        let result = unsafe { PyThread_tss_create(self.inner.get()) };
        assert_eq!(result, 0, "failed to created thread specific storage");
        self.state.store(LOCAL_KEY_CREATED, Ordering::SeqCst);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! thread_local {
    ($($(#[$attr:meta])* $vis:vis static $name:ident : $ty:ty = $init:expr;)+) => {
        $(
            const _: () = assert!(!core::mem::needs_drop::<$ty>(), "thread local values must be trivially droppable for `no_std` support");

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
