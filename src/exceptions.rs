// Copyright (c) 2017-present PyO3 Project and Contributors

//! Exception and warning types defined by Python.
//!
//! The structs in this module represent Python's built-in exceptions and
//! warnings, while the modules comprise structs representing errors defined in
//! Python code.
//!
//! The latter are created with the
//! [`import_exception`](crate::import_exception) macro, which you can use
//! yourself to import Python classes that are ultimately derived from
//! `BaseException`.

use crate::{ffi, PyResult, Python};
use std::ffi::CStr;
use std::ops;
use std::os::raw::c_char;

/// The boilerplate to convert between a Rust type and a Python exception.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_exception_boilerplate {
    ($name: ident) => {
        impl ::std::convert::From<&$name> for $crate::PyErr {
            #[inline]
            fn from(err: &$name) -> $crate::PyErr {
                $crate::PyErr::from_value(err)
            }
        }

        impl $name {
            /// Creates a new [`PyErr`] of this type.
            ///
            /// [`PyErr`]: https://docs.rs/pyo3/latest/pyo3/struct.PyErr.html "PyErr in pyo3"
            #[inline]
            pub fn new_err<A>(args: A) -> $crate::PyErr
            where
                A: $crate::PyErrArguments + ::std::marker::Send + ::std::marker::Sync + 'static,
            {
                $crate::PyErr::new::<$name, A>(args)
            }
        }

        impl ::std::error::Error for $name {
            fn source(&self) -> ::std::option::Option<&(dyn ::std::error::Error + 'static)> {
                unsafe {
                    use $crate::AsPyPointer;
                    let cause: &$crate::exceptions::PyBaseException = self
                        .py()
                        .from_owned_ptr_or_opt($crate::ffi::PyException_GetCause(self.as_ptr()))?;

                    ::std::option::Option::Some(cause)
                }
            }
        }
    };
}

/// Defines a Rust type for an exception defined in Python code.
///
/// # Syntax
///
/// ```import_exception!(module, MyError)```
///
/// * `module` is the name of the containing module.
/// * `MyError` is the name of the new exception type.
///
/// # Examples
/// ```
/// use pyo3::import_exception;
/// use pyo3::types::IntoPyDict;
/// use pyo3::Python;
///
/// import_exception!(socket, gaierror);
///
/// Python::with_gil(|py| {
///     let ctx = [("gaierror", py.get_type::<gaierror>())].into_py_dict(py);
///     pyo3::py_run!(py, *ctx, "import socket; assert gaierror is socket.gaierror");
/// });
///
/// ```
#[macro_export]
macro_rules! import_exception {
    ($module: expr, $name: ident) => {
        /// A Rust type representing an exception defined in Python code.
        ///
        /// This type was created by the [`pyo3::import_exception!`] macro - see its documentation
        /// for more information.
        ///
        /// [`pyo3::import_exception!`]: https://docs.rs/pyo3/latest/pyo3/macro.import_exception.html "import_exception in pyo3"
        #[repr(transparent)]
        #[allow(non_camel_case_types)] // E.g. `socket.herror`
        pub struct $name($crate::PyAny);

        $crate::impl_exception_boilerplate!($name);

        $crate::pyobject_native_type_core!(
            $name,
            *$name::type_object_raw($crate::Python::assume_gil_acquired()),
            #module=::std::option::Option::Some(stringify!($module))
        );

        impl $name {
            fn type_object_raw(py: $crate::Python<'_>) -> *mut $crate::ffi::PyTypeObject {
                use $crate::sync::GILOnceCell;
                use $crate::AsPyPointer;
                static TYPE_OBJECT: GILOnceCell<$crate::Py<$crate::types::PyType>> =
                    GILOnceCell::new();

                TYPE_OBJECT
                    .get_or_init(py, || {
                        let imp = py
                            .import(stringify!($module))
                            .unwrap_or_else(|err| {
                                let traceback = err
                                    .traceback(py)
                                    .map(|tb| tb.format().expect("raised exception will have a traceback"))
                                    .unwrap_or_default();
                                ::std::panic!("Can not import module {}: {}\n{}", stringify!($module), err, traceback);
                            });
                        let cls = imp.getattr(stringify!($name)).expect(concat!(
                            "Can not load exception class: {}.{}",
                            stringify!($module),
                            ".",
                            stringify!($name)
                        ));

                        cls.extract()
                            .expect("Imported exception should be a type object")
                    })
                    .as_ptr() as *mut _
            }
        }
    };
}

