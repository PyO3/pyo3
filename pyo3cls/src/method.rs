// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};
use utils::for_err_msg;


#[derive(Clone, Debug)]
pub struct FnArg<'a> {
    pub name: &'a syn::Ident,
    pub mode: &'a syn::BindingMode,
    pub ty: &'a syn::Ty,
    pub optional: Option<&'a syn::Ty>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FnType {
    Getter(Option<String>),
    Setter(Option<String>),
    Fn,
    FnNew,
    FnCall,
}

#[derive(Clone, Debug)]
pub enum FnAttr {
    Args(syn::Ident),
    Kwargs(syn::Ident),
    Default(syn::Ident, Tokens),
}

pub struct FnSpec<'a> {
    pub tp: FnType,
    pub attrs: Vec<FnAttr>,
    pub args: Vec<FnArg<'a>>,
    pub output: syn::Ty,
}

impl<'a> FnSpec<'a> {

    pub fn parse(name: &'a syn::Ident,
                 sig: &'a syn::MethodSig,
                 meth_attrs: &'a mut Vec<syn::Attribute>) -> FnSpec<'a> {
        let (fn_type, fn_attrs) = parse_attributes(meth_attrs);

        //let mut has_self = false;
        let mut py = false;
        let mut arguments = Vec::new();

        for input in sig.decl.inputs[1..].iter() {
            match input {
                &syn::FnArg::SelfRef(_, _) => {
                    //has_self = true;
                },
                &syn::FnArg::SelfValue(_) => {
                    //has_self = true;
                }
                &syn::FnArg::Captured(ref pat, ref ty) => {
                    let (mode, ident) = match pat {
                        &syn::Pat::Ident(ref mode, ref ident, _) =>
                            (mode, ident),
                        _ =>
                            panic!("unsupported argument: {:?}", pat),
                    };
                    // TODO add check for first py: Python arg
                    if py {
                        let opt = check_arg_ty_and_optional(name, ty);
                        arguments.push(FnArg{name: ident, mode: mode, ty: ty, optional: opt});
                    } else {
                        py = true;
                    }
                }
                &syn::FnArg::Ignored(_) =>
                    panic!("ignored argument: {:?}", name),
            }
        }

        let ty = match sig.decl.output {
            syn::FunctionRetTy::Default => syn::Ty::Infer,
            syn::FunctionRetTy::Ty(ref ty) => ty.clone()
        };

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
                FnAttr::Args(ref ident) =>
                    return name == ident,
                _ => (),
            }
        }
        false
    }

    pub fn accept_args(&self) -> bool {
        for s in self.attrs.iter() {
            match *s {
                FnAttr::Args(_) => return true,
                _ => (),
            }
        }
        false
    }

    pub fn is_kwargs(&self, name: &syn::Ident) -> bool {
        for s in self.attrs.iter() {
            match *s {
                FnAttr::Kwargs(ref ident) =>
                    return name == ident,
                _ => (),
            }
        }
        false
    }

    pub fn accept_kwargs(&self) -> bool {
        for s in self.attrs.iter() {
            match *s {
                FnAttr::Kwargs(_) => return true,
                _ => (),
            }
        }
        false
    }

    pub fn default_value(&self, name: &syn::Ident) -> Option<Tokens> {
        for s in self.attrs.iter() {
            match *s {
                FnAttr::Default(ref ident, ref val) => {
                    if ident == name {
                        return Some(val.clone())
                    }
                },
                _ => (),
            }
        }
        None
    }
}

