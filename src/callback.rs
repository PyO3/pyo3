/// Utilities for a Python callable object that invokes a Rust function.

use std::os::raw::c_int;
use std::{any, mem, ptr, isize, io, marker, panic};

use libc;
use python::{Python, PythonObject};
use objects::exc;
use conversion::ToPyObject;
use ffi::{self, Py_hash_t};
use err::{PyErr, PyResult};


pub trait CallbackConverter<S> {
    type R;

    fn convert(S, Python) -> Self::R;
    fn error_value() -> Self::R;
}

pub struct PyObjectCallbackConverter;

impl <S> CallbackConverter<S> for PyObjectCallbackConverter
    where S: ToPyObject
{
    type R = *mut ffi::PyObject;

    fn convert(val: S, py: Python) -> *mut ffi::PyObject {
        val.into_py_object(py).into_object().steal_ptr()
    }

    #[inline]
    fn error_value() -> *mut ffi::PyObject {
        ptr::null_mut()
    }
}

pub struct PythonObjectCallbackConverter<T>(pub marker::PhantomData<T>);

impl <T, S> CallbackConverter<S> for PythonObjectCallbackConverter<T>
    where T: PythonObject,
          S: ToPyObject
{
    type R = *mut ffi::PyObject;

    fn convert(val: S, py: Python) -> *mut ffi::PyObject {
        val.into_py_object(py).into_object().steal_ptr()
    }

    #[inline]
    fn error_value() -> *mut ffi::PyObject {
        ptr::null_mut()
    }
}

pub struct BoolConverter;

impl CallbackConverter<bool> for BoolConverter {
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
            PyErr::new_lazy_init(py.get_type::<exc::OverflowError>(), None).restore(py);
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

pub struct VoidCallbackConverter;

impl CallbackConverter<()> for VoidCallbackConverter {
    type R = ();

    #[inline]
    fn convert(_: (), _: Python) -> () {
        ()
    }

    #[inline]
    fn error_value() -> () {
        ()
    }
}

pub struct IterNextResultConverter;

impl <T> CallbackConverter<Option<T>>
    for IterNextResultConverter
    where T: ToPyObject
{
    type R = *mut ffi::PyObject;

    fn convert(val: Option<T>, py: Python) -> *mut ffi::PyObject {
        match val {
            Some(val) => val.into_py_object(py).into_object().steal_ptr(),
            None => unsafe {
                ffi::PyErr_SetNone(ffi::PyExc_StopIteration);
                ptr::null_mut()
            }
        }
    }

    #[inline]
    fn error_value() -> *mut ffi::PyObject {
        ptr::null_mut()
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


pub unsafe fn handle_callback<F, T, C>(location: &str, _c: C, f: F) -> C::R
    where F: FnOnce(Python) -> PyResult<T>,
          F: panic::UnwindSafe,
          C: CallbackConverter<T>
{
    let guard = AbortOnDrop(location);
    let ret = panic::catch_unwind(|| {
        let py = Python::assume_gil_acquired();
        match f(py) {
            Ok(val) => {
                C::convert(val, py)
            }
            Err(e) => {
                e.restore(py);
                C::error_value()
            }
        }
    });
    let ret = match ret {
        Ok(r) => r,
        Err(ref err) => {
            handle_panic(Python::assume_gil_acquired(), err);
            C::error_value()
        }
    };
    mem::forget(guard);
    ret
}

fn handle_panic(_py: Python, _panic: &any::Any) {
    let msg = cstr!("Rust panic");
    unsafe {
        ffi::PyErr_SetString(ffi::PyExc_SystemError, msg.as_ptr());
    }
}

pub struct AbortOnDrop<'a>(pub &'a str);

impl <'a> Drop for AbortOnDrop<'a> {
    fn drop(&mut self) {
        use std::io::Write;
        let _ = writeln!(&mut io::stderr(), "Cannot unwind out of {}", self.0);
        unsafe { libc::abort() }
    }
}
