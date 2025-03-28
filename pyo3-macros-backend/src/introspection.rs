//! Generates introspection data i.e. JSON strings in the .pyo3i0 section.
//!
//! There is a JSON per PyO3 proc macro (pyclass, pymodule, pyfunction...).
//!
//! These JSON blobs can refer to each others via the _PYO3_INTROSPECTION_ID constants
//! providing unique ids for each element.
//!
//! The JSON blobs format must be synchronized with the `pyo3_introspection::introspection.rs::Chunk`
//! type that is used to parse them.

use crate::utils::PyO3CratePath;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::mem::take;
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::{Attribute, Ident};

static GLOBAL_COUNTER_FOR_UNIQUE_NAMES: AtomicUsize = AtomicUsize::new(0);

pub fn module_introspection_code<'a>(
    pyo3_crate_path: &PyO3CratePath,
    name: &str,
    members: impl IntoIterator<Item = &'a Ident>,
    members_cfg_attrs: impl IntoIterator<Item = &'a Vec<Attribute>>,
) -> TokenStream {
    let stub = IntrospectionNode::Map(
        [
            ("type", IntrospectionNode::String("module")),
            ("id", IntrospectionNode::IntrospectionId(None)),
            ("name", IntrospectionNode::String(name)),
            (
                "members",
                IntrospectionNode::List(
                    members
                        .into_iter()
                        .zip(members_cfg_attrs)
                        .filter_map(|(member, attributes)| {
                            if attributes.is_empty() {
                                Some(IntrospectionNode::IntrospectionId(Some(member)))
                            } else {
                                None // TODO: properly interpret cfg attributes
                            }
                        })
                        .collect(),
                ),
            ),
        ]
        .into(),
    )
    .emit(pyo3_crate_path);
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
    let stub = IntrospectionNode::Map(
        [
            ("type", IntrospectionNode::String("class")),
            ("id", IntrospectionNode::IntrospectionId(Some(ident))),
            ("name", IntrospectionNode::String(name)),
        ]
        .into(),
    )
    .emit(pyo3_crate_path);
    let introspection_id = introspection_id_const();
    quote! {
        #stub
        impl #ident {
            #introspection_id
        }
    }
}

pub fn function_introspection_code(pyo3_crate_path: &PyO3CratePath, name: &str) -> TokenStream {
    let stub = IntrospectionNode::Map(
        [
            ("type", IntrospectionNode::String("function")),
            ("id", IntrospectionNode::IntrospectionId(None)),
            ("name", IntrospectionNode::String(name)),
        ]
        .into(),
    )
    .emit(pyo3_crate_path);
    let introspection_id = introspection_id_const();
    quote! {
        #stub
        #introspection_id
    }
}

enum IntrospectionNode<'a> {
    String(&'a str),
    IntrospectionId(Option<&'a Ident>),
    Map(HashMap<&'static str, IntrospectionNode<'a>>),
    List(Vec<IntrospectionNode<'a>>),
}

impl IntrospectionNode<'_> {
    fn emit(self, pyo3_crate_path: &PyO3CratePath) -> TokenStream {
        let mut content = ConcatenationBuilder::default();
        self.add_to_serialization(&mut content);
        let content = content.into_token_stream(pyo3_crate_path);

        let static_name = format_ident!("PYO3_INTROSPECTION_0_{}", unique_element_id());
        // #[no_mangle] is required to make sure some linkers like Linux ones do not mangle the section name too.
        quote! {
            const _: () = {
                #[used]
                #[no_mangle]
                static #static_name: &'static str = #content;
            };
        }
    }

    fn add_to_serialization(self, content: &mut ConcatenationBuilder) {
        match self {
            Self::String(string) => {
                content.push_str_to_escape(string);
            }
            Self::IntrospectionId(ident) => {
                content.push_str("\"");
                content.push_tokens(if let Some(ident) = ident {
                    quote! { #ident::_PYO3_INTROSPECTION_ID }
                } else {
                    Ident::new("_PYO3_INTROSPECTION_ID", Span::call_site()).into_token_stream()
                });
                content.push_str("\"");
            }
            Self::Map(map) => {
                content.push_str("{");
                for (i, (key, value)) in map.into_iter().enumerate() {
                    if i > 0 {
                        content.push_str(",");
                    }
                    content.push_str_to_escape(key);
                    content.push_str(":");
                    value.add_to_serialization(content);
                }
                content.push_str("}");
            }
            Self::List(list) => {
                content.push_str("[");
                for (i, value) in list.into_iter().enumerate() {
                    if i > 0 {
                        content.push_str(",");
                    }
                    value.add_to_serialization(content);
                }
                content.push_str("]");
            }
        }
    }
}

#[derive(Default)]
struct ConcatenationBuilder {
    elements: Vec<ConcatenationBuilderElement>,
    current_string: String,
}

impl ConcatenationBuilder {
    fn push_tokens(&mut self, token_stream: TokenStream) {
        if !self.current_string.is_empty() {
            self.elements.push(ConcatenationBuilderElement::String(take(
                &mut self.current_string,
            )));
        }
        self.elements
            .push(ConcatenationBuilderElement::TokenStream(token_stream));
    }

    fn push_str(&mut self, value: &str) {
        self.current_string.push_str(value);
    }

    fn push_str_to_escape(&mut self, value: &str) {
        self.current_string.push('"');
        for c in value.chars() {
            match c {
                '\\' => self.current_string.push_str("\\\\"),
                '"' => self.current_string.push_str("\\\""),
                c => {
                    if c < char::from(32) {
                        panic!("ASCII chars below 32 are not allowed")
                    } else {
                        self.current_string.push(c);
                    }
                }
            }
        }
        self.current_string.push('"');
    }

    fn into_token_stream(self, pyo3_crate_path: &PyO3CratePath) -> TokenStream {
        let mut elements = self.elements;
        if !self.current_string.is_empty() {
            elements.push(ConcatenationBuilderElement::String(self.current_string));
        }

        if let [ConcatenationBuilderElement::String(string)] = elements.as_slice() {
            // We avoid the const_concat! macro if there is only a single string
            return string.to_token_stream();
        }

        quote! {
            #pyo3_crate_path::impl_::concat::const_concat!(#(#elements , )*)
        }
    }
}

enum ConcatenationBuilderElement {
    String(String),
    TokenStream(TokenStream),
}

impl ToTokens for ConcatenationBuilderElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::String(s) => s.to_tokens(tokens),
            Self::TokenStream(ts) => ts.to_tokens(tokens),
        }
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
