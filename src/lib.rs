#![cfg_attr(feature = "nightly", feature(specialization))]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Rust bindings to the Python interpreter.
//!
//! PyO3 can be used to write native Python modules or run Python code and modules from Rust.
//!
//! See [the guide](https://pyo3.rs/) for a detailed introduction.
//!
//! # PyO3's object types
//!
//! PyO3 has several core types that you should familiarize yourself with:
//!
//! ## The Python<'py> object
//!
//! Holding the [global interpreter lock](https://docs.python.org/3/glossary.html#term-global-interpreter-lock)
//! (GIL) is modeled with the [`Python<'py>`](crate::Python) token.
//! All APIs that require that the GIL is held require this token as proof
//! that you really are holding the GIL. It can be explicitly acquired and
//! is also implicitly acquired by PyO3 as it wraps Rust functions and structs
//! into Python functions and objects.
//!
//! ## The GIL-dependent types
//!
//! For example `&`[`PyAny`](crate::types::PyAny).
//! These are only ever seen as references, with a lifetime that is only valid for as long
//! as the GIL is held, which is why using them doesn't require a  [`Python<'py>`](crate::Python) token.
//!  The underlying Python object, if mutable, can be mutated through any reference.
//!
//! See the [guide](https://pyo3.rs/latest/types.html) for an explanation of the different Python object types.
//!
//! ## The GIL-independent types
//!
//! When wrapped in [`Py`]`<...>`, like with [`Py`]`<`[`PyAny`](crate::types::PyAny)`>` or [`Py`]`<SomePyClass>`, Python objects
//! no longer have a limited lifetime which makes them easier to store in structs and pass between functions.
//! However, you cannot do much with them without a
//! [`Python<'py>`](crate::Python) token, for which you’d need to reacquire the GIL.
//!
//! ## PyErr
//!
//! The vast majority of operations in this library will return [`PyResult<...>`](PyResult).
//! This is an alias for the type `Result<..., PyErr>`.
//!
//! A `PyErr` represents a Python exception. A `PyErr` returned to Python code will be raised as a Python exception.
//! Errors from `PyO3` itself are also exposed as Python exceptions.
//!
//! # Feature flags
//!
//! PyO3 uses [feature flags](https://doc.rust-lang.org/cargo/reference/features.html)
//! to enable you to opt-in to additional functionality. For a detailed description, see
//! the [Features Reference chapter of the guide](https://pyo3.rs/latest/features.html#features-reference).
//!
//! ## Default feature flags
//!
//! The following features are turned on by default:
//! - `macros`: Enables various macros, including all the attribute macros.
//!
//! ## Optional feature flags
//!
//! The following features are optional:
//! - `abi3`: Restricts PyO3's API to a subset of the full Python API which is guaranteed
//! by [PEP 384](https://www.python.org/dev/peps/pep-0384/) to be forward-compatible with future Python versions.
//
//! - `auto-initialize`: Changes [`Python::with_gil`](crate::Python::with_gil) and
//! [`Python::acquire_gil`](crate::Python::acquire_gil) to automatically initialize the
//! Python interpreter if needed.
//
//! - `extension-module`: This will tell the linker to keep the Python symbols unresolved,
//! so that your module can also be used with statically linked Python interpreters.
//! Use this feature when building an extension module.
//
//! - `hashbrown`: Enables conversions between Python objects and
//! [hashbrown](https://docs.rs/hashbrown)'s
//! [`HashMap`](https://docs.rs/hashbrown/latest/hashbrown/struct.HashMap.html) and
//! [`HashSet`](https://docs.rs/hashbrown/latest/hashbrown/struct.HashSet.html) types.
//
//! - [`indexmap`](crate::indexmap): Enables conversions between Python dictionary and
//! [indexmap](https://docs.rs/indexmap)'s
//! [`IndexMap`](https://docs.rs/indexmap/latest/indexmap/map/struct.IndexMap.html).
//
//! - `multiple-pymethods`: Enables the use of multiple
//! [`#[pymethods]`](crate::proc_macro::pymethods) blocks per
//! [`#[pyclass]`](crate::proc_macro::pyclass). This adds a dependency on the
//! [`inventory`](https://docs.rs/inventory) crate, which is not supported on all platforms.
//
//! - [`num-bigint`](./num_bigint/index.html): Enables conversions between Python objects and
//! [num-bigint](https://docs.rs/num-bigint)'s
//! [`BigInt`](https://docs.rs/num-bigint/latest/num_bigint/struct.BigInt.html) and
//! [`BigUint`](https://docs.rs/num-bigint/latest/num_bigint/struct.BigUint.html) types.
//
//! - [`num-complex`](crate::num_complex): Enables conversions between Python objects and
//! [num-complex](https://docs.rs/num-complex)'s
//! [`Complex`](https://docs.rs/num-complex/latest/num_complex/struct.Complex.html) type.
//
//! - `serde`: Allows implementing [serde](https://docs.rs/serde)'s
//! [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) and
//! [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) traits for
//! [`Py`]`<T>` for all `T` that implement
//! [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) and
//! [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html).
//!
//! ## Unstable features
//!
//! - `nightly`: Gates some optimizations that rely on
//!  [`#![feature(specialization)]`](https://github.com/rust-lang/rfcs/blob/master/text/1210-impl-specialization.md),
//! for which you'd also need nightly Rust. You should not use this feature.
//
//! ## `rustc` environment flags
//!
//! PyO3 uses `rustc`'s `--cfg` flags to enable or disable code used for different Python versions.
//! If you want to do this for your own crate, you can do so with the [`pyo3-build-config`](https://docs.rs/pyo3-build-config) crate.
//!
//! - `Py_3_6`, `Py_3_7`, `Py_3_8`, `Py_3_9`, `Py_3_10`: Marks code that is only enabled when compiling for a given minimum Python version.
//
//! - `Py_LIMITED_API`: Marks code enabled when the `abi3` feature flag is enabled.
//
//! - `PyPy` - Marks code enabled when compiling for PyPy.
//!
//! # Minimum supported Rust and Python versions
//!
//! PyO3 supports Python 3.6+ and Rust 1.41+.
//!
//! Building with PyPy is also possible (via cpyext) for Python 3.6,
//! targeted PyPy version is 7.3+. Please refer to the
//! [pypy section](https://pyo3.rs/latest/building_and_distribution/pypy.html)
//! in the guide for more information.
//!
//! # Example: Building a native Python module
//!
//! To build, test and publish your crate as a Python module, it is recommended that you use
//! [maturin](https://github.com/PyO3/maturin) or
//! [setuptools-rust](https://github.com/PyO3/setuptools-rust). You can also do this manually. See the
//! [Building and Distribution chapter of the guide](https://pyo3.rs/latest/building_and_distribution.html)
//! for more information.
//!
//! Add these files to your crate's root:
//!
//! **`Cargo.toml`**
//!
//! ```toml
//! [package]
//! name = "string-sum"
//! version = "0.1.0"
//! edition = "2018"
//!
//! [lib]
//! name = "string_sum"
//! # "cdylib" is necessary to produce a shared library for Python to import from.
//! #
//! # Downstream Rust code (including code in `bin/`, `examples/`, and `tests/`) will not be able
//! # to `use string_sum;` unless the "rlib" or "lib" crate type is also included, e.g.:
//! # crate-type = ["cdylib", "rlib"]
//! crate-type = ["cdylib"]
//!
//! [dependencies.pyo3]
// workaround for `extended_key_value_attributes`: https://github.com/rust-lang/rust/issues/82768#issuecomment-803935643
#![cfg_attr(docsrs, cfg_attr(docsrs, doc = concat!("version = \"", env!("CARGO_PKG_VERSION"),  "\"")))]
#![cfg_attr(not(docsrs), doc = "version = \"*\"")]
//! features = ["extension-module"]
//! ```
//!
//! **`src/lib.rs`**
//!
//! ```rust
//! use pyo3::prelude::*;
//!
//! /// Formats the sum of two numbers as string.
//! #[pyfunction]
//! fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//!     Ok((a + b).to_string())
//! }
//!
//! /// A Python module implemented in Rust.
//! #[pymodule]
//! fn string_sum(py: Python, m: &PyModule) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! **`.cargo/config.toml`**
//! ```toml
//! # These flags must be passed to rustc when compiling for macOS
//! # They can be omitted if you pass the flags yourself
//! # or don't care about macOS
//!
//! [target.x86_64-apple-darwin]
//! rustflags = [
//!   "-C", "link-arg=-undefined",
//!   "-C", "link-arg=dynamic_lookup",
//! ]
//!
//! [target.aarch64-apple-darwin]
//! rustflags = [
//!   "-C", "link-arg=-undefined",
//!   "-C", "link-arg=dynamic_lookup",
//! ]
//! ```
//!
//! # Example: Using Python from Rust
//!
//! You can use PyO3 to call Python functions from Rust.
//!
//! Add `pyo3` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies.pyo3]
// workaround for `extended_key_value_attributes`: https://github.com/rust-lang/rust/issues/82768#issuecomment-803935643
#![cfg_attr(docsrs, cfg_attr(docsrs, doc = concat!("version = \"", env!("CARGO_PKG_VERSION"),  "\"")))]
#![cfg_attr(not(docsrs), doc = "version = \"*\"")]
//! # this is necessary to automatically initialize the Python interpreter
//! features = ["auto-initialize"]
//! ```
//!
//! Example program displaying the value of `sys.version`:
//!
//! ```rust
//! use pyo3::prelude::*;
//! use pyo3::types::IntoPyDict;
//!
//! fn main() -> PyResult<()> {
//!     let gil = Python::acquire_gil();
//!     let py = gil.python();
//!     let sys = py.import("sys")?;
//!     let version: String = sys.get("version")?.extract()?;
//!
//!     let locals = [("os", py.import("os")?)].into_py_dict(py);
//!     let code = "os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";
//!     let user: String = py.eval(code, None, Some(&locals))?.extract()?;
//!
//!     println!("Hello {}, I'm Python {}", user, version);
//!     Ok(())
//! }
//! ```