/// Defines a new exception type.
///
/// # Syntax
///
/// * `module` is the name of the containing module.
/// * `name` is the name of the new exception type.
/// * `base` is the base class of `MyError`, usually [`PyException`].
/// * `doc` (optional) is the docstring visible to users (with `.__doc__` and `help()`) and
/// accompanies your error type in your crate's documentation.
///
/// # Examples
///
/// ```
/// use pyo3::prelude::*;
/// use pyo3::create_exception;
/// use pyo3::exceptions::PyException;
///
/// create_exception!(my_module, MyError, PyException, "Some description.");
///
/// #[pyfunction]
/// fn raise_myerror() -> PyResult<()> {
///     let err = MyError::new_err("Some error happened.");
///     Err(err)
/// }
///
/// #[pymodule]
/// fn my_module(py: Python<'_>, m: &PyModule) -> PyResult<()> {
///     m.add("MyError", py.get_type::<MyError>())?;
///     m.add_function(wrap_pyfunction!(raise_myerror, py)?)?;
///     Ok(())
/// }
/// # fn main() -> PyResult<()> {
/// #     Python::with_gil(|py| -> PyResult<()> {
/// #         let fun = wrap_pyfunction!(raise_myerror, py)?;
/// #         let locals = pyo3::types::PyDict::new(py);
/// #         locals.set_item("MyError", py.get_type::<MyError>())?;
/// #         locals.set_item("raise_myerror", fun)?;
/// #
/// #         py.run(
/// # "try:
/// #     raise_myerror()
/// # except MyError as e:
/// #     assert e.__doc__ == 'Some description.'
/// #     assert str(e) == 'Some error happened.'",
/// #             None,
/// #             Some(locals),
/// #         )?;
/// #
/// #         Ok(())
/// #     })
/// # }
/// ```
///
/// Python code can handle this exception like any other exception:
///
/// ```python
/// from my_module import MyError, raise_myerror
///
/// try:
///     raise_myerror()
/// except MyError as e:
///     assert e.__doc__ == 'Some description.'
///     assert str(e) == 'Some error happened.'
/// ```
///
#[macro_export]
macro_rules! create_exception {
    ($module: expr, $name: ident, $base: ty) => {
        #[repr(transparent)]
        #[allow(non_camel_case_types)] // E.g. `socket.herror`
        pub struct $name($crate::PyAny);

        $crate::impl_exception_boilerplate!($name);

        $crate::create_exception_type_object!($module, $name, $base, ::std::option::Option::None);
    };
    ($module: expr, $name: ident, $base: ty, $doc: expr) => {
        #[repr(transparent)]
        #[allow(non_camel_case_types)] // E.g. `socket.herror`
        #[doc = $doc]
        pub struct $name($crate::PyAny);

        $crate::impl_exception_boilerplate!($name);

        $crate::create_exception_type_object!(
            $module,
            $name,
            $base,
            ::std::option::Option::Some($doc)
        );
    };
}

/// `impl PyTypeInfo for $name` where `$name` is an
/// exception newly defined in Rust code.
#[doc(hidden)]
#[macro_export]
macro_rules! create_exception_type_object {
    ($module: expr, $name: ident, $base: ty, $doc: expr) => {
        $crate::pyobject_native_type_core!(
            $name,
            *$name::type_object_raw($crate::Python::assume_gil_acquired()),
            #module=::std::option::Option::Some(stringify!($module))
        );

        impl $name {
            fn type_object_raw(py: $crate::Python<'_>) -> *mut $crate::ffi::PyTypeObject {
                use $crate::sync::GILOnceCell;
                use $crate::AsPyPointer;
                static TYPE_OBJECT: GILOnceCell<$crate::Py<$crate::types::PyType>> =
                    GILOnceCell::new();

                TYPE_OBJECT
                    .get_or_init(py, ||
                        $crate::PyErr::new_type(
                            py,
                            concat!(stringify!($module), ".", stringify!($name)),
                            $doc,
                            ::std::option::Option::Some(py.get_type::<$base>()),
                            ::std::option::Option::None,
                        ).expect("Failed to initialize new exception type.")
                ).as_ptr() as *mut $crate::ffi::PyTypeObject
            }
        }
    };
}

