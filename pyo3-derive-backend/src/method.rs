// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::pyfunction::Argument;
use crate::pyfunction::PyFunctionAttr;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

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
    FnInit,
    FnCall,
    FnClass,
    FnStatic,
    PySelf(syn::TypePath),
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
        sig: &'a syn::MethodSig,
        meth_attrs: &'a mut Vec<syn::Attribute>,
    ) -> syn::Result<FnSpec<'a>> {
        let (mut fn_type, fn_attrs) = parse_attributes(meth_attrs)?;

        let mut has_self = false;
        let mut arguments = Vec::new();
        for input in sig.decl.inputs.iter() {
            match input {
                syn::FnArg::SelfRef(_) => {
                    has_self = true;
                }
                syn::FnArg::SelfValue(_) => {
                    has_self = true;
                }
                syn::FnArg::Captured(syn::ArgCaptured {
                    ref pat, ref ty, ..
                }) => {
                    // skip first argument (cls)
                    if (fn_type == FnType::FnClass || fn_type == FnType::FnNew) && !has_self {
                        has_self = true;
                        continue;
                    }

                    let (ident, by_ref, mutability) = match pat {
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

                    let py = match ty {
                        syn::Type::Path(syn::TypePath { ref path, .. }) => {
                            if let Some(segment) = path.segments.last() {
                                segment.value().ident == "Python"
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };

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
                syn::FnArg::Ignored(_) => {
                    return Err(syn::Error::new_spanned(name, "ignored argument"));
                }
                syn::FnArg::Inferred(_) => {
                    return Err(syn::Error::new_spanned(name, "inferred argument"));
                }
            }
        }

        let ty = get_return_info(&sig.decl.output);

        if fn_type == FnType::Fn && !has_self {
            if arguments.is_empty() {
                panic!("Static method needs #[staticmethod] attribute");
            }
            let tp = match arguments.remove(0).ty {
                syn::Type::Path(p) => replace_self(p),
                _ => panic!("Invalid type as self"),
            };
            fn_type = FnType::PySelf(tp);
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
            if let Argument::VarArgs(ref ident) = s {
                return name == ident;
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
            if let Argument::KeywordArgs(ref ident) = s {
                return name == ident;
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
                Argument::Arg(ref ident, ref opt) => {
                    if ident == name {
                        if let Some(ref val) = opt {
                            let i: syn::Expr = syn::parse_str(&val).unwrap();
                            return Some(i.into_token_stream());
                        }
                    }
                }
                Argument::Kwarg(ref ident, ref opt) => {
                    if ident == name {
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
            if let Argument::Kwarg(ref ident, _) = s {
                if ident == name {
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
                if "Option" == segment.value().ident.to_string().as_str() {
                    match segment.value().arguments {
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
                match segment.value().ident.to_string().as_str() {
                    "Option" => match segment.value().arguments {
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
        match attr.interpret_meta().unwrap() {
            syn::Meta::Word(ref name) => match name.to_string().as_ref() {
                "new" | "__new__" => res = Some(FnType::FnNew),
                "init" | "__init__" => res = Some(FnType::FnInit),
                "call" | "__call__" => res = Some(FnType::FnCall),
                "classmethod" => res = Some(FnType::FnClass),
                "staticmethod" => res = Some(FnType::FnStatic),
                "setter" | "getter" => {
                    if let syn::AttrStyle::Inner(_) = attr.style {
                        panic!(
                            "Inner style attribute is not
                                    supported for setter and getter"
                        );
                    }
                    if res != None {
                        panic!("setter/getter attribute can not be used mutiple times");
                    }
                    if name == "setter" {
                        res = Some(FnType::Setter(None))
                    } else {
                        res = Some(FnType::Getter(None))
                    }
                }
                _ => new_attrs.push(attr.clone()),
            },
            syn::Meta::List(syn::MetaList {
                ref ident,
                ref nested,
                ..
            }) => match ident.to_string().as_str() {
                "new" => res = Some(FnType::FnNew),
                "init" => res = Some(FnType::FnInit),
                "call" => res = Some(FnType::FnCall),
                "setter" | "getter" => {
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
                    match nested.first().unwrap().value() {
                        syn::NestedMeta::Meta(syn::Meta::Word(ref w)) => {
                            if ident == "setter" {
                                res = Some(FnType::Setter(Some(w.to_string())))
                            } else {
                                res = Some(FnType::Getter(Some(w.to_string())))
                            }
                        }
                        syn::NestedMeta::Literal(ref lit) => match *lit {
                            syn::Lit::Str(ref s) => {
                                if ident == "setter" {
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
                            println!("cannot parse {:?} attribute: {:?}", ident, nested);
                        }
                    }
                }
                "args" => {
                    let attrs = PyFunctionAttr::from_meta(nested)?;
                    spec.extend(attrs.arguments)
                }
                _ => new_attrs.push(attr.clone()),
            },
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

// Replace A<Self> with A<_>
fn replace_self(path: &syn::TypePath) -> syn::TypePath {
    fn infer(span: proc_macro2::Span) -> syn::GenericArgument {
        syn::GenericArgument::Type(syn::Type::Infer(syn::TypeInfer {
            underscore_token: syn::token::Underscore { spans: [span] },
        }))
    }
    let mut res = path.to_owned();
    for seg in &mut res.path.segments {
        if let syn::PathArguments::AngleBracketed(ref mut g) = seg.arguments {
            for arg in &mut g.args {
                if let syn::GenericArgument::Type(syn::Type::Path(p)) = arg {
                    if p.path.segments.len() == 1 && p.path.segments[0].ident == "Self" {
                        *arg = infer(p.path.segments[0].ident.span());
                    }
                }
            }
        }
    }
    res
}
