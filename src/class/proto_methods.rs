use crate::class::{basic::PyObjectMethods, descr::PyDescrMethods, iter::PyIterMethods};
use crate::ffi::PyBufferProcs;
use std::{
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

/// For rust-numpy, we need a stub implementation.
pub trait PyProtoMethods {
    fn basic_methods() -> Option<NonNull<PyObjectMethods>>;
    fn buffer_methods() -> Option<NonNull<PyBufferProcs>>;
    fn descr_methods() -> Option<NonNull<PyDescrMethods>>;
    fn iter_methods() -> Option<NonNull<PyIterMethods>>;
}

#[doc(hidden)]
pub trait HasPyProtoRegistry: Sized + 'static {
    fn registory() -> &'static PyProtoRegistry;
}

impl<T: HasPyProtoRegistry> PyProtoMethods for T {
    fn basic_methods() -> Option<NonNull<PyObjectMethods>> {
        NonNull::new(Self::registory().basic_methods.load(Ordering::SeqCst))
    }
    fn buffer_methods() -> Option<NonNull<PyBufferProcs>> {
        NonNull::new(Self::registory().buffer_methods.load(Ordering::SeqCst))
    }
    fn descr_methods() -> Option<NonNull<PyDescrMethods>> {
        NonNull::new(Self::registory().descr_methods.load(Ordering::SeqCst))
    }
    fn iter_methods() -> Option<NonNull<PyIterMethods>> {
        NonNull::new(Self::registory().iter_methods.load(Ordering::SeqCst))
    }
}

#[doc(hidden)]
pub struct PyProtoRegistry {
    // Basic Protocols
    basic_methods: AtomicPtr<PyObjectMethods>,
    // Buffer Protocols
    buffer_methods: AtomicPtr<PyBufferProcs>,
    // Descr Protocols
    descr_methods: AtomicPtr<PyDescrMethods>,
    // Iterator Protocols
    iter_methods: AtomicPtr<PyIterMethods>,
}

impl PyProtoRegistry {
    pub const fn new() -> Self {
        PyProtoRegistry {
            basic_methods: AtomicPtr::new(ptr::null_mut()),
            buffer_methods: AtomicPtr::new(ptr::null_mut()),
            descr_methods: AtomicPtr::new(ptr::null_mut()),
            iter_methods: AtomicPtr::new(ptr::null_mut()),
        }
    }
    pub fn set_basic_methods(&self, methods: PyObjectMethods) {
        self.basic_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::SeqCst)
    }
    pub fn set_buffer_methods(&self, methods: PyBufferProcs) {
        self.buffer_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::SeqCst)
    }
    pub fn set_descr_methods(&self, methods: PyDescrMethods) {
        self.descr_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::SeqCst)
    }
    pub fn set_iter_methods(&self, methods: PyIterMethods) {
        self.iter_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::SeqCst)
    }
}
