// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, Ident};

use args::{Argument, parse_arguments};
use utils::for_err_msg;


#[derive(Clone, Debug)]
pub struct FnArg<'a> {
    pub name: &'a syn::Ident,
    pub mode: &'a syn::BindingMode,
    pub ty: &'a syn::Ty,
    pub optional: Option<&'a syn::Ty>,
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

pub struct FnSpec<'a> {
    pub tp: FnType,
    pub attrs: Vec<Argument>,
    pub args: Vec<FnArg<'a>>,
    pub output: syn::Ty,
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
                &syn::FnArg::SelfRef(_, _) => {
                    has_self = true;
                },
                &syn::FnArg::SelfValue(_) => {
                    has_self = true;
                }
                &syn::FnArg::Captured(ref pat, ref ty) => {
                    // skip first argument (cls)
                    if (fn_type == FnType::FnClass || fn_type == FnType::FnNew) && !has_self {
                        has_self = true;
                        continue
                    }

                    let (mode, ident) = match pat {
                        &syn::Pat::Ident(ref mode, ref ident, _) =>
                            (mode, ident),
                        _ =>
                            panic!("unsupported argument: {:?}", pat),
                    };

                    let py = match ty {
                        &syn::Ty::Path(_, ref path) =>
                            if let Some(segment) = path.segments.last() {
                                segment.ident.as_ref() == "Python"
                            } else {
                                false
                            },
                        _ => false
                    };

                    let opt = check_arg_ty_and_optional(name, ty);
                    arguments.push(
                        FnArg {
                            name: ident,
                            mode: mode,
                            ty: ty,
                            optional: opt,
                            py: py,
                            reference: is_ref(name, ty),
                        }
                    );
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
                Argument::VarArgs(ref ident) =>
                    return name.as_ref() == ident.as_str(),
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
                    return name.as_ref() == ident.as_str(),
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
                    if ident.as_str() == name.as_ref() {
                        if let &Some(ref val) = opt {
                            let i = Ident::from(val.as_str());
                            return Some(quote!(#i))
                        }
                    }
                },
                Argument::Kwarg(ref ident, ref opt) => {
                    if ident.as_str() == name.as_ref() {
                        let i = Ident::from(opt.as_str());
                        return Some(quote!(#i))
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
                    if ident.as_str() == name.as_ref() {
                        return true
                    }
                },
                _ => (),
            }
        }
        false
    }
}

pub fn is_ref<'a>(name: &'a syn::Ident, ty: &'a syn::Ty) -> bool {
    match ty {
        &syn::Ty::Rptr(_, _) => {
            return true
        }
        &syn::Ty::Path(_, ref path) => {
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
                                match &params.types[params.types.len()-1] {
                                    &syn::Ty::Rptr(_, _) => {
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

pub fn check_arg_ty_and_optional<'a>(name: &'a syn::Ident, ty: &'a syn::Ty)
                                     -> Option<&'a syn::Ty>
{
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

fn parse_attributes(attrs: &mut Vec<syn::Attribute>) -> (FnType, Vec<Argument>) {
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
                    "init" => {
                        res = Some(FnType::FnInit)
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
                        spec.extend(parse_arguments(meta.as_slice()))
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
