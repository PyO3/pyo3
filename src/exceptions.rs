// Copyright (c) 2017-present PyO3 Project and Contributors

//! Exception types defined by python.

use crate::err::{PyErr, PyResult};
use crate::ffi;
use crate::type_object::PyTypeObject;
use crate::types::{PyAny, PyTuple};
use crate::Python;
use crate::{AsPyPointer, ToPyObject};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::{self, ops};

/// The boilerplate to convert between a rust type and a python exception
#[macro_export]
macro_rules! impl_exception_boilerplate {
    ($name: ident) => {
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
            pub fn py_err<T: $crate::ToPyObject + 'static>(args: T) -> $crate::PyErr {
                $crate::PyErr::new::<Self, T>(args)
            }

            pub fn into<R, T: $crate::ToPyObject + 'static>(args: T) -> $crate::PyResult<R> {
                $crate::PyErr::new::<Self, T>(args).into()
            }
        }
    };
}

/// Defines rust type for exception defined in Python code.
///
/// # Syntax
///
/// `import_exception!(module, MyError)`
///
/// * `module` is the name of the containing module.
/// * `MyError` is the name of the new exception type.
///
/// # Example
/// ```
/// use pyo3::import_exception;
/// use pyo3::types::IntoPyDict;
/// use pyo3::Python;
///
/// import_exception!(socket, gaierror);
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///
///     let ctx = [("gaierror", py.get_type::<gaierror>())].into_py_dict(py);
///     py.run(
///         "import socket; assert gaierror is socket.gaierror",
///         None,
///         Some(ctx),
///     )
///     .unwrap();
/// }
///
/// ```
#[macro_export]
macro_rules! import_exception {
    ($module: expr, $name: ident) => {
        #[allow(non_camel_case_types)] // E.g. `socket.herror`
        pub struct $name;

        $crate::impl_exception_boilerplate!($name);

        $crate::import_exception_type_object!($module, $name);
    };
}

/// `impl $crate::type_object::PyTypeObject for $name` where `$name` is an exception defined in python
/// code.
#[macro_export]
macro_rules! import_exception_type_object {
    ($module: expr, $name: ident) => {
        unsafe impl $crate::type_object::PyTypeObject for $name {
            fn init_type() -> std::ptr::NonNull<$crate::ffi::PyTypeObject> {
                // We can't use lazy_static here because raw pointers aren't Send
                static TYPE_OBJECT_ONCE: ::std::sync::Once = ::std::sync::Once::new();
                static mut TYPE_OBJECT: *mut $crate::ffi::PyTypeObject = ::std::ptr::null_mut();

                TYPE_OBJECT_ONCE.call_once(|| {
                    let gil = $crate::Python::acquire_gil();
                    let py = gil.python();

                    unsafe {
                        let imp = py
                            .import(stringify!($module))
                            .expect(concat!("Can not import module: ", stringify!($module)));
                        let cls = imp.get(stringify!($name)).expect(concat!(
                            "Can not load exception class: {}.{}",
                            stringify!($module),
                            ".",
                            stringify!($name)
                        ));
                        TYPE_OBJECT =
                            $crate::IntoPyPointer::into_ptr(cls) as *mut $crate::ffi::PyTypeObject;
                    }
                });

                unsafe { std::ptr::NonNull::new_unchecked(TYPE_OBJECT) }
            }
        }
    };
}

