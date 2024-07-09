use crate::ffi;
use crate::PyAny;

/// Represents a Python code object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyCode>`][crate::Py] or [`Bound<'py, PyCode>`][crate::Bound].
#[repr(transparent)]
pub struct PyCode(PyAny);

pyobject_native_type_core!(
    PyCode,
    pyobject_native_static_type_object!(ffi::PyCode_Type),
    #checkfunction=ffi::PyCode_Check
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PyTypeMethods;
    use crate::{PyTypeInfo, Python};

    #[test]
    fn test_type_object() {
        Python::with_gil(|py| {
            assert_eq!(PyCode::type_object_bound(py).name().unwrap(), "code");
        })
    }
}
