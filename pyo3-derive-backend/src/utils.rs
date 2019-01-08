// Copyright (c) 2017-present PyO3 Project and Contributors
use syn;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use proc_macro2::TokenStream;

pub fn print_err(msg: String, t: TokenStream) {
    println!("Error: {} in '{}'", msg, t.to_string());
}

/// Parse the macro arguments into a list of expressions.
pub fn parse_attrs(tokens: proc_macro::TokenStream) -> Vec<syn::Expr> {
    let parser = Punctuated::<syn::Expr, Token![,]>::parse_terminated;
    let error_message = "The macro attributes should be a list of comma separated expressions";

    parser
        .parse(tokens)
        .expect(error_message)
        .into_iter()
        .collect()
}

/// Parses variant attributes like `variants("MyTypeU32<u32>", "MyTypeF32<f32>")` into pairs
/// of names and type arguments.
pub fn parse_variants(call: &syn::ExprCall) -> Vec<(String, syn::AngleBracketedGenericArguments)> {
    use syn::Expr::*;

    let path = match *call.func {
        Path(ref expr_path) => expr_path,
        _ => panic!("Unsupported argument syntax"),
    };
    let path_segments = &path.path.segments;

    if path_segments.len() != 1
        || path_segments.first().unwrap().value().ident.to_string() != "variants"
    {
        panic!("Unsupported argument syntax");
    }

    call.args
        .iter()
        .map(|x| {
            // Extract string argument.
            let lit = match x {
                Lit(syn::ExprLit {
                    lit: syn::Lit::Str(ref lit),
                    ..
                }) => lit.value(),
                _ => panic!("Unsupported argument syntax"),
            };

            // Parse string as type.
            let ty: syn::Type = syn::parse_str(&lit).expect("Invalid type definition");

            let path_segs = match ty {
                syn::Type::Path(syn::TypePath { ref path, .. }) => path.segments.clone(),
                _ => panic!("Unsupported type syntax"),
            };

            if path_segs.len() != 1 {
                panic!("Type path is expected to have exactly one segment.");
            }

            let seg = path_segs.iter().nth(0).unwrap();
            let args = match seg.arguments {
                syn::PathArguments::AngleBracketed(ref args) => args.clone(),
                _ => panic!("Expected angle bracketed type arguments"),
            };

            (seg.ident.to_string(), args)
        })
        .collect()
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
    })
    .unwrap()
}
