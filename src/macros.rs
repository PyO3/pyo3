/// A convenient macro to execute a Python code snippet, with some local variables set.
///
/// # Panics
///
/// This macro internally calls [`Python::run`](crate::Python::run) and panics
/// if it returns `Err`, after printing the error to stdout.
///
/// If you need to handle failures, please use [`Python::run`](crate::marker::Python::run) instead.
///
/// # Examples
/// ```
/// use pyo3::{prelude::*, py_run, types::PyList};
///
/// # fn main() -> PyResult<()> {
/// Python::attach(|py| {
///     let list = PyList::new(py, &[1, 2, 3])?;
///     py_run!(py, list, "assert list == [1, 2, 3]");
/// # Ok(())
/// })
/// # }
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
/// Python::attach(|py| {
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
/// # fn main() -> PyResult<()> {
/// Python::attach(|py| {
///     let locals = [("C", py.get_type::<MyClass>())].into_py_dict(py)?;
///     pyo3::py_run!(py, *locals, "c = C()");
/// #   Ok(())
/// })
/// # }
/// ```
#[macro_export]
macro_rules! py_run {
    ($py:expr, $($val:ident)+, $code:literal) => {{
        $crate::py_run_impl!($py, $($val)+, $crate::indoc::indoc!($code))
    }};
    ($py:expr, $($val:ident)+, $code:expr) => {{
        $crate::py_run_impl!($py, $($val)+, $crate::unindent::unindent($code))
    }};
    ($py:expr, *$dict:expr, $code:literal) => {{
        $crate::py_run_impl!($py, *$dict, $crate::indoc::indoc!($code))
    }};
    ($py:expr, *$dict:expr, $code:expr) => {{
        $crate::py_run_impl!($py, *$dict, $crate::unindent::unindent($code))
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_run_impl {
    ($py:expr, $($val:ident)+, $code:expr) => {{
        use $crate::types::IntoPyDict;
        use $crate::conversion::IntoPyObject;
        use $crate::BoundObject;
        let d = [$((stringify!($val), (&$val).into_pyobject($py).unwrap().into_any().into_bound()),)+].into_py_dict($py).unwrap();
        $crate::py_run_impl!($py, *d, $code)
    }};
    ($py:expr, *$dict:expr, $code:expr) => {{
        use ::std::option::Option::*;
        #[allow(unused_imports)]
        if let ::std::result::Result::Err(e) = $py.run(&::std::ffi::CString::new($code).unwrap(), None, Some(&$dict)) {
            e.print($py);
            // So when this c api function the last line called printed the error to stderr,
            // the output is only written into a buffer which is never flushed because we
            // panic before flushing. This is where this hack comes into place
            $py.run($crate::ffi::c_str!("import sys; sys.stderr.flush()"), None, None)
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
        $crate::impl_::pyfunction::WrapPyFunctionArg::wrap_pyfunction(
            $py_or_module,
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
                .make_module(py, wrapped_pymodule::__PYO3_GIL_USED)
                .expect("failed to wrap pymodule")
        }
    };
}

/// Add the module to the initialization table in order to make embedded Python code to use it.
/// Module name is the argument.
///
/// Use it before [`Python::initialize`](crate::marker::Python::initialize) and
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
                $module::__PYO3_NAME.as_ptr(),
                ::std::option::Option::Some($module::__pyo3_init),
            );
        }
    };
}