macro_rules! impl_native_exception (
    ($name:ident, $exc_name:ident, $doc:expr, $layout:path) => (
        #[doc = $doc]
        #[allow(clippy::upper_case_acronyms)]
        pub struct $name($crate::PyAny);

        $crate::impl_exception_boilerplate!($name);
        $crate::pyobject_native_type!($name, $layout, *($crate::ffi::$exc_name as *mut $crate::ffi::PyTypeObject));
    );
    ($name:ident, $exc_name:ident, $doc:expr) => (
        impl_native_exception!($name, $exc_name, $doc, $crate::ffi::PyBaseExceptionObject);
    )
);

#[cfg(windows)]
macro_rules! impl_windows_native_exception (
    ($name:ident, $exc_name:ident, $doc:expr, $layout:path) => (
        #[cfg(windows)]
        #[doc = $doc]
        #[allow(clippy::upper_case_acronyms)]
        pub struct $name($crate::PyAny);

        $crate::impl_exception_boilerplate!($name);
        $crate::pyobject_native_type!($name, $layout, *($crate::ffi::$exc_name as *mut $crate::ffi::PyTypeObject));
    );
    ($name:ident, $exc_name:ident, $doc:expr) => (
        impl_windows_native_exception!($name, $exc_name, $doc, $crate::ffi::PyBaseExceptionObject);
    )
);

macro_rules! native_doc(
    ($name: literal, $alt: literal) => (
        concat!(
"Represents Python's [`", $name, "`](https://docs.python.org/3/library/exceptions.html#", $name, ") exception.

", $alt
        )
    );
    ($name: literal) => (
        concat!(
"
Represents Python's [`", $name, "`](https://docs.python.org/3/library/exceptions.html#", $name, ") exception.

# Example: Raising ", $name, " from Rust

This exception can be sent to Python code by converting it into a
[`PyErr`](crate::PyErr), where Python code can then catch it.
```
use pyo3::prelude::*;
use pyo3::exceptions::Py", $name, ";

#[pyfunction]
fn always_throws() -> PyResult<()> {
    let message = \"I'm ", $name ,", and I was raised from Rust.\";
    Err(Py", $name, "::new_err(message))
}
#
# Python::with_gil(|py| {
#     let fun = pyo3::wrap_pyfunction!(always_throws, py).unwrap();
#     let err = fun.call0().expect_err(\"called a function that should always return an error but the return value was Ok\");
#     assert!(err.is_instance_of::<Py", $name, ">(py))
# });
```

Python code:
 ```python
 from my_module import always_throws

try:
    always_throws()
except ", $name, " as e:
    print(f\"Caught an exception: {e}\")
```

# Example: Catching ", $name, " in Rust

```
use pyo3::prelude::*;
use pyo3::exceptions::Py", $name, ";

Python::with_gil(|py| {
    let result: PyResult<()> = py.run(\"raise ", $name, "\", None, None);

    let error_type = match result {
        Ok(_) => \"Not an error\",
        Err(error) if error.is_instance_of::<Py", $name, ">(py) => \"" , $name, "\",
        Err(_) => \"Some other error\",
    };

    assert_eq!(error_type, \"", $name, "\");
});
```
"
        )
    );
);

impl_native_exception!(
    PyBaseException,
    PyExc_BaseException,
    native_doc!("BaseException")
);
impl_native_exception!(PyException, PyExc_Exception, native_doc!("Exception"));
impl_native_exception!(
    PyStopAsyncIteration,
    PyExc_StopAsyncIteration,
    native_doc!("StopAsyncIteration")
);
impl_native_exception!(
    PyStopIteration,
    PyExc_StopIteration,
    native_doc!("StopIteration"),
    ffi::PyStopIterationObject
);
impl_native_exception!(
    PyGeneratorExit,
    PyExc_GeneratorExit,
    native_doc!("GeneratorExit")
);
impl_native_exception!(
    PyArithmeticError,
    PyExc_ArithmeticError,
    native_doc!("ArithmeticError")
);
impl_native_exception!(PyLookupError, PyExc_LookupError, native_doc!("LookupError"));

