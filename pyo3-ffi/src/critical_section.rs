#![cfg(PyRustPython)]

use crate::{PyMutex, PyObject};
use std::collections::HashMap;
use std::ptr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

struct ObjectCriticalLock {
    bits: AtomicU8,
}

impl ObjectCriticalLock {
    const fn new() -> Self {
        Self {
            bits: AtomicU8::new(0),
        }
    }
}

#[repr(C)]
pub struct PyCriticalSection {
    prev: usize,
    mutex: *const AtomicU8,
}

#[repr(C)]
pub struct PyCriticalSection2 {
    base: PyCriticalSection,
    mutex2: *const AtomicU8,
}

fn object_lock_registry() -> &'static Mutex<HashMap<usize, &'static ObjectCriticalLock>> {
    static REGISTRY: OnceLock<Mutex<HashMap<usize, &'static ObjectCriticalLock>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn object_lock(op: *mut PyObject) -> &'static ObjectCriticalLock {
    let key = op as usize;
    let mut registry = object_lock_registry().lock().unwrap();
    *registry
        .entry(key)
        .or_insert_with(|| Box::leak(Box::new(ObjectCriticalLock::new())))
}

#[inline]
unsafe fn lock_bits(bits: *const AtomicU8) {
    debug_assert!(!bits.is_null());
    let bits = unsafe { &*bits };
    while bits
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        std::thread::yield_now();
    }
}

#[inline]
unsafe fn unlock_bits(bits: *const AtomicU8) {
    debug_assert!(!bits.is_null());
    unsafe { &*bits }.store(0, Ordering::Release);
}

#[inline]
unsafe fn begin_one(c: *mut PyCriticalSection, bits: *const AtomicU8) {
    unsafe {
        (*c).prev = 0;
        (*c).mutex = bits;
        lock_bits(bits);
    }
}

#[inline]
unsafe fn begin_two(c: *mut PyCriticalSection2, bits1: *const AtomicU8, bits2: *const AtomicU8) {
    unsafe {
        (*c).base.prev = 0;
        if std::ptr::eq(bits1, bits2) {
            (*c).base.mutex = bits1;
            (*c).mutex2 = ptr::null();
            lock_bits(bits1);
            return;
        }
        let (first, second) = if (bits1 as usize) <= (bits2 as usize) {
            (bits1, bits2)
        } else {
            (bits2, bits1)
        };
        (*c).base.mutex = first;
        (*c).mutex2 = second;
        lock_bits(first);
        lock_bits(second);
    }
}

#[allow(non_snake_case)]
pub unsafe fn PyCriticalSection_Begin(c: *mut PyCriticalSection, op: *mut PyObject) {
    unsafe { begin_one(c, ptr::addr_of!(object_lock(op).bits)) }
}

#[allow(non_snake_case)]
pub unsafe fn PyCriticalSection_BeginMutex(c: *mut PyCriticalSection, m: *mut PyMutex) {
    unsafe { begin_one(c, ptr::addr_of!((*m).bits)) }
}

#[allow(non_snake_case)]
pub unsafe fn PyCriticalSection_End(c: *mut PyCriticalSection) {
    unsafe {
        if !(*c).mutex.is_null() {
            unlock_bits((*c).mutex);
            (*c).mutex = ptr::null();
        }
    }
}

#[allow(non_snake_case)]
pub unsafe fn PyCriticalSection2_Begin(
    c: *mut PyCriticalSection2,
    a: *mut PyObject,
    b: *mut PyObject,
) {
    unsafe {
        begin_two(
            c,
            ptr::addr_of!(object_lock(a).bits),
            ptr::addr_of!(object_lock(b).bits),
        )
    }
}

#[allow(non_snake_case)]
pub unsafe fn PyCriticalSection2_BeginMutex(
    c: *mut PyCriticalSection2,
    m1: *mut PyMutex,
    m2: *mut PyMutex,
) {
    unsafe { begin_two(c, ptr::addr_of!((*m1).bits), ptr::addr_of!((*m2).bits)) }
}

#[allow(non_snake_case)]
pub unsafe fn PyCriticalSection2_End(c: *mut PyCriticalSection2) {
    unsafe {
        if !(*c).mutex2.is_null() {
            unlock_bits((*c).mutex2);
            (*c).mutex2 = ptr::null();
        }
        if !(*c).base.mutex.is_null() {
            unlock_bits((*c).base.mutex);
            (*c).base.mutex = ptr::null();
        }
    }
}
