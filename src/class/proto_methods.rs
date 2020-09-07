#[cfg(not(Py_LIMITED_API))]
use crate::class::buffer::PyBufferProcs;
use crate::class::{
    basic::PyObjectMethods, descr::PyDescrMethods, gc::PyGCMethods, iter::PyIterMethods,
    mapping::PyMappingMethods, number::PyNumberMethods, pyasync::PyAsyncMethods,
    sequence::PySequenceMethods,
};
use std::{
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

/// Defines all method tables we need for object protocols.
// Note(kngwyu): default implementations are for rust-numpy. Please don't remove them.
pub trait PyProtoMethods {
    fn async_methods() -> Option<NonNull<PyAsyncMethods>> {
        None
    }
    fn basic_methods() -> Option<NonNull<PyObjectMethods>> {
        None
    }
    #[cfg(not(Py_LIMITED_API))]
    fn buffer_methods() -> Option<NonNull<PyBufferProcs>> {
        None
    }
    fn descr_methods() -> Option<NonNull<PyDescrMethods>> {
        None
    }
    fn gc_methods() -> Option<NonNull<PyGCMethods>> {
        None
    }
    fn mapping_methods() -> Option<NonNull<PyMappingMethods>> {
        None
    }
    fn number_methods() -> Option<NonNull<PyNumberMethods>> {
        None
    }
    fn iter_methods() -> Option<NonNull<PyIterMethods>> {
        None
    }
    fn sequence_methods() -> Option<NonNull<PySequenceMethods>> {
        None
    }
}

/// Indicates that a type has a protocol registry. Implemented by `#[pyclass]`.
#[doc(hidden)]
pub trait HasProtoRegistry: Sized + 'static {
    fn registry() -> &'static PyProtoRegistry;
}

impl<T: HasProtoRegistry> PyProtoMethods for T {
    fn async_methods() -> Option<NonNull<PyAsyncMethods>> {
        NonNull::new(Self::registry().async_methods.load(Ordering::Relaxed))
    }
    fn basic_methods() -> Option<NonNull<PyObjectMethods>> {
        NonNull::new(Self::registry().basic_methods.load(Ordering::Relaxed))
    }
    #[cfg(not(Py_LIMITED_API))]
    fn buffer_methods() -> Option<NonNull<PyBufferProcs>> {
        NonNull::new(Self::registry().buffer_methods.load(Ordering::Relaxed))
    }
    fn descr_methods() -> Option<NonNull<PyDescrMethods>> {
        NonNull::new(Self::registry().descr_methods.load(Ordering::Relaxed))
    }
    fn gc_methods() -> Option<NonNull<PyGCMethods>> {
        NonNull::new(Self::registry().gc_methods.load(Ordering::Relaxed))
    }
    fn mapping_methods() -> Option<NonNull<PyMappingMethods>> {
        NonNull::new(Self::registry().mapping_methods.load(Ordering::Relaxed))
    }
    fn number_methods() -> Option<NonNull<PyNumberMethods>> {
        NonNull::new(Self::registry().number_methods.load(Ordering::Relaxed))
    }
    fn iter_methods() -> Option<NonNull<PyIterMethods>> {
        NonNull::new(Self::registry().iter_methods.load(Ordering::Relaxed))
    }
    fn sequence_methods() -> Option<NonNull<PySequenceMethods>> {
        NonNull::new(Self::registry().sequence_methods.load(Ordering::Relaxed))
    }
}

/// Stores all method protocols.
/// Used in the proc-macro code as a static variable.
#[doc(hidden)]
pub struct PyProtoRegistry {
    /// Async protocols.
    async_methods: AtomicPtr<PyAsyncMethods>,
    /// Basic protocols.
    basic_methods: AtomicPtr<PyObjectMethods>,
    /// Buffer protocols.
    #[cfg(not(Py_LIMITED_API))]
    buffer_methods: AtomicPtr<PyBufferProcs>,
    /// Descr pProtocols.
    descr_methods: AtomicPtr<PyDescrMethods>,
    /// GC protocols.
    gc_methods: AtomicPtr<PyGCMethods>,
    /// Mapping protocols.
    mapping_methods: AtomicPtr<PyMappingMethods>,
    /// Number protocols.
    number_methods: AtomicPtr<PyNumberMethods>,
    /// Iterator protocols.
    iter_methods: AtomicPtr<PyIterMethods>,
    /// Sequence protocols.
    sequence_methods: AtomicPtr<PySequenceMethods>,
}

impl PyProtoRegistry {
    pub const fn new() -> Self {
        PyProtoRegistry {
            async_methods: AtomicPtr::new(ptr::null_mut()),
            basic_methods: AtomicPtr::new(ptr::null_mut()),
            #[cfg(not(Py_LIMITED_API))]
            buffer_methods: AtomicPtr::new(ptr::null_mut()),
            descr_methods: AtomicPtr::new(ptr::null_mut()),
            gc_methods: AtomicPtr::new(ptr::null_mut()),
            mapping_methods: AtomicPtr::new(ptr::null_mut()),
            number_methods: AtomicPtr::new(ptr::null_mut()),
            iter_methods: AtomicPtr::new(ptr::null_mut()),
            sequence_methods: AtomicPtr::new(ptr::null_mut()),
        }
    }
    pub fn set_async_methods(&self, methods: PyAsyncMethods) {
        self.async_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::Relaxed)
    }
    pub fn set_basic_methods(&self, methods: PyObjectMethods) {
        self.basic_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::Relaxed)
    }
    #[cfg(not(Py_LIMITED_API))]
    pub fn set_buffer_methods(&self, methods: PyBufferProcs) {
        self.buffer_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::Relaxed)
    }
    pub fn set_descr_methods(&self, methods: PyDescrMethods) {
        self.descr_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::Relaxed)
    }
    pub fn set_gc_methods(&self, methods: PyGCMethods) {
        self.gc_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::Relaxed)
    }
    pub fn set_mapping_methods(&self, methods: PyMappingMethods) {
        self.mapping_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::Relaxed)
    }
    pub fn set_number_methods(&self, methods: PyNumberMethods) {
        self.number_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::Relaxed)
    }
    pub fn set_iter_methods(&self, methods: PyIterMethods) {
        self.iter_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::Relaxed)
    }
    pub fn set_sequence_methods(&self, methods: PySequenceMethods) {
        self.sequence_methods
            .store(Box::into_raw(Box::new(methods)), Ordering::Relaxed)
    }
}
