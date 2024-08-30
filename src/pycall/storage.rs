use std::alloc::{handle_alloc_error, Layout};
use std::mem::MaybeUninit;

use crate::ffi;

use super::PPPyObject;

pub trait RawStorage: Sized {
    type InitParam<'a>
    where
        Self: 'a;
    fn new(len: usize) -> Self;
    fn as_init_param(&mut self) -> Self::InitParam<'_>;
    fn as_ptr(&mut self) -> PPPyObject;
    fn len(&self) -> usize;
    fn init_param_from_ptr<'a>(ptr: PPPyObject) -> Self::InitParam<'a>;
}

impl<T: 'static> RawStorage for MaybeUninit<T> {
    type InitParam<'a> = PPPyObject;
    #[inline(always)]
    fn new(_len: usize) -> Self {
        MaybeUninit::uninit()
    }
    #[inline(always)]
    fn as_init_param(&mut self) -> PPPyObject {
        self.as_mut_ptr().cast::<*mut ffi::PyObject>()
    }
    #[inline(always)]
    fn as_ptr(&mut self) -> PPPyObject {
        (self as *mut Self).cast::<*mut ffi::PyObject>()
    }
    #[inline(always)]
    fn len(&self) -> usize {
        size_of::<Self>() / size_of::<*mut ffi::PyObject>()
    }
    #[inline(always)]
    fn init_param_from_ptr<'a>(ptr: PPPyObject) -> Self::InitParam<'a> {
        ptr
    }
}

pub struct DynKnownSizeRawStorage {
    ptr: PPPyObject,
    len: usize,
}

impl Drop for DynKnownSizeRawStorage {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                self.ptr.cast::<u8>(),
                Layout::array::<*mut ffi::PyObject>(self.len).unwrap_unchecked(),
            );
        }
    }
}

impl RawStorage for DynKnownSizeRawStorage {
    type InitParam<'a> = PPPyObject;
    #[inline]
    fn new(len: usize) -> Self {
        unsafe {
            let layout =
                Layout::array::<*mut ffi::PyObject>(len).expect("too much memory requested");
            let ptr = std::alloc::alloc(layout).cast::<*mut ffi::PyObject>();
            if ptr.is_null() {
                handle_alloc_error(layout);
            }
            Self { ptr, len }
        }
    }
    #[inline(always)]
    fn as_init_param(&mut self) -> PPPyObject {
        self.ptr
    }
    #[inline(always)]
    fn as_ptr(&mut self) -> PPPyObject {
        self.ptr
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    fn init_param_from_ptr<'a>(ptr: PPPyObject) -> Self::InitParam<'a> {
        ptr
    }
}

pub(super) type UnsizedStorage = Vec<*mut ffi::PyObject>;
pub(super) type UnsizedInitParam<'a> = &'a mut Vec<*mut ffi::PyObject>;

impl RawStorage for UnsizedStorage {
    type InitParam<'a> = UnsizedInitParam<'a>;
    #[inline]
    fn new(len: usize) -> Self {
        Vec::with_capacity(len)
    }
    #[inline(always)]
    fn as_init_param(&mut self) -> UnsizedInitParam<'_> {
        self
    }
    #[inline(always)]
    fn as_ptr(&mut self) -> PPPyObject {
        self.as_mut_ptr()
    }
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn init_param_from_ptr<'a>(_ptr: PPPyObject) -> Self::InitParam<'a> {
        unreachable!("UnsizedStorage does not use small stack optimization")
    }
}
