// Copyright (c) 2017-present PyO3 Project and Contributors

#![recursion_limit="1024"]
#![feature(proc_macro)]

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;
#[macro_use] extern crate log;

use std::str::FromStr;
use proc_macro::TokenStream;

use quote::{Tokens, ToTokens};

mod py_class;
mod py_impl;
mod py_proto;
mod py_method;
mod args;
mod defs;
mod func;
mod method;
mod module;
mod utils;


#[proc_macro_attribute]
pub fn mod2init(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let mut ast = syn::parse_item(&source).unwrap();

    // Build the output
    let init = module::build_py2_module_init(&mut ast, attr.to_string());

    // Return the generated impl as a TokenStream
    let mut tokens = Tokens::new();
    ast.to_tokens(&mut tokens);
    let s = String::from(tokens.as_str()) + init.as_str();

    TokenStream::from_str(s.as_str()).unwrap()
}

#[proc_macro_attribute]
pub fn mod3init(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let mut ast = syn::parse_item(&source).unwrap();

    // Build the output
    let init = module::build_py3_module_init(&mut ast, attr.to_string());

    // Return the generated impl as a TokenStream
    let mut tokens = Tokens::new();
    ast.to_tokens(&mut tokens);
    let s = String::from(tokens.as_str()) + init.as_str();

    TokenStream::from_str(s.as_str()).unwrap()
}

#[proc_macro_attribute]
pub fn proto(_: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    // Parse the string representation into a syntax tree
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
pub fn class(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let mut ast = syn::parse_derive_input(&source).unwrap();

    // Build the output
    let expanded = py_class::build_py_class(&mut ast, attr.to_string());

    // Return the generated impl as a TokenStream
    let mut tokens = Tokens::new();
    ast.to_tokens(&mut tokens);
    let s = String::from(tokens.as_str()) + expanded.as_str();

    TokenStream::from_str(s.as_str()).unwrap()
}

#[proc_macro_attribute]
pub fn methods(_: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let mut ast = syn::parse_item(&source).unwrap();

    // Build the output
    let expanded = py_impl::build_py_methods(&mut ast);

    // Return the generated impl as a TokenStream
    let mut tokens = Tokens::new();
    ast.to_tokens(&mut tokens);
    let s = String::from(tokens.as_str()) + expanded.as_str();

    TokenStream::from_str(s.as_str()).unwrap()
}
