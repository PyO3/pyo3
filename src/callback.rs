// Copyright (c) 2017-present PyO3 Project and Contributors

//! Utilities for a Python callable object that invokes a Rust function.

use std::os::raw::c_int;
use std::{ptr, isize};

use err::PyResult;
use ffi::{self, Py_hash_t};
use python::{Python, IntoPyPointer};
use objects::exc::OverflowError;
use conversion::IntoPyObject;


pub trait CallbackConverter<S> {
    type R;

    fn convert(S, Python) -> Self::R;
    fn error_value() -> Self::R;
}

pub struct PyObjectCallbackConverter;

impl<S> CallbackConverter<S> for PyObjectCallbackConverter
    where S: IntoPyObject
{
    type R = *mut ffi::PyObject;

    fn convert(val: S, py: Python) -> *mut ffi::PyObject {
        val.into_object(py).into_ptr()
    }

    #[inline]
    fn error_value() -> *mut ffi::PyObject {
        ptr::null_mut()
    }
}


pub struct BoolCallbackConverter;

impl CallbackConverter<bool> for BoolCallbackConverter {
    type R = c_int;

    #[inline]
    fn convert(val: bool, _py: Python) -> c_int {
        val as c_int
    }

    #[inline]
    fn error_value() -> c_int {
        -1
    }
}

pub struct LenResultConverter;

impl CallbackConverter<usize> for LenResultConverter {
    type R = isize;

    fn convert(val: usize, py: Python) -> isize {
        if val <= (isize::MAX as usize) {
            val as isize
        } else {
            OverflowError::new(()).restore(py);
            -1
        }
    }

    #[inline]
    fn error_value() -> isize {
        -1
    }
}


pub struct UnitCallbackConverter;

impl CallbackConverter<()> for UnitCallbackConverter {
    type R = c_int;

    #[inline]
    fn convert(_: (), _: Python) -> c_int {
        0
    }

    #[inline]
    fn error_value() -> c_int {
        -1
    }
}

pub trait WrappingCastTo<T> {
    fn wrapping_cast(self) -> T;
}

macro_rules! wrapping_cast {
    ($from:ty, $to:ty) => {
        impl WrappingCastTo<$to> for $from {
            #[inline]
            fn wrapping_cast(self) -> $to {
                self as $to
            }
        }
    }
}
wrapping_cast!(u8, Py_hash_t);
wrapping_cast!(u16, Py_hash_t);
wrapping_cast!(u32, Py_hash_t);
wrapping_cast!(usize, Py_hash_t);
wrapping_cast!(u64, Py_hash_t);
wrapping_cast!(i8, Py_hash_t);
wrapping_cast!(i16, Py_hash_t);
wrapping_cast!(i32, Py_hash_t);
wrapping_cast!(isize, Py_hash_t);
wrapping_cast!(i64, Py_hash_t);

pub struct HashConverter;

impl <T> CallbackConverter<T> for HashConverter
    where T: WrappingCastTo<Py_hash_t>
{
    type R = Py_hash_t;

    #[inline]
    fn convert(val: T, _py: Python) -> Py_hash_t {
        let hash = val.wrapping_cast();
        if hash == -1 {
            -2
        } else {
            hash
        }
    }

    #[inline]
    fn error_value() -> Py_hash_t {
        -1
    }
}

#[inline]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
pub unsafe fn cb_convert<C, T>(_c: C, py: Python, value: PyResult<T>) -> C::R
    where C: CallbackConverter<T>
{
    match value {
        Ok(val) => C::convert(val, py),
        Err(e) => {
            e.restore(py);
            C::error_value()
        }
    }
}
