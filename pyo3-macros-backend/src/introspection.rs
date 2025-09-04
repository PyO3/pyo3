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
use crate::utils::{PyO3CratePath, TypeExt};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::mem::take;
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::visit_mut::{visit_type_mut, VisitMut};
use syn::{Attribute, Ident, ReturnType, Type, TypePath};

static GLOBAL_COUNTER_FOR_UNIQUE_NAMES: AtomicUsize = AtomicUsize::new(0);

#[allow(clippy::too_many_arguments)]
pub fn module_introspection_code<'a>(
    pyo3_crate_path: &PyO3CratePath,
    name: &str,
    members: impl IntoIterator<Item = &'a Ident>,
    members_cfg_attrs: impl IntoIterator<Item = &'a Vec<Attribute>>,
    incomplete: bool,
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
                        .map(|(member, attributes)| AttributedIntrospectionNode {
                            node: IntrospectionNode::IntrospectionId(Some(ident_to_type(member))),
                            attributes,
                        })
                        .collect(),
                ),
            ),
            ("incomplete", IntrospectionNode::Bool(incomplete)),
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

#[allow(clippy::too_many_arguments)]
pub fn function_introspection_code(
    pyo3_crate_path: &PyO3CratePath,
    ident: Option<&Ident>,
    name: &str,
    signature: &FunctionSignature<'_>,
    first_argument: Option<&'static str>,
    returns: ReturnType,
    decorators: impl IntoIterator<Item = String>,
    parent: Option<&Type>,
) -> TokenStream {
    let mut desc = HashMap::from([
        ("type", IntrospectionNode::String("function".into())),
        ("name", IntrospectionNode::String(name.into())),
        (
            "arguments",
            arguments_introspection_data(signature, first_argument, parent),
        ),
        (
            "returns",
            if let Some((_, returns)) = signature
                .attribute
                .as_ref()
                .and_then(|attribute| attribute.value.returns.as_ref())
            {
                IntrospectionNode::String(returns.to_python().into())
            } else {
                match returns {
                    ReturnType::Default => IntrospectionNode::String("None".into()),
                    ReturnType::Type(_, ty) => match *ty {
                        Type::Tuple(t) if t.elems.is_empty() => {
                            // () is converted to None in return types
                            IntrospectionNode::String("None".into())
                        }
                        mut ty => {
                            if let Some(class_type) = parent {
                                replace_self(&mut ty, class_type);
                            }
                            ty = ty.elide_lifetimes();
                            IntrospectionNode::OutputType {
                                rust_type: ty,
                                is_final: false,
                            }
                        }
                    },
                }
            },
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
        .map(|d| IntrospectionNode::String(d.into()).into())
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

pub fn attribute_introspection_code(
    pyo3_crate_path: &PyO3CratePath,
    parent: Option<&Type>,
    name: String,
    value: String,
    mut rust_type: Type,
    is_final: bool,
) -> TokenStream {
    let mut desc = HashMap::from([
        ("type", IntrospectionNode::String("attribute".into())),
        ("name", IntrospectionNode::String(name.into())),
        (
            "parent",
            IntrospectionNode::IntrospectionId(parent.map(Cow::Borrowed)),
        ),
    ]);
    if value == "..." {
        // We need to set a type, but not need to set the value to ..., all attributes have a value
        if let Some(parent) = parent {
            replace_self(&mut rust_type, parent);
        }
        rust_type = rust_type.elide_lifetimes();
        desc.insert(
            "annotation",
            IntrospectionNode::OutputType {
                rust_type,
                is_final,
            },
        );
    } else {
        desc.insert(
            "annotation",
            if is_final {
                // Type checkers can infer the type from the value because it's typing.Literal[value]
                // So, following stubs best practices, we only write typing.Final and not
                // typing.Final[typing.literal[value]]
                IntrospectionNode::String("typing.Final".into())
            } else {
                IntrospectionNode::OutputType {
                    rust_type,
                    is_final,
                }
            },
        );
        desc.insert("value", IntrospectionNode::String(value.into()));
    }
    IntrospectionNode::Map(desc).emit(pyo3_crate_path)
}

fn arguments_introspection_data<'a>(
    signature: &'a FunctionSignature<'a>,
    first_argument: Option<&'a str>,
    class_type: Option<&Type>,
) -> IntrospectionNode<'a> {
    let mut argument_desc = signature.arguments.iter().filter(|arg| {
        matches!(
            arg,
            FnArg::Regular(_) | FnArg::VarArgs(_) | FnArg::KwArgs(_)
        )
    });

    let mut posonlyargs = Vec::new();
    let mut args = Vec::new();
    let mut vararg = None;
    let mut kwonlyargs = Vec::new();
    let mut kwarg = None;

    if let Some(first_argument) = first_argument {
        posonlyargs.push(
            IntrospectionNode::Map(
                [("name", IntrospectionNode::String(first_argument.into()))].into(),
            )
            .into(),
        );
    }

    for (i, param) in signature
        .python_signature
        .positional_parameters
        .iter()
        .enumerate()
    {
        let arg_desc = if let Some(FnArg::Regular(arg_desc)) = argument_desc.next() {
            arg_desc
        } else {
            panic!("Less arguments than in python signature");
        };
        let arg = argument_introspection_data(param, arg_desc, class_type);
        if i < signature.python_signature.positional_only_parameters {
            posonlyargs.push(arg);
        } else {
            args.push(arg)
        }
    }

    if let Some(param) = &signature.python_signature.varargs {
        let Some(FnArg::VarArgs(arg_desc)) = argument_desc.next() else {
            panic!("Fewer arguments than in python signature");
        };
        let mut params = HashMap::from([("name", IntrospectionNode::String(param.into()))]);
        if let Some(annotation) = &arg_desc.annotation {
            params.insert("annotation", IntrospectionNode::String(annotation.into()));
        }
        vararg = Some(IntrospectionNode::Map(params));
    }

    for (param, _) in &signature.python_signature.keyword_only_parameters {
        let Some(FnArg::Regular(arg_desc)) = argument_desc.next() else {
            panic!("Less arguments than in python signature");
        };
        kwonlyargs.push(argument_introspection_data(param, arg_desc, class_type));
    }

    if let Some(param) = &signature.python_signature.kwargs {
        let Some(FnArg::KwArgs(arg_desc)) = argument_desc.next() else {
            panic!("Less arguments than in python signature");
        };
        let mut params = HashMap::from([("name", IntrospectionNode::String(param.into()))]);
        if let Some(annotation) = &arg_desc.annotation {
            params.insert("annotation", IntrospectionNode::String(annotation.into()));
        }
        kwarg = Some(IntrospectionNode::Map(params));
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
    class_type: Option<&Type>,
) -> AttributedIntrospectionNode<'a> {
    let mut params: HashMap<_, _> = [("name", IntrospectionNode::String(name.into()))].into();
    if desc.default_value.is_some() {
        params.insert(
            "default",
            IntrospectionNode::String(desc.default_value().into()),
        );
    }

    if let Some(annotation) = &desc.annotation {
        params.insert("annotation", IntrospectionNode::String(annotation.into()));
    } else if desc.from_py_with.is_none() {
        // If from_py_with is set we don't know anything on the input type
        if let Some(ty) = desc.option_wrapped_type {
            // Special case to properly generate a `T | None` annotation
            let mut ty = ty.clone();
            if let Some(class_type) = class_type {
                replace_self(&mut ty, class_type);
            }
            ty = ty.elide_lifetimes();
            params.insert(
                "annotation",
                IntrospectionNode::InputType {
                    rust_type: ty,
                    nullable: true,
                },
            );
        } else {
            let mut ty = desc.ty.clone();
            if let Some(class_type) = class_type {
                replace_self(&mut ty, class_type);
            }
            ty = ty.elide_lifetimes();
            params.insert(
                "annotation",
                IntrospectionNode::InputType {
                    rust_type: ty,
                    nullable: false,
                },
            );
        }
    }
    IntrospectionNode::Map(params).into()
}

enum IntrospectionNode<'a> {
    String(Cow<'a, str>),
    Bool(bool),
    IntrospectionId(Option<Cow<'a, Type>>),
    InputType { rust_type: Type, nullable: bool },
    OutputType { rust_type: Type, is_final: bool },
    Map(HashMap<&'static str, IntrospectionNode<'a>>),
    List(Vec<AttributedIntrospectionNode<'a>>),
}

impl IntrospectionNode<'_> {
    fn emit(self, pyo3_crate_path: &PyO3CratePath) -> TokenStream {
        let mut content = ConcatenationBuilder::default();
        self.add_to_serialization(&mut content, pyo3_crate_path);
        let content = content.into_token_stream(pyo3_crate_path);

        let static_name = format_ident!("PYO3_INTROSPECTION_0_{}", unique_element_id());
        // #[no_mangle] is required to make sure some linkers like Linux ones do not mangle the section name too.
        quote! {
            const _: () = {
                #[used]
                #[no_mangle]
                static #static_name: &'static [u8] = #content;
            };
        }
    }

    fn add_to_serialization(
        self,
        content: &mut ConcatenationBuilder,
        pyo3_crate_path: &PyO3CratePath,
    ) {
        match self {
            Self::String(string) => {
                content.push_str_to_escape(&string);
            }
            Self::Bool(value) => content.push_str(if value { "true" } else { "false" }),
            Self::IntrospectionId(ident) => {
                content.push_str("\"");
                content.push_tokens(if let Some(ident) = ident {
                    quote! { #ident::_PYO3_INTROSPECTION_ID.as_bytes() }
                } else {
                    quote! { _PYO3_INTROSPECTION_ID.as_bytes() }
                });
                content.push_str("\"");
            }
            Self::InputType {
                rust_type,
                nullable,
            } => {
                content.push_str("\"");
                content.push_tokens(quote! { <#rust_type as #pyo3_crate_path::impl_::extract_argument::PyFunctionArgument<false>>::INPUT_TYPE.as_bytes() });
                if nullable {
                    content.push_str(" | None");
                }
                content.push_str("\"");
            }
            Self::OutputType {
                rust_type,
                is_final,
            } => {
                content.push_str("\"");
                if is_final {
                    content.push_str("typing.Final[");
                }
                content.push_tokens(quote! { <#rust_type as #pyo3_crate_path::impl_::introspection::PyReturnType>::OUTPUT_TYPE.as_bytes() });
                if is_final {
                    content.push_str("]");
                }
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
                    value.add_to_serialization(content, pyo3_crate_path);
                }
                content.push_str("}");
            }
            Self::List(list) => {
                content.push_str("[");
                for (i, AttributedIntrospectionNode { node, attributes }) in
                    list.into_iter().enumerate()
                {
                    if attributes.is_empty() {
                        if i > 0 {
                            content.push_str(",");
                        }
                        node.add_to_serialization(content, pyo3_crate_path);
                    } else {
                        // We serialize the element to easily gate it behind the attributes
                        let mut nested_builder = ConcatenationBuilder::default();
                        if i > 0 {
                            nested_builder.push_str(",");
                        }
                        node.add_to_serialization(&mut nested_builder, pyo3_crate_path);
                        let nested_content = nested_builder.into_token_stream(pyo3_crate_path);
                        content.push_tokens(quote! { #(#attributes)* #nested_content });
                    }
                }
                content.push_str("]");
            }
        }
    }
}

struct AttributedIntrospectionNode<'a> {
    node: IntrospectionNode<'a>,
    attributes: &'a [Attribute],
}

impl<'a> From<IntrospectionNode<'a>> for AttributedIntrospectionNode<'a> {
    fn from(node: IntrospectionNode<'a>) -> Self {
        Self {
            node,
            attributes: &[],
        }
    }
}

#[derive(Default)]
pub struct ConcatenationBuilder {
    elements: Vec<ConcatenationBuilderElement>,
    current_string: String,
}

impl ConcatenationBuilder {
    pub fn push_tokens(&mut self, token_stream: TokenStream) {
        if !self.current_string.is_empty() {
            self.elements.push(ConcatenationBuilderElement::String(take(
                &mut self.current_string,
            )));
        }
        self.elements
            .push(ConcatenationBuilderElement::TokenStream(token_stream));
    }

    pub fn push_str(&mut self, value: &str) {
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

    pub fn into_token_stream(self, pyo3_crate_path: &PyO3CratePath) -> TokenStream {
        let mut elements = self.elements;
        if !self.current_string.is_empty() {
            elements.push(ConcatenationBuilderElement::String(self.current_string));
        }

        if let [ConcatenationBuilderElement::String(string)] = elements.as_slice() {
            // We avoid the const_concat! macro if there is only a single string
            return quote! { #string.as_bytes() };
        }

        quote! {
            {
                const PIECES: &[&[u8]] = &[#(#elements , )*];
                &#pyo3_crate_path::impl_::concat::combine_to_array::<{
                    #pyo3_crate_path::impl_::concat::combined_len(PIECES)
                }>(PIECES)
            }
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
            Self::String(s) => quote! { #s.as_bytes() }.to_tokens(tokens),
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

pub fn unique_element_id() -> u64 {
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

// Replace Self in types with the given type
fn replace_self(ty: &mut Type, self_target: &Type) {
    struct SelfReplacementVisitor<'a> {
        self_target: &'a Type,
    }

    impl VisitMut for SelfReplacementVisitor<'_> {
        fn visit_type_mut(&mut self, ty: &mut Type) {
            if let syn::Type::Path(type_path) = ty {
                if type_path.qself.is_none()
                    && type_path.path.segments.len() == 1
                    && type_path.path.segments[0].ident == "Self"
                    && type_path.path.segments[0].arguments.is_empty()
                {
                    // It is Self
                    *ty = self.self_target.clone();
                    return;
                }
            }
            visit_type_mut(self, ty);
        }
    }

    SelfReplacementVisitor { self_target }.visit_type_mut(ty);
}
