// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate declares only the proc macro attributes, as a crate defining proc macro attributes
//! must not contain any other public items

#![recursion_limit = "1024"]
#![feature(proc_macro)]

extern crate proc_macro;
extern crate proc_macro2;
extern crate pyo3_derive_backend;
#[macro_use]
extern crate quote;
extern crate syn;

use syn::buffer::TokenBuffer;
use syn::punctuated::Punctuated;
use syn::token::Comma;

use pyo3_derive_backend::*;


#[proc_macro_attribute]
pub fn mod2init(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemFn = syn::parse(input)
        .expect("#[modinit] must be used on a function");

    // Extract the mod name
    let modname: syn::Ident = syn::parse(attr)
        .expect("could not parse module name");

    // Process the functions within the module
    module::process_functions_in_module(&mut ast);

    // Create the module initialisation function
    let expanded = module::py2_init(&ast.ident, &modname, utils::get_doc(&ast.attrs, false));

    quote! (
        #ast
        #expanded
    ).into()
}

#[proc_macro_attribute]
pub fn mod3init(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemFn = syn::parse(input)
        .expect("#[modinit] must be used on a `fn` block");

    // Extract the mod name
    let modname: syn::Ident = syn::parse(attr)
        .expect("could not parse module name");

    // Process the functions within the module
    module::process_functions_in_module(&mut ast);

    // Create the module initialisation function
    let expanded = module::py3_init(&ast.ident, &modname, utils::get_doc(&ast.attrs, false));

    quote! (
        #ast
        #expanded
    ).into()
}

#[proc_macro_attribute]
pub fn proto(_: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemImpl = syn::parse(input)
        .expect("#[proto] must be used on an `impl` block");

    // Build the output
    let expanded = py_proto::build_py_proto(&mut ast);

    quote! (
        #ast
        #expanded
    ).into()
}

#[proc_macro_attribute]
pub fn class(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::DeriveInput = syn::parse(input)
        .expect("#[class] must be used on a `struct`");

    // Parse the macro arguments into a list of expressions
    let args: Vec<syn::Expr> = {
        let buffer = TokenBuffer::new(attr);
        let punc = Punctuated::<syn::Expr,Comma>::parse_terminated(buffer.begin());
        punc.expect("could not parse macro arguments").0.into_iter().collect()
    };

    // Build the output
    let expanded = py_class::build_py_class(&mut ast, &args);

    quote! (
        #ast
        #expanded
    ).into()
}

#[proc_macro_attribute]
pub fn methods(_: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemImpl = syn::parse(input.clone())
        .expect("#[methods] must be used on an `impl` block");

    // Build the output
    let expanded = py_impl::build_py_methods(&mut ast);

    quote! (
        #ast
        #expanded
    ).into()
}

#[proc_macro_attribute]
pub fn function(_: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemFn = syn::parse(input)
        .expect("#[function] must be used on a `fn` block");

    // Build the output
    let python_name = ast.ident.clone();
    let expanded = module::add_fn_to_module(&mut ast, &python_name, Vec::new());

    quote! (
        #ast
        #expanded
    ).into()
}
