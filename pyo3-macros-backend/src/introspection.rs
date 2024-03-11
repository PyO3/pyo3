//! Generates introspection data i.e. JSON strings in the .pyo3i0 section.
//!
//! There is a JSON per PyO3 proc macro (pyclass, pymodule, pyfunction...).
//!
//! These JSON blobs can refer to each others via the _PYO3_INTROSPECTION_ID constants
//! providing unique ids for each element.

use crate::utils::PyO3CratePath;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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
    to_concat.push(quote! { _PYO3_INTROSPECTION_ID });
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
            #member::_PYO3_INTROSPECTION_ID
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
    to_concat.push(quote! { #ident::_PYO3_INTROSPECTION_ID });
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
    to_concat.push(quote! { _PYO3_INTROSPECTION_ID });
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
    let static_name = format_ident!("PYO3_INTRS_{}", unique_element_id());
    // #[no_mangle] is required to make sure some linkers like Linux ones do not mangle the section name too.
    quote! {
        const _: () = {
            #[used]
            #[cfg(not(target_family = "wasm"))]
            #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos", target_os = "watchos"), link_section = "__TEXT,__pyo3i0")]
            #[cfg_attr(not(any(target_os = "macos", target_os = "ios", target_os = "tvos", target_os = "watchos")), link_section = ".pyo3i0")]
            #[no_mangle]
            static #static_name: &'static str = #content;
        };
    }
}

fn introspection_id_const() -> TokenStream {
    let id = unique_element_id().to_string();
    quote! {
        #[doc(hidden)]
        pub const _PYO3_INTROSPECTION_ID: &'static str = #id;
    }
}

fn unique_element_id() -> u64 {
    let mut hasher = DefaultHasher::new();
    format!("{:?}", Span::call_site()).hash(&mut hasher); // Distinguishes between call sites
    GLOBAL_COUNTER_FOR_UNIQUE_NAMES
        .fetch_add(1, Ordering::Relaxed)
        .hash(&mut hasher); // If there are multiple elements in the same call site
    hasher.finish()
}
