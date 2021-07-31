// Copyright (c) 2017-present PyO3 Project and Contributors
use proc_macro2::Span;
use syn::spanned::Spanned;

use crate::attributes::TextSignatureAttribute;

/// Macro inspired by `anyhow::anyhow!` to create a compiler error with the given span.
macro_rules! err_spanned {
    ($span:expr => $msg:expr) => {
        syn::Error::new($span, $msg)
    };
}

/// Macro inspired by `anyhow::bail!` to return a compiler error with the given span.
macro_rules! bail_spanned {
    ($span:expr => $msg:expr) => {
        return Err(err_spanned!($span => $msg));
    };
}

/// Macro inspired by `anyhow::ensure!` to return a compiler error with the given span if the
/// specified condition is not met.
macro_rules! ensure_spanned {
    ($condition:expr, $span:expr => $msg:expr) => {
        if !($condition) {
            bail_spanned!($span => $msg);
        }
    }
}

/// Check if the given type `ty` is `pyo3::Python`.
pub fn is_python(ty: &syn::Type) -> bool {
    match unwrap_ty_group(ty) {
        syn::Type::Path(typath) => typath
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "Python")
            .unwrap_or(false),
        _ => false,
    }
}

/// If `ty` is Option<T>, return `Some(T)`, else None.
pub fn option_type_argument(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        let seg = path.segments.last().filter(|s| s.ident == "Option")?;
        if let syn::PathArguments::AngleBracketed(params) = &seg.arguments {
            if let syn::GenericArgument::Type(ty) = params.args.first()? {
                return Some(ty);
            }
        }
    }
    None
}

// Returns a null-terminated syn::LitStr for use as a Python docstring.
pub fn get_doc(
    attrs: &[syn::Attribute],
    text_signature: Option<(&syn::Ident, &TextSignatureAttribute)>,
) -> syn::Result<syn::LitStr> {
    let mut doc = String::new();
    let mut span = Span::call_site();

    if let Some((python_name, text_signature)) = text_signature {
        // create special doc string lines to set `__text_signature__`
        doc.push_str(&python_name.to_string());
        span = text_signature.lit.span();
        doc.push_str(&text_signature.lit.value());
        doc.push_str("\n--\n\n");
    }

    let mut separator = "";
    let mut first = true;

    for attr in attrs.iter() {
        if attr.path.is_ident("doc") {
            if let Ok(DocArgs { _eq_token, lit_str }) = syn::parse2(attr.tokens.clone()) {
                if first {
                    first = false;
                    span = lit_str.span();
                }
                let d = lit_str.value();
                doc.push_str(separator);
                if d.starts_with(' ') {
                    doc.push_str(&d[1..d.len()]);
                } else {
                    doc.push_str(&d);
                };
                separator = "\n";
            }
        }
    }

    doc.push('\0');

    Ok(syn::LitStr::new(&doc, span))
}

struct DocArgs {
    _eq_token: syn::Token![=],
    lit_str: syn::LitStr,
}

impl syn::parse::Parse for DocArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let this = Self {
            _eq_token: input.parse()?,
            lit_str: input.parse()?,
        };
        ensure_spanned!(input.is_empty(), input.span() => "expected end of doc attribute");
        Ok(this)
    }
}

pub fn ensure_not_async_fn(sig: &syn::Signature) -> syn::Result<()> {
    if let Some(asyncness) = &sig.asyncness {
        bail_spanned!(
            asyncness.span() => "`async fn` is not yet supported for Python functions.\n\n\
            Additional crates such as `pyo3-asyncio` can be used to integrate async Rust and \
            Python. For more information, see https://github.com/PyO3/pyo3/issues/1632"
        );
    };
    Ok(())
}

pub fn unwrap_group(mut expr: &syn::Expr) -> &syn::Expr {
    while let syn::Expr::Group(g) = expr {
        expr = &*g.expr;
    }
    expr
}

pub fn unwrap_ty_group(mut ty: &syn::Type) -> &syn::Type {
    while let syn::Type::Group(g) = ty {
        ty = &*g.elem;
    }
    ty
}
