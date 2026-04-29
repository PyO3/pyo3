use crate::types::{PyAny, PyType};
use crate::{Bound, PyResult, Python};

use super::construct::build_native_enum;
use super::spec::NativeEnumSpec;

/// A Rust enum that can be exposed to Python as a `enum.Enum` subclass.
///
/// Implement this trait (or derive it with `#[derive(NativeEnum)]`) to enable conversion
/// between a Rust enum and its Python counterpart.
///
/// # Class caching
///
/// `#[derive(NativeEnum)]` generates a `py_enum_class` override that stores the Python
/// class in a `PyOnceLock`, constructing it only once per interpreter session.
///
/// The **default** `py_enum_class` provided by this trait does **not** cache — it calls
/// [`build_native_enum`] on every invocation, which happens inside `to_py_member` and
/// `from_py_member`. If you implement this trait manually, override `py_enum_class` with
/// a cached version to avoid reconstructing the Python class on every conversion:
///
/// ```rust,ignore
/// use pyo3::native_enum::{build_native_enum, NativeEnum, NativeEnumSpec};
/// use pyo3::sync::PyOnceLock;
/// use pyo3::types::PyType;
/// use pyo3::{Bound, Py, PyResult, Python};
///
/// static MY_ENUM_CLASS: PyOnceLock<Py<PyType>> = PyOnceLock::new();
///
/// impl NativeEnum for MyEnum {
///     const SPEC: NativeEnumSpec = /* ... */;
///
///     fn py_enum_class(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
///         MY_ENUM_CLASS
///             .get_or_try_init(py, || {
///                 build_native_enum(py, &Self::SPEC).map(|cls| cls.unbind())
///             })
///             .map(|cls| cls.clone_ref(py).into_bound(py))
///     }
///
///     // implement to_py_member and from_py_member ...
/// }
/// ```
///
/// [`build_native_enum`]: super::build_native_enum
pub trait NativeEnum: Sized + 'static {
    /// Static specification describing how to build the Python class.
    const SPEC: NativeEnumSpec;

    /// Builds and returns the Python `enum` subclass for this type.
    ///
    /// **Uncached by default** — override with a `PyOnceLock`-based implementation when
    /// implementing the trait manually. See the [trait-level docs](NativeEnum) for an example.
    fn py_enum_class(py: Python<'_>) -> PyResult<Bound<'_, PyType>> {
        build_native_enum(py, &Self::SPEC)
    }

    /// Converts `self` into the corresponding Python enum member.
    fn to_py_member<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>>;

    /// Extracts `Self` from a Python enum member.
    fn from_py_member(obj: &Bound<'_, PyAny>) -> PyResult<Self>;
}