pub use crate::class::*;
pub use crate::conversion::{
    AsPyPointer, FromPyObject, FromPyPointer, IntoPy, IntoPyPointer, PyTryFrom, PyTryInto,
    ToBorrowedObject, ToPyObject,
};
pub use crate::err::{PyDowncastError, PyErr, PyErrArguments, PyResult};
#[cfg(not(PyPy))]
#[cfg_attr(docsrs, doc(cfg(not(PyPy))))]
pub use crate::gil::{prepare_freethreaded_python, with_embedded_python_interpreter};
pub use crate::gil::{GILGuard, GILPool};
pub use crate::instance::{Py, PyNativeType, PyObject};
pub use crate::pycell::{PyCell, PyRef, PyRefMut};
pub use crate::pyclass::PyClass;
pub use crate::pyclass_init::PyClassInitializer;
pub use crate::python::{Python, PythonVersionInfo};
pub use crate::type_object::PyTypeInfo;
// Since PyAny is as important as PyObject, we expose it to the top level.
pub use crate::types::PyAny;

#[cfg(feature = "macros")]
#[doc(hidden)]
pub use {
    indoc,    // Re-exported for py_run
    paste,    // Re-exported for wrap_function
    unindent, // Re-exported for py_run
};

#[cfg(all(feature = "macros", feature = "multiple-pymethods"))]
pub use inventory; // Re-exported for `#[pyclass]` and `#[pymethods]` with `multiple-pymethods`.

