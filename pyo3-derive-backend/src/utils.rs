// Copyright (c) 2017-present PyO3 Project and Contributors

use proc_macro2::Span;
use proc_macro2::TokenStream;

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
            .map(|seg| seg.value().ident == "Python")
            .unwrap_or(false),
        _ => false,
    }
}

// FIXME(althonos): not sure the docstring formatting is on par here.
pub fn get_doc(attrs: &[syn::Attribute], null_terminated: bool) -> syn::Lit {
    let mut doc = Vec::new();

    // TODO(althonos): set span on produced doc str literal
    // let mut span = None;

    for attr in attrs.iter() {
        if let Some(syn::Meta::NameValue(ref metanv)) = attr.interpret_meta() {
            if metanv.ident == "doc" {
                // span = Some(metanv.span());
                if let syn::Lit::Str(ref litstr) = metanv.lit {
                    let d = litstr.value();
                    doc.push(if d.starts_with(' ') {
                        d[1..d.len()].to_string()
                    } else {
                        d
                    });
                } else {
                    panic!("Invalid doc comment");
                }
            }
        }
    }

    let mut docstr = doc.join("\n");
    if null_terminated {
        docstr.push('\0');
    }

    syn::Lit::Str(syn::LitStr::new(&docstr, Span::call_site()))
}
