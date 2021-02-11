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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum SelfType {
    Receiver { mutable: bool },
    TryFromPyCell(proc_macro2::Span),
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
            SelfType::TryFromPyCell(span) => {
                quote_spanned! { *span =>
                    let _cell = _py.from_borrowed_ptr::<pyo3::PyCell<#cls>>(_slf);
                    #[allow(clippy::useless_conversion)]  // In case _slf is PyCell<Self>
                    let _slf = std::convert::TryFrom::try_from(_cell)?;
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
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
        syn::ReturnType::Type(_, ty) => *ty.clone(),
    }
}

pub fn parse_method_receiver(arg: &syn::FnArg) -> syn::Result<SelfType> {
    match arg {
        syn::FnArg::Receiver(recv) => Ok(SelfType::Receiver {
            mutable: recv.mutability.is_some(),
        }),
        syn::FnArg::Typed(syn::PatType { ty, .. }) => Ok(SelfType::TryFromPyCell(ty.span())),
    }
}

impl<'a> FnSpec<'a> {
    /// Parser function signature and function attributes
    #[allow(clippy::manual_strip)] // for strip_prefix replacement supporting rust < 1.45
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
                .ok_or_else(|| err_spanned!(sig.span() => msg))
                .and_then(parse_method_receiver)
        };

        // strip get_ or set_
        let strip_fn_name = |prefix: &'static str| {
            let ident = name.unraw().to_string();
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
                ensure_spanned!(
                    sig.inputs.is_empty(),
                    sig.inputs.span() => "class attribute methods cannot take arguments"
                );
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
                "static method needs #[staticmethod] attribute",
            )?),
        };

        // parse rest of arguments
        for input in inputs_iter {
            match input {
                syn::FnArg::Receiver(recv) => {
                    bail_spanned!(recv.span() => "unexpected receiver for method")
                }
                syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => {
                    let (ident, by_ref, mutability) = match &**pat {
                        syn::Pat::Ident(syn::PatIdent {
                            ident,
                            by_ref,
                            mutability,
                            ..
                        }) => (ident, by_ref, mutability),
                        _ => bail_spanned!(pat.span() => "unsupported argument"),
                    };

                    arguments.push(FnArg {
                        name: ident,
                        by_ref,
                        mutability,
                        ty,
                        optional: utils::option_type_argument(ty),
                        py: utils::is_python(ty),
                    });
                }
            }
        }

        let ty = get_return_info(&sig.output);
        let python_name = python_name.as_ref().unwrap_or(name).unraw();

        let mut parse_erroneous_text_signature = |error_msg: &str| {
            // try to parse anyway to give better error messages
            if let Some(text_signature) =
                utils::parse_text_signature_attrs(meth_attrs, &python_name)?
            {
                bail_spanned!(text_signature.span() => error_msg)
            } else {
                Ok(None)
            }
        };

        let text_signature = match &fn_type {
            FnType::Fn(_) | FnType::FnClass | FnType::FnStatic => {
                utils::parse_text_signature_attrs(&mut *meth_attrs, &python_name)?
            }
            FnType::FnNew => parse_erroneous_text_signature(
                "text_signature not allowed on __new__; if you want to add a signature on \
                 __new__, put it on the struct definition instead",
            )?,
            FnType::FnCall(_) | FnType::Getter(_) | FnType::Setter(_) | FnType::ClassAttribute => {
                parse_erroneous_text_signature("text_signature not allowed with this method type")?
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
            if let Argument::VarArgs(path) = s {
                return path.is_ident(name);
            }
        }
        false
    }

    pub fn is_kwargs(&self, name: &syn::Ident) -> bool {
        for s in self.attrs.iter() {
            if let Argument::KeywordArgs(path) = s {
                return path.is_ident(name);
            }
        }
        false
    }

    pub fn default_value(&self, name: &syn::Ident) -> Option<TokenStream> {
        for s in self.attrs.iter() {
            match s {
                Argument::Arg(path, opt) | Argument::Kwarg(path, opt) => {
                    if path.is_ident(name) {
                        if let Some(val) = opt {
                            let i: syn::Expr = syn::parse_str(&val).unwrap();
                            return Some(i.into_token_stream());
                        }
                    }
                }
                _ => (),
            }
        }
        None
    }

    pub fn is_kw_only(&self, name: &syn::Ident) -> bool {
        for s in self.attrs.iter() {
            if let Argument::Kwarg(path, _) = s {
                if path.is_ident(name) {
                    return true;
                }
            }
        }
        false
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
            ensure_spanned!(
               ty.replace($new_ty).is_none(),
               $ident.span() => "cannot specify a second method type"
            );
        };
    }

    for attr in attrs.drain(..) {
        match attr.parse_meta()? {
            syn::Meta::Path(name) => {
                if name.is_ident("new") || name.is_ident("__new__") {
                    set_ty!(MethodTypeAttribute::New, name);
                } else if name.is_ident("init") || name.is_ident("__init__") {
                    bail_spanned!(name.span() => "#[init] is disabled since PyO3 0.9.0");
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
                        bail_spanned!(
                            attr.span() => "inner attribute is not supported for setter and getter"
                        );
                    }
                    if name.is_ident("setter") {
                        set_ty!(MethodTypeAttribute::Setter, name);
                    } else {
                        set_ty!(MethodTypeAttribute::Getter, name);
                    }
                } else {
                    new_attrs.push(attr)
                }
            }
            syn::Meta::List(syn::MetaList {
                path, mut nested, ..
            }) => {
                if path.is_ident("new") {
                    set_ty!(MethodTypeAttribute::New, path);
                } else if path.is_ident("init") {
                    bail_spanned!(path.span() => "#[init] is disabled since PyO3 0.9.0");
                } else if path.is_ident("call") {
                    set_ty!(MethodTypeAttribute::Call, path);
                } else if path.is_ident("setter") || path.is_ident("getter") {
                    if let syn::AttrStyle::Inner(_) = attr.style {
                        bail_spanned!(
                            attr.span() => "inner attribute is not supported for setter and getter"
                        );
                    }
                    ensure_spanned!(
                        nested.len() == 1,
                        attr.span() => "setter/getter requires one value"
                    );

                    if path.is_ident("setter") {
                        set_ty!(MethodTypeAttribute::Setter, path);
                    } else {
                        set_ty!(MethodTypeAttribute::Getter, path);
                    };

                    property_name = match nested.pop().unwrap().into_value() {
                        syn::NestedMeta::Meta(syn::Meta::Path(w)) if w.segments.len() == 1 => {
                            Some(w.segments[0].ident.clone())
                        }
                        syn::NestedMeta::Lit(lit) => match lit {
                            syn::Lit::Str(s) => Some(s.parse()?),
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
                    let attrs = PyFunctionAttr::from_meta(&nested)?;
                    args.extend(attrs.arguments)
                } else {
                    new_attrs.push(attr)
                }
            }
            syn::Meta::NameValue(_) => new_attrs.push(attr),
        }
    }

    *attrs = new_attrs;

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
        if let New | Call | Getter | Setter = ty {
            bail_spanned!(name.span() => "name not allowed with this method type");
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
