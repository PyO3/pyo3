#![deny(clippy::undocumented_unsafe_blocks)]

use crate::{ffi, PyAny};

/// Represents a Python [`contextvars.Context`][1] object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyContext>`][crate::Py] or [`Bound<'py, PyContext>`][crate::Bound].
///
/// [1]: https://docs.python.org/3/library/contextvars.html#contextvars.Context
#[repr(transparent)]
pub struct PyContext(PyAny);

pyobject_native_type_core!(
    PyContext,
    pyobject_native_static_type_object!(ffi::PyContext_Type),
    "contextvars",
    "Context",
    #module=Some("contextvars"),
    #checkfunction=ffi::PyContext_CheckExact
);

#[cfg(test)]
mod tests {
    use super::PyContext;
    use crate::types::PyAnyMethods;
    use crate::Python;

    #[test]
    fn context_type() {
        Python::attach(|py| {
            let context = py
                .import(c"contextvars")
                .unwrap()
                .getattr(c"Context")
                .unwrap()
                .call0()
                .unwrap();

            assert!(context.is_exact_instance_of::<PyContext>());
            context.cast::<PyContext>().unwrap();
        });
    }
}
