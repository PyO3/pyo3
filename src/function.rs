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

use std::{mem, ptr};
use python::{Python, PythonObject};
use objects::{PyObject, PyTuple, PyDict, PyString, exc};
use conversion::ToPyObject;
use rustobject::{TypeBuilder, TypeConstructor};
use ffi;
use err::{self, PyResult};

#[macro_export]
#[doc(hidden)]
macro_rules! py_method_def {
    ($f: ident, $flags: expr, $wrap: expr) => {{
        static mut method_def: $crate::_detail::ffi::PyMethodDef = $crate::_detail::ffi::PyMethodDef {
            //ml_name: bytes!(stringify!($f), "\0"),
            ml_name: 0 as *const $crate::_detail::libc::c_char,
            ml_meth: None,
            ml_flags: $crate::_detail::ffi::METH_VARARGS | $crate::_detail::ffi::METH_KEYWORDS | $flags,
            ml_doc: 0 as *const $crate::_detail::libc::c_char
        };
        method_def.ml_name = concat!(stringify!($f), "\0").as_ptr() as *const _;
        method_def.ml_meth = Some(
            std::mem::transmute::<$crate::_detail::ffi::PyCFunctionWithKeywords,
                                  $crate::_detail::ffi::PyCFunction>($wrap)
        );
        &mut method_def
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_fn_wrap {
    // * $f: function name, used as part of wrapper function name
    // * |py, args, kwargs| { body }
    ($f: ident, | $py: ident, $args: ident, $kwargs: ident | $body: block) => {{
        unsafe extern "C" fn wrap<DUMMY>(
            _slf: *mut $crate::_detail::ffi::PyObject,
            $args: *mut $crate::_detail::ffi::PyObject,
            $kwargs: *mut $crate::_detail::ffi::PyObject)
        -> *mut $crate::_detail::ffi::PyObject
        {
            py_wrap_body!($py, concat!("Rust panic in py_fn!(", stringify!($f), ")"),
                $args, $kwargs, { $body })
        }
        wrap::<()>
    }};
}

#[macro_export]
#[doc(hidden)] // TODO: eliminate this macro
macro_rules! py_wrap_body {
    ($py: ident, $location: expr, $args: ident, $kwargs: ident, $body: block) => {{
        let _guard = $crate::_detail::PanicGuard::with_message(
            concat!("Rust panic in ", $location));
        let $py: $crate::Python = $crate::_detail::bounded_assume_gil_acquired(&$args);
        let $args: $crate::PyTuple = $crate::PyObject::from_borrowed_ptr($py, $args).unchecked_cast_into();
        let $kwargs: Option<$crate::PyDict> = $crate::_detail::get_kwargs($py, $kwargs);
        let ret = {
            let $args = &$args;
            let $kwargs = $kwargs.as_ref();
            $crate::_detail::result_to_ptr($py, $body)
        };
        $crate::PyDrop::release_ref($args, $py);
        $crate::PyDrop::release_ref($kwargs, $py);
        ret
    }}
}

#[macro_export]
#[doc(hidden)] // combines py_wrap_body with py_argparse
macro_rules! py_wrap_argparse {
    ($py: ident, $location: expr, $args: expr, $kwargs: expr,
        ($( $pname:ident : $ptype:ty ),*) $body:block
    ) => {{
        let args = $args;
        let kwargs = $kwargs;
        py_wrap_body!($py, $location, args, kwargs, {
            py_argparse!($py, Some($location), args, kwargs,
                ( $($pname : $ptype),* ) $body)
        })
    }}
}

#[inline]
pub unsafe fn get_kwargs(py: Python, ptr: *mut ffi::PyObject) -> Option<PyDict> {
    if ptr.is_null() {
        None
    } else {
        Some(PyObject::from_borrowed_ptr(py, ptr).unchecked_cast_into())
    }
}

pub fn result_to_ptr<T>(py: Python, result: PyResult<T>) -> *mut ffi::PyObject
    where T: ToPyObject
{
    match result {
        Ok(val) => {
            return val.into_py_object(py).into_object().steal_ptr();
        }
        Err(e) => {
            e.restore(py);
            return ptr::null_mut();
        }
    }
}

/// Creates a Python callable object that invokes a Rust function.
///
/// There are two forms of this macro:
/// 1) py_fn!(f)
///     `f` is the name of a rust function with the signature
///     `fn(Python, &PyTuple, Option<&PyDict>) -> PyResult<R>`
///      for some `R` that implements `ToPyObject`.
///
/// 2) py_fn!(f(parameter_list))
///     This form automatically converts the arguments into
///     the Rust types specified in the parameter list,
///     and then calls `f(Python, Parameters)`.
///     See `py_argparse!()` for details on argument parsing.
///
/// The macro returns an unspecified type that implements `ToPyObject`.
/// The resulting python object is a callable object that invokes
/// the Rust function.
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
///     dict.set_item(py, "multiply", py_fn!(multiply(lhs: i32, rhs: i32))).unwrap();
///     py.run("print(multiply(6, 7))", None, Some(&dict)).unwrap();
/// }
/// ```
#[macro_export]
macro_rules! py_fn {
    ($f: ident) => ({
        let wrap = py_fn_wrap!($f, |py, args, kwargs| {
            $f(py, args, kwargs)
        });
        unsafe { $crate::_detail::py_fn_impl(py_method_def!($f, 0, wrap)) }
    });
    ($f: ident ( $( $pname:ident : $ptype:ty ),* ) ) => ({
        let wrap = py_fn_wrap!($f, |py, args, kwargs| {
            py_argparse!(py, Some(stringify!($f)), args, kwargs,
                    ( $($pname : $ptype),* ) { $f( py, $($pname),* ) })
        });
        unsafe { $crate::_detail::py_fn_impl(py_method_def!($f, 0, wrap)) }
    });
}

/// Result type of the `py_fn!()` macro.
///
/// Use the `ToPyObject` implementation to create a python callable object.
pub struct PyFn(*mut ffi::PyMethodDef);

#[inline]
pub unsafe fn py_fn_impl(def: *mut ffi::PyMethodDef) -> PyFn {
    PyFn(def)
}

impl ToPyObject for PyFn {
    type ObjectType = PyObject;

    fn to_py_object(&self, py: Python) -> PyObject {
        unsafe {
            err::from_owned_ptr_or_panic(py, ffi::PyCFunction_New(self.0, ptr::null_mut()))
        }
    }
}

unsafe impl TypeConstructor for PyFn {
    fn tp_new(&self) -> ffi::newfunc {
        unsafe {
            mem::transmute::<ffi::PyCFunction, ffi::newfunc>((*self.0).ml_meth.unwrap())
        }
    }
}

// Tests for this file are in tests/test_function.rs

