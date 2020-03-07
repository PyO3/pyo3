// Copyright (c) 2017-present PyO3 Project and Contributors
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use std::fmt::Display;

pub(crate) fn borrow_self(is_mut: bool) -> TokenStream {
    if is_mut {
        quote! {
            let mut _slf = _slf.try_borrow_mut()?;
        }
    } else {
        quote! {
            let _slf = _slf.try_borrow()?;
        }
    }
}

pub fn print_err(msg: String, t: TokenStream) {
    println!("Error: {} in '{}'", msg, t.to_string());
}

/// Check if the given type `ty` is `pyo3::Python`.
pub fn if_type_is_python(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(ref typath) => typath
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "Python")
            .unwrap_or(false),
        _ => false,
    }
}

pub fn is_text_signature_attr(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("text_signature")
}

fn parse_text_signature_attr<T: Display + quote::ToTokens + ?Sized>(
    attr: &syn::Attribute,
    python_name: &T,
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
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &python_name,
                format!("failed to parse python name: {}", python_name),
            )
        })?;
    match attr.parse_meta()? {
        syn::Meta::NameValue(syn::MetaNameValue {
            lit: syn::Lit::Str(lit),
            ..
        }) => {
            let value = lit.value();
            if value.starts_with('(') && value.ends_with(')') {
                Ok(Some(syn::LitStr::new(
                    &(python_name_str.to_owned() + &value),
                    lit.span(),
                )))
            } else {
                Err(syn::Error::new_spanned(
                    lit,
                    "text_signature must start with \"(\" and end with \")\"",
                ))
            }
        }
        meta => Err(syn::Error::new_spanned(
            meta,
            "text_signature must be of the form #[text_signature = \"\"]",
        )),
    }
}

pub fn parse_text_signature_attrs<T: Display + quote::ToTokens + ?Sized>(
    attrs: &mut Vec<syn::Attribute>,
    python_name: &T,
) -> syn::Result<Option<syn::LitStr>> {
    let mut text_signature = None;
    let mut attrs_out = Vec::with_capacity(attrs.len());
    for attr in attrs.drain(..) {
        if let Some(value) = parse_text_signature_attr(&attr, python_name)? {
            if text_signature.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "text_signature attribute already specified previously",
                ));
            } else {
                text_signature = Some(value);
            }
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
        if let Ok(syn::Meta::NameValue(ref metanv)) = attr.parse_meta() {
            if metanv.path.is_ident("doc") {
                if let syn::Lit::Str(ref litstr) = metanv.lit {
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
                    return Err(syn::Error::new_spanned(metanv, "Invalid doc comment"));
                }
            }
        }
    }

    if null_terminated {
        doc.push('\0');
    }

    Ok(syn::LitStr::new(&doc, span))
}
