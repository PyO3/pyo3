// Copyright (c) 2017-present PyO3 Project and Contributors

#![recursion_limit = "1024"]
#![feature(proc_macro)]

extern crate proc_macro;
extern crate pyo3_derive_backend;
extern crate quote;
#[macro_use]
extern crate syn;

use std::str::FromStr;

use proc_macro::TokenStream;
use pyo3_derive_backend::*;
use quote::{ToTokens, Tokens};
use syn::buffer::TokenBuffer;
use syn::punctuated::Punctuated;


#[proc_macro_attribute]
pub fn mod2init(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemFn = syn::parse(input).unwrap();

    // Extract the mod name
    let modname: syn::Ident = syn::parse(attr).unwrap();

    // Process the functions within the module
    module::process_functions_in_module(&mut ast);

    // Create the module initialisation function
    let init = module::py2_init(&ast.ident, &modname, utils::get_doc(&ast.attrs, false));

    // Return the generated code as a TokenStream
    let mut tokens = ast.into_tokens();
    tokens.append_all(init);
    tokens.into()
}

#[proc_macro_attribute]
pub fn mod3init(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemFn = syn::parse(input).unwrap();

    // Extract the mod name
    let modname: syn::Ident = syn::parse(attr).unwrap();

    // Process the functions within the module
    module::process_functions_in_module(&mut ast);

    // Create the module initialisation function
    let init = module::py3_init(&ast.ident, &modname, utils::get_doc(&ast.attrs, false));

    // Return the generated code as a TokenStream
    let mut tokens = ast.into_tokens();
    tokens.append_all(init);
    tokens.into()
}

#[proc_macro_attribute]
pub fn proto(_: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::Item = syn::parse(input).unwrap();

    // Build the output
    let expanded = py_proto::build_py_proto(&mut ast);

    // Return the generated impl as a TokenStream
    let mut tokens = ast.into_tokens();
    tokens.append_all(expanded);
    tokens.into()
}

#[proc_macro_attribute]
pub fn class(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::DeriveInput = syn::parse(input).unwrap();

    // Parse the macro arguments into a list of expressions
    let mut args: Vec<syn::Expr> = {
        let buffer = TokenBuffer::new(attr);
        let punc = Punctuated::<syn::Expr, Token![,]>::parse_terminated(buffer.begin());
        punc.expect("could not parse arguments").0.into_iter().collect()
    };

    // Build the output
    let expanded = py_class::build_py_class(&mut ast, &args);

    // Return the generated impl as a TokenStream
    let mut tokens = ast.into_tokens();
    tokens.append_all(expanded);
    tokens.into()
}

#[proc_macro_attribute]
pub fn methods(_: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::Item = syn::parse(input).unwrap();

    // Build the output
    let expanded = py_impl::build_py_methods(&mut ast);

    // Return the generated impl as a TokenStream
    let mut tokens = ast.into_tokens();
    tokens.append_all(expanded);
    tokens.into()
}

#[proc_macro_attribute]
pub fn function(_: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemFn = syn::parse(input)
        .expect("#[function] must be used on function definitions");

    // Build the output
    let python_name = ast.ident.clone();
    let expanded = module::add_fn_to_module(&mut ast, &python_name, Vec::new());

    // Return the generated impl as a TokenStream
    let mut tokens = ast.into_tokens();
    tokens.append_all(expanded);
    tokens.into()
}
