//! Generates introspection data i.e. JSON strings in the .pyo3_data0 section
//!
//! There is a JSON per PyO3 proc macro (pyclass, pymodule, pyfunction...)
//!
//! These JSON blobs can refer to each others via the PYO3_INTROSPECTION_ID constants
//! providing unique ids for each element.

use crate::utils::PyO3CratePath;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::Ident;

static GLOBAL_COUNTER_FOR_UNIQUE_NAMES: AtomicUsize = AtomicUsize::new(0);

pub fn module_introspection_code<'a>(
    pyo3_crate_path: &PyO3CratePath,
    name: &str,
    members: impl IntoIterator<Item = &'a Ident>,
) -> TokenStream {
    let mut to_concat = Vec::new();
    to_concat.push(quote! { "{\"type\":\"module\",\"id\":\"" });
    to_concat.push(quote! { PYO3_INTROSPECTION_ID });
    to_concat.push(quote! { "\",\"name\":\""});
    to_concat.push(quote! { #name });
    to_concat.push(quote! { "\",\"members\":["});
    let mut start = true;
    for member in members {
        if start {
            start = false;
        } else {
            to_concat.push(quote! { "," });
        }
        to_concat.push(quote! { "\"" });
        to_concat.push(quote! {
            #member::PYO3_INTROSPECTION_ID
        });
        to_concat.push(quote! { "\"" });
    }
    to_concat.push(quote! { "]}" });
    let stub = stub_section(quote! {
        #pyo3_crate_path::impl_::concat::const_concat!(#(#to_concat , )*)
    });
    let introspection_id = introspection_id_const();
    quote! {
        #stub
        #introspection_id
    }
}

pub fn class_introspection_code(
    pyo3_crate_path: &PyO3CratePath,
    ident: &Ident,
    name: &str,
) -> TokenStream {
    let mut to_concat = Vec::new();
    to_concat.push(quote! { "{\"type\":\"class\",\"id\":\"" });
    to_concat.push(quote! { #ident::PYO3_INTROSPECTION_ID });
    to_concat.push(quote! { "\",\"name\":\""});
    to_concat.push(quote! { #name });
    to_concat.push(quote! { "\"}" });
    let stub = stub_section(quote! {
        #pyo3_crate_path::impl_::concat::const_concat!(#(#to_concat , )*)
    });
    let introspection_id = introspection_id_const();
    quote! {
        #stub
        impl #ident {
            #introspection_id
        }
    }
}

pub fn function_introspection_code(pyo3_crate_path: &PyO3CratePath, name: &str) -> TokenStream {
    let mut to_concat = Vec::new();
    to_concat.push(quote! { "{\"type\":\"function\",\"id\":\"" });
    to_concat.push(quote! { PYO3_INTROSPECTION_ID });
    to_concat.push(quote! { "\",\"name\":\""});
    to_concat.push(quote! { #name });
    to_concat.push(quote! { "\"}" });
    let stub = stub_section(quote! {
        #pyo3_crate_path::impl_::concat::const_concat!(#(#to_concat , )*)
    });
    let introspection_id = introspection_id_const();
    quote! {
        #stub
        #introspection_id
    }
}

fn stub_section(content: impl ToTokens) -> TokenStream {
    let section_name = if cfg!(any(target_os = "macos", target_os = "ios")) {
        "__TEXT,__pyo3_data0"
    } else {
        ".pyo3_data0"
    };
    quote! {
        const _: () = {
            #[used]
            #[link_section = #section_name]
            static PYO3_INTROSPECTION_DATA: &'static str = #content;
        };
    }
}

fn introspection_id_const() -> TokenStream {
    let id = GLOBAL_COUNTER_FOR_UNIQUE_NAMES.fetch_add(1, Ordering::Relaxed);
    quote! {
        pub const PYO3_INTROSPECTION_ID: &'static str = concat!(
            env!("CARGO_CRATE_NAME"),
            env!("CARGO_PKG_VERSION"),
            stringify!(#id)
        );
    }
}