/// Defines a new exception type.
///
/// # Syntax
/// `create_exception!(module, MyError, BaseException)`
///
/// * `module` is the name of the containing module.
/// * `MyError` is the name of the new exception type.
/// * `BaseException` is the superclass of MyError, usually `pyo3::exceptions::Exception`
///
/// # Example
/// ```
/// use pyo3::prelude::*;
/// use pyo3::create_exception;
/// use pyo3::types::IntoPyDict;
/// use pyo3::exceptions::Exception;
///
/// create_exception!(mymodule, CustomError, Exception);
///
/// fn main() {
///     let gil = Python::acquire_gil();
///     let py = gil.python();
///     let error_type = py.get_type::<CustomError>();
///     let ctx = [("CustomError", error_type)].into_py_dict(py);
///     let type_description: String = py
///         .eval("str(CustomError)", None, Some(&ctx))
///         .unwrap()
///         .extract()
///         .unwrap();
///     assert_eq!(type_description, "<class 'mymodule.CustomError'>");
///     py.run(
///         "assert CustomError('oops').args == ('oops',)",
///         None,
///         Some(ctx),
///     )
///     .unwrap();
/// }
/// ```
#[macro_export]
macro_rules! create_exception {
    ($module: ident, $name: ident, $base: ty) => {
        #[allow(non_camel_case_types)] // E.g. `socket.herror`
        pub struct $name;

        $crate::impl_exception_boilerplate!($name);

        $crate::create_exception_type_object!($module, $name, $base);
    };
}

/// `impl $crate::type_object::PyTypeObject for $name` where `$name` is an exception newly defined in
/// rust code.
#[macro_export]
macro_rules! create_exception_type_object {
    ($module: ident, $name: ident, $base: ty) => {
        unsafe impl $crate::type_object::PyTypeObject for $name {
            fn init_type() -> std::ptr::NonNull<$crate::ffi::PyTypeObject> {
                // We can't use lazy_static here because raw pointers aren't Send
                static TYPE_OBJECT_ONCE: ::std::sync::Once = ::std::sync::Once::new();
                static mut TYPE_OBJECT: *mut $crate::ffi::PyTypeObject = ::std::ptr::null_mut();

                TYPE_OBJECT_ONCE.call_once(|| {
                    let gil = $crate::Python::acquire_gil();
                    let py = gil.python();

                    unsafe {
                        TYPE_OBJECT = $crate::PyErr::new_type(
                            py,
                            concat!(stringify!($module), ".", stringify!($name)),
                            Some(py.get_type::<$base>()),
                            None,
                        );
                    }
                });

                unsafe { std::ptr::NonNull::new_unchecked(TYPE_OBJECT) }
            }
        }
    };
}

macro_rules! impl_native_exception (
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
        unsafe impl PyTypeObject for $name {
            fn init_type() -> std::ptr::NonNull<$crate::ffi::PyTypeObject> {
                unsafe { std::ptr::NonNull::new_unchecked(ffi::$exc_name as *mut _) }
            }
        }
    );
);

impl_native_exception!(BaseException, PyExc_BaseException);
impl_native_exception!(Exception, PyExc_Exception);
impl_native_exception!(StopAsyncIteration, PyExc_StopAsyncIteration);
impl_native_exception!(StopIteration, PyExc_StopIteration);
impl_native_exception!(GeneratorExit, PyExc_GeneratorExit);
impl_native_exception!(ArithmeticError, PyExc_ArithmeticError);
impl_native_exception!(LookupError, PyExc_LookupError);

impl_native_exception!(AssertionError, PyExc_AssertionError);
impl_native_exception!(AttributeError, PyExc_AttributeError);
impl_native_exception!(BufferError, PyExc_BufferError);
impl_native_exception!(EOFError, PyExc_EOFError);
impl_native_exception!(FloatingPointError, PyExc_FloatingPointError);
impl_native_exception!(OSError, PyExc_OSError);
impl_native_exception!(ImportError, PyExc_ImportError);

#[cfg(Py_3_6)]
impl_native_exception!(ModuleNotFoundError, PyExc_ModuleNotFoundError);

