#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

#[macro_use]
mod test_utils;

#[pyclass]
/// The MacroDocs class.
#[doc = concat!("Some macro ", "class ", "docs.")]
/// A very interesting type!
struct MacroDocs {}

#[pymethods]
impl MacroDocs {
    #[doc = concat!("A macro ", "example.")]
    /// With mixed doc types.
    fn macro_doc(&self) {}
}

#[test]
fn meth_doc() {
    Python::attach(|py| {
        let d = [("C", py.get_type::<MacroDocs>())]
            .into_py_dict(py)
            .unwrap();
        py_assert!(
            py,
            *d,
            "C.__doc__ == 'The MacroDocs class.\\nSome macro class docs.\\nA very interesting type!'"
        );
        py_assert!(
            py,
            *d,
            "C.macro_doc.__doc__ == 'A macro example.\\nWith mixed doc types.'"
        );
    });
}
