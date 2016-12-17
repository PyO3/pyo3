// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use libc;
use std::{mem, ptr, io, any, marker};
use std::panic;
use std::ffi::{CString, CStr};
use python::{Python, PythonObject, PyDrop};
use objects::{PyObject, PyTuple, PyDict, PyString, exc};
use conversion::ToPyObject;
use ffi;
use err::{self, PyResult};

#[macro_export]
#[doc(hidden)]
macro_rules! py_method_def {
    ($name: expr, $flags: expr, $wrap: expr) => {{
        static mut METHOD_DEF: $crate::_detail::ffi::PyMethodDef = $crate::_detail::ffi::PyMethodDef {
            //ml_name: bytes!(stringify!($name), "\0"),
            ml_name: 0 as *const $crate::_detail::libc::c_char,
            ml_meth: None,
            ml_flags: $crate::_detail::ffi::METH_VARARGS | $crate::_detail::ffi::METH_KEYWORDS | $flags,
            ml_doc: 0 as *const $crate::_detail::libc::c_char
        };
        METHOD_DEF.ml_name = concat!($name, "\0").as_ptr() as *const _;
        METHOD_DEF.ml_meth = Some(
            ::std::mem::transmute::<$crate::_detail::ffi::PyCFunctionWithKeywords,
                                  $crate::_detail::ffi::PyCFunction>($wrap)
        );
        &mut METHOD_DEF
    }}
}

/// Creates a Python callable object that invokes a Rust function.
///
/// There are two forms of this macro:
///
/// 1. `py_fn!(py, f(parameter_list))`
/// 1. `py_fn!(py, f(parameter_list) -> PyResult<T> { body })`
///
/// both forms return a value of type `PyObject`.
/// This python object is a callable object that invokes
/// the Rust function when called.
///
/// When called, the arguments are converted into
/// the Rust types specified in the parameter list.
/// See `py_argparse!()` for details on argument parsing.
///
/// Form 1:
///
/// * `py` must be an expression of type `Python`
/// * `f` must be the name of a function that is compatible with the specified
///    parameter list, except that a single parameter of type `Python` is prepended.
///    The function must return `PyResult<T>` for some `T` that implements `ToPyObject`.
///
/// Form 2:
///
/// * `py` must be an identifier refers to a `Python` value.
///   The function body will also have access to a `Python` variable of this name.
/// * `f` must be an identifier.
/// * The function return type must be `PyResult<T>` for some `T` that
///   implements `ToPyObject`.
///
/// # Example
/// ```
/// #[macro_use] extern crate cpython;
/// use cpython::{Python, PyResult, PyErr, PyDict};
/// use cpython::{exc};
///
/// fn multiply(py: Python, lhs: i32, rhs: i32) -> PyResult<i32> {
///     match lhs.checked_mul(rhs) {
///         Some(val) => Ok(val),
///         None => Err(PyErr::new_lazy_init(py.get_type::<exc::OverflowError>(), None))
///     }
/// }
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     let dict = PyDict::new(py);
///     dict.set_item(py, "multiply", py_fn!(py, multiply(lhs: i32, rhs: i32))).unwrap();
///     py.run("print(multiply(6, 7))", None, Some(&dict)).unwrap();
/// }
/// ```
#[macro_export]
macro_rules! py_fn {
    ($py:expr, $f:ident $plist:tt ) => {
        py_argparse_parse_plist! { py_fn_impl { $py, $f } $plist }
    };
    ($py:ident, $f:ident $plist:tt -> $ret:ty { $($body:tt)* } ) => {
        py_argparse_parse_plist! { py_fn_impl { $py, $f, $ret, { $($body)* } } $plist }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_fn_impl {
    // Form 1: reference existing function
    { $py:expr, $f:ident [ $( { $pname:ident : $ptype:ty = $detail:tt } )* ] } => {{
        unsafe extern "C" fn wrap(
            _slf: *mut $crate::_detail::ffi::PyObject,
            args: *mut $crate::_detail::ffi::PyObject,
            kwargs: *mut $crate::_detail::ffi::PyObject)
        -> *mut $crate::_detail::ffi::PyObject
        {
            $crate::_detail::handle_callback(
                stringify!($f), $crate::_detail::PyObjectCallbackConverter,
                |py| {
                    py_argparse_raw!(py, Some(stringify!($f)), args, kwargs,
                        [ $( { $pname : $ptype = $detail } )* ]
                        {
                            $f(py $(, $pname )* )
                        })
                })
        }
        unsafe {
            $crate::_detail::py_fn_impl($py,
                py_method_def!(stringify!($f), 0, wrap))
        }
    }};
    // Form 2: inline function definition
    { $py:ident, $f:ident, $ret:ty, $body:block [ $( { $pname:ident : $ptype:ty = $detail:tt } )* ] } => {{
        fn $f($py: $crate::Python $( , $pname : $ptype )* ) -> $ret $body
        py_fn_impl!($py, $f [ $( { $pname : $ptype = $detail } )* ])
    }}
}

pub unsafe fn py_fn_impl(py: Python, method_def: *mut ffi::PyMethodDef) -> PyObject {
    err::from_owned_ptr_or_panic(py, ffi::PyCFunction_New(method_def, ptr::null_mut()))
}

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
          S: ToPyObject<ObjectType=T>
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

// Tests for this file are in tests/test_function.rs