impl_native_exception!(
    PyAssertionError,
    PyExc_AssertionError,
    native_doc!("AssertionError")
);
impl_native_exception!(
    PyAttributeError,
    PyExc_AttributeError,
    native_doc!("AttributeError")
);
impl_native_exception!(PyBufferError, PyExc_BufferError, native_doc!("BufferError"));
impl_native_exception!(PyEOFError, PyExc_EOFError, native_doc!("EOFError"));
impl_native_exception!(
    PyFloatingPointError,
    PyExc_FloatingPointError,
    native_doc!("FloatingPointError")
);
#[cfg(not(PyPy))]
impl_native_exception!(
    PyOSError,
    PyExc_OSError,
    native_doc!("OSError"),
    ffi::PyOSErrorObject
);
#[cfg(PyPy)]
impl_native_exception!(PyOSError, PyExc_OSError, native_doc!("OSError"));
impl_native_exception!(PyImportError, PyExc_ImportError, native_doc!("ImportError"));

impl_native_exception!(
    PyModuleNotFoundError,
    PyExc_ModuleNotFoundError,
    native_doc!("ModuleNotFoundError")
);

impl_native_exception!(PyIndexError, PyExc_IndexError, native_doc!("IndexError"));
impl_native_exception!(PyKeyError, PyExc_KeyError, native_doc!("KeyError"));
impl_native_exception!(
    PyKeyboardInterrupt,
    PyExc_KeyboardInterrupt,
    native_doc!("KeyboardInterrupt")
);
impl_native_exception!(PyMemoryError, PyExc_MemoryError, native_doc!("MemoryError"));
impl_native_exception!(PyNameError, PyExc_NameError, native_doc!("NameError"));
impl_native_exception!(
    PyOverflowError,
    PyExc_OverflowError,
    native_doc!("OverflowError")
);
impl_native_exception!(
    PyRuntimeError,
    PyExc_RuntimeError,
    native_doc!("RuntimeError")
);
impl_native_exception!(
    PyRecursionError,
    PyExc_RecursionError,
    native_doc!("RecursionError")
);
impl_native_exception!(
    PyNotImplementedError,
    PyExc_NotImplementedError,
    native_doc!("NotImplementedError")
);
#[cfg(not(PyPy))]
impl_native_exception!(
    PySyntaxError,
    PyExc_SyntaxError,
    native_doc!("SyntaxError"),
    ffi::PySyntaxErrorObject
);
#[cfg(PyPy)]
impl_native_exception!(PySyntaxError, PyExc_SyntaxError, native_doc!("SyntaxError"));
impl_native_exception!(
    PyReferenceError,
    PyExc_ReferenceError,
    native_doc!("ReferenceError")
);
impl_native_exception!(PySystemError, PyExc_SystemError, native_doc!("SystemError"));
#[cfg(not(PyPy))]
impl_native_exception!(
    PySystemExit,
    PyExc_SystemExit,
    native_doc!("SystemExit"),
    ffi::PySystemExitObject
);
#[cfg(PyPy)]
impl_native_exception!(PySystemExit, PyExc_SystemExit, native_doc!("SystemExit"));
impl_native_exception!(PyTypeError, PyExc_TypeError, native_doc!("TypeError"));
impl_native_exception!(
    PyUnboundLocalError,
    PyExc_UnboundLocalError,
    native_doc!("UnboundLocalError")
);
#[cfg(not(PyPy))]
impl_native_exception!(
    PyUnicodeError,
    PyExc_UnicodeError,
    native_doc!("UnicodeError"),
    ffi::PyUnicodeErrorObject
);
#[cfg(PyPy)]
impl_native_exception!(
    PyUnicodeError,
    PyExc_UnicodeError,
    native_doc!("UnicodeError")
);
// these three errors need arguments, so they're too annoying to write tests for using macros...
impl_native_exception!(
    PyUnicodeDecodeError,
    PyExc_UnicodeDecodeError,
    native_doc!("UnicodeDecodeError", "")
);
impl_native_exception!(
    PyUnicodeEncodeError,
    PyExc_UnicodeEncodeError,
    native_doc!("UnicodeEncodeError", "")
);
impl_native_exception!(
    PyUnicodeTranslateError,
    PyExc_UnicodeTranslateError,
    native_doc!("UnicodeTranslateError", "")
);
impl_native_exception!(PyValueError, PyExc_ValueError, native_doc!("ValueError"));
impl_native_exception!(
    PyZeroDivisionError,
    PyExc_ZeroDivisionError,
    native_doc!("ZeroDivisionError")
);

