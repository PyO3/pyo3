// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;
use quote::ToTokens;

use args::{Argument, parse_arguments};
use utils::for_err_msg;


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
        syn::ReturnType::Default => syn::Type::Infer(parse_quote!{_}),
        syn::ReturnType::Type(_, ref ty) => *ty.clone()
    }
}


impl<'a> FnSpec<'a> {
    /// Parser function signature and function attributes
    pub fn parse(name: &'a syn::Ident,
                 sig: &'a syn::MethodSig,
                 meth_attrs: &'a mut Vec<syn::Attribute>) -> FnSpec<'a> {
        let (fn_type, fn_attrs) = parse_attributes(meth_attrs);

        let mut has_self = false;
        let mut arguments = Vec::new();

        for input in sig.decl.inputs.iter() {
            match input {
                &syn::FnArg::SelfRef(_) => {
                    has_self = true;
                },
                &syn::FnArg::SelfValue(_) => {
                    has_self = true;
                }
                &syn::FnArg::Captured(syn::ArgCaptured {ref pat, ref ty, ..}) => {
                    // skip first argument (cls)
                    if (fn_type == FnType::FnClass || fn_type == FnType::FnNew) && !has_self {
                        has_self = true;
                        continue
                    }

                    let (ident, by_ref, mutability) = match pat {
                        &syn::Pat::Ident(syn::PatIdent {ref ident, ref by_ref, ref mutability, .. }) =>
                            (ident, by_ref, mutability),
                        _ =>
                            panic!("unsupported argument: {:?}", pat),
                    };

                    let py = match ty {
                        &syn::Type::Path(syn::TypePath {ref path, ..}) =>
                            if let Some(segment) = path.segments.last() {
                                segment.value().ident.as_ref() == "Python"
                            } else {
                                false
                            },
                        _ => false
                    };

                    let opt = check_arg_ty_and_optional(name, ty);
                    arguments.push(
                        FnArg {
                            name: ident,
                            by_ref,
                            mutability,
                            // mode: mode,
                            ty: ty,
                            optional: opt,
                            py: py,
                            reference: is_ref(name, ty),
                        }
                    );
                }
                &syn::FnArg::Ignored(_) =>
                    panic!("ignored argument: {:?}", name),
                &syn::FnArg::Inferred(_) =>
                    panic!("ingerred argument: {:?}", name),
            }
        }

        let ty = get_return_info(&sig.decl.output);

        FnSpec {
            tp: fn_type,
            attrs: fn_attrs,
            args: arguments,
            output: ty,
        }
    }

    pub fn is_args(&self, name: &syn::Ident) -> bool {
        for s in self.attrs.iter() {
            match *s {
                Argument::VarArgs(ref ident) =>
                    return name == ident,
                _ => (),
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
            match *s {
                Argument::KeywordArgs(ref ident) =>
                    return name == ident,
                _ => (),
            }
        }
        false
    }

    pub fn accept_kwargs(&self) -> bool {
        for s in self.attrs.iter() {
            match *s {
                Argument::KeywordArgs(_) => return true,
                _ => (),
            }
        }
        false
    }

    pub fn default_value(&self, name: &syn::Ident) -> Option<Tokens> {
        for s in self.attrs.iter() {
            match *s {
                Argument::Arg(ref ident, ref opt) => {
                    if ident == name {
                        if let &Some(ref val) = opt {
                            let i: syn::Expr = syn::parse_str(&val).unwrap();
                            return Some(i.into_tokens())
                        }
                    }
                },
                Argument::Kwarg(ref ident, ref opt) => {
                    if ident == name {
                        let i: syn::Expr = syn::parse_str(&opt).unwrap();
                        return Some(i.into_tokens())
                    }
                },
                _ => (),
            }
        }
        None
    }

    pub fn is_kw_only(&self, name: &syn::Ident) -> bool {
        for s in self.attrs.iter() {
            match *s {
                Argument::Kwarg(ref ident, _) => {
                    if ident == name {
                        return true
                    }
                },
                _ => (),
            }
        }
        false
    }
}

pub fn is_ref<'a>(name: &'a syn::Ident, ty: &'a syn::Type) -> bool {
    match ty {
        &syn::Type::Reference(_) => {
            return true
        }
        &syn::Type::Path(syn::TypePath {ref path, ..}) => {
            if let Some(segment) = path.segments.last() {
                match segment.value().ident.as_ref() {
                    "Option" => {
                        match segment.value().arguments {
                            syn::PathArguments::AngleBracketed(ref params) => {
                                if params.args.len() != 1 {
                                    panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                           for_err_msg(name),
                                           for_err_msg(ty),
                                           for_err_msg(path));
                                }
                                match &params.args[params.args.len()-1] {
                                    &syn::GenericArgument::Type(syn::Type::Reference(_)) => {
                                        return true
                                    },
                                    _ => ()
                                }
                            },
                            _ => {
                                panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                       for_err_msg(name),
                                       for_err_msg(ty),
                                       for_err_msg(path));
                            }
                        }
                    },
                    _ => (),
                }
            }
        },
        _ => ()
    }
    false
}

