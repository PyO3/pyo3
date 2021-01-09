// Copyright (c) 2017-present PyO3 Project and Contributors
use proc_macro2::Span;
use syn::spanned::Spanned;

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
    match ty {
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

pub fn is_text_signature_attr(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("text_signature")
}

fn parse_text_signature_attr(
    attr: &syn::Attribute,
    python_name: &syn::Ident,
) -> syn::Result<Option<syn::LitStr>> {
    if !is_text_signature_attr(attr) {
        return Ok(None);
    }
    let python_name_str = python_name.to_string();
    let python_name_str = python_name_str
        .rsplit('.')
        .next()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| err_spanned!(python_name.span() => "failed to parse python name"))?;
    match attr.parse_meta()? {
        syn::Meta::NameValue(syn::MetaNameValue {
            lit: syn::Lit::Str(lit),
            ..
        }) => {
            let value = lit.value();
            ensure_spanned!(
                value.starts_with('(') && value.ends_with(')'),
                lit.span() => "text_signature must start with \"(\" and end with \")\""
            );
            Ok(Some(syn::LitStr::new(
                &(python_name_str.to_owned() + &value),
                lit.span(),
            )))
        }
        meta => bail_spanned!(
            meta.span() => "text_signature must be of the form #[text_signature = \"\"]"
        ),
    }
}

pub fn parse_text_signature_attrs(
    attrs: &mut Vec<syn::Attribute>,
    python_name: &syn::Ident,
) -> syn::Result<Option<syn::LitStr>> {
    let mut text_signature = None;
    let mut attrs_out = Vec::with_capacity(attrs.len());
    for attr in attrs.drain(..) {
        if let Some(value) = parse_text_signature_attr(&attr, python_name)? {
            ensure_spanned!(
                text_signature.is_none(),
                attr.span() => "text_signature attribute already specified previously"
            );
            text_signature = Some(value);
        } else {
            attrs_out.push(attr);
        }
    }
    *attrs = attrs_out;
    Ok(text_signature)
}

// FIXME(althonos): not sure the docstring formatting is on par here.
pub fn get_doc(
    attrs: &[syn::Attribute],
    text_signature: Option<syn::LitStr>,
    null_terminated: bool,
) -> syn::Result<syn::LitStr> {
    let mut doc = String::new();
    let mut span = Span::call_site();

    if let Some(text_signature) = text_signature {
        // create special doc string lines to set `__text_signature__`
        span = text_signature.span();
        doc.push_str(&text_signature.value());
        doc.push_str("\n--\n\n");
    }

    let mut separator = "";
    let mut first = true;

    for attr in attrs.iter() {
        if let Ok(syn::Meta::NameValue(metanv)) = attr.parse_meta() {
            if metanv.path.is_ident("doc") {
                if let syn::Lit::Str(litstr) = metanv.lit {
                    if first {
                        first = false;
                        span = litstr.span();
                    }
                    let d = litstr.value();
                    doc.push_str(separator);
                    if d.starts_with(' ') {
                        doc.push_str(&d[1..d.len()]);
                    } else {
                        doc.push_str(&d);
                    };
                    separator = "\n";
                } else {
                    bail_spanned!(metanv.span() => "invalid doc comment")
                }
            }
        }
    }

    if null_terminated {
        doc.push('\0');
    }

    Ok(syn::LitStr::new(&doc, span))
}