impl_native_exception!(
    PyBlockingIOError,
    PyExc_BlockingIOError,
    native_doc!("BlockingIOError")
);
impl_native_exception!(
    PyBrokenPipeError,
    PyExc_BrokenPipeError,
    native_doc!("BrokenPipeError")
);
impl_native_exception!(
    PyChildProcessError,
    PyExc_ChildProcessError,
    native_doc!("ChildProcessError")
);
impl_native_exception!(
    PyConnectionError,
    PyExc_ConnectionError,
    native_doc!("ConnectionError")
);
impl_native_exception!(
    PyConnectionAbortedError,
    PyExc_ConnectionAbortedError,
    native_doc!("ConnectionAbortedError")
);
impl_native_exception!(
    PyConnectionRefusedError,
    PyExc_ConnectionRefusedError,
    native_doc!("ConnectionRefusedError")
);
impl_native_exception!(
    PyConnectionResetError,
    PyExc_ConnectionResetError,
    native_doc!("ConnectionResetError")
);
impl_native_exception!(
    PyFileExistsError,
    PyExc_FileExistsError,
    native_doc!("FileExistsError")
);
impl_native_exception!(
    PyFileNotFoundError,
    PyExc_FileNotFoundError,
    native_doc!("FileNotFoundError")
);
impl_native_exception!(
    PyInterruptedError,
    PyExc_InterruptedError,
    native_doc!("InterruptedError")
);
impl_native_exception!(
    PyIsADirectoryError,
    PyExc_IsADirectoryError,
    native_doc!("IsADirectoryError")
);
impl_native_exception!(
    PyNotADirectoryError,
    PyExc_NotADirectoryError,
    native_doc!("NotADirectoryError")
);
impl_native_exception!(
    PyPermissionError,
    PyExc_PermissionError,
    native_doc!("PermissionError")
);
impl_native_exception!(
    PyProcessLookupError,
    PyExc_ProcessLookupError,
    native_doc!("ProcessLookupError")
);
impl_native_exception!(
    PyTimeoutError,
    PyExc_TimeoutError,
    native_doc!("TimeoutError")
);

impl_native_exception!(
    PyEnvironmentError,
    PyExc_EnvironmentError,
    native_doc!("EnvironmentError")
);
impl_native_exception!(PyIOError, PyExc_IOError, native_doc!("IOError"));

#[cfg(windows)]
impl_windows_native_exception!(
    PyWindowsError,
    PyExc_WindowsError,
    native_doc!("WindowsError")
);

impl PyUnicodeDecodeError {
    /// Creates a Python `UnicodeDecodeError`.
    pub fn new<'p>(
        py: Python<'p>,
        encoding: &CStr,
        input: &[u8],
        range: ops::Range<usize>,
        reason: &CStr,
    ) -> PyResult<&'p PyUnicodeDecodeError> {
        unsafe {
            py.from_owned_ptr_or_err(ffi::PyUnicodeDecodeError_Create(
                encoding.as_ptr(),
                input.as_ptr() as *const c_char,
                input.len() as ffi::Py_ssize_t,
                range.start as ffi::Py_ssize_t,
                range.end as ffi::Py_ssize_t,
                reason.as_ptr(),
            ))
        }
    }

    /// Creates a Python `UnicodeDecodeError` from a Rust UTF-8 decoding error.
    ///
    /// # Examples
    ///
    /// ```
    /// use pyo3::prelude::*;
    /// use pyo3::exceptions::PyUnicodeDecodeError;
    ///
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let invalid_utf8 = b"fo\xd8o";
    ///     let err = std::str::from_utf8(invalid_utf8).expect_err("should be invalid utf8");
    ///     let decode_err = PyUnicodeDecodeError::new_utf8(py, invalid_utf8, err)?;
    ///     assert_eq!(
    ///         decode_err.to_string(),
    ///         "'utf-8' codec can't decode byte 0xd8 in position 2: invalid utf-8"
    ///     );
    ///     Ok(())
    /// })
    /// # }
    pub fn new_utf8<'p>(
        py: Python<'p>,
        input: &[u8],
        err: std::str::Utf8Error,
    ) -> PyResult<&'p PyUnicodeDecodeError> {
        let pos = err.valid_up_to();
        PyUnicodeDecodeError::new(
            py,
            CStr::from_bytes_with_nul(b"utf-8\0").unwrap(),
            input,
            pos..(pos + 1),
            CStr::from_bytes_with_nul(b"invalid utf-8\0").unwrap(),
        )
    }
}

