// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::pyfunction::Argument;
use crate::pyfunction::{parse_name_attribute, PyFunctionAttr};
use crate::utils;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
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

#[derive(Clone, PartialEq, Debug)]
pub enum FnType {
    Getter,
    Setter,
    Fn,
    FnNew,
    FnCall,
    FnClass,
    FnStatic,
    /// For methods taht have `self_: &PyCell<Self>` instead of self receiver
    PySelfRef(syn::TypeReference),
    /// For methods taht have `self_: PyRef<Self>` or `PyRefMut<Self>` instead of self receiver
    PySelfPath(syn::TypePath),
}

#[derive(Clone, PartialEq, Debug)]
pub struct FnSpec<'a> {
    pub tp: FnType,
    pub self_: Option<bool>,
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

impl<'a> FnSpec<'a> {
    /// Generate the code for borrowing self
    pub(crate) fn borrow_self(&self) -> TokenStream {
        let is_mut = self
            .self_
            .expect("impl_borrow_self is called for non-self fn");
        crate::utils::borrow_self(is_mut)
    }

    /// Parser function signature and function attributes
    pub fn parse(
        sig: &'a syn::Signature,
        meth_attrs: &mut Vec<syn::Attribute>,
        allow_custom_name: bool,
    ) -> syn::Result<FnSpec<'a>> {
        let name = &sig.ident;
        let MethodAttributes {
            ty: mut fn_type,
            args: fn_attrs,
            mut python_name,
        } = parse_method_attributes(meth_attrs, allow_custom_name)?;

