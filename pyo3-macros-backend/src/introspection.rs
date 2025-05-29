//! Generates introspection data i.e. JSON strings in the .pyo3i0 section.
//!
//! There is a JSON per PyO3 proc macro (pyclass, pymodule, pyfunction...).
//!
//! These JSON blobs can refer to each others via the _PYO3_INTROSPECTION_ID constants
//! providing unique ids for each element.
//!
//! The JSON blobs format must be synchronized with the `pyo3_introspection::introspection.rs::Chunk`
//! type that is used to parse them.

use crate::method::{FnArg, RegularArg};
use crate::pyfunction::FunctionSignature;
use crate::utils::PyO3CratePath;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::mem::take;
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::ext::IdentExt;
use syn::{Attribute, Ident, Type, TypePath};

static GLOBAL_COUNTER_FOR_UNIQUE_NAMES: AtomicUsize = AtomicUsize::new(0);

pub fn module_introspection_code<'a>(
    pyo3_crate_path: &PyO3CratePath,
    name: &str,
    members: impl IntoIterator<Item = &'a Ident>,
    members_cfg_attrs: impl IntoIterator<Item = &'a Vec<Attribute>>,
    consts: impl IntoIterator<Item = &'a Ident>,
    consts_values: impl IntoIterator<Item = &'a String>,
    consts_cfg_attrs: impl IntoIterator<Item = &'a Vec<Attribute>>,
) -> TokenStream {
    IntrospectionNode::Map(
        [
            ("type", IntrospectionNode::String("module".into())),
            ("id", IntrospectionNode::IntrospectionId(None)),
            ("name", IntrospectionNode::String(name.into())),
            (
                "members",
                IntrospectionNode::List(
                    members
                        .into_iter()
                        .zip(members_cfg_attrs)
                        .filter_map(|(member, attributes)| {
                            if attributes.is_empty() {
                                Some(IntrospectionNode::IntrospectionId(Some(ident_to_type(
                                    member,
                                ))))
                            } else {
                                None // TODO: properly interpret cfg attributes
                            }
                        })
                        .collect(),
                ),
            ),
            (
                "consts",
                IntrospectionNode::List(
                    consts
                        .into_iter()
                        .zip(consts_values)
                        .zip(consts_cfg_attrs)
                        .filter_map(|((ident, value), attributes)| {
                            if attributes.is_empty() {
                                Some(const_introspection_code(ident, value))
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
    .emit(pyo3_crate_path)
}

pub fn class_introspection_code(
    pyo3_crate_path: &PyO3CratePath,
    ident: &Ident,
    name: &str,
) -> TokenStream {
    IntrospectionNode::Map(
        [
            ("type", IntrospectionNode::String("class".into())),
            (
                "id",
                IntrospectionNode::IntrospectionId(Some(ident_to_type(ident))),
            ),
            ("name", IntrospectionNode::String(name.into())),
        ]
        .into(),
    )
    .emit(pyo3_crate_path)
}

pub fn function_introspection_code(
    pyo3_crate_path: &PyO3CratePath,
    ident: Option<&Ident>,
    name: &str,
    signature: &FunctionSignature<'_>,
    first_argument: Option<&'static str>,
    decorators: impl IntoIterator<Item = String>,
    parent: Option<&Type>,
) -> TokenStream {
    let mut desc = HashMap::from([
        ("type", IntrospectionNode::String("function".into())),
        ("name", IntrospectionNode::String(name.into())),
        (
            "arguments",
            arguments_introspection_data(signature, first_argument),
        ),
    ]);
    if let Some(ident) = ident {
        desc.insert(
            "id",
            IntrospectionNode::IntrospectionId(Some(ident_to_type(ident))),
        );
    }
    let decorators = decorators
        .into_iter()
        .map(|d| IntrospectionNode::String(d.into()))
        .collect::<Vec<_>>();
    if !decorators.is_empty() {
        desc.insert("decorators", IntrospectionNode::List(decorators));
    }
    if let Some(parent) = parent {
        desc.insert(
            "parent",
            IntrospectionNode::IntrospectionId(Some(Cow::Borrowed(parent))),
        );
    }
    IntrospectionNode::Map(desc).emit(pyo3_crate_path)
}

fn const_introspection_code<'a>(ident: &'a Ident, value: &'a String) -> IntrospectionNode<'a> {
    IntrospectionNode::Map(
        [
            ("type", IntrospectionNode::String("const".into())),
            (
                "name",
                IntrospectionNode::String(ident.unraw().to_string().into()),
            ),
            ("value", IntrospectionNode::String(value.into())),
        ]
        .into(),
    )
}

fn arguments_introspection_data<'a>(
    signature: &'a FunctionSignature<'a>,
    first_argument: Option<&'a str>,
) -> IntrospectionNode<'a> {
    let mut argument_desc = signature.arguments.iter().filter_map(|arg| {
        if let FnArg::Regular(arg) = arg {
            Some(arg)
        } else {
            None
        }
    });

    let mut posonlyargs = Vec::new();
    let mut args = Vec::new();
    let mut vararg = None;
    let mut kwonlyargs = Vec::new();
    let mut kwarg = None;

    if let Some(first_argument) = first_argument {
        posonlyargs.push(IntrospectionNode::Map(
            [("name", IntrospectionNode::String(first_argument.into()))].into(),
        ));
    }

    for (i, param) in signature
        .python_signature
        .positional_parameters
        .iter()
        .enumerate()
    {
        let arg_desc = if let Some(arg_desc) = argument_desc.next() {
            arg_desc
        } else {
            panic!("Less arguments than in python signature");
        };
        let arg = argument_introspection_data(param, arg_desc);
        if i < signature.python_signature.positional_only_parameters {
            posonlyargs.push(arg);
        } else {
            args.push(arg)
        }
    }

    if let Some(param) = &signature.python_signature.varargs {
        vararg = Some(IntrospectionNode::Map(
            [("name", IntrospectionNode::String(param.into()))].into(),
        ));
    }

    for (param, _) in &signature.python_signature.keyword_only_parameters {
        let arg_desc = if let Some(arg_desc) = argument_desc.next() {
            arg_desc
        } else {
            panic!("Less arguments than in python signature");
        };
        kwonlyargs.push(argument_introspection_data(param, arg_desc));
    }

    if let Some(param) = &signature.python_signature.kwargs {
        kwarg = Some(IntrospectionNode::Map(
            [
                ("name", IntrospectionNode::String(param.into())),
                ("kind", IntrospectionNode::String("VAR_KEYWORD".into())),
            ]
            .into(),
        ));
    }

    let mut map = HashMap::new();
    if !posonlyargs.is_empty() {
        map.insert("posonlyargs", IntrospectionNode::List(posonlyargs));
    }
    if !args.is_empty() {
        map.insert("args", IntrospectionNode::List(args));
    }
    if let Some(vararg) = vararg {
        map.insert("vararg", vararg);
    }
    if !kwonlyargs.is_empty() {
        map.insert("kwonlyargs", IntrospectionNode::List(kwonlyargs));
    }
    if let Some(kwarg) = kwarg {
        map.insert("kwarg", kwarg);
    }
    IntrospectionNode::Map(map)
}

fn argument_introspection_data<'a>(
    name: &'a str,
    desc: &'a RegularArg<'_>,
) -> IntrospectionNode<'a> {
    let mut params: HashMap<_, _> = [("name", IntrospectionNode::String(name.into()))].into();
    if desc.default_value.is_some() {
        params.insert(
            "default",
            IntrospectionNode::String(desc.default_value().into()),
        );
    }
    IntrospectionNode::Map(params)
}

enum IntrospectionNode<'a> {
    String(Cow<'a, str>),
    IntrospectionId(Option<Cow<'a, Type>>),
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
                content.push_str_to_escape(&string);
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

/// Generates a new unique identifier for linking introspection objects together
pub fn introspection_id_const() -> TokenStream {
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

fn ident_to_type(ident: &Ident) -> Cow<'static, Type> {
    Cow::Owned(
        TypePath {
            path: ident.clone().into(),
            qself: None,
        }
        .into(),
    )
}