impl_native_exception!(PyWarning, PyExc_Warning, native_doc!("Warning"));
impl_native_exception!(PyUserWarning, PyExc_UserWarning, native_doc!("UserWarning"));
impl_native_exception!(
    PyDeprecationWarning,
    PyExc_DeprecationWarning,
    native_doc!("DeprecationWarning")
);
impl_native_exception!(
    PyPendingDeprecationWarning,
    PyExc_PendingDeprecationWarning,
    native_doc!("PendingDeprecationWarning")
);
impl_native_exception!(
    PySyntaxWarning,
    PyExc_SyntaxWarning,
    native_doc!("SyntaxWarning")
);
impl_native_exception!(
    PyRuntimeWarning,
    PyExc_RuntimeWarning,
    native_doc!("RuntimeWarning")
);
impl_native_exception!(
    PyFutureWarning,
    PyExc_FutureWarning,
    native_doc!("FutureWarning")
);
impl_native_exception!(
    PyImportWarning,
    PyExc_ImportWarning,
    native_doc!("ImportWarning")
);
impl_native_exception!(
    PyUnicodeWarning,
    PyExc_UnicodeWarning,
    native_doc!("UnicodeWarning")
);
impl_native_exception!(
    PyBytesWarning,
    PyExc_BytesWarning,
    native_doc!("BytesWarning")
);
impl_native_exception!(
    PyResourceWarning,
    PyExc_ResourceWarning,
    native_doc!("ResourceWarning")
);

#[cfg(Py_3_10)]
impl_native_exception!(
    PyEncodingWarning,
    PyExc_EncodingWarning,
    native_doc!("EncodingWarning")
);

#[cfg(test)]
macro_rules! test_exception {
    ($exc_ty:ident $(, $constructor:expr)?) => {
        #[allow(non_snake_case)]
        #[test]
        fn $exc_ty () {
            use super::$exc_ty;

            $crate::Python::with_gil(|py| {
                use std::error::Error;
                let err: $crate::PyErr = {
                    None
                    $(
                        .or(Some($constructor(py)))
                    )?
                        .unwrap_or($exc_ty::new_err("a test exception"))
                };

                assert!(err.is_instance_of::<$exc_ty>(py));

                let value: &$exc_ty = err.value(py).downcast().unwrap();
                assert!(value.source().is_none());

                err.set_cause(py, Some($crate::exceptions::PyValueError::new_err("a cause")));
                assert!(value.source().is_some());

                assert!($crate::PyErr::from(value).is_instance_of::<$exc_ty>(py));
            })
        }
    };
}

/// Exceptions defined in Python's [`asyncio`](https://docs.python.org/3/library/asyncio.html)
/// module.
pub mod asyncio {
    import_exception!(asyncio, CancelledError);
    import_exception!(asyncio, InvalidStateError);
    import_exception!(asyncio, TimeoutError);
    import_exception!(asyncio, IncompleteReadError);
    import_exception!(asyncio, LimitOverrunError);
    import_exception!(asyncio, QueueEmpty);
    import_exception!(asyncio, QueueFull);

    #[cfg(test)]
    mod tests {
        test_exception!(CancelledError);
        test_exception!(InvalidStateError);
        test_exception!(TimeoutError);
        test_exception!(IncompleteReadError, |_| {
            IncompleteReadError::new_err(("partial", "expected"))
        });
        test_exception!(LimitOverrunError, |_| {
            LimitOverrunError::new_err(("message", "consumed"))
        });
        test_exception!(QueueEmpty);
        test_exception!(QueueFull);
    }
}

/// Exceptions defined in Python's [`socket`](https://docs.python.org/3/library/socket.html)
/// module.
pub mod socket {
    import_exception!(socket, herror);
    import_exception!(socket, gaierror);
    import_exception!(socket, timeout);

