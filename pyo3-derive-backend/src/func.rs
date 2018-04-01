// Copyright (c) 2017-present PyO3 Project and Contributors
use syn;
use quote::Tokens;
use utils::print_err;

// TODO:
//   Add lifetime support for args with Rptr

#[derive(Debug)]
pub enum MethodProto {
    Free{name: &'static str, proto: &'static str, },
    Unary{name: &'static str, pyres: bool, proto: &'static str, },
    Binary{name: &'static str, arg: &'static str, pyres: bool, proto: &'static str},
    BinaryS{name: &'static str,
            arg1: &'static str, arg2: &'static str, pyres: bool, proto: &'static str},
    Ternary{name: &'static str,
            arg1: &'static str,
            arg2: &'static str,
            pyres: bool, proto: &'static str},
    TernaryS{name: &'static str,
             arg1: &'static str,
             arg2: &'static str,
             arg3: &'static str,
             pyres: bool, proto: &'static str},
    Quaternary{name: &'static str,
               arg1: &'static str,
               arg2: &'static str,
               arg3: &'static str, proto: &'static str},
}

impl MethodProto {

    pub fn eq(&self, name: &str) -> bool {
        match *self {
            MethodProto::Free{name: n, proto: _} => n == name,
            MethodProto::Unary{name: n, pyres: _, proto: _} => n == name,
            MethodProto::Binary{name: n, arg: _, pyres: _, proto: _} => n == name,
            MethodProto::BinaryS{name: n, arg1: _, arg2: _, pyres: _, proto: _} => n == name,
            MethodProto::Ternary{name: n, arg1: _, arg2: _, pyres: _, proto: _} => n == name,
            MethodProto::TernaryS{name: n, arg1: _, arg2: _, arg3: _,
                                  pyres: _, proto: _} => n == name,
            MethodProto::Quaternary{name: n, arg1: _, arg2: _, arg3: _, proto: _} => n == name,
        }
    }
}


pub fn impl_method_proto(cls: &Box<syn::Ty>,
                         sig: &mut syn::MethodSig,
                         meth: &MethodProto) -> Tokens {
    let decl = sig.decl.clone();

    match *meth {
        MethodProto::Free{name: _, proto} => {
            let p = syn::Ident::from(proto);
            return quote! {
                impl<'p> #p<'p> for #cls {}
            }
        },
        _ => (),
    };

    match decl.output {
        syn::FunctionRetTy::Ty(ref ty) => {
            match *meth {
                MethodProto::Free{name: _, proto: _} => unreachable!(),
                MethodProto::Unary{name: _, pyres, proto} => {
                    let p = syn::Ident::from(proto);
                    let (ty, succ) = get_res_success(ty);

                    let tmp = extract_decl(syn::parse_item(
                        quote! {fn test(&self)
                                        -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    sig.decl.output = tmp.output.clone();
                    modify_self_ty(sig);

                    if pyres {
                        quote! {
                            impl<'p> #p<'p> for #cls {
                                type Success = #succ;
                                type Result = #ty;
                            }
                        }
                    } else {
                        quote! {
                            impl<'p> #p<'p> for #cls {
                                type Result = #ty;
                            }
                        }
                    }
                },
                MethodProto::Binary{name: n, arg, pyres, proto} => {
                    if sig.decl.inputs.len() <= 1 {
                        println!("Not enough arguments for {}", n);
                        return Tokens::new();
                    }
                    let p = syn::Ident::from(proto);
                    let arg_name = syn::Ident::from(arg);
                    let arg_ty = get_arg_ty(sig, 1);
                    let (ty, succ) = get_res_success(ty);

                    let tmp = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg: <#cls as #p<'p>>::#arg_name)
                                -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    let tmp2 = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg: Option<<#cls as #p<'p>>::#arg_name>)
                                -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    modify_arg_ty(sig, 1, &tmp, &tmp2);
                    modify_self_ty(sig);

                    if pyres {
                        quote! {
                            impl<'p> #p<'p> for #cls {
                                type #arg_name = #arg_ty;
                                type Success = #succ;
                                type Result = #ty;
                            }
                        }
                    } else {
                        quote! {
                            impl<'p> #p<'p> for #cls {
                                type #arg_name = #arg_ty;
                                type Result = #ty;
                            }
                        }
                    }
                },
                MethodProto::BinaryS{name: n, arg1, arg2, pyres, proto} => {
                    if sig.decl.inputs.len() <= 1 {
                        print_err(format!("Not enough arguments {}", n), quote!(sig));
                        return Tokens::new();
                    }
                    let p = syn::Ident::from(proto);
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 0);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 1);
                    let (ty, succ) = get_res_success(ty);

                    // rewrite ty
                    let tmp = extract_decl(syn::parse_item(
                        quote! {fn test(
                            arg1: <#cls as #p<'p>>::#arg1_name,
                            arg2: <#cls as #p<'p>>::#arg2_name)
                                -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    let tmp2 = extract_decl(syn::parse_item(
                        quote! {fn test(
                            arg1: Option<<#cls as #p<'p>>::#arg1_name>,
                            arg2: Option<<#cls as #p<'p>>::#arg2_name>)
                                -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    modify_arg_ty(sig, 0, &tmp, &tmp2);
                    modify_arg_ty(sig, 1, &tmp, &tmp2);

                    if pyres {
                        quote! {
                            impl<'p> #p<'p> for #cls {
                                type #arg1_name = #arg1_ty;
                                type #arg2_name = #arg2_ty;
                                type Success = #succ;
                                type Result = #ty;
                            }
                        }
                    } else {
                        quote! {
                            impl<'p> #p<'p> for #cls {
                                type #arg1_name = #arg1_ty;
                                type #arg2_name = #arg2_ty;
                                type Result = #ty;
                            }
                        }
                    }
                },
                MethodProto::Ternary{name: n, arg1, arg2, pyres, proto} => {
                    if sig.decl.inputs.len() <= 2 {
                        print_err(format!("Not enough arguments {}", n), quote!(sig));
                        return Tokens::new();
                    }
                    let p = syn::Ident::from(proto);
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 1);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 2);
                    let (ty, succ) = get_res_success(ty);

                    // rewrite ty
                    let tmp = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg1: <#cls as #p<'p>>::#arg1_name,
                            arg2: <#cls as #p<'p>>::#arg2_name)
                                -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    let tmp2 = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg1: Option<<#cls as #p<'p>>::#arg1_name>,
                            arg2: Option<<#cls as #p<'p>>::#arg2_name>)
                                -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    modify_arg_ty(sig, 1, &tmp, &tmp2);
                    modify_arg_ty(sig, 2, &tmp, &tmp2);
                    modify_self_ty(sig);

                    if pyres {
                        quote! {
                            impl<'p> #p<'p> for #cls {
                                type #arg1_name = #arg1_ty;
                                type #arg2_name = #arg2_ty;
                                type Success = #succ;
                                type Result = #ty;
                            }
                        }
                    } else {
                        quote! {
                            impl<'p> #p<'p> for #cls {
                                type #arg1_name = #arg1_ty;
                                type #arg2_name = #arg2_ty;
                                type Result = #ty;
                            }
                        }
                    }
                },
                MethodProto::TernaryS{name: n, arg1, arg2, arg3, pyres, proto} => {
                    if sig.decl.inputs.len() <= 2 {
                        print_err(format!("Not enough arguments {}", n), quote!(sig));
                        return Tokens::new();
                    }
                    let p = syn::Ident::from(proto);
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 0);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 1);
                    let arg3_name = syn::Ident::from(arg3);
                    let arg3_ty = get_arg_ty(sig, 2);
                    let (ty, succ) = get_res_success(ty);

                    // rewrite ty
                    let tmp = extract_decl(syn::parse_item(
                        quote! {fn test(
                            arg1: <#cls as #p<'p>>::#arg1_name,
                            arg2: <#cls as #p<'p>>::#arg2_name,
                            arg3: <#cls as #p<'p>>::#arg3_name)
                                -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    let tmp2 = extract_decl(syn::parse_item(
                        quote! {fn test(
                            arg1: Option<<#cls as #p<'p>>::#arg1_name>,
                            arg2: Option<<#cls as #p<'p>>::#arg2_name>,
                            arg3: Option<<#cls as #p<'p>>::#arg3_name>)
                                -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    modify_arg_ty(sig, 0, &tmp, &tmp2);
                    modify_arg_ty(sig, 1, &tmp, &tmp2);
                    modify_arg_ty(sig, 2, &tmp, &tmp2);

                    if pyres {
                        quote! {
                            impl<'p> #p<'p> for #cls {
                                type #arg1_name = #arg1_ty;
                                type #arg2_name = #arg2_ty;
                                type #arg3_name = #arg3_ty;
                                type Success = #succ;
                                type Result = #ty;
                            }
                        }
                    } else {
                        quote! {
                            impl<'p> #p<'p> for #cls {
                                type #arg1_name = #arg1_ty;
                                type #arg2_name = #arg2_ty;
                                type #arg3_name = #arg3_ty;
                                type Result = #ty;
                            }
                        }
                    }
                },
                MethodProto::Quaternary{name: n, arg1, arg2, arg3, proto} => {
                    if sig.decl.inputs.len() <= 3 {
                        print_err(format!("Not enough arguments {}", n), quote!(sig));
                        return Tokens::new();
                    }
                    let p = syn::Ident::from(proto);
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 1);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 2);
                    let arg3_name = syn::Ident::from(arg3);
                    let arg3_ty = get_arg_ty(sig, 3);
                    let (ty, succ) = get_res_success(ty);

                    // rewrite ty
                    let tmp = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg1: <#cls as #p<'p>>::#arg1_name,
                            arg2: <#cls as #p<'p>>::#arg2_name,
                            arg3: <#cls as #p<'p>>::#arg3_name)
                                -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    let tmp2 = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg1: Option<<#cls as #p<'p>>::#arg1_name>,
                            arg2: Option<<#cls as #p<'p>>::#arg2_name>,
                            arg3: Option<<#cls as #p<'p>>::#arg3_name>)
                                -> <#cls as #p<'p>>::Result {}}.as_str()).unwrap());
                    modify_arg_ty(sig, 1, &tmp, &tmp2);
                    modify_arg_ty(sig, 2, &tmp, &tmp2);
                    modify_arg_ty(sig, 3, &tmp, &tmp2);
                    modify_self_ty(sig);

                    quote! {
                        impl<'p> #p<'p> for #cls {
                            type #arg1_name = #arg1_ty;
                            type #arg2_name = #arg2_ty;
                            type #arg3_name = #arg3_ty;
                            type Success = #succ;
                            type Result = #ty;
                        }
                    }
                },
            }
        },
        _ => panic!("fn return type is not supported"),
    }
}


// TODO: better arg ty detection
fn get_arg_ty(sig: &syn::MethodSig, idx: usize) -> syn::Ty {
    let mut ty = match sig.decl.inputs[idx] {
        syn::FnArg::Captured(_, ref arg_ty) => {
            match arg_ty {
                &syn::Ty::Path(_, ref path) => {
                    // use only last path segment for Option<>
                    let seg = path.segments.last().unwrap().clone();
                    if seg.ident.as_ref() == "Option" {
                        match seg.parameters {
                            syn::PathParameters::AngleBracketed(ref data) => {
                                if let Some(ty) = data.types.last() {
                                    return ty.clone()
                                }
                            }
                            _ => (),
                        }
                    }
                    arg_ty.clone()
                },
                _ => arg_ty.clone()
            }
        },
        _ => panic!("fn arg type is not supported"),
    };

    match ty {
        syn::Ty::Rptr(ref mut lifetime, _) => {
            match lifetime {
                &mut None => {
                    *lifetime = Some(syn::Lifetime {ident: syn::Ident::from("'p")})
                }
                _ => (),
            }
        }
        _ => ()
    }

    ty
}

// Success
fn get_res_success(ty: &syn::Ty) -> (Tokens, syn::Ty) {
    let mut result;
    let mut succ;

    match ty {
        &syn::Ty::Path(_, ref path) => {
            if let Some(segment) = path.segments.last() {
                match segment.ident.as_ref() {
                    // check for PyResult<T>
                    "PyResult" => match segment.parameters {
                        syn::PathParameters::AngleBracketed(ref data) => {
                            result = true;
                            succ = data.types[0].clone();

                            // check for PyResult<Option<T>>
                            match data.types[0] {
                                syn::Ty::Path(_, ref path) =>
                                    if let Some(segment) = path.segments.last() {
                                        match segment.ident.as_ref() {
                                            // get T from Option<T>
                                            "Option" => match segment.parameters {
                                                syn::PathParameters::AngleBracketed(ref data) =>
                                                {
                                                    result = false;
                                                    succ = data.types[0].clone();
                                                },
                                                _ => (),
                                            },
                                            _ => (),
                                        }
                                    },
                                _ => ()
                            }
                        },
                        _ => panic!("fn result type is not supported"),
                    },
                    _ => panic!("fn result type has to be PyResult or (), got {:?}",
                                segment.ident.as_ref())
                }
            } else {
                panic!("fn result is not supported {:?}", path)
            }
        }
        _ => panic!("not supported: {:?}", ty),
    };

    // result
    let res = if result {
        quote! {PyResult<#succ>}
    } else {
        quote! {#ty}
    };

    (res, succ)
}


fn extract_decl(spec: syn::Item) -> syn::FnDecl {
    match spec.node {
        syn::ItemKind::Fn(decl, _, _, _, _, _) => *decl,
        _ => panic!()
    }
}

// modify method signature
fn modify_arg_ty(sig: &mut syn::MethodSig, idx: usize,
                 decl1: &syn::FnDecl, decl2: &syn::FnDecl)
{
    let arg = sig.decl.inputs[idx].clone();
    match arg {
        syn::FnArg::Captured(ref pat, ref arg_ty) => {
            match arg_ty {
                &syn::Ty::Path(_, ref path) => {
                    let seg = path.segments.last().unwrap().clone();
                    if seg.ident.as_ref() == "Option" {
                        sig.decl.inputs[idx] = fix_name(pat, &decl2.inputs[idx]);
                    } else {
                        sig.decl.inputs[idx] = fix_name(pat, &decl1.inputs[idx]);
                    }
                },
                _ => {
                    sig.decl.inputs[idx] = fix_name(pat, &decl1.inputs[idx]);
                }
            }
        },
        _ => panic!("not supported"),
    }

    sig.decl.output = decl1.output.clone();
}

fn modify_self_ty(sig: &mut syn::MethodSig)
{
    match sig.decl.inputs[0] {
        syn::FnArg::SelfRef(ref mut lifetime, _) => {
            *lifetime = Some(syn::Lifetime {ident: syn::Ident::from("'p")})
        },
        _ => panic!("not supported"),
    }
}

fn fix_name(pat: &syn::Pat, arg: &syn::FnArg) -> syn::FnArg {
    match arg {
        &syn::FnArg::Captured(_, ref arg_ty) =>
            syn::FnArg::Captured(pat.clone(), arg_ty.clone()),
        _ => panic!("func.rs::296"),
    }
}
