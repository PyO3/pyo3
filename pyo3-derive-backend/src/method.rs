// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::pyfunction::Argument;
use crate::pyfunction::PyFunctionAttr;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
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
    Getter(Option<String>),
    Setter(Option<String>),
    Fn,
    FnNew,
    FnCall,
    FnClass,
    FnStatic,
    PySelfNew(syn::TypeReference),
}

#[derive(Clone, PartialEq, Debug)]
pub struct FnSpec<'a> {
    pub tp: FnType,
    pub attrs: Vec<Argument>,
    pub args: Vec<FnArg<'a>>,
    pub output: syn::Type,
}

pub fn get_return_info(output: &syn::ReturnType) -> syn::Type {
    match output {
        syn::ReturnType::Default => syn::Type::Infer(syn::parse_quote! {_}),
        syn::ReturnType::Type(_, ref ty) => *ty.clone(),
    }
}

impl<'a> FnSpec<'a> {
    /// Parser function signature and function attributes
    pub fn parse(
        name: &'a syn::Ident,
        sig: &'a syn::Signature,
        meth_attrs: &mut Vec<syn::Attribute>,
    ) -> syn::Result<FnSpec<'a>> {
        let (mut fn_type, fn_attrs) = parse_attributes(meth_attrs)?;

        let mut has_self = false;
        let mut arguments = Vec::new();
        for input in sig.inputs.iter() {
            match input {
                syn::FnArg::Receiver(_) => {
                    has_self = true;
                }
                syn::FnArg::Typed(syn::PatType {
                    ref pat, ref ty, ..
                }) => {
                    // skip first argument (cls)
                    if fn_type == FnType::FnClass && !has_self {
                        has_self = true;
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

                    let opt = check_arg_ty_and_optional(name, ty);
                    arguments.push(FnArg {
                        name: ident,
                        by_ref,
                        mutability,
                        // mode: mode,
                        ty,
                        optional: opt,
                        py,
                        reference: is_ref(name, ty),
                    });
                }
            }
        }

        let ty = get_return_info(&sig.output);

        if fn_type == FnType::Fn && !has_self {
            if arguments.is_empty() {
                return Err(syn::Error::new_spanned(
                    name,
                    "Static method needs #[staticmethod] attribute",
                ));
            }
            let tp = match arguments.remove(0).ty {
                syn::Type::Reference(r) => replace_self(r)?,
                x => return Err(syn::Error::new_spanned(x, "Invalid type as custom self")),
            };
            fn_type = FnType::PySelfNew(tp);
        }

        Ok(FnSpec {
            tp: fn_type,
            attrs: fn_attrs,
            args: arguments,
            output: ty,
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

    pub fn accept_args(&self) -> bool {
        for s in self.attrs.iter() {
            match *s {
                Argument::VarArgs(_) => return true,
                Argument::VarArgsSeparator => return true,
                _ => (),
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

    pub fn accept_kwargs(&self) -> bool {
        for s in self.attrs.iter() {
            if let Argument::KeywordArgs(_) = s {
                return true;
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

pub fn check_arg_ty_and_optional<'a>(
    name: &'a syn::Ident,
    ty: &'a syn::Type,
) -> Option<&'a syn::Type> {
    match ty {
        syn::Type::Path(syn::TypePath { ref path, .. }) => {
            //if let Some(ref qs) = qs {
            //    panic!("explicit Self type in a 'qualified path' is not supported: {:?} - {:?}",
            //           name, qs);
            //}

            if let Some(segment) = path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "Option" => match segment.arguments {
                        syn::PathArguments::AngleBracketed(ref params) => {
                            if params.args.len() != 1 {
                                panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                       name,
                                       ty,
                                       path);
                            }

                            match &params.args[0] {
                                syn::GenericArgument::Type(ref ty) => Some(ty),
                                _ => panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                            name,
                                            ty,
                                            path),
                            }
                        }
                        _ => {
                            panic!(
                                "argument type is not supported by python method: {:?} ({:?}) {:?}",
                                name, ty, path
                            );
                        }
                    },
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => {
            None
            //panic!("argument type is not supported by python method: {:?} ({:?})",
            //name,
            //ty);
        }
    }
}

fn parse_attributes(attrs: &mut Vec<syn::Attribute>) -> syn::Result<(FnType, Vec<Argument>)> {
    let mut new_attrs = Vec::new();
    let mut spec = Vec::new();
    let mut res: Option<FnType> = None;

    for attr in attrs.iter() {
        match attr.parse_meta()? {
            syn::Meta::Path(ref name) => {
                if name.is_ident("new") || name.is_ident("__new__") {
                    res = Some(FnType::FnNew)
                } else if name.is_ident("init") || name.is_ident("__init__") {
                    return Err(syn::Error::new_spanned(
                        name,
                        "#[init] is disabled from PyO3 0.9.0",
                    ));
                } else if name.is_ident("call") || name.is_ident("__call__") {
                    res = Some(FnType::FnCall)
                } else if name.is_ident("classmethod") {
                    res = Some(FnType::FnClass)
                } else if name.is_ident("staticmethod") {
                    res = Some(FnType::FnStatic)
                } else if name.is_ident("setter") || name.is_ident("getter") {
                    if let syn::AttrStyle::Inner(_) = attr.style {
                        panic!("Inner style attribute is not supported for setter and getter");
                    }
                    if res != None {
                        panic!("setter/getter attribute can not be used mutiple times");
                    }
                    if name.is_ident("setter") {
                        res = Some(FnType::Setter(None))
                    } else {
                        res = Some(FnType::Getter(None))
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
                        "#[init] is disabled from PyO3 0.9.0",
                    ));
                } else if path.is_ident("call") {
                    res = Some(FnType::FnCall)
                } else if path.is_ident("setter") || path.is_ident("getter") {
                    if let syn::AttrStyle::Inner(_) = attr.style {
                        panic!(
                            "Inner style attribute is not
                                    supported for setter and getter"
                        );
                    }
                    if res != None {
                        panic!("setter/getter attribute can not be used mutiple times");
                    }
                    if nested.len() != 1 {
                        panic!("setter/getter requires one value");
                    }
                    match nested.first().unwrap() {
                        syn::NestedMeta::Meta(syn::Meta::Path(ref w)) => {
                            if path.is_ident("setter") {
                                res = Some(FnType::Setter(Some(w.segments[0].ident.to_string())))
                            } else {
                                res = Some(FnType::Getter(Some(w.segments[0].ident.to_string())))
                            }
                        }
                        syn::NestedMeta::Lit(ref lit) => match *lit {
                            syn::Lit::Str(ref s) => {
                                if path.is_ident("setter") {
                                    res = Some(FnType::Setter(Some(s.value())))
                                } else {
                                    res = Some(FnType::Getter(Some(s.value())))
                                }
                            }
                            _ => {
                                panic!("setter/getter attribute requires str value");
                            }
                        },
                        _ => {
                            println!("cannot parse {:?} attribute: {:?}", path, nested);
                        }
                    }
                } else if path.is_ident("args") {
                    let attrs = PyFunctionAttr::from_meta(nested)?;
                    spec.extend(attrs.arguments)
                } else {
                    new_attrs.push(attr.clone())
                }
            }
            syn::Meta::NameValue(_) => new_attrs.push(attr.clone()),
        }
    }
    attrs.clear();
    attrs.extend(new_attrs);

    match res {
        Some(tp) => Ok((tp, spec)),
        None => Ok((FnType::Fn, spec)),
    }
}

// Replace &A<Self> with &A<_>
fn replace_self(refn: &syn::TypeReference) -> syn::Result<syn::TypeReference> {
    fn infer(span: proc_macro2::Span) -> syn::GenericArgument {
        syn::GenericArgument::Type(syn::Type::Infer(syn::TypeInfer {
            underscore_token: syn::token::Underscore { spans: [span] },
        }))
    }
    let mut res = refn.to_owned();
    let tp = match &mut *res.elem {
        syn::Type::Path(p) => p,
        _ => return Err(syn::Error::new_spanned(refn, "unsupported argument")),
    };
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
    res.lifetime = None;
    Ok(res)
}
