use crate::exceptions::PyImportError;
use crate::sync::PyOnceLock;
use crate::types::{PyAny, PyType};
use crate::{Bound, Py, PyResult, Python};

use super::spec::NativeEnumBase;

static ENUM_BASE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static INT_ENUM_BASE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static STR_ENUM_BASE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static FLAG_BASE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static INT_FLAG_BASE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static ENUM_AUTO_FN: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

/// Returns the cached Python base class for `base`, importing `enum` at most once.
pub(crate) fn get_cached_base(
    py: Python<'_>,
    base: NativeEnumBase,
) -> PyResult<&Bound<'_, PyType>> {
    if base == NativeEnumBase::StrEnum && py.version_info() < (3, 11) {
        return Err(PyImportError::new_err(
            "StrEnum requires Python 3.11 or later",
        ));
    }
    let cell = match base {
        NativeEnumBase::Enum => &ENUM_BASE,
        NativeEnumBase::IntEnum => &INT_ENUM_BASE,
        NativeEnumBase::StrEnum => &STR_ENUM_BASE,
        NativeEnumBase::Flag => &FLAG_BASE,
        NativeEnumBase::IntFlag => &INT_FLAG_BASE,
    };
    cell.import(py, "enum", base.class_name())
}

/// Returns the cached `enum.auto` callable.
pub(crate) fn get_enum_auto(py: Python<'_>) -> PyResult<&Bound<'_, PyAny>> {
    ENUM_AUTO_FN.import(py, "enum", "auto")
}
