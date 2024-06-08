/// A convenient macro to execute a Python code snippet, with some local variables set.
///
/// # Panics
///
/// This macro internally calls [`Python::run_bound`](crate::Python::run_bound) and panics
/// if it returns `Err`, after printing the error to stdout.
///
/// If you need to handle failures, please use [`Python::run_bound`](crate::marker::Python::run_bound) instead.
///
/// # Examples
/// ```
/// use pyo3::{prelude::*, py_run, types::PyList};
///
/// Python::with_gil(|py| {
///     let list = PyList::new_bound(py, &[1, 2, 3]);
///     py_run!(py, list, "assert list == [1, 2, 3]");
/// });
/// ```
///
/// You can use this macro to test pyfunctions or pyclasses quickly.
///
/// ```
/// use pyo3::{prelude::*, py_run};
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
///     let time = Py::new(py, Time {hour: 8, minute: 43, second: 16}).unwrap();
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
///     let locals = [("C", py.get_type_bound::<MyClass>())].into_py_dict_bound(py);
///     pyo3::py_run!(py, *locals, "c = C()");
/// });
/// ```
#[macro_export]
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
macro_rules! py_run_impl {
    ($py:expr, $($val:ident)+, $code:expr) => {{
        use $crate::types::IntoPyDict;
        use $crate::ToPyObject;
        let d = [$((stringify!($val), $val.to_object($py)),)+].into_py_dict_bound($py);
        $crate::py_run_impl!($py, *d, $code)
    }};
    ($py:expr, *$dict:expr, $code:expr) => {{
        use ::std::option::Option::*;
        #[allow(unused_imports)]
        #[cfg(feature = "gil-refs")]
        use $crate::PyNativeType;
        if let ::std::result::Result::Err(e) = $py.run_bound($code, None, Some(&$dict.as_borrowed())) {
            e.print($py);
            // So when this c api function the last line called printed the error to stderr,
            // the output is only written into a buffer which is never flushed because we
            // panic before flushing. This is where this hack comes into place
            $py.run_bound("import sys; sys.stderr.flush()", None, None)
                .unwrap();
            ::std::panic!("{}", $code)
        }
    }};
}

/// Wraps a Rust function annotated with [`#[pyfunction]`](macro@crate::pyfunction).
///
/// This can be used with [`PyModule::add_function`](crate::types::PyModuleMethods::add_function) to
/// add free functions to a [`PyModule`](crate::types::PyModule) - see its documentation for more
/// information.
///
/// During the migration from the GIL Ref API to the Bound API, the return type of this macro will
/// be either the `&'py PyModule` GIL Ref or `Bound<'py, PyModule>` according to the second
/// argument.
///
/// For backwards compatibility, if the second argument is `Python<'py>` then the return type will
/// be `&'py PyModule` GIL Ref. To get `Bound<'py, PyModule>`, use the [`crate::wrap_pyfunction_bound!`]
/// macro instead.
#[macro_export]
macro_rules! wrap_pyfunction {
    ($function:path) => {
        &|py_or_module| {
            use $function as wrapped_pyfunction;
            $crate::impl_::pyfunction::WrapPyFunctionArg::wrap_pyfunction(
                py_or_module,
                &wrapped_pyfunction::_PYO3_DEF,
            )
        }
    };
    ($function:path, $py_or_module:expr) => {{
        use $function as wrapped_pyfunction;
        let check_gil_refs = $crate::impl_::deprecations::GilRefs::new();
        let py_or_module =
            $crate::impl_::deprecations::inspect_type($py_or_module, &check_gil_refs);
        check_gil_refs.is_python();
        $crate::impl_::pyfunction::WrapPyFunctionArg::wrap_pyfunction(
            py_or_module,
            &wrapped_pyfunction::_PYO3_DEF,
        )
    }};
}

/// Wraps a Rust function annotated with [`#[pyfunction]`](macro@crate::pyfunction).
///
/// This can be used with [`PyModule::add_function`](crate::types::PyModuleMethods::add_function) to
/// add free functions to a [`PyModule`](crate::types::PyModule) - see its documentation for more
/// information.
#[macro_export]
macro_rules! wrap_pyfunction_bound {
    ($function:path) => {
        &|py_or_module| {
            use $function as wrapped_pyfunction;
            $crate::impl_::pyfunction::WrapPyFunctionArg::wrap_pyfunction(
                $crate::impl_::pyfunction::OnlyBound(py_or_module),
                &wrapped_pyfunction::_PYO3_DEF,
            )
        }
    };
    ($function:path, $py_or_module:expr) => {{
        use $function as wrapped_pyfunction;
        $crate::impl_::pyfunction::WrapPyFunctionArg::wrap_pyfunction(
            $crate::impl_::pyfunction::OnlyBound($py_or_module),
            &wrapped_pyfunction::_PYO3_DEF,
        )
    }};
}

/// Returns a function that takes a [`Python`](crate::Python) instance and returns a
/// Python module.
///
/// Use this together with [`#[pymodule]`](crate::pymodule) and
/// [`PyModule::add_wrapped`](crate::types::PyModuleMethods::add_wrapped).
#[macro_export]
macro_rules! wrap_pymodule {
    ($module:path) => {
        &|py| {
            use $module as wrapped_pymodule;
            wrapped_pymodule::_PYO3_DEF
                .make_module(py)
                .expect("failed to wrap pymodule")
        }
    };
}

/// Add the module to the initialization table in order to make embedded Python code to use it.
/// Module name is the argument.
///
/// Use it before [`prepare_freethreaded_python`](crate::prepare_freethreaded_python) and
/// leave feature `auto-initialize` off
#[cfg(not(any(PyPy, GraalPy)))]
#[macro_export]
macro_rules! append_to_inittab {
    ($module:ident) => {
        unsafe {
            if $crate::ffi::Py_IsInitialized() != 0 {
                ::std::panic!(
                    "called `append_to_inittab` but a Python interpreter is already running."
                );
            }
            $crate::ffi::PyImport_AppendInittab(
                $module::__PYO3_NAME.as_ptr().cast(),
                ::std::option::Option::Some($module::__pyo3_init),
            );
        }
    };
}
