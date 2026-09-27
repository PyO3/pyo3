//! Example of custom annotations.

use pyo3::impl_::concat::combine_to_array;
use pyo3::impl_::introspection::SerializedIntrospectionFragment;
use pyo3::inspect::{PyClassNameStaticExpr, PyStaticExpr};
use pyo3::prelude::*;

/// Emits an introspection chunk the way a library crate has to: without knowing the id of the
/// module it ends up in.
macro_rules! introspection_chunk {
    ($symbol:ident, $json:literal) => {
        const _: () = {
            const JSON: &[u8] = $json.as_bytes();
            #[used]
            #[no_mangle]
            static $symbol: SerializedIntrospectionFragment<{ JSON.len() }> =
                SerializedIntrospectionFragment {
                    length: JSON.len() as u32,
                    fragment: combine_to_array(&[JSON]),
                };
        };
    };
}

introspection_chunk!(
    PYO3_INTROSPECTION_1_PYTESTS_SUPPORTS_CLOSE,
    r#"{"type": "class", "id": "pytests:SupportsClose", "name": "SupportsClose", "attach_to_root": true, "bases": [{"type": "attribute", "value": {"type": "name", "id": "typing"}, "attr": "Protocol"}]}"#
);
introspection_chunk!(
    PYO3_INTROSPECTION_1_PYTESTS_SUPPORTS_CLOSE_CLOSE,
    r#"{"type": "function", "name": "close", "parent": "pytests:SupportsClose", "arguments": {"posonlyargs": [{"name": "self"}]}, "returns": {"type": "constant", "kind": "none"}}"#
);
// Not referenced by any annotation, so left out of the stubs
introspection_chunk!(
    PYO3_INTROSPECTION_1_PYTESTS_SUPPORTS_UNUSED,
    r#"{"type": "class", "id": "pytests:SupportsUnused", "name": "SupportsUnused", "attach_to_root": true, "bases": [{"type": "attribute", "value": {"type": "name", "id": "typing"}, "attr": "Protocol"}]}"#
);

/// Any object with a `close()` method, typed as the `SupportsClose` protocol
pub struct Closeable<'py>(Bound<'py, PyAny>);

impl<'py> FromPyObject<'_, 'py> for Closeable<'py> {
    type Error = PyErr;

    const INPUT_TYPE: PyStaticExpr = PyStaticExpr::PyClass(PyClassNameStaticExpr::new(
        &PyStaticExpr::Name {
            id: "SupportsClose",
        },
        "pytests:SupportsClose",
    ));

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        Ok(Self(obj.to_owned()))
    }
}

#[pymodule]
pub mod annotations {
    use crate::pyclasses::EmptyClass;
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyTuple};

    #[pyfunction(signature = (a: "list[int]", *_args: "str", _b: "int | None" = None, **_kwargs: "bool") -> "int")]
    fn with_custom_type_annotations<'py>(
        a: Bound<'py, PyAny>,
        _args: Bound<'py, PyTuple>,
        _b: Option<Bound<'py, PyAny>>,
        _kwargs: Option<Bound<'py, PyDict>>,
    ) -> Bound<'py, PyAny> {
        a
    }

    #[pyfunction]
    fn cross_module_imports(_a: &EmptyClass) {}

    #[pyfunction]
    fn close(obj: super::Closeable<'_>) -> PyResult<()> {
        obj.0.call_method0("close")?;
        Ok(())
    }
}
