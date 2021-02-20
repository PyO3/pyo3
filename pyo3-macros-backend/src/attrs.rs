use syn::spanned::Spanned;
use syn::{ExprPath, Lit, Meta, MetaNameValue, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct FromPyWithAttribute(pub ExprPath);

impl FromPyWithAttribute {
    pub fn from_meta(meta: Meta) -> Result<Self> {
        let string_literal = match meta {
            Meta::NameValue(MetaNameValue {
                lit: Lit::Str(string_literal),
                ..
            }) => string_literal,
            meta => {
                bail_spanned!(meta.span() => "expected a name-value: `pyo3(from_py_with = \"func\")`")
            }
        };

        let expr_path = string_literal.parse::<ExprPath>()?;
        Ok(FromPyWithAttribute(expr_path))
    }
}
