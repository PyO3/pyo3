// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate declares only the proc macro attributes, as a crate defining proc macro attributes
//! must not contain any other public items.

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use pyo3_derive_backend::{
    add_fn_to_module, build_py_class, build_py_methods, build_py_proto, get_doc,
    process_functions_in_module, py_init, PyClassArgs, PyFunctionAttr,
};
use quote::quote;
use syn::ext::IdentExt;
use syn::parse_macro_input;

/// Internally, this proc macro create a new c function called `PyInit_{my_module}`
/// that then calls the init function you provided
#[proc_macro_attribute]
pub fn pymodule(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemFn);

    let modname = if attr.is_empty() {
        ast.ident.clone()
    } else {
        parse_macro_input!(attr as syn::Ident)
    };

    process_functions_in_module(&mut ast);

    let expanded = py_init(&ast.ident, &modname, get_doc(&ast.attrs, false));

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pyproto(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemImpl);
    let expanded = build_py_proto(&mut ast).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pyclass(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemStruct);
    let args = parse_macro_input!(attr as PyClassArgs);
    let expanded = build_py_class(&mut ast, &args).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pymethods(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemImpl);
    let expanded = build_py_methods(&mut ast).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pyfunction(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::ItemFn);
    let args = parse_macro_input!(attr as PyFunctionAttr);

    let python_name = syn::Ident::new(&ast.ident.unraw().to_string(), Span::call_site());
    let expanded = add_fn_to_module(&ast, &python_name, args.arguments);

    quote!(
        #ast
        #expanded
    )
    .into()
}
