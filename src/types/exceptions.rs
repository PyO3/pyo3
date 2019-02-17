// Copyright (c) 2017-present PyO3 Project and Contributors

//! This module contains the standard python exception types.

use crate::conversion::ToPyObject;
use crate::err::{PyErr, PyResult};
use crate::ffi;
use crate::instance::Py;
use crate::python::{Python, ToPyPointer};
use crate::typeob::PyTypeObject;
use crate::types::{PyObjectRef, PyTuple, PyType};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::{self, ops};

// Copyright (c) 2017-present PyO3 Project and Contributors

/// Stringify a dotted path.
#[doc(hidden)]
#[macro_export]
macro_rules! dot_stringify {
    ($e:ident) => (
        stringify!($e)
    );
    ($e:ident. $($es:ident).+) => (
        concat!(stringify!($e), ".", dot_stringify!($($es).*))
    );
}

/// Defines rust type for exception defined in Python code.
///
/// # Syntax
/// `import_exception!(module, MyError)`
///
/// * `module` is the name of the containing module.
/// * `MyError` is the name of the new exception type.
///
/// # Example
/// ```
/// #[macro_use] extern crate pyo3;
///
/// use pyo3::Python;
/// use pyo3::types::PyDict;
///
/// import_exception!(socket, gaierror);
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     let ctx = PyDict::new(py);
///
///     ctx.set_item("gaierror", py.get_type::<gaierror>()).unwrap();
///     py.run("import socket; assert gaierror is socket.gaierror", None, Some(ctx)).unwrap();
/// }
/// ```
#[macro_export]
macro_rules! import_exception {
    ($($module:ident).+ , $name: ident) => {
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl ::std::convert::From<$name> for $crate::PyErr {
            fn from(_err: $name) -> $crate::PyErr {
                $crate::PyErr::new::<$name, _>(())
            }
        }

        impl<T> ::std::convert::Into<$crate::PyResult<T>> for $name {
            fn into(self) -> $crate::PyResult<T> {
                $crate::PyErr::new::<$name, _>(()).into()
            }
        }

        impl $name {
            pub fn py_err<T: $crate::ToPyObject + 'static>(args: T) -> $crate::PyErr
                where Self: $crate::typeob::PyTypeObject + Sized
            {
                $crate::PyErr::new::<Self, T>(args)
            }
            pub fn into<R, T: $crate::ToPyObject + 'static>(args: T) -> $crate::PyResult<R>
                where Self: $crate::typeob::PyTypeObject + Sized
            {
                $crate::PyErr::new::<Self, T>(args).into()
            }
        }

        impl $crate::typeob::PyTypeObject for $name {
            #[inline]
            fn init_type() {}

            #[inline]
            fn type_object() -> $crate::Py<$crate::types::PyType> {
                use $crate::IntoPyPointer;
                static mut TYPE_OBJECT: *mut $crate::ffi::PyTypeObject = ::std::ptr::null_mut();

                unsafe {
                    if TYPE_OBJECT.is_null() {
                        let gil = $crate::Python::acquire_gil();
                        let py = gil.python();

                        let imp = py.import(dot_stringify!($($module).*))
                            .expect(concat!(
                                "Can not import module: ", dot_stringify!($($module).*)));
                        let cls = imp.get(stringify!($name))
                            .expect(concat!(
                                "Can not load exception class: {}.{}", dot_stringify!($($module).*),
                                ".", stringify!($name)));
                        TYPE_OBJECT = cls.into_ptr() as *mut $crate::ffi::PyTypeObject;
                    }

                    $crate::Py::from_borrowed_ptr(
                        TYPE_OBJECT as *const _ as *mut $crate::ffi::PyObject)
                }
            }
        }
    };
}

