// Copyright (c) 2017-present PyO3 Project and Contributors
use syn;
use syn::spanned::Spanned;
use quote::{Tokens, ToTokens};
use proc_macro::TokenStream;


/// https://github.com/rust-lang/rust/pull/50120 removed the parantheses from
/// the attr TokenStream, so we need to re-add them manually.
///
/// nightly-2018-04-05: ( name=CustomName )
/// nightly-2018-04-28: name=CustomName
// pub fn attr_with_parentheses(attr: TokenStream) -> String {
//     let attr = attr.to_string();
//     if attr.len() > 0 && !attr.starts_with("(") {
//         return format!("({})", attr);
//     } else {
//         return attr;
//     }
// }

pub fn print_err(msg: String, t: Tokens) {
    println!("Error: {} in '{}'", msg, t.to_string());
}

pub fn for_err_msg(i: &ToTokens) -> String {
    let mut tokens = Tokens::new();

    i.to_tokens(&mut tokens);
    format!("{:?}", tokens).to_string()
}


// FIXME(althonos): not sure the docstring formatting is on par here.
pub fn get_doc(attrs: &Vec<syn::Attribute>, null_terminated: bool) -> syn::Lit {

    let mut doc = Vec::new();
    let mut span = None;

    for attr in attrs.iter() {
        if let Some(syn::Meta::NameValue(ref metanv)) = attr.interpret_meta() {
            if metanv.ident == "doc" {
                span = Some(metanv.span());
                if let syn::Lit::Str(ref litstr) = metanv.lit {
                    let d = litstr.value();
                    doc.push(if d.starts_with(" ") { d[1..d.len()].to_string() } else {d});
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
