// Copyright (c) 2017-present PyO3 Project and Contributors

use proc_macro2::Span;
use proc_macro2::TokenStream;
use std::fmt::Display;

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
    text_signature: &mut Option<syn::LitStr>,
) -> syn::Result<Option<()>> {
    if !is_text_signature_attr(attr) {
        return Ok(None);
    }
    if text_signature.is_some() {
        return Err(syn::Error::new_spanned(
            attr,
            "text_signature attribute already specified previously",
        ));
    }
    let value: String;
    match attr.parse_meta()? {
        syn::Meta::NameValue(syn::MetaNameValue {
            lit: syn::Lit::Str(lit),
            ..
        }) => {
            value = lit.value();
            *text_signature = Some(lit);
        }
        meta => {
            return Err(syn::Error::new_spanned(
                meta,
                "text_signature must be of the form #[text_signature = \"\"]",
            ));
        }
    };
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
    if !value.starts_with(&python_name_str) || !value[python_name_str.len()..].starts_with('(') {
        return Err(syn::Error::new_spanned(
            text_signature,
            format!("text_signature must start with \"{}(\"", python_name_str),
        ));
    }
    if !value.ends_with(')') {
        return Err(syn::Error::new_spanned(
            text_signature,
            "text_signature must end with \")\"",
        ));
    }
    Ok(Some(()))
}

pub fn parse_text_signature_attrs<T: Display + quote::ToTokens + ?Sized>(
    attrs: &mut Vec<syn::Attribute>,
    python_name: &T,
) -> syn::Result<Option<syn::LitStr>> {
    let mut parse_error: Option<syn::Error> = None;
    let mut text_signature = None;
    attrs.retain(|attr| {
        match parse_text_signature_attr(attr, python_name, &mut text_signature) {
            Ok(None) => return true,
            Ok(Some(_)) => {}
            Err(err) => {
                if let Some(parse_error) = &mut parse_error {
                    parse_error.combine(err);
                } else {
                    parse_error = Some(err);
                }
            }
        }
        false
    });
    if let Some(parse_error) = parse_error {
        return Err(parse_error);
    }
    Ok(text_signature)
}

// FIXME(althonos): not sure the docstring formatting is on par here.
pub fn get_doc(
    attrs: &[syn::Attribute],
    text_signature: Option<syn::LitStr>,
    null_terminated: bool,
) -> syn::Result<syn::Lit> {
    let mut doc = Vec::new();
    let mut needs_terminating_newline = false;

    if let Some(text_signature) = text_signature {
        doc.push(text_signature.value());
        doc.push("--".to_string());
        doc.push(String::new());
        needs_terminating_newline = true;
    }

    // TODO(althonos): set span on produced doc str literal
    // let mut span = None;

    for attr in attrs.iter() {
        if let Ok(syn::Meta::NameValue(ref metanv)) = attr.parse_meta() {
            if metanv.path.is_ident("doc") {
                // span = Some(metanv.span());
                if let syn::Lit::Str(ref litstr) = metanv.lit {
                    let d = litstr.value();
                    doc.push(if d.starts_with(' ') {
                        d[1..d.len()].to_string()
                    } else {
                        d
                    });
                    needs_terminating_newline = false;
                } else {
                    return Err(syn::Error::new_spanned(metanv, "Invalid doc comment"));
                }
            }
        }
    }

    if needs_terminating_newline {
        doc.push(String::new());
    }

    let mut docstr = doc.join("\n");
    if null_terminated {
        docstr.push('\0');
    }

    Ok(syn::Lit::Str(syn::LitStr::new(&docstr, Span::call_site())))
}