fn check_arg_ty_and_optional<'a>(name: &'a syn::Ident, ty: &'a syn::Ty) -> Option<&'a syn::Ty> {
    match ty {
        &syn::Ty::Path(_, ref path) => {
            //if let &Some(ref qs) = qs {
            //    panic!("explicit Self type in a 'qualified path' is not supported: {:?} - {:?}",
            //           name, qs);
            //}

            if let Some(segment) = path.segments.last() {
                match segment.ident.as_ref() {
                    "Option" => {
                        match segment.parameters {
                            syn::PathParameters::AngleBracketed(ref params) => {
                                if params.types.len() != 1 {
                                    panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                           for_err_msg(name),
                                           for_err_msg(ty),
                                           for_err_msg(path));
                                }
                                Some(&params.types[0])
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

fn parse_attributes(attrs: &mut Vec<syn::Attribute>) -> (FnType, Vec<FnAttr>) {
    let mut new_attrs = Vec::new();
    let mut spec = Vec::new();
    let mut res: Option<FnType> = None;

    for attr in attrs.iter() {
        match attr.value {
            syn::MetaItem::Word(ref name) => {
                match name.as_ref() {
                    "new" | "__new__" => {
                        res = Some(FnType::FnNew)
                    },
                    "call" | "__call__" => {
                        res = Some(FnType::FnCall)
                    },
                    "setter" | "getter" => {
                        if attr.style == syn::AttrStyle::Inner {
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
            syn::MetaItem::List(ref name, ref meta) => {
                match name.as_ref() {
                    "new" => {
                        res = Some(FnType::FnNew)
                    },
                    "call" => {
                        res = Some(FnType::FnCall)
                    },
                    "setter" | "getter" => {
                        if attr.style == syn::AttrStyle::Inner {
                            panic!("Inner style attribute is not
                                    supported for setter and getter");
                        }
                        if res != None {
                            panic!("setter/getter attribute can not be used mutiple times");
                        }
                        if meta.len() != 1 {
                            panic!("setter/getter requires one value");
                        }
                        match *meta.first().unwrap() {
                            syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref w)) => {
                                if name.as_ref() == "setter" {
                                    res = Some(FnType::Setter(Some(w.to_string())))
                                } else {
                                    res = Some(FnType::Getter(Some(w.to_string())))
                                }
                            },
                            syn::NestedMetaItem::Literal(ref lit) => {
                                match *lit {
                                    syn::Lit::Str(ref s, syn::StrStyle::Cooked) => {
                                        if name.as_ref() == "setter" {
                                            res = Some(FnType::Setter(Some(s.clone())))
                                        } else {
                                            res = Some(FnType::Getter(Some(s.clone())))
                                        }
                                    },
                                    _ => {
                                        panic!("setter/getter attribute requires str value");
                                    },
                                }
                            }
                            _ => {
                                println!("cannot parse {:?} attribute: {:?}", name, meta);
                            },
                        }
                    },
                    "args" => {
                        spec.extend(parse_args(meta))
                    }
                    "defaults" => {
                        // parse: #[defaults(param2=12, param3=12)]
                        for item in meta.iter() {
                            if let Some(el) = parse_args_default(item) {
                                spec.push(el)
                            }
                        }
                    }
                    _ => {
                        new_attrs.push(attr.clone())
                    }
                }
            },
            syn::MetaItem::NameValue(_, _) => {
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

/// parse: #[args(args="args", kw="kwargs")]
fn parse_args(items: &Vec<syn::NestedMetaItem>) -> Vec<FnAttr> {
    let mut spec = Vec::new();

    for item in items.iter() {
        match item {
            &syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref ident, ref name)) => {
                match *name {
                    syn::Lit::Str(ref name, _) => match ident.as_ref() {
                        "args" =>
                            spec.push(FnAttr::Args(syn::Ident::from(name.clone()))),
                        "kw" =>
                            spec.push(FnAttr::Kwargs(syn::Ident::from(name.clone()))),
                        _ => (),
                    },
                    _ => (),
                }
            },
            _ => (),
        }
    }

    spec
}

fn parse_args_default(item: &syn::NestedMetaItem) -> Option<FnAttr> {
    match *item {
        syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref name, ref lit)) => {
            let mut t = Tokens::new();
            match lit {
                &syn::Lit::Str(ref val, _) => {
                    syn::Ident::from(val.as_str()).to_tokens(&mut t);
                },
                _ => {
                    lit.to_tokens(&mut t);
                }
            }
            Some(FnAttr::Default(name.clone(), t))
        }
        _ => {
            println!("expected name value {:?}", item);
            None
        }
    }
}
