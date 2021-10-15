/// Wraps a Rust function annotated with [`#[pyfunction]`](crate::pyfunction), turning it into a
/// [`PyCFunction`](crate::types::PyCFunction).
///
/// This can be used with [`PyModule::add_function`](crate::types::PyModule::add_function) to add
/// free functions to a [`PyModule`](crate::types::PyModule) - see its documention for more information.
#[macro_export]
macro_rules! wrap_pyfunction {
    ($function_name: ident) => {{
        &|py| $crate::paste::expr! { [<__pyo3_get_function_ $function_name>] }(py)
    }};

    ($function_name: ident, $arg: expr) => {
        $crate::wrap_pyfunction!($function_name)(
            <$crate::derive_utils::PyFunctionArguments as ::std::convert::From<_>>::from($arg),
        )
    };
}

/// Returns a function that takes a [`Python`](crate::python::Python) instance and returns a Python module.
///
/// Use this together with [`#[pymodule]`](crate::pymodule) and [crate::types::PyModule::add_wrapped].
#[macro_export]
macro_rules! wrap_pymodule {
    ($module_name:ident) => {{
        $crate::paste::expr! {
            &|py| unsafe { $crate::PyObject::from_owned_ptr(py, [<PyInit_ $module_name>]()) }
        }
    }};
}

/// A convenient macro to execute a Python code snippet, with some local variables set.
///
/// # Panics
///
/// This macro internally calls [`Python::run`](crate::Python::run) and panics
/// if it returns `Err`, after printing the error to stdout.
///
/// If you need to handle failures, please use [`Python::run`](crate::python::Python::run) instead.
///
/// # Examples
/// ```
/// use pyo3::{prelude::*, py_run, types::PyList};
///
/// Python::with_gil(|py| {
///     let list = PyList::new(py, &[1, 2, 3]);
///     py_run!(py, list, "assert list == [1, 2, 3]");
/// });
/// ```
///
/// You can use this macro to test pyfunctions or pyclasses quickly.
///
/// ```
/// use pyo3::{prelude::*, py_run, PyCell};
///
/// #[pyclass]
/// #[derive(Debug)]
/// struct Time {
///     hour: u32,
///     minute: u32,
///     second: u32,
/// }
///
/// #[pymethods]
/// impl Time {
///     fn repl_japanese(&self) -> String {
///         format!("{}時{}分{}秒", self.hour, self.minute, self.second)
///     }
///     #[getter]
///     fn hour(&self) -> u32 {
///         self.hour
///     }
///     fn as_tuple(&self) -> (u32, u32, u32) {
///         (self.hour, self.minute, self.second)
///     }
/// }
///
/// Python::with_gil(|py| {
///     let time = PyCell::new(py, Time {hour: 8, minute: 43, second: 16}).unwrap();
///     let time_as_tuple = (8, 43, 16);
///     py_run!(py, time time_as_tuple, r#"
///         assert time.hour == 8
///         assert time.repl_japanese() == "8時43分16秒"
///         assert time.as_tuple() == time_as_tuple
///     "#);
/// });
/// ```
///
/// If you need to prepare the `locals` dict by yourself, you can pass it as `*locals`.
///
/// ```
/// use pyo3::prelude::*;
/// use pyo3::types::IntoPyDict;
///
/// #[pyclass]
/// struct MyClass;
///
/// #[pymethods]
/// impl MyClass {
///     #[new]
///     fn new() -> Self {
///         MyClass {}
///     }
/// }
///
/// Python::with_gil(|py| {
///     let locals = [("C", py.get_type::<MyClass>())].into_py_dict(py);
///     pyo3::py_run!(py, *locals, "c = C()");
/// });
/// ```
#[macro_export]
#[cfg(feature = "macros")]
macro_rules! py_run {
    ($py:expr, $($val:ident)+, $code:literal) => {{
        $crate::py_run_impl!($py, $($val)+, $crate::indoc::indoc!($code))
    }};
    ($py:expr, $($val:ident)+, $code:expr) => {{
        $crate::py_run_impl!($py, $($val)+, &$crate::unindent::unindent($code))
    }};
    ($py:expr, *$dict:expr, $code:literal) => {{
        $crate::py_run_impl!($py, *$dict, $crate::indoc::indoc!($code))
    }};
    ($py:expr, *$dict:expr, $code:expr) => {{
        $crate::py_run_impl!($py, *$dict, &$crate::unindent::unindent($code))
    }};
}

#[macro_export]
#[doc(hidden)]
#[cfg(feature = "macros")]
macro_rules! py_run_impl {
    ($py:expr, $($val:ident)+, $code:expr) => {{
        use $crate::types::IntoPyDict;
        use $crate::ToPyObject;
        let d = [$((stringify!($val), $val.to_object($py)),)+].into_py_dict($py);
        $crate::py_run_impl!($py, *d, $code)
    }};
    ($py:expr, *$dict:expr, $code:expr) => {{
        use ::std::option::Option::*;
        if let ::std::result::Result::Err(e) = $py.run($code, None, Some($dict)) {
            e.print($py);
            // So when this c api function the last line called printed the error to stderr,
            // the output is only written into a buffer which is never flushed because we
            // panic before flushing. This is where this hack comes into place
            $py.run("import sys; sys.stderr.flush()", None, None)
                .unwrap();
            ::std::panic!("{}", $code)
        }
    }};
}
