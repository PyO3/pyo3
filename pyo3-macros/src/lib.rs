// Copyright (c) 2017-present PyO3 Project and Contributors
//! This crate declares only the proc macro attributes, as a crate defining proc macro attributes
//! must not contain any other public items.

#![cfg_attr(docsrs, feature(doc_cfg))]
extern crate proc_macro;

use proc_macro::TokenStream;
use pyo3_macros_backend::{
    build_derive_from_pyobject, build_py_class, build_py_function, build_py_methods,
    build_py_proto, get_doc, process_functions_in_module, py_init, PyClassArgs, PyClassMethodsType,
    PyFunctionAttr,
};
use quote::quote;
use syn::parse_macro_input;

/// Internally, this proc macro create a new c function called `PyInit_{my_module}`
/// that then calls the init function you provided
#[proc_macro_attribute]
pub fn pymodule(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemFn);

    let modname = if attr.is_empty() {
        ast.sig.ident.clone()
    } else {
        parse_macro_input!(attr as syn::Ident)
    };

    if let Err(err) = process_functions_in_module(&mut ast) {
        return err.to_compile_error().into();
    }

    let doc = match get_doc(&ast.attrs, None, false) {
        Ok(doc) => doc,
        Err(err) => return err.to_compile_error().into(),
    };

    let expanded = py_init(&ast.sig.ident, &modname, doc);

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
    pyclass_impl(attr, input, PyClassMethodsType::Specialization)
}

#[proc_macro_attribute]
pub fn pyclass_with_inventory(attr: TokenStream, input: TokenStream) -> TokenStream {
    pyclass_impl(attr, input, PyClassMethodsType::Inventory)
}

#[proc_macro_attribute]
pub fn pymethods(_: TokenStream, input: TokenStream) -> TokenStream {
    pymethods_impl(input, PyClassMethodsType::Specialization)
}

#[proc_macro_attribute]
pub fn pymethods_with_inventory(_: TokenStream, input: TokenStream) -> TokenStream {
    pymethods_impl(input, PyClassMethodsType::Inventory)
}

#[proc_macro_attribute]
pub fn pyfunction(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemFn);
    let args = parse_macro_input!(attr as PyFunctionAttr);

    let expanded = build_py_function(&mut ast, args).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

#[proc_macro_derive(FromPyObject, attributes(pyo3))]
pub fn derive_from_py_object(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as syn::DeriveInput);
    let expanded = build_derive_from_pyobject(&ast).unwrap_or_else(|e| e.to_compile_error());
    quote!(
        #expanded
    )
    .into()
}

fn pyclass_impl(
    attr: TokenStream,
    input: TokenStream,
    methods_type: PyClassMethodsType,
) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemStruct);
    let args = parse_macro_input!(attr as PyClassArgs);
    let expanded =
        build_py_class(&mut ast, &args, methods_type).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}

fn pymethods_impl(input: TokenStream, methods_type: PyClassMethodsType) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::ItemImpl);
    let expanded =
        build_py_methods(&mut ast, methods_type).unwrap_or_else(|e| e.to_compile_error());

    quote!(
        #ast
        #expanded
    )
    .into()
}
