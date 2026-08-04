use proc_macro2::TokenStream;
use syn::parse::Parse;
use syn::{Expr, Ident, Path, Token};

use crate::introspection::class_introspection_code;
use crate::py_expr::PyExpr;
use crate::utils::{PyO3CratePath, PythonDoc, StrOrExpr};

pub struct ExceptionIntrospectionArgs {
    crate_path: Path,
    name: Ident,
    base: Path, // the super class of this exception
    doc: Option<Expr>,
}

impl Parse for ExceptionIntrospectionArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let crate_path = input.parse()?;
        input.parse::<Token![,]>()?;
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let base = input.parse()?;
        let doc = if input.parse::<Option<Token![,]>>()?.is_some() {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            crate_path,
            name,
            base,
            doc,
        })
    }
}

pub fn build_exception_introspection(
    ExceptionIntrospectionArgs {
        crate_path,
        name,
        base,
        doc,
    }: ExceptionIntrospectionArgs,
) -> TokenStream {
    let doc = doc.map(|doc| PythonDoc {
        parts: vec![StrOrExpr::Expr(doc)],
    });
    class_introspection_code(
        &PyO3CratePath::Given(crate_path),
        &name,
        &name.to_string(),
        Some(PyExpr::from_type(
            syn::TypePath {
                qself: None,
                path: base,
            }
            .into(),
            None,
        )),
        false,
        None,
        doc.as_ref(),
    )
}