macro_rules! exc_type (
    ($name:ident, $exc_name:ident) => (
        pub struct $name;

        impl std::convert::From<$name> for PyErr {
            fn from(_err: $name) -> PyErr {
                PyErr::new::<$name, _>(())
            }
        }
        impl<T> std::convert::Into<$crate::PyResult<T>> for $name {
            fn into(self) -> $crate::PyResult<T> {
                PyErr::new::<$name, _>(()).into()
            }
        }
        impl $name {
            pub fn py_err<V: ToPyObject + 'static>(args: V) -> PyErr {
                PyErr::new::<$name, V>(args)
            }
            pub fn into<R, V: ToPyObject + 'static>(args: V) -> PyResult<R> {
                PyErr::new::<$name, V>(args).into()
            }
        }
        impl PyTypeObject for $name {
            #[inline]
            fn init_type() {}

            #[inline]
            fn type_object() -> Py<PyType> {
                unsafe {
                    Py::from_borrowed_ptr(ffi::$exc_name)
                }
            }
        }
    );
);

exc_type!(BaseException, PyExc_BaseException);
exc_type!(Exception, PyExc_Exception);
#[cfg(Py_3)]
exc_type!(StopAsyncIteration, PyExc_StopAsyncIteration);
exc_type!(StopIteration, PyExc_StopIteration);
exc_type!(GeneratorExit, PyExc_GeneratorExit);
exc_type!(ArithmeticError, PyExc_ArithmeticError);
exc_type!(LookupError, PyExc_LookupError);

exc_type!(AssertionError, PyExc_AssertionError);
exc_type!(AttributeError, PyExc_AttributeError);
exc_type!(BufferError, PyExc_BufferError);
exc_type!(EOFError, PyExc_EOFError);
exc_type!(FloatingPointError, PyExc_FloatingPointError);
exc_type!(OSError, PyExc_OSError);
exc_type!(ImportError, PyExc_ImportError);

#[cfg(Py_3_6)]
exc_type!(ModuleNotFoundError, PyExc_ModuleNotFoundError);

exc_type!(IndexError, PyExc_IndexError);
exc_type!(KeyError, PyExc_KeyError);
exc_type!(KeyboardInterrupt, PyExc_KeyboardInterrupt);
exc_type!(MemoryError, PyExc_MemoryError);
exc_type!(NameError, PyExc_NameError);
exc_type!(OverflowError, PyExc_OverflowError);
exc_type!(RuntimeError, PyExc_RuntimeError);
#[cfg(Py_3)]
exc_type!(RecursionError, PyExc_RecursionError);
exc_type!(NotImplementedError, PyExc_NotImplementedError);
exc_type!(SyntaxError, PyExc_SyntaxError);
exc_type!(ReferenceError, PyExc_ReferenceError);
exc_type!(SystemError, PyExc_SystemError);
exc_type!(SystemExit, PyExc_SystemExit);
exc_type!(TypeError, PyExc_TypeError);
exc_type!(UnboundLocalError, PyExc_UnboundLocalError);
exc_type!(UnicodeError, PyExc_UnicodeError);
exc_type!(UnicodeDecodeError, PyExc_UnicodeDecodeError);
exc_type!(UnicodeEncodeError, PyExc_UnicodeEncodeError);
exc_type!(UnicodeTranslateError, PyExc_UnicodeTranslateError);
exc_type!(ValueError, PyExc_ValueError);
exc_type!(ZeroDivisionError, PyExc_ZeroDivisionError);

#[cfg(Py_3)]
exc_type!(BlockingIOError, PyExc_BlockingIOError);
#[cfg(Py_3)]
exc_type!(BrokenPipeError, PyExc_BrokenPipeError);
#[cfg(Py_3)]
exc_type!(ChildProcessError, PyExc_ChildProcessError);
#[cfg(Py_3)]
exc_type!(ConnectionError, PyExc_ConnectionError);
#[cfg(Py_3)]
exc_type!(ConnectionAbortedError, PyExc_ConnectionAbortedError);
#[cfg(Py_3)]
exc_type!(ConnectionRefusedError, PyExc_ConnectionRefusedError);
#[cfg(Py_3)]
exc_type!(ConnectionResetError, PyExc_ConnectionResetError);
#[cfg(Py_3)]
exc_type!(FileExistsError, PyExc_FileExistsError);
#[cfg(Py_3)]
exc_type!(FileNotFoundError, PyExc_FileNotFoundError);
#[cfg(Py_3)]
exc_type!(InterruptedError, PyExc_InterruptedError);
#[cfg(Py_3)]
exc_type!(IsADirectoryError, PyExc_IsADirectoryError);
#[cfg(Py_3)]
exc_type!(NotADirectoryError, PyExc_NotADirectoryError);
#[cfg(Py_3)]
exc_type!(PermissionError, PyExc_PermissionError);
#[cfg(Py_3)]
exc_type!(ProcessLookupError, PyExc_ProcessLookupError);
#[cfg(Py_3)]
exc_type!(TimeoutError, PyExc_TimeoutError);