#[macro_use]
mod internal_tricks;

// The CPython stable ABI does not include PyBuffer.
#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(docsrs, doc(cfg(not(Py_LIMITED_API))))]
pub mod buffer;

#[doc(hidden)]
pub mod callback;
pub mod class;
pub mod conversion;
mod conversions;
#[macro_use]
#[doc(hidden)]
pub mod derive_utils;
mod err;
pub mod exceptions;
pub mod ffi;
mod gil;
pub mod impl_;
mod instance;

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(docsrs, doc(cfg(not(Py_LIMITED_API))))]
pub mod marshal;

pub mod once_cell;
pub mod panic;
pub mod prelude;
pub mod pycell;
pub mod pyclass;
pub mod pyclass_init;
pub mod pyclass_slots;
mod python;

pub mod type_object;
pub mod types;

pub mod num_bigint;

pub mod num_complex;

#[cfg_attr(docsrs, doc(cfg(feature = "indexmap")))]
#[cfg(feature = "indexmap")]
pub use crate::conversions::indexmap;

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
pub mod serde;

/// The proc macros, all of which are part of the prelude.
///
/// Import these with `use pyo3::prelude::*;`
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[cfg(feature = "macros")]
pub mod proc_macro {
    pub use pyo3_macros::pymodule;

    pub use pyo3_macros::{pyfunction, pyproto};

    #[cfg(not(feature = "multiple-pymethods"))]
    pub use pyo3_macros::{pyclass, pymethods};

    #[cfg(feature = "multiple-pymethods")]
    pub use pyo3_macros::{
        pyclass_with_inventory as pyclass, pymethods_with_inventory as pymethods,
    };
}

