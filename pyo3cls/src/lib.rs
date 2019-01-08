// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate declares only the proc macro attributes, as a crate defining proc macro attributes
//! must not contain any other public items.

#![recursion_limit = "1024"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate pyo3_derive_backend;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro2::Span;
use pyo3_derive_backend::*;

#[proc_macro_attribute]
pub fn pymodule2(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemFn = syn::parse(input).expect("#[pymodule] must be used on a function");

    let modname: syn::Ident;
    if attr.is_empty() {
        modname = ast.ident.clone();
    } else {
        modname = syn::parse(attr).expect("could not parse module name");
    }

    // Process the functions within the module
    module::process_functions_in_module(&mut ast);

    // Create the module initialisation function
    let expanded = module::py2_init(&ast.ident, &modname, utils::get_doc(&ast.attrs, false));

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pymodule3(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemFn = syn::parse(input).expect("#[pymodule] must be used on a `fn` block");

    let modname: syn::Ident;
    if attr.is_empty() {
        modname = ast.ident.clone();
    } else {
        modname = syn::parse(attr).expect("could not parse module name");
    }

    // Process the functions within the module
    module::process_functions_in_module(&mut ast);

    // Create the module initialisation function
    let expanded = module::py3_init(&ast.ident, &modname, utils::get_doc(&ast.attrs, false));

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pyproto(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemImpl =
        syn::parse(input).expect("#[pyproto] must be used on an `impl` block");

    // Build the output
    let expanded = py_proto::build_py_proto(&mut ast);

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pyclass(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemStruct =
        syn::parse(input).expect("#[pyclass] must be used on a `struct`");
    let args = utils::parse_attrs(attr);

    // Build the output
    let expanded = py_class::build_py_class(&mut ast, &args);

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pymethods(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the token stream into a syntax tree
    let mut ast: syn::ItemImpl =
        syn::parse(input.clone()).expect("#[pymethods] must be used on an `impl` block");
    let args = utils::parse_attrs(attr);

    // Build the output
    let expanded = py_impl::build_py_methods(&mut ast, &args);

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_attribute]
pub fn pyfunction(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut ast: syn::ItemFn = syn::parse(input).expect("#[function] must be used on a `fn` block");

    // Workaround for https://github.com/dtolnay/syn/issues/478
    let python_name = syn::Ident::new(
        &ast.ident.to_string().trim_start_matches("r#"),
        Span::call_site(),
    );
    let expanded = module::add_fn_to_module(&mut ast, &python_name, Vec::new());

    quote!(
        #ast
        #expanded
    )
    .into()
}
