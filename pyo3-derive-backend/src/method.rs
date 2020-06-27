// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::pyfunction::Argument;
use crate::pyfunction::{parse_name_attribute, PyFunctionAttr};
use crate::utils;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::{quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;

#[derive(Clone, PartialEq, Debug)]
pub struct FnArg<'a> {
    pub name: &'a syn::Ident,
    pub by_ref: &'a Option<syn::token::Ref>,
    pub mutability: &'a Option<syn::token::Mut>,
    pub ty: &'a syn::Type,
    pub optional: Option<&'a syn::Type>,
    pub py: bool,
    pub reference: bool,
}

#[derive(Clone, PartialEq, Debug, Copy, Eq)]
pub enum MethodTypeAttribute {
    /// #[new]
    New,
    /// #[call]
    Call,
    /// #[classmethod]
    ClassMethod,
    /// #[classattr]
    ClassAttribute,
    /// #[staticmethod]
    StaticMethod,
    /// #[getter]
    Getter,
    /// #[setter]
    Setter,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FnType {
    Getter(SelfType),
    Setter(SelfType),
    Fn(SelfType),
    FnCall(SelfType),
    FnNew,
    FnClass,
    FnStatic,
    ClassAttribute,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SelfType {
    Receiver { mutable: bool },
    TryFromPyCell(syn::Type),
}

impl SelfType {
    pub fn receiver(&self, cls: &syn::Type) -> TokenStream {
        match self {
            SelfType::Receiver { mutable: false } => {
                quote! {
                    let _cell = _py.from_borrowed_ptr::<pyo3::PyCell<#cls>>(_slf);
                    let _ref = _cell.try_borrow()?;
                    let _slf = &_ref;
                }
            }
            SelfType::Receiver { mutable: true } => {
                quote! {
                    let _cell = _py.from_borrowed_ptr::<pyo3::PyCell<#cls>>(_slf);
                    let mut _ref = _cell.try_borrow_mut()?;
                    let _slf = &mut _ref;
                }
            }
            SelfType::TryFromPyCell(ty) => {
                quote_spanned! { ty.span() =>
                    let _cell = _py.from_borrowed_ptr::<pyo3::PyCell<#cls>>(_slf);
                    let _slf = std::convert::TryFrom::try_from(_cell)?;
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct FnSpec<'a> {
    pub tp: FnType,
    // Rust function name
    pub name: &'a syn::Ident,
    // Wrapped python name. This should not have any leading r#.
    // r# can be removed by syn::ext::IdentExt::unraw()
    pub python_name: syn::Ident,
    pub attrs: Vec<Argument>,
    pub args: Vec<FnArg<'a>>,
    pub output: syn::Type,
    pub doc: syn::LitStr,
}

pub fn get_return_info(output: &syn::ReturnType) -> syn::Type {
    match output {
        syn::ReturnType::Default => syn::Type::Infer(syn::parse_quote! {_}),
        syn::ReturnType::Type(_, ref ty) => *ty.clone(),
    }
}

pub fn parse_method_receiver(arg: &syn::FnArg) -> syn::Result<SelfType> {
    match arg {
        syn::FnArg::Receiver(recv) => Ok(SelfType::Receiver {
            mutable: recv.mutability.is_some(),
        }),
        syn::FnArg::Typed(syn::PatType { ref ty, .. }) => {
            Ok(SelfType::TryFromPyCell(ty.as_ref().clone()))
        }
    }
}

impl<'a> FnSpec<'a> {
    /// Parser function signature and function attributes
    pub fn parse(
        sig: &'a syn::Signature,
        meth_attrs: &mut Vec<syn::Attribute>,
        allow_custom_name: bool,
    ) -> syn::Result<FnSpec<'a>> {
        let name = &sig.ident;
        let MethodAttributes {
            ty: fn_type_attr,
            args: fn_attrs,
            mut python_name,
        } = parse_method_attributes(meth_attrs, allow_custom_name)?;

        let mut arguments = Vec::new();
        let mut inputs_iter = sig.inputs.iter();

        let mut parse_receiver = |msg: &'static str| {
            inputs_iter
                .next()
                .ok_or_else(|| syn::Error::new_spanned(sig, msg))
                .and_then(parse_method_receiver)
        };

        // strip get_ or set_
        let strip_fn_name = |prefix: &'static str| {
            let ident = sig.ident.unraw().to_string();
            if ident.starts_with(prefix) {
                Some(syn::Ident::new(&ident[prefix.len()..], ident.span()))
            } else {
                None
            }
        };

        // Parse receiver & function type for various method types
        let fn_type = match fn_type_attr {
            Some(MethodTypeAttribute::StaticMethod) => FnType::FnStatic,
            Some(MethodTypeAttribute::ClassAttribute) => {
                if !sig.inputs.is_empty() {
                    return Err(syn::Error::new_spanned(
                        name,
                        "Class attribute methods cannot take arguments",
                    ));
                }
                FnType::ClassAttribute
            }
            Some(MethodTypeAttribute::New) => FnType::FnNew,
            Some(MethodTypeAttribute::ClassMethod) => {
                // Skip first argument for classmethod - always &PyType
                let _ = inputs_iter.next();
                FnType::FnClass
            }
            Some(MethodTypeAttribute::Call) => {
                FnType::FnCall(parse_receiver("expected receiver for #[call]")?)
            }
            Some(MethodTypeAttribute::Getter) => {
                // Strip off "get_" prefix if needed
                if python_name.is_none() {
                    python_name = strip_fn_name("get_");
                }

                FnType::Getter(parse_receiver("expected receiver for #[getter]")?)
            }
            Some(MethodTypeAttribute::Setter) => {
                // Strip off "set_" prefix if needed
                if python_name.is_none() {
                    python_name = strip_fn_name("set_");
                }

                FnType::Setter(parse_receiver("expected receiver for #[setter]")?)
            }
            None => FnType::Fn(parse_receiver(
                "Static method needs #[staticmethod] attribute",
            )?),
        };

        // parse rest of arguments
        for input in inputs_iter {
            match input {
                syn::FnArg::Receiver(recv) => {
                    return Err(syn::Error::new_spanned(
                        recv,
                        "Unexpected receiver for method",
                    ));
                }
                syn::FnArg::Typed(syn::PatType {
                    ref pat, ref ty, ..
                }) => {
                    let (ident, by_ref, mutability) = match **pat {
                        syn::Pat::Ident(syn::PatIdent {
                            ref ident,
                            ref by_ref,
                            ref mutability,
                            ..
                        }) => (ident, by_ref, mutability),
                        _ => {
                            return Err(syn::Error::new_spanned(pat, "unsupported argument"));
                        }
                    };

                    let py = crate::utils::if_type_is_python(ty);

                    let opt = check_ty_optional(ty);
                    arguments.push(FnArg {
                        name: ident,
                        by_ref,
                        mutability,
                        ty,
                        optional: opt,
                        py,
                        reference: is_ref(name, ty),
                    });
                }
            }
        }

        let ty = get_return_info(&sig.output);
        let python_name = python_name.unwrap_or_else(|| name.unraw());

        let mut parse_erroneous_text_signature = |error_msg: &str| {
            // try to parse anyway to give better error messages
            if let Some(text_signature) =
                utils::parse_text_signature_attrs(meth_attrs, &python_name)?
            {
                Err(syn::Error::new_spanned(text_signature, error_msg))
            } else {
                Ok(None)
            }
        };

        let text_signature = match &fn_type {
            FnType::Fn(_) | FnType::FnClass | FnType::FnStatic => {
                utils::parse_text_signature_attrs(&mut *meth_attrs, name)?
            }
            FnType::FnNew => parse_erroneous_text_signature(
                "text_signature not allowed on __new__; if you want to add a signature on \
                 __new__, put it on the struct definition instead",
            )?,
            FnType::FnCall(_) | FnType::Getter(_) | FnType::Setter(_) | FnType::ClassAttribute => {
                parse_erroneous_text_signature("text_signature not allowed with this attribute")?
            }
        };

        let doc = utils::get_doc(&meth_attrs, text_signature, true)?;

        Ok(FnSpec {
            tp: fn_type,
            name,
            python_name,
            attrs: fn_attrs,
            args: arguments,
            output: ty,
            doc,
        })
    }

    pub fn is_args(&self, name: &syn::Ident) -> bool {
        for s in self.attrs.iter() {
            if let Argument::VarArgs(ref path) = s {
                return path.is_ident(name);
            }
        }
        false
    }

    pub fn is_kwargs(&self, name: &syn::Ident) -> bool {
        for s in self.attrs.iter() {
            if let Argument::KeywordArgs(ref path) = s {
                return path.is_ident(name);
            }
        }
        false
    }

    pub fn default_value(&self, name: &syn::Ident) -> Option<TokenStream> {
        for s in self.attrs.iter() {
            match *s {
                Argument::Arg(ref path, ref opt) => {
                    if path.is_ident(name) {
                        if let Some(ref val) = opt {
                            let i: syn::Expr = syn::parse_str(&val).unwrap();
                            return Some(i.into_token_stream());
                        }
                    }
                }
                Argument::Kwarg(ref path, ref opt) => {
                    if path.is_ident(name) {
                        let i: syn::Expr = syn::parse_str(&opt).unwrap();
                        return Some(quote!(#i));
                    }
                }
                _ => (),
            }
        }
        None
    }

    pub fn is_kw_only(&self, name: &syn::Ident) -> bool {
        for s in self.attrs.iter() {
            if let Argument::Kwarg(ref path, _) = s {
                if path.is_ident(name) {
                    return true;
                }
            }
        }
        false
    }
}

pub fn is_ref(name: &syn::Ident, ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(_) => return true,
        syn::Type::Path(syn::TypePath { ref path, .. }) => {
            if let Some(segment) = path.segments.last() {
                if "Option" == segment.ident.to_string().as_str() {
                    match segment.arguments {
                        syn::PathArguments::AngleBracketed(ref params) => {
                            if params.args.len() != 1 {
                                panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                       name,
                                       ty,
                                       path);
                            }
                            let last = &params.args[params.args.len() - 1];
                            if let syn::GenericArgument::Type(syn::Type::Reference(_)) = last {
                                return true;
                            }
                        }
                        _ => {
                            panic!(
                                "argument type is not supported by python method: {:?} ({:?}) {:?}",
                                name, ty, path
                            );
                        }
                    }
                }
            }
        }
        _ => (),
    }
    false
}

pub(crate) fn check_ty_optional(ty: &syn::Type) -> Option<&syn::Type> {
    let path = match ty {
        syn::Type::Path(syn::TypePath { ref path, .. }) => path,
        _ => return None,
    };
    let seg = path.segments.last().filter(|s| s.ident == "Option")?;
    match seg.arguments {
        syn::PathArguments::AngleBracketed(ref params) => match params.args.first() {
            Some(syn::GenericArgument::Type(ref ty)) => Some(ty),
            _ => None,
        },
        _ => None,
    }
}

#[derive(Clone, PartialEq, Debug)]
struct MethodAttributes {
    ty: Option<MethodTypeAttribute>,
    args: Vec<Argument>,
    python_name: Option<syn::Ident>,
}

fn parse_method_attributes(
    attrs: &mut Vec<syn::Attribute>,
    allow_custom_name: bool,
) -> syn::Result<MethodAttributes> {
    let mut new_attrs = Vec::new();
    let mut args = Vec::new();
    let mut ty: Option<MethodTypeAttribute> = None;
    let mut property_name = None;

    macro_rules! set_ty {
        ($new_ty:expr, $ident:expr) => {
            if ty.replace($new_ty).is_some() {
                return Err(syn::Error::new_spanned(
                    $ident,
                    "Cannot specify a second method type",
                ));
            }
        };
    }

    for attr in attrs.iter() {
        match attr.parse_meta()? {
            syn::Meta::Path(ref name) => {
                if name.is_ident("new") || name.is_ident("__new__") {
                    set_ty!(MethodTypeAttribute::New, name);
                } else if name.is_ident("init") || name.is_ident("__init__") {
                    return Err(syn::Error::new_spanned(
                        name,
                        "#[init] is disabled since PyO3 0.9.0",
                    ));
                } else if name.is_ident("call") || name.is_ident("__call__") {
                    set_ty!(MethodTypeAttribute::Call, name);
                } else if name.is_ident("classmethod") {
                    set_ty!(MethodTypeAttribute::ClassMethod, name);
                } else if name.is_ident("staticmethod") {
                    set_ty!(MethodTypeAttribute::StaticMethod, name);
                } else if name.is_ident("classattr") {
                    set_ty!(MethodTypeAttribute::ClassAttribute, name);
                } else if name.is_ident("setter") || name.is_ident("getter") {
                    if let syn::AttrStyle::Inner(_) = attr.style {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "Inner style attribute is not supported for setter and getter",
                        ));
                    }
                    if name.is_ident("setter") {
                        set_ty!(MethodTypeAttribute::Setter, name);
                    } else {
                        set_ty!(MethodTypeAttribute::Getter, name);
                    }
                } else {
                    new_attrs.push(attr.clone())
                }
            }
            syn::Meta::List(syn::MetaList {
                ref path,
                ref nested,
                ..
            }) => {
                if path.is_ident("new") {
                    set_ty!(MethodTypeAttribute::New, path);
                } else if path.is_ident("init") {
                    return Err(syn::Error::new_spanned(
                        path,
                        "#[init] is disabled since PyO3 0.9.0",
                    ));
                } else if path.is_ident("call") {
                    set_ty!(MethodTypeAttribute::Call, path);
                } else if path.is_ident("setter") || path.is_ident("getter") {
                    if let syn::AttrStyle::Inner(_) = attr.style {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "Inner style attribute is not supported for setter and getter",
                        ));
                    }
                    if nested.len() != 1 {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "setter/getter requires one value",
                        ));
                    }

                    if path.is_ident("setter") {
                        set_ty!(MethodTypeAttribute::Setter, path);
                    } else {
                        set_ty!(MethodTypeAttribute::Getter, path);
                    };

                    property_name = match nested.first().unwrap() {
                        syn::NestedMeta::Meta(syn::Meta::Path(ref w)) if w.segments.len() == 1 => {
                            Some(w.segments[0].ident.clone())
                        }
                        syn::NestedMeta::Lit(ref lit) => match *lit {
                            syn::Lit::Str(ref s) => Some(s.parse()?),
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    lit,
                                    "setter/getter attribute requires str value",
                                ))
                            }
                        },
                        _ => {
                            return Err(syn::Error::new_spanned(
                                nested.first().unwrap(),
                                "expected ident or string literal for property name",
                            ))
                        }
                    };
                } else if path.is_ident("args") {
                    let attrs = PyFunctionAttr::from_meta(nested)?;
                    args.extend(attrs.arguments)
                } else {
                    new_attrs.push(attr.clone())
                }
            }
            syn::Meta::NameValue(_) => new_attrs.push(attr.clone()),
        }
    }

    attrs.clear();
    attrs.extend(new_attrs);

    let python_name = if allow_custom_name {
        parse_method_name_attribute(ty.as_ref(), attrs, property_name)?
    } else {
        property_name
    };

    Ok(MethodAttributes {
        ty,
        args,
        python_name,
    })
}

fn parse_method_name_attribute(
    ty: Option<&MethodTypeAttribute>,
    attrs: &mut Vec<syn::Attribute>,
    property_name: Option<syn::Ident>,
) -> syn::Result<Option<syn::Ident>> {
    use MethodTypeAttribute::*;
    let name = parse_name_attribute(attrs)?;

    // Reject some invalid combinations
    if let (Some(name), Some(ty)) = (&name, ty) {
        match ty {
            New | Call | Getter | Setter => {
                return Err(syn::Error::new_spanned(
                    name,
                    "name not allowed with this method type",
                ))
            }
            _ => {}
        }
    }

    // Thanks to check above we can be sure that this generates the right python name
    Ok(match ty {
        Some(New) => Some(syn::Ident::new("__new__", proc_macro2::Span::call_site())),
        Some(Call) => Some(syn::Ident::new("__call__", proc_macro2::Span::call_site())),
        Some(Getter) | Some(Setter) => property_name,
        _ => name,
    })
}
