// Copyright (c) 2017-present PyO3 Project and Contributors

#![recursion_limit = "1024"]
#![feature(proc_macro)]

extern crate proc_macro;
extern crate pyo3_derive_backend;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use pyo3_derive_backend::*;
use quote::{ToTokens, Tokens};
use std::str::FromStr;

#[proc_macro_attribute]
pub fn mod2init(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let mut ast = syn::parse_item(&source).unwrap();

    // Build the output
    module::process_functions_in_module(&mut ast);

    let attr = utils::attr_with_parentheses(attr);

    let modname = &attr[1..attr.len() - 1].to_string();

    let init = module::py2_init(&ast.ident, &modname, utils::get_doc(&ast.attrs, false));

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
    module::process_functions_in_module(&mut ast);

    let attr = utils::attr_with_parentheses(attr);

    let modname = &attr[1..attr.len() - 1].to_string();

    let init = module::py3_init(&ast.ident, &modname, utils::get_doc(&ast.attrs, false));

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
    let expanded = py_class::build_py_class(&mut ast, utils::attr_with_parentheses(attr));

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

#[proc_macro_attribute]
pub fn function(_: TokenStream, input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let mut ast = syn::parse_item(&source).unwrap();

    // Build the output
    let python_name = ast.ident.clone();
    let expanded = module::add_fn_to_module(&mut ast, python_name, Vec::new());

    // Return the generated impl as a TokenStream
    let mut tokens = Tokens::new();
    ast.to_tokens(&mut tokens);
    let s = String::from(tokens.as_str()) + expanded.as_str();

    TokenStream::from_str(s.as_str()).unwrap()
}