impl_native_exception!(IndexError, PyExc_IndexError);
impl_native_exception!(KeyError, PyExc_KeyError);
impl_native_exception!(KeyboardInterrupt, PyExc_KeyboardInterrupt);
impl_native_exception!(MemoryError, PyExc_MemoryError);
impl_native_exception!(NameError, PyExc_NameError);
impl_native_exception!(OverflowError, PyExc_OverflowError);
impl_native_exception!(RuntimeError, PyExc_RuntimeError);
impl_native_exception!(RecursionError, PyExc_RecursionError);
impl_native_exception!(NotImplementedError, PyExc_NotImplementedError);
impl_native_exception!(SyntaxError, PyExc_SyntaxError);
impl_native_exception!(ReferenceError, PyExc_ReferenceError);
impl_native_exception!(SystemError, PyExc_SystemError);
impl_native_exception!(SystemExit, PyExc_SystemExit);
impl_native_exception!(TypeError, PyExc_TypeError);
impl_native_exception!(UnboundLocalError, PyExc_UnboundLocalError);
impl_native_exception!(UnicodeError, PyExc_UnicodeError);
impl_native_exception!(UnicodeDecodeError, PyExc_UnicodeDecodeError);
impl_native_exception!(UnicodeEncodeError, PyExc_UnicodeEncodeError);
impl_native_exception!(UnicodeTranslateError, PyExc_UnicodeTranslateError);
impl_native_exception!(ValueError, PyExc_ValueError);
impl_native_exception!(ZeroDivisionError, PyExc_ZeroDivisionError);

impl_native_exception!(BlockingIOError, PyExc_BlockingIOError);
impl_native_exception!(BrokenPipeError, PyExc_BrokenPipeError);
impl_native_exception!(ChildProcessError, PyExc_ChildProcessError);
impl_native_exception!(ConnectionError, PyExc_ConnectionError);
impl_native_exception!(ConnectionAbortedError, PyExc_ConnectionAbortedError);
impl_native_exception!(ConnectionRefusedError, PyExc_ConnectionRefusedError);
impl_native_exception!(ConnectionResetError, PyExc_ConnectionResetError);
impl_native_exception!(FileExistsError, PyExc_FileExistsError);
impl_native_exception!(FileNotFoundError, PyExc_FileNotFoundError);
impl_native_exception!(InterruptedError, PyExc_InterruptedError);
impl_native_exception!(IsADirectoryError, PyExc_IsADirectoryError);
impl_native_exception!(NotADirectoryError, PyExc_NotADirectoryError);
impl_native_exception!(PermissionError, PyExc_PermissionError);
impl_native_exception!(ProcessLookupError, PyExc_ProcessLookupError);
impl_native_exception!(TimeoutError, PyExc_TimeoutError);

impl_native_exception!(EnvironmentError, PyExc_EnvironmentError);
impl_native_exception!(IOError, PyExc_IOError);
#[cfg(target_os = "windows")]
impl_native_exception!(WindowsError, PyExc_WindowsError);

impl UnicodeDecodeError {
    pub fn new_err<'p>(
        py: Python<'p>,
        encoding: &CStr,
        input: &[u8],
        range: ops::Range<usize>,
        reason: &CStr,
    ) -> PyResult<&'p PyAny> {
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

    #[allow(clippy::range_plus_one)] // False positive, ..= returns the wrong type
    pub fn new_utf8<'p>(
        py: Python<'p>,
        input: &[u8],
        err: std::str::Utf8Error,
    ) -> PyResult<&'p PyAny> {
        let pos = err.valid_up_to();
        UnicodeDecodeError::new_err(
            py,
            CStr::from_bytes_with_nul(b"utf-8\0").unwrap(),
            input,
            pos..pos + 1,
            CStr::from_bytes_with_nul(b"invalid utf-8\0").unwrap(),
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
    use crate::exceptions::Exception;
    use crate::objectprotocol::ObjectProtocol;
    use crate::types::{IntoPyDict, PyDict};
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

    #[test]
    fn custom_exception() {
        create_exception!(mymodule, CustomError, Exception);

        let gil = Python::acquire_gil();
        let py = gil.python();
        let error_type = py.get_type::<CustomError>();
        let ctx = [("CustomError", error_type)].into_py_dict(py);
        let type_description: String = py
            .eval("str(CustomError)", None, Some(&ctx))
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(type_description, "<class 'mymodule.CustomError'>");
        py.run(
            "assert CustomError('oops').args == ('oops',)",
            None,
            Some(&ctx),
        )
        .unwrap();
    }
}