/// Returns a function that takes a [Python] instance and returns a Python function.
///
/// Use this together with `#[pyfunction]` and [types::PyModule::add_wrapped].
#[macro_export]
macro_rules! wrap_pyfunction {
    ($function_name: ident) => {{
        &|py| pyo3::paste::expr! { [<__pyo3_get_function_ $function_name>] }(py)
    }};

    ($function_name: ident, $arg: expr) => {
        pyo3::wrap_pyfunction!($function_name)(pyo3::derive_utils::PyFunctionArguments::from($arg))
    };
}

/// Returns a function that takes a [Python] instance and returns a Python module.
///
/// Use this together with `#[pymodule]` and [types::PyModule::add_wrapped].
#[macro_export]
macro_rules! wrap_pymodule {
    ($module_name:ident) => {{
        pyo3::paste::expr! {
            &|py| unsafe { pyo3::PyObject::from_owned_ptr(py, [<PyInit_ $module_name>]()) }
        }
    }};
}

/// A convenient macro to execute a Python code snippet, with some local variables set.
///
/// # Panics
/// This macro internally calls [`Python::run`](struct.Python.html#method.run) and panics
/// if it returns `Err`, after printing the error to stdout.
///
/// # Examples
/// ```
/// use pyo3::{prelude::*, py_run, types::PyList};
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
/// #[pyclass]
/// #[derive(Debug)]
/// struct Time {
///     hour: u32,
///     minute: u32,
///     second: u32,
/// }
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
/// #[pyclass]
/// struct MyClass {}
/// #[pymethods]
/// impl MyClass {
///     #[new]
///     fn new() -> Self { MyClass {} }
/// }
/// Python::with_gil(|py| {
///    let locals = [("C", py.get_type::<MyClass>())].into_py_dict(py);
///    pyo3::py_run!(py, *locals, "c = C()");
/// });
/// ```
///
/// **Note**
/// Since this macro is intended to use for testing, it **causes panic** when
/// [Python::run] returns `Err` internally.
/// If you need to handle failures, please use [Python::run] directly.
///
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
        if let Err(e) = $py.run($code, None, Some($dict)) {
            e.print($py);
            // So when this c api function the last line called printed the error to stderr,
            // the output is only written into a buffer which is never flushed because we
            // panic before flushing. This is where this hack comes into place
            $py.run("import sys; sys.stderr.flush()", None, None)
                .unwrap();
            panic!("{}", $code)
        }
    }};
}

/// Test readme and user guide
#[cfg(doctest)]
pub mod doc_test {
    macro_rules! doctest_impl {
        ($doc:expr, $mod:ident) => {
            #[doc = $doc]
            mod $mod {}
        };
    }

    macro_rules! doctest {
        ($path:expr, $mod:ident) => {
            doctest_impl!(include_str!(concat!("../", $path)), $mod);
        };
    }

    doctest!("README.md", readme_md);
    doctest!("guide/src/advanced.md", guide_advanced_md);
    doctest!(
        "guide/src/building_and_distribution.md",
        guide_building_and_distribution_md
    );
    doctest!(
        "guide/src/building_and_distribution/pypy.md",
        guide_building_and_distribution_pypy_md
    );
    doctest!("guide/src/class.md", guide_class_md);
    doctest!("guide/src/class/protocols.md", guide_class_protocols_md);
    doctest!("guide/src/conversions.md", guide_conversions_md);
    doctest!(
        "guide/src/conversions/tables.md",
        guide_conversions_tables_md
    );
    doctest!(
        "guide/src/conversions/traits.md",
        guide_conversions_traits_md
    );
    doctest!("guide/src/debugging.md", guide_debugging_md);
    doctest!("guide/src/exception.md", guide_exception_md);
    doctest!("guide/src/function.md", guide_function_md);
    doctest!("guide/src/migration.md", guide_migration_md);
    doctest!("guide/src/module.md", guide_module_md);
    doctest!("guide/src/parallelism.md", guide_parallelism_md);
    doctest!("guide/src/python_from_rust.md", guide_python_from_rust_md);
    doctest!("guide/src/rust_cpython.md", guide_rust_cpython_md);
    doctest!("guide/src/trait_bounds.md", guide_trait_bounds_md);
    doctest!("guide/src/types.md", guide_types_md);
    doctest!("guide/src/faq.md", faq);
}
