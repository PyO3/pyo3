// Copyright (c) 2017-present PyO3 Project and Contributors

#![recursion_limit="1024"]
#![feature(proc_macro)]

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

use std::str::FromStr;
use proc_macro::TokenStream;

use quote::{Tokens, ToTokens};

mod py_class;
mod py_proto;
mod py_method;


#[proc_macro_attribute]
pub fn proto(_: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    //let ast: syn::Crate = source.parse().unwrap();
    let mut ast = syn::parse_item(&source).unwrap();

    // Build the output
    let expanded = py_proto::build_py_proto(&mut ast);

    // Return the generated impl as a TokenStream
    let mut tokens = Tokens::new();
    ast.to_tokens(&mut tokens);
    let s = String::from(tokens.as_str()) + expanded.as_str();

    TokenStream::from_str(s.as_str()).unwrap()
}


#[proc_macro_attribute]
pub fn class(_: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    //let ast: syn::Crate = source.parse().unwrap();
    let mut ast = syn::parse_derive_input(&source).unwrap();

    // Build the output
    let expanded = py_class::build_py_class(&mut ast);

    // Return the generated impl as a TokenStream
    let mut tokens = Tokens::new();
    ast.to_tokens(&mut tokens);
    let s = String::from(tokens.as_str()) + expanded.as_str();

    TokenStream::from_str(s.as_str()).unwrap()
}
