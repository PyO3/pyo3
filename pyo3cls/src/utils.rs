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

pub fn get_doc(attrs: &Vec<syn::Attribute>) -> syn::Lit {
    let mut doc = Vec::new();

    for attr in attrs.iter() {
        match attr.value {
            syn::MetaItem::NameValue(ref ident, ref lit) => {
                if ident.as_ref() == "doc" {
                    let s = quote!{ #lit }.to_string();
                    doc.push(s[1..s.len()-1].to_owned())
                }
            }
            _ => (),
        }
    }
    let doc = doc.join("\n");
    syn::Lit::Str(format!("{}\0", doc), syn::StrStyle::Cooked)
}
