// Copyright (c) 2017-present PyO3 Project and Contributors
use syn;
use quote::{Tokens, ToTokens};


pub fn print_err(msg: String, t: Tokens) {
    println!("Error: {} in '{}'", msg, t.to_string());
}

pub fn for_err_msg(i: &ToTokens) -> String {
    let mut tokens = Tokens::new();

    i.to_tokens(&mut tokens);
    tokens.as_str().to_string()
}

pub fn get_doc(attrs: &Vec<syn::Attribute>, null_terminated: bool) -> syn::Lit {
    let mut doc = Vec::new();

    for attr in attrs.iter() {
        match attr.value {
            syn::MetaItem::NameValue(ref ident, ref lit) => {
                if ident.as_ref() == "doc" {
                    let s = quote!{ #lit }.to_string();
                    let mut s = s[1..s.len()-1].to_string();
                    if s.starts_with("/// ") {
                        // Remove leading whitespace and ///
                        s = s[4..].to_string();
                    } else {
                        // Remove only ///
                        s = s[3..].to_string();
                    }
                    doc.push(s)
                }
            }
            _ => (),
        }
    }
    let doc = doc.join("\n");
    if null_terminated {
        syn::Lit::Str(format!("{}\0", doc), syn::StrStyle::Cooked)
    } else {
        syn::Lit::Str(doc, syn::StrStyle::Cooked)
    }
}