exc_type!(EnvironmentError, PyExc_EnvironmentError);
exc_type!(IOError, PyExc_IOError);
#[cfg(target_os = "windows")]
exc_type!(WindowsError, PyExc_WindowsError);

impl UnicodeDecodeError {
    pub fn new_err<'p>(
        py: Python<'p>,
        encoding: &CStr,
        input: &[u8],
        range: ops::Range<usize>,
        reason: &CStr,
    ) -> PyResult<&'p PyObjectRef> {
        unsafe {
            let input: &[c_char] = &*(input as *const [u8] as *const [c_char]);
            py.from_owned_ptr_or_err(ffi::PyUnicodeDecodeError_Create(
                encoding.as_ptr(),
                input.as_ptr(),
                input.len() as ffi::Py_ssize_t,
                range.start as ffi::Py_ssize_t,
                range.end as ffi::Py_ssize_t,
                reason.as_ptr(),
            ))
        }
    }

    pub fn new_utf8<'p>(
        py: Python<'p>,
        input: &[u8],
        err: std::str::Utf8Error,
    ) -> PyResult<&'p PyObjectRef> {
        let pos = err.valid_up_to();
        UnicodeDecodeError::new_err(
            py,
            cstr!("utf-8"),
            input,
            pos..pos + 1,
            cstr!("invalid utf-8"),
        )
    }
}

impl StopIteration {
    pub fn stop_iteration(_py: Python, args: &PyTuple) {
        unsafe {
            ffi::PyErr_SetObject(
                ffi::PyExc_StopIteration as *mut ffi::PyObject,
                args.as_ptr(),
            );
        }
    }
}

/// Exceptions defined in `asyncio` module
pub mod asyncio {
    import_exception!(asyncio, CancelledError);
    import_exception!(asyncio, InvalidStateError);
    import_exception!(asyncio, TimeoutError);
    import_exception!(asyncio, IncompleteReadError);
    import_exception!(asyncio, LimitOverrunError);
    import_exception!(asyncio, QueueEmpty);
    import_exception!(asyncio, QueueFull);
}

/// Exceptions defined in `socket` module
pub mod socket {
    import_exception!(socket, herror);
    import_exception!(socket, gaierror);
    import_exception!(socket, timeout);
}

#[cfg(test)]
mod test {
    use crate::types::PyDict;
    use crate::{PyErr, Python};

    import_exception!(socket, gaierror);
    import_exception!(email.errors, MessageError);

    #[test]
    fn test_check_exception() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let err: PyErr = gaierror.into();
        let socket = py
            .import("socket")
            .map_err(|e| e.print(py))
            .expect("could not import socket");

        let d = PyDict::new(py);
        d.set_item("socket", socket)
            .map_err(|e| e.print(py))
            .expect("could not setitem");
        d.set_item("exc", err)
            .map_err(|e| e.print(py))
            .expect("could not setitem");

        py.run("assert isinstance(exc, socket.gaierror)", None, Some(d))
            .map_err(|e| e.print(py))
            .expect("assertion failed");
    }

    #[test]
    fn test_check_exception_nested() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let err: PyErr = MessageError.into();
        let email = py
            .import("email")
            .map_err(|e| e.print(py))
            .expect("could not import email");

        let d = PyDict::new(py);
        d.set_item("email", email)
            .map_err(|e| e.print(py))
            .expect("could not setitem");
        d.set_item("exc", err)
            .map_err(|e| e.print(py))
            .expect("could not setitem");

        py.run(
            "assert isinstance(exc, email.errors.MessageError)",
            None,
            Some(d),
        )
        .map_err(|e| e.print(py))
        .expect("assertion failed");
    }
}
