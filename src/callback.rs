// Copyright (c) 2017-present PyO3 Project and Contributors

//! Utilities for a Python callable object that invokes a Rust function.

use crate::err::PyResult;
use crate::exceptions::OverflowError;
use crate::ffi::{self, Py_hash_t};
use crate::IntoPyPointer;
use crate::{IntoPy, PyObject, Python};
use std::os::raw::c_int;
use std::{isize, ptr};

/// Convert the result of callback function into the appropriate return value.
///
/// Used by PyO3 macros.
pub trait CallbackConverter {
    type Source;
    type Result: Copy;
    const ERR_VALUE: Self::Result;

    fn convert(s: Self::Source, py: Python) -> Self::Result;

    #[inline]
    fn convert_result(py: Python, value: PyResult<Self::Source>) -> Self::Result {
        match value {
            Ok(val) => Self::convert(val, py),
            Err(e) => {
                e.restore(py);
                Self::ERR_VALUE
            }
        }
    }
}

pub struct PyObjectCallbackConverter<T>(pub std::marker::PhantomData<T>);

impl<T> CallbackConverter for PyObjectCallbackConverter<T>
where
    T: IntoPy<PyObject>,
{
    type Source = T;
    type Result = *mut ffi::PyObject;
    const ERR_VALUE: Self::Result = ptr::null_mut();

    fn convert(s: Self::Source, py: Python) -> Self::Result {
        s.into_py(py).into_ptr()
    }
}

pub struct BoolCallbackConverter;

impl CallbackConverter for BoolCallbackConverter {
    type Source = bool;
    type Result = c_int;
    const ERR_VALUE: Self::Result = -1;

    #[inline]
    fn convert(s: Self::Source, _py: Python) -> Self::Result {
        s as c_int
    }
}

pub struct LenResultConverter;

impl CallbackConverter for LenResultConverter {
    type Source = usize;
    type Result = isize;
    const ERR_VALUE: Self::Result = -1;

    fn convert(val: Self::Source, py: Python) -> Self::Result {
        if val <= (isize::MAX as usize) {
            val as isize
        } else {
            OverflowError::py_err(()).restore(py);
            -1
        }
    }
}

pub struct UnitCallbackConverter;

impl CallbackConverter for UnitCallbackConverter {
    type Source = ();
    type Result = c_int;
    const ERR_VALUE: Self::Result = -1;

    #[inline]
    fn convert(_s: Self::Source, _py: Python) -> Self::Result {
        0
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
    };
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

pub struct HashConverter<T>(pub std::marker::PhantomData<T>);

impl<T> CallbackConverter for HashConverter<T>
where
    T: WrappingCastTo<Py_hash_t>,
{
    type Source = T;
    type Result = Py_hash_t;
    const ERR_VALUE: Self::Result = -1;

    #[inline]
    fn convert(val: T, _py: Python) -> Py_hash_t {
        let hash = val.wrapping_cast();
        if hash == -1 {
            -2
        } else {
            hash
        }
    }
}

// Short hands methods for macros
#[inline]
pub fn cb_convert<C, T>(_c: C, py: Python, value: PyResult<T>) -> C::Result
where
    C: CallbackConverter<Source = T>,
{
    C::convert_result(py, value)
}

// Same as cb_convert(PyObjectCallbackConverter<T>, py, value)
#[inline]
pub fn cb_obj_convert<T: IntoPy<PyObject>>(
    py: Python,
    value: PyResult<T>,
) -> <PyObjectCallbackConverter<T> as CallbackConverter>::Result {
    PyObjectCallbackConverter::<T>::convert_result(py, value)
}

#[inline]
pub unsafe fn cb_err<C>(_c: C, py: Python, err: impl Into<crate::PyErr>) -> C::Result
where
    C: CallbackConverter,
{
    err.into().restore(py);
    C::ERR_VALUE
}
