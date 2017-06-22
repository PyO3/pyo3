/// Utilities for a Python callable object that invokes a Rust function.

use std::os::raw::c_int;
use std::{any, mem, ptr, isize, io, panic};
use libc;

use pythonrun;
use python::{Python, IntoPyPointer};
use objects::exc;
use conversion::IntoPyObject;
use ffi::{self, Py_hash_t};
use err::{PyErr, PyResult};
use instance::{Py, AsPyRef};
use typeob::PyTypeInfo;


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
            PyErr::new_lazy_init(
                py.get_type::<exc::OverflowError>(), None).restore(py);
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

pub struct IterNextResultConverter;

impl <T> CallbackConverter<Option<T>> for IterNextResultConverter
    where T: IntoPyObject
{
    type R = *mut ffi::PyObject;

    fn convert(val: Option<T>, py: Python) -> *mut ffi::PyObject {
        match val {
            Some(val) => val.into_object(py).into_ptr(),
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


pub unsafe fn handle<'p, F, T, C>(location: &str, _c: C, f: F) -> C::R
    where F: FnOnce(Python<'p>) -> PyResult<T>,
          F: panic::UnwindSafe,
          C: CallbackConverter<T>
{
    let guard = AbortOnDrop(location);
    let pool = pythonrun::Pool::new();
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
    drop(pool);
    mem::forget(guard);
    ret
}

#[allow(unused_mut)]
pub unsafe fn cb_unary<Slf, F, T, C>(location: &str,
                                     slf: *mut ffi::PyObject, _c: C, f: F) -> C::R
    where F: for<'p> FnOnce(Python<'p>, &'p mut Slf) -> PyResult<T>,
          F: panic::UnwindSafe,
          Slf: PyTypeInfo,
          C: CallbackConverter<T>
{
    let guard = AbortOnDrop(location);
    let pool = pythonrun::Pool::new();
    let ret = panic::catch_unwind(|| {
        let py = Python::assume_gil_acquired();
        let slf = Py::<Slf>::from_borrowed_ptr(slf);

        let result = match f(py, slf.as_mut(py)) {
            Ok(val) => {
                C::convert(val, py)
            }
            Err(e) => {
                e.restore(py);
                C::error_value()
            }
        };
        py.release(slf);
        result
    });
    let ret = match ret {
        Ok(r) => r,
        Err(ref err) => {
            handle_panic(Python::assume_gil_acquired(), err);
            C::error_value()
        }
    };
    drop(pool);
    mem::forget(guard);
    ret
}

#[allow(unused_mut)]
pub unsafe fn cb_unary_unit<Slf, F>(location: &str, slf: *mut ffi::PyObject, f: F) -> c_int
    where F: for<'p> FnOnce(Python<'p>, &'p mut Slf) -> c_int,
          F: panic::UnwindSafe,
          Slf: PyTypeInfo,
{
    let guard = AbortOnDrop(location);
    let pool = pythonrun::Pool::new();
    let ret = panic::catch_unwind(|| {
        let py = Python::assume_gil_acquired();
        let slf = Py::<Slf>::from_borrowed_ptr(slf);

        let result = f(py, slf.as_mut(py));
        py.release(slf);
        result
    });
    let ret = match ret {
        Ok(r) => r,
        Err(ref err) => {
            handle_panic(Python::assume_gil_acquired(), err);
            -1
        }
    };
    drop(pool);
    mem::forget(guard);
    ret
}

pub unsafe fn cb_meth<F>(location: &str, f: F) -> *mut ffi::PyObject
    where F: for<'p> FnOnce(Python<'p>) -> *mut ffi::PyObject,
          F: panic::UnwindSafe
{
    let guard = AbortOnDrop(location);
    let pool = pythonrun::Pool::new();
    let ret = panic::catch_unwind(|| {
        let py = Python::assume_gil_acquired();
        f(py)
    });
    let ret = match ret {
        Ok(r) => r,
        Err(ref err) => {
            handle_panic(Python::assume_gil_acquired(), err);
            ptr::null_mut()
        }
    };
    drop(pool);
    mem::forget(guard);
    ret
}

pub unsafe fn cb_pyfunc<F, C, T>(location: &str, _c: C, f: F) -> C::R
    where F: for<'p> FnOnce(Python<'p>) -> C::R,
          F: panic::UnwindSafe,
          C: CallbackConverter<T>
{
    let guard = AbortOnDrop(location);
    let pool = pythonrun::Pool::new();
    let ret = panic::catch_unwind(|| {
        let py = Python::assume_gil_acquired();
        f(py)
    });
    let ret = match ret {
        Ok(r) => r,
        Err(ref err) => {
            handle_panic(Python::assume_gil_acquired(), err);
            C::error_value()
        }
    };
    drop(pool);
    mem::forget(guard);
    ret
}

pub unsafe fn cb_setter<F>(location: &str, f: F) -> c_int
    where F: for<'p> FnOnce(Python<'p>) -> c_int,
          F: panic::UnwindSafe
{
    let guard = AbortOnDrop(location);
    let pool = pythonrun::Pool::new();
    let ret = panic::catch_unwind(|| {
        let py = Python::assume_gil_acquired();
        f(py)
    });
    let ret = match ret {
        Ok(r) => r,
        Err(ref err) => {
            handle_panic(Python::assume_gil_acquired(), err);
            -1
        }
    };
    drop(pool);
    mem::forget(guard);
    ret
}

#[inline]
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


pub fn handle_panic(_py: Python, _panic: &any::Any) {
    unsafe {
        ffi::PyErr_SetString(ffi::PyExc_SystemError, "Rust panic\0".as_ptr() as *const i8);
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
