// Copyright (c) 2017-present PyO3 Project and Contributors
use syn;
use quote::{ToTokens, Tokens};

pub fn print_err(msg: String, t: Tokens) {
    println!("Error: {} in '{}'", msg, t.to_string());
}

pub fn for_err_msg(i: &ToTokens) -> String {
    format!("{:?}", i.into_tokens())
}

// FIXME(althonos): not sure the docstring formatting is on par here.
pub fn get_doc(attrs: &Vec<syn::Attribute>, null_terminated: bool) -> syn::Lit {
    let mut doc = Vec::new();

    // TODO(althonos): set span on produced doc str literal
    // let mut span = None;

    for attr in attrs.iter() {
        if let Some(syn::Meta::NameValue(ref metanv)) = attr.interpret_meta() {
            if metanv.ident == "doc" {
                // span = Some(metanv.span());
                if let syn::Lit::Str(ref litstr) = metanv.lit {
                    let d = litstr.value();
                    doc.push(if d.starts_with(" ") {
                        d[1..d.len()].to_string()
                    } else {
                        d
                    });
                } else {
                    panic!("could not parse doc");
                }
            }
        }
    }

    let doc = doc.join("\n");

    // FIXME: add span
    syn::parse_str(&if null_terminated {
        format!("\"{}\0\"", doc)
    } else {
        format!("\"{}\"", doc)
    }).unwrap()
}
