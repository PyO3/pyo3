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


pub fn impl_method_proto(
    cls: &Box<syn::Type>,
    sig: &mut syn::MethodSig,
    meth: &MethodProto
) -> Tokens {

    let decl = sig.decl.clone();

    match *meth {
        MethodProto::Free{name: _, proto} => {
            let p: syn::Path = syn::parse_str(proto).unwrap();
            return quote! {
                impl<'p> #p<'p> for #cls {}
            }
        },
        _ => (),
    };

    match decl.output {
        syn::ReturnType::Type(_, ref ty) => {
            match *meth {
                MethodProto::Free{name: _, proto: _} => unreachable!(),
                MethodProto::Unary{name: _, pyres, proto} => {
                    let p: syn::Path = syn::parse_str(proto).unwrap();
                    let (ty, succ) = get_res_success(ty);

                    let tmp = extract_decl(parse_quote!{
                        fn test(&self) -> <#cls as #p<'p>>::Result {}
                    });
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

                    let p: syn::Path = syn::parse_str(proto).unwrap();
                    let arg_name = syn::Ident::from(arg);
                    let arg_ty = get_arg_ty(sig, 1);
                    let (ty, succ) = get_res_success(ty);

                    let tmp = extract_decl(
                        parse_quote!{
                            fn test(&self,arg: <#cls as #p<'p>>::#arg_name)-> <#cls as #p<'p>>::Result {}
                        });

                    let tmp2 = extract_decl(
                        parse_quote!{
                            fn test( &self, arg: Option<<#cls as #p<'p>>::#arg_name>) -> <#cls as #p<'p>>::Result {}
                        });

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
                    let p: syn::Path = syn::parse_str(proto).unwrap();
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 0);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 1);
                    let (ty, succ) = get_res_success(ty);

                    // rewrite ty
                    let tmp = extract_decl(
                        parse_quote!{fn test(
                            arg1: <#cls as #p<'p>>::#arg1_name,
                            arg2: <#cls as #p<'p>>::#arg2_name)
                                -> <#cls as #p<'p>>::Result {}});
                    let tmp2 = extract_decl(
                        parse_quote!{fn test(
                            arg1: Option<<#cls as #p<'p>>::#arg1_name>,
                            arg2: Option<<#cls as #p<'p>>::#arg2_name>)
                                -> <#cls as #p<'p>>::Result {}});
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
                    let p: syn::Path = syn::parse_str(proto).unwrap();
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 1);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 2);
                    let (ty, succ) = get_res_success(ty);

                    // rewrite ty
                    let tmp = extract_decl(
                        parse_quote! {fn test(
                            &self,
                            arg1: <#cls as #p<'p>>::#arg1_name,
                            arg2: <#cls as #p<'p>>::#arg2_name)
                                -> <#cls as #p<'p>>::Result {}});
                    let tmp2 = extract_decl(
                        parse_quote! {fn test(
                            &self,
                            arg1: Option<<#cls as #p<'p>>::#arg1_name>,
                            arg2: Option<<#cls as #p<'p>>::#arg2_name>)
                                -> <#cls as #p<'p>>::Result {}});
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
                    let p: syn::Path = syn::parse_str(proto).unwrap();
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 0);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 1);
                    let arg3_name = syn::Ident::from(arg3);
                    let arg3_ty = get_arg_ty(sig, 2);
                    let (ty, succ) = get_res_success(ty);

                    // rewrite ty
                    let tmp = extract_decl(
                        parse_quote! {fn test(
                            arg1: <#cls as #p<'p>>::#arg1_name,
                            arg2: <#cls as #p<'p>>::#arg2_name,
                            arg3: <#cls as #p<'p>>::#arg3_name)
                                -> <#cls as #p<'p>>::Result {}});
                    let tmp2 = extract_decl(
                        parse_quote! {fn test(
                            arg1: Option<<#cls as #p<'p>>::#arg1_name>,
                            arg2: Option<<#cls as #p<'p>>::#arg2_name>,
                            arg3: Option<<#cls as #p<'p>>::#arg3_name>)
                                -> <#cls as #p<'p>>::Result {}});
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
                    let p: syn::Path = syn::parse_str(proto).unwrap();
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 1);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 2);
                    let arg3_name = syn::Ident::from(arg3);
                    let arg3_ty = get_arg_ty(sig, 3);
                    let (ty, succ) = get_res_success(ty);

                    // rewrite ty
                    let tmp = extract_decl(
                        parse_quote! {fn test(
                            &self,
                            arg1: <#cls as #p<'p>>::#arg1_name,
                            arg2: <#cls as #p<'p>>::#arg2_name,
                            arg3: <#cls as #p<'p>>::#arg3_name)
                                -> <#cls as #p<'p>>::Result {}});
                    let tmp2 = extract_decl(
                        parse_quote! {fn test(
                            &self,
                            arg1: Option<<#cls as #p<'p>>::#arg1_name>,
                            arg2: Option<<#cls as #p<'p>>::#arg2_name>,
                            arg3: Option<<#cls as #p<'p>>::#arg3_name>)
                                -> <#cls as #p<'p>>::Result {}});
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
fn get_arg_ty(sig: &syn::MethodSig, idx: usize) -> syn::Type {
    let mut ty = match sig.decl.inputs[idx] {
        syn::FnArg::Captured(ref cap) => {
            match cap.ty {
                syn::Type::Path(ref ty) => {
                    // use only last path segment for Option<>
                    let seg = ty.path.segments.last().unwrap().value().clone();
                    if seg.ident.as_ref() == "Option" {
                        match seg.arguments {
                            syn::PathArguments::AngleBracketed(ref data) => {
                                if let Some(pair) = data.args.last() {
                                    match pair.value() {
                                        syn::GenericArgument::Type(ref ty) => return ty.clone(),
                                        _ => panic!("Option only accepted for concrete types"),
                                    }
                                };
                            }
                            _ => (),
                        }
                    }
                    cap.ty.clone()
                },
                _ => cap.ty.clone()
            }
        },
        _ => panic!("fn arg type is not supported"),
    };


    if let syn::Type::Reference(ref mut r) = ty {
        r.lifetime.get_or_insert(parse_quote!{'p});
    }

    ty
}

// Success
fn get_res_success(ty: &syn::Type) -> (Tokens, syn::GenericArgument) {
    let mut result;
    let mut succ;

    match ty {
        &syn::Type::Path(ref typath) => {
            if let Some(segment) = typath.path.segments.last() {
                match segment.value().ident.as_ref() {
                    // check for PyResult<T>
                    "PyResult" => match segment.value().arguments {
                        syn::PathArguments::AngleBracketed(ref data) => {
                            result = true;
                            succ = data.args[0].clone();

                            // check for PyResult<Option<T>>
                            match data.args[0] {
                                syn::GenericArgument::Type(syn::Type::Path(ref typath)) =>
                                    if let Some(segment) = typath.path.segments.last() {
                                        match segment.value().ident.as_ref() {
                                            // get T from Option<T>
                                            "Option" => match segment.value().arguments {
                                                syn::PathArguments::AngleBracketed(ref data) =>
                                                {
                                                    result = false;
                                                    succ = data.args[0].clone();
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
                                segment.value().ident.as_ref())
                }
            } else {
                panic!("fn result is not supported {:?}", typath)
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
    match spec {
        syn::Item::Fn(f) => *f.decl,
        _ => panic!()
    }
}

// modify method signature
fn modify_arg_ty(sig: &mut syn::MethodSig, idx: usize,
                 decl1: &syn::FnDecl, decl2: &syn::FnDecl)
{
    let arg = sig.decl.inputs[idx].clone();
    match arg {
        syn::FnArg::Captured(ref cap) => {
            match cap.ty {
                syn::Type::Path(ref typath) => {
                    let seg = typath.path.segments.last().unwrap().value().clone();
                    if seg.ident.as_ref() == "Option" {
                        sig.decl.inputs[idx] = fix_name(&cap.pat, &decl2.inputs[idx]);
                    } else {
                        sig.decl.inputs[idx] = fix_name(&cap.pat, &decl1.inputs[idx]);
                    }
                },
                _ => {
                    sig.decl.inputs[idx] = fix_name(&cap.pat, &decl1.inputs[idx]);
                }
            }
        },
        _ => panic!("not supported"),
    }

    sig.decl.output = decl1.output.clone();
}

fn modify_self_ty(sig: &mut syn::MethodSig) {
    if let syn::FnArg::SelfRef(ref mut r) = sig.decl.inputs[0] {
        r.lifetime = Some(parse_quote!{'p});
    } else {
        panic!("not supported")
    }
}

fn fix_name(pat: &syn::Pat, arg: &syn::FnArg) -> syn::FnArg {
    if let syn::FnArg::Captured(ref cap) = arg {
        syn::FnArg::Captured(syn::ArgCaptured {
            pat: pat.clone(),
            colon_token: cap.colon_token,
            ty: cap.ty.clone(),
        })
    } else {
        panic!("func.rs::296")
    }
}