pub fn check_arg_ty_and_optional<'a>(name: &'a syn::Ident, ty: &'a syn::Type)
                                     -> Option<&'a syn::Type>
{
    match ty {
        &syn::Type::Path(syn::TypePath {ref path, ..}) => {
            //if let &Some(ref qs) = qs {
            //    panic!("explicit Self type in a 'qualified path' is not supported: {:?} - {:?}",
            //           name, qs);
            //}

            if let Some(segment) = path.segments.last() {
                match segment.value().ident.as_ref() {
                    "Option" => {
                        match segment.value().arguments {
                            syn::PathArguments::AngleBracketed(ref params) => {
                                if params.args.len() != 1 {
                                    panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                           for_err_msg(name),
                                           for_err_msg(ty),
                                           for_err_msg(path));
                                }

                                match &params.args[0] {
                                    &syn::GenericArgument::Type(ref ty) => Some(ty),
                                    _ => panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                                    for_err_msg(name),
                                                    for_err_msg(ty),
                                                    for_err_msg(path)),
                                }

                            },
                            _ => {
                                panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                       for_err_msg(name),
                                       for_err_msg(ty),
                                       for_err_msg(path));
                            }
                        }
                    },
                    _ => None,
                }
            } else {
                None
            }
        },
        _ => {
            None
            //panic!("argument type is not supported by python method: {:?} ({:?})",
            //for_err_msg(name),
            //for_err_msg(ty));
        },
    }
}

fn parse_attributes(attrs: &mut Vec<syn::Attribute>) -> (FnType, Vec<Argument>) {
    let mut new_attrs = Vec::new();
    let mut spec = Vec::new();
    let mut res: Option<FnType> = None;

    for attr in attrs.iter() {
        match attr.interpret_meta().unwrap() {
            syn::Meta::Word(ref name) => {
                match name.as_ref() {
                    "new" | "__new__" => {
                        res = Some(FnType::FnNew)
                    },
                    "init" | "__init__" => {
                        res = Some(FnType::FnInit)
                    },
                    "call" | "__call__" => {
                        res = Some(FnType::FnCall)
                    },
                    "classmethod" => {
                        res = Some(FnType::FnClass)
                    },
                    "staticmethod" => {
                        res = Some(FnType::FnStatic)
                    },
                    "setter" | "getter" => {
                        if let syn::AttrStyle::Inner(_) = attr.style  {
                            panic!("Inner style attribute is not
                                    supported for setter and getter");
                        }
                        if res != None {
                            panic!("setter/getter attribute can not be used mutiple times");
                        }
                        if name.as_ref() == "setter" {
                            res = Some(FnType::Setter(None))
                        } else {
                            res = Some(FnType::Getter(None))
                        }
                    },
                    _ => {
                        new_attrs.push(attr.clone())
                    }
                }
            },
            syn::Meta::List(syn::MetaList {ref ident, ref nested, ..}) => {
                match ident.as_ref() {
                    "new" => {
                        res = Some(FnType::FnNew)
                    },
                    "init" => {
                        res = Some(FnType::FnInit)
                    },
                    "call" => {
                        res = Some(FnType::FnCall)
                    },
                    "setter" | "getter" => {
                        if let syn::AttrStyle::Inner(_) = attr.style {
                            panic!("Inner style attribute is not
                                    supported for setter and getter");
                        }
                        if res != None {
                            panic!("setter/getter attribute can not be used mutiple times");
                        }
                        if nested.len() != 1 {
                            panic!("setter/getter requires one value");
                        }
                        match nested.first().unwrap().value() {
                            syn::NestedMeta::Meta(syn::Meta::Word(ref w)) => {
                                if ident.as_ref() == "setter" {
                                    res = Some(FnType::Setter(Some(w.to_string())))
                                } else {
                                    res = Some(FnType::Getter(Some(w.to_string())))
                                }
                            },
                            syn::NestedMeta::Literal(ref lit) => {
                                match *lit {
                                    syn::Lit::Str(ref s) => {
                                        if ident.as_ref() == "setter" {
                                            res = Some(FnType::Setter(Some(s.value())))
                                        } else {
                                            res = Some(FnType::Getter(Some(s.value())))
                                        }
                                    },
                                    _ => {
                                        panic!("setter/getter attribute requires str value");
                                    },
                                }
                            }
                            _ => {
                                println!("cannot parse {:?} attribute: {:?}", ident, nested);
                            },
                        }
                    },
                    "args" => {
                        let args = nested.iter().cloned().collect::<Vec<_>>();
                        spec.extend(parse_arguments(args.as_slice()))
                    }
                    _ => {
                        new_attrs.push(attr.clone())
                    }
                }
            },
            syn::Meta::NameValue(_) => {
                new_attrs.push(attr.clone())
            },
        }
    }
    attrs.clear();
    attrs.extend(new_attrs);

    match res {
        Some(tp) => (tp, spec),
        None => (FnType::Fn, spec),
    }
}