        let mut self_ = None;
        let mut arguments = Vec::new();
        for input in sig.inputs.iter() {
            match input {
                syn::FnArg::Receiver(recv) => {
                    self_ = Some(recv.mutability.is_some());
                }
                syn::FnArg::Typed(syn::PatType {
                    ref pat, ref ty, ..
                }) => {
                    // skip first argument (cls)
                    if fn_type == FnType::FnClass && self_.is_none() {
                        self_ = Some(false);
                        continue;
                    }

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

        if fn_type == FnType::Fn && self_.is_none() {
            if arguments.is_empty() {
                return Err(syn::Error::new_spanned(
                    name,
                    "Static method needs #[staticmethod] attribute",
                ));
            }
            fn_type = match arguments.remove(0).ty {
                syn::Type::Reference(r) => FnType::PySelfRef(replace_self_in_ref(r)?),
                syn::Type::Path(p) => FnType::PySelfPath(replace_self_in_path(p)),
                x => return Err(syn::Error::new_spanned(x, "Invalid type as custom self")),
            };
        }

        // "Tweak" getter / setter names: strip off set_ and get_ if needed
        if let FnType::Getter | FnType::Setter = &fn_type {
            if python_name.is_none() {
                let prefix = match &fn_type {
                    FnType::Getter => "get_",
                    FnType::Setter => "set_",
                    _ => unreachable!(),
                };

                let ident = sig.ident.unraw().to_string();
                if ident.starts_with(prefix) {
                    python_name = Some(syn::Ident::new(&ident[prefix.len()..], ident.span()))
                }
            }
        }

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
            FnType::Fn
            | FnType::PySelfRef(_)
            | FnType::PySelfPath(_)
            | FnType::FnClass
            | FnType::FnStatic => utils::parse_text_signature_attrs(&mut *meth_attrs, name)?,
            FnType::FnNew => parse_erroneous_text_signature(
                "text_signature not allowed on __new__; if you want to add a signature on \
                 __new__, put it on the struct definition instead",
            )?,
            FnType::FnCall | FnType::Getter | FnType::Setter => {
                parse_erroneous_text_signature("text_signature not allowed with this attribute")?
            }
        };

        let doc = utils::get_doc(&meth_attrs, text_signature, true)?;

        Ok(FnSpec {
            tp: fn_type,
            self_,
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
    ty: FnType,
    args: Vec<Argument>,
    python_name: Option<syn::Ident>,
}

fn parse_method_attributes(
    attrs: &mut Vec<syn::Attribute>,
    allow_custom_name: bool,
) -> syn::Result<MethodAttributes> {
    let mut new_attrs = Vec::new();
    let mut args = Vec::new();
    let mut res: Option<FnType> = None;
    let mut property_name = None;

    for attr in attrs.iter() {
        match attr.parse_meta()? {
            syn::Meta::Path(ref name) => {
                if name.is_ident("new") || name.is_ident("__new__") {
                    res = Some(FnType::FnNew)
                } else if name.is_ident("init") || name.is_ident("__init__") {
                    return Err(syn::Error::new_spanned(
                        name,
                        "#[init] is disabled since PyO3 0.9.0",
                    ));
                } else if name.is_ident("call") || name.is_ident("__call__") {
                    res = Some(FnType::FnCall)
                } else if name.is_ident("classmethod") {
                    res = Some(FnType::FnClass)
                } else if name.is_ident("staticmethod") {
                    res = Some(FnType::FnStatic)
                } else if name.is_ident("setter") || name.is_ident("getter") {
                    if let syn::AttrStyle::Inner(_) = attr.style {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "Inner style attribute is not supported for setter and getter",
                        ));
                    }
                    if res != None {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "setter/getter attribute can not be used mutiple times",
                        ));
                    }
                    if name.is_ident("setter") {
                        res = Some(FnType::Setter)
                    } else {
                        res = Some(FnType::Getter)
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
                    res = Some(FnType::FnNew)
                } else if path.is_ident("init") {
                    return Err(syn::Error::new_spanned(
                        path,
                        "#[init] is disabled since PyO3 0.9.0",
                    ));
                } else if path.is_ident("call") {
                    res = Some(FnType::FnCall)
                } else if path.is_ident("setter") || path.is_ident("getter") {
                    if let syn::AttrStyle::Inner(_) = attr.style {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "Inner style attribute is not supported for setter and getter",
                        ));
                    }
                    if res != None {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "setter/getter attribute can not be used mutiple times",
                        ));
                    }
                    if nested.len() != 1 {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "setter/getter requires one value",
                        ));
                    }

                    res = if path.is_ident("setter") {
                        Some(FnType::Setter)
                    } else {
                        Some(FnType::Getter)
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

    let ty = res.unwrap_or(FnType::Fn);
    let python_name = if allow_custom_name {
        parse_method_name_attribute(&ty, attrs, property_name)?
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
    ty: &FnType,
    attrs: &mut Vec<syn::Attribute>,
    property_name: Option<syn::Ident>,
) -> syn::Result<Option<syn::Ident>> {
    let name = parse_name_attribute(attrs)?;

    // Reject some invalid combinations
    if let Some(name) = &name {
        match ty {
            FnType::FnNew | FnType::FnCall | FnType::Getter | FnType::Setter => {
                return Err(syn::Error::new_spanned(
                    name,
                    "name not allowed with this attribute",
                ))
            }
            _ => {}
        }
    }

    // Thanks to check above we can be sure that this generates the right python name
    Ok(match ty {
        FnType::FnNew => Some(syn::Ident::new("__new__", proc_macro2::Span::call_site())),
        FnType::FnCall => Some(syn::Ident::new("__call__", proc_macro2::Span::call_site())),
        FnType::Getter | FnType::Setter => property_name,
        _ => name,
    })
}

// Replace &A<Self> with &A<_>
fn replace_self_in_ref(refn: &syn::TypeReference) -> syn::Result<syn::TypeReference> {
    let mut res = refn.to_owned();
    let tp = match &mut *res.elem {
        syn::Type::Path(p) => p,
        _ => return Err(syn::Error::new_spanned(refn, "unsupported argument")),
    };
    replace_self_impl(tp);
    res.lifetime = None;
    Ok(res)
}

fn replace_self_in_path(tp: &syn::TypePath) -> syn::TypePath {
    let mut res = tp.to_owned();
    replace_self_impl(&mut res);
    res
}

fn replace_self_impl(tp: &mut syn::TypePath) {
    for seg in &mut tp.path.segments {
        if let syn::PathArguments::AngleBracketed(ref mut g) = seg.arguments {
            let mut args = syn::punctuated::Punctuated::new();
            for arg in &g.args {
                let mut add_arg = true;
                if let syn::GenericArgument::Lifetime(_) = arg {
                    add_arg = false;
                }
                if let syn::GenericArgument::Type(syn::Type::Path(p)) = arg {
                    if p.path.segments.len() == 1 && p.path.segments[0].ident == "Self" {
                        args.push(infer(p.span()));
                        add_arg = false;
                    }
                }
                if add_arg {
                    args.push(arg.clone());
                }
            }
            g.args = args;
        }
    }
    fn infer(span: proc_macro2::Span) -> syn::GenericArgument {
        syn::GenericArgument::Type(syn::Type::Infer(syn::TypeInfer {
            underscore_token: syn::token::Underscore { spans: [span] },
        }))
    }
}