    #[cfg(test)]
    mod tests {
        test_exception!(herror);
        test_exception!(gaierror);
        test_exception!(timeout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{IntoPyDict, PyDict};
    use crate::{PyErr, Python};

    import_exception!(socket, gaierror);
    import_exception!(email.errors, MessageError);

    #[test]
    fn test_check_exception() {
        Python::with_gil(|py| {
            let err: PyErr = gaierror::new_err(());
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
        });
    }

    #[test]
    fn test_check_exception_nested() {
        Python::with_gil(|py| {
            let err: PyErr = MessageError::new_err(());
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
        });
    }

    #[test]
    fn custom_exception() {
        create_exception!(mymodule, CustomError, PyException);

        Python::with_gil(|py| {
            let error_type = py.get_type::<CustomError>();
            let ctx = [("CustomError", error_type)].into_py_dict(py);
            let type_description: String = py
                .eval("str(CustomError)", None, Some(ctx))
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(type_description, "<class 'mymodule.CustomError'>");
            py.run(
                "assert CustomError('oops').args == ('oops',)",
                None,
                Some(ctx),
            )
            .unwrap();
            py.run("assert CustomError.__doc__ is None", None, Some(ctx))
                .unwrap();
        });
    }

    #[test]
    fn custom_exception_dotted_module() {
        create_exception!(mymodule.exceptions, CustomError, PyException);
        Python::with_gil(|py| {
            let error_type = py.get_type::<CustomError>();
            let ctx = [("CustomError", error_type)].into_py_dict(py);
            let type_description: String = py
                .eval("str(CustomError)", None, Some(ctx))
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(
                type_description,
                "<class 'mymodule.exceptions.CustomError'>"
            );
        });
    }

    #[test]
    fn custom_exception_doc() {
        create_exception!(mymodule, CustomError, PyException, "Some docs");

        Python::with_gil(|py| {
            let error_type = py.get_type::<CustomError>();
            let ctx = [("CustomError", error_type)].into_py_dict(py);
            let type_description: String = py
                .eval("str(CustomError)", None, Some(ctx))
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(type_description, "<class 'mymodule.CustomError'>");
            py.run(
                "assert CustomError('oops').args == ('oops',)",
                None,
                Some(ctx),
            )
            .unwrap();
            py.run("assert CustomError.__doc__ == 'Some docs'", None, Some(ctx))
                .unwrap();
        });
    }

    #[test]
    fn custom_exception_doc_expr() {
        create_exception!(
            mymodule,
            CustomError,
            PyException,
            concat!("Some", " more ", stringify!(docs))
        );

        Python::with_gil(|py| {
            let error_type = py.get_type::<CustomError>();
            let ctx = [("CustomError", error_type)].into_py_dict(py);
            let type_description: String = py
                .eval("str(CustomError)", None, Some(ctx))
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(type_description, "<class 'mymodule.CustomError'>");
            py.run(
                "assert CustomError('oops').args == ('oops',)",
                None,
                Some(ctx),
            )
            .unwrap();
            py.run(
                "assert CustomError.__doc__ == 'Some more docs'",
                None,
                Some(ctx),
            )
            .unwrap();
        });
    }

    #[test]
    fn native_exception_debug() {
        Python::with_gil(|py| {
            let exc = py
                .run("raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error")
                .into_value(py)
                .into_ref(py);
            assert_eq!(
                format!("{:?}", exc),
                exc.repr().unwrap().extract::<String>().unwrap()
            );
        });
    }

    #[test]
    fn native_exception_display() {
        Python::with_gil(|py| {
            let exc = py
                .run("raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error")
                .into_value(py)
                .into_ref(py);
            assert_eq!(
                exc.to_string(),
                exc.str().unwrap().extract::<String>().unwrap()
            );
        });
    }

    #[test]
    fn native_exception_chain() {
        use std::error::Error;

        Python::with_gil(|py| {
            let exc = py
                .run(
                    "raise Exception('banana') from TypeError('peach')",
                    None,
                    None,
                )
                .expect_err("raising should have given us an error")
                .into_value(py)
                .into_ref(py);

            assert_eq!(format!("{:?}", exc), "Exception('banana')");

            let source = exc.source().expect("cause should exist");

            assert_eq!(format!("{:?}", source), "TypeError('peach')");

            let source_source = source.source();
            assert!(source_source.is_none(), "source_source should be None");
        });
    }

    #[test]
    fn unicode_decode_error() {
        let invalid_utf8 = b"fo\xd8o";
        let err = std::str::from_utf8(invalid_utf8).expect_err("should be invalid utf8");
        Python::with_gil(|py| {
            let decode_err = PyUnicodeDecodeError::new_utf8(py, invalid_utf8, err).unwrap();
            assert_eq!(
                format!("{:?}", decode_err),
                "UnicodeDecodeError('utf-8', b'fo\\xd8o', 2, 3, 'invalid utf-8')"
            );

            // Restoring should preserve the same error
            let e: PyErr = decode_err.into();
            e.restore(py);

            assert_eq!(
                PyErr::fetch(py).to_string(),
                "UnicodeDecodeError: \'utf-8\' codec can\'t decode byte 0xd8 in position 2: invalid utf-8"
            );
        });
    }

    test_exception!(PyBaseException);
    test_exception!(PyException);
    test_exception!(PyStopAsyncIteration);
    test_exception!(PyStopIteration);
    test_exception!(PyGeneratorExit);
    test_exception!(PyArithmeticError);
    test_exception!(PyLookupError);
    test_exception!(PyAssertionError);
    test_exception!(PyAttributeError);
    test_exception!(PyBufferError);
    test_exception!(PyEOFError);
    test_exception!(PyFloatingPointError);
    test_exception!(PyOSError);
    test_exception!(PyImportError);
    test_exception!(PyModuleNotFoundError);
    test_exception!(PyIndexError);
    test_exception!(PyKeyError);
    test_exception!(PyKeyboardInterrupt);
    test_exception!(PyMemoryError);
    test_exception!(PyNameError);
    test_exception!(PyOverflowError);
    test_exception!(PyRuntimeError);
    test_exception!(PyRecursionError);
    test_exception!(PyNotImplementedError);
    test_exception!(PySyntaxError);
    test_exception!(PyReferenceError);
    test_exception!(PySystemError);
    test_exception!(PySystemExit);
    test_exception!(PyTypeError);
    test_exception!(PyUnboundLocalError);
    test_exception!(PyUnicodeError);
    test_exception!(PyUnicodeDecodeError, |py| {
        let invalid_utf8 = b"fo\xd8o";
        let err = std::str::from_utf8(invalid_utf8).expect_err("should be invalid utf8");
        PyErr::from_value(PyUnicodeDecodeError::new_utf8(py, invalid_utf8, err).unwrap())
    });
    test_exception!(PyUnicodeEncodeError, |py: Python<'_>| {
        py.eval("chr(40960).encode('ascii')", None, None)
            .unwrap_err()
    });
    test_exception!(PyUnicodeTranslateError, |_| {
        PyUnicodeTranslateError::new_err(("\u{3042}", 0, 1, "ouch"))
    });
    test_exception!(PyValueError);
    test_exception!(PyZeroDivisionError);
    test_exception!(PyBlockingIOError);
    test_exception!(PyBrokenPipeError);
    test_exception!(PyChildProcessError);
    test_exception!(PyConnectionError);
    test_exception!(PyConnectionAbortedError);
    test_exception!(PyConnectionRefusedError);
    test_exception!(PyConnectionResetError);
    test_exception!(PyFileExistsError);
    test_exception!(PyFileNotFoundError);
    test_exception!(PyInterruptedError);
    test_exception!(PyIsADirectoryError);
    test_exception!(PyNotADirectoryError);
    test_exception!(PyPermissionError);
    test_exception!(PyProcessLookupError);
    test_exception!(PyTimeoutError);
    test_exception!(PyEnvironmentError);
    test_exception!(PyIOError);
    #[cfg(windows)]
    test_exception!(PyWindowsError);

    test_exception!(PyWarning);
    test_exception!(PyUserWarning);
    test_exception!(PyDeprecationWarning);
    test_exception!(PyPendingDeprecationWarning);
    test_exception!(PySyntaxWarning);
    test_exception!(PyRuntimeWarning);
    test_exception!(PyFutureWarning);
    test_exception!(PyImportWarning);
    test_exception!(PyUnicodeWarning);
    test_exception!(PyBytesWarning);
    #[cfg(Py_3_10)]
    test_exception!(PyEncodingWarning);
}
