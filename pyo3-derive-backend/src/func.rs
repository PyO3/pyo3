// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::utils::print_err;
use proc_macro2::{Span, TokenStream};
use quote::quote;

// TODO:
//   Add lifetime support for args with Rptr

#[derive(Debug)]
pub enum MethodProto {
    Free {
        name: &'static str,
        proto: &'static str,
    },
    Unary {
        name: &'static str,
        pyres: bool,
        proto: &'static str,
    },
    Binary {
        name: &'static str,
        arg: &'static str,
        pyres: bool,
        proto: &'static str,
    },
    BinaryS {
        name: &'static str,
        arg1: &'static str,
        arg2: &'static str,
        pyres: bool,
        proto: &'static str,
    },
    Ternary {
        name: &'static str,
        arg1: &'static str,
        arg2: &'static str,
        pyres: bool,
        proto: &'static str,
    },
    TernaryS {
        name: &'static str,
        arg1: &'static str,
        arg2: &'static str,
        arg3: &'static str,
        pyres: bool,
        proto: &'static str,
    },
    Quaternary {
        name: &'static str,
        arg1: &'static str,
        arg2: &'static str,
        arg3: &'static str,
        proto: &'static str,
    },
}

impl PartialEq<str> for MethodProto {
    fn eq(&self, name: &str) -> bool {
        match *self {
            MethodProto::Free { name: n, .. } => n == name,
            MethodProto::Unary { name: n, .. } => n == name,
            MethodProto::Binary { name: n, .. } => n == name,
            MethodProto::BinaryS { name: n, .. } => n == name,
            MethodProto::Ternary { name: n, .. } => n == name,
            MethodProto::TernaryS { name: n, .. } => n == name,
            MethodProto::Quaternary { name: n, .. } => n == name,
        }
    }
}

pub fn impl_method_proto(
    cls: &syn::Type,
    sig: &mut syn::MethodSig,
    meth: &MethodProto,
) -> TokenStream {
    if let MethodProto::Free { proto, .. } = meth {
        let p: syn::Path = syn::parse_str(proto).unwrap();
        return quote! {
            impl<'p> #p<'p> for #cls {}
        };
    }

    let ty = &*if let syn::ReturnType::Type(_, ref ty) = sig.decl.output {
        ty.clone()
    } else {
        panic!("fn return type is not supported")
    };

    match *meth {
        MethodProto::Free { .. } => unreachable!(),
        MethodProto::Unary { pyres, proto, .. } => {
            let p: syn::Path = syn::parse_str(proto).unwrap();
            let (ty, succ) = get_res_success(ty);

            let tmp: syn::ItemFn = syn::parse_quote! {
                fn test(&self) -> <#cls as #p<'p>>::Result {}
            };
            sig.decl.output = tmp.decl.output;
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
        }
        MethodProto::Binary {
            name,
            arg,
            pyres,
            proto,
        } => {
            if sig.decl.inputs.len() <= 1 {
                println!("Not enough arguments for {}", name);
                return TokenStream::new();
            }

            let p: syn::Path = syn::parse_str(proto).unwrap();
            let arg_name = syn::Ident::new(arg, Span::call_site());
            let arg_ty = get_arg_ty(sig, 1);
            let (ty, succ) = get_res_success(ty);

            let tmp = extract_decl(syn::parse_quote! {
                fn test(&self,arg: <#cls as #p<'p>>::#arg_name)-> <#cls as #p<'p>>::Result {}
            });

            let tmp2 = extract_decl(syn::parse_quote! {
                fn test(&self, arg: Option<<#cls as #p<'p>>::#arg_name>) -> <#cls as #p<'p>>::Result {}
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
        }
        MethodProto::BinaryS {
            name,
            arg1,
            arg2,
            pyres,
            proto,
        } => {
            if sig.decl.inputs.len() <= 1 {
                print_err(format!("Not enough arguments {}", name), quote!(sig));
                return TokenStream::new();
            }
            let p: syn::Path = syn::parse_str(proto).unwrap();
            let arg1_name = syn::Ident::new(arg1, Span::call_site());
            let arg1_ty = get_arg_ty(sig, 0);
            let arg2_name = syn::Ident::new(arg2, Span::call_site());
            let arg2_ty = get_arg_ty(sig, 1);
            let (ty, succ) = get_res_success(ty);

            // rewrite ty
            let tmp = extract_decl(syn::parse_quote! {fn test(
            arg1: <#cls as #p<'p>>::#arg1_name,
            arg2: <#cls as #p<'p>>::#arg2_name)
                -> <#cls as #p<'p>>::Result {}});
            let tmp2 = extract_decl(syn::parse_quote! {fn test(
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
        }
        MethodProto::Ternary {
            name,
            arg1,
            arg2,
            pyres,
            proto,
        } => {
            if sig.decl.inputs.len() <= 2 {
                print_err(format!("Not enough arguments {}", name), quote!(sig));
                return TokenStream::new();
            }
            let p: syn::Path = syn::parse_str(proto).unwrap();
            let arg1_name = syn::Ident::new(arg1, Span::call_site());
            let arg1_ty = get_arg_ty(sig, 1);
            let arg2_name = syn::Ident::new(arg2, Span::call_site());
            let arg2_ty = get_arg_ty(sig, 2);
            let (ty, succ) = get_res_success(ty);

            // rewrite ty
            let tmp = extract_decl(syn::parse_quote! {fn test(
            &self,
            arg1: <#cls as #p<'p>>::#arg1_name,
            arg2: <#cls as #p<'p>>::#arg2_name)
                -> <#cls as #p<'p>>::Result {}});
            let tmp2 = extract_decl(syn::parse_quote! {fn test(
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
        }
        MethodProto::TernaryS {
            name,
            arg1,
            arg2,
            arg3,
            pyres,
            proto,
        } => {
            if sig.decl.inputs.len() <= 2 {
                print_err(format!("Not enough arguments {}", name), quote!(sig));
                return TokenStream::new();
            }
            let p: syn::Path = syn::parse_str(proto).unwrap();
            let arg1_name = syn::Ident::new(arg1, Span::call_site());
            let arg1_ty = get_arg_ty(sig, 0);
            let arg2_name = syn::Ident::new(arg2, Span::call_site());
            let arg2_ty = get_arg_ty(sig, 1);
            let arg3_name = syn::Ident::new(arg3, Span::call_site());
            let arg3_ty = get_arg_ty(sig, 2);
            let (ty, succ) = get_res_success(ty);

            // rewrite ty
            let tmp = extract_decl(syn::parse_quote! {fn test(
            arg1: <#cls as #p<'p>>::#arg1_name,
            arg2: <#cls as #p<'p>>::#arg2_name,
            arg3: <#cls as #p<'p>>::#arg3_name)
                -> <#cls as #p<'p>>::Result {}});
            let tmp2 = extract_decl(syn::parse_quote! {fn test(
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
        }
        MethodProto::Quaternary {
            name,
            arg1,
            arg2,
            arg3,
            proto,
        } => {
            if sig.decl.inputs.len() <= 3 {
                print_err(format!("Not enough arguments {}", name), quote!(sig));
                return TokenStream::new();
            }
            let p: syn::Path = syn::parse_str(proto).unwrap();
            let arg1_name = syn::Ident::new(arg1, Span::call_site());
            let arg1_ty = get_arg_ty(sig, 1);
            let arg2_name = syn::Ident::new(arg2, Span::call_site());
            let arg2_ty = get_arg_ty(sig, 2);
            let arg3_name = syn::Ident::new(arg3, Span::call_site());
            let arg3_ty = get_arg_ty(sig, 3);
            let (ty, succ) = get_res_success(ty);

            // rewrite ty
            let tmp = extract_decl(syn::parse_quote! {fn test(
            &self,
            arg1: <#cls as #p<'p>>::#arg1_name,
            arg2: <#cls as #p<'p>>::#arg2_name,
            arg3: <#cls as #p<'p>>::#arg3_name)
                -> <#cls as #p<'p>>::Result {}});
            let tmp2 = extract_decl(syn::parse_quote! {fn test(
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
        }
    }
}

// TODO: better arg ty detection
fn get_arg_ty(sig: &syn::MethodSig, idx: usize) -> syn::Type {
    let mut ty = match sig.decl.inputs[idx] {
        syn::FnArg::Captured(ref cap) => {
            match cap.ty {
                syn::Type::Path(ref ty) => {
                    // use only last path segment for Option<>
                    let seg = *ty.path.segments.last().unwrap().value();
                    if seg.ident == "Option" {
                        if let syn::PathArguments::AngleBracketed(ref data) = seg.arguments {
                            if let Some(pair) = data.args.last() {
                                match pair.value() {
                                    syn::GenericArgument::Type(ref ty) => return ty.clone(),
                                    _ => panic!("Option only accepted for concrete types"),
                                }
                            };
                        }
                    }
                    cap.ty.clone()
                }
                _ => cap.ty.clone(),
            }
        }
        _ => panic!("fn arg type is not supported"),
    };

    // Add a lifetime if there is none
    if let syn::Type::Reference(ref mut r) = ty {
        r.lifetime.get_or_insert(syn::parse_quote! {'p});
    }

    ty
}

// Success
fn get_res_success(ty: &syn::Type) -> (TokenStream, syn::GenericArgument) {
    let mut result;
    let mut succ;

    match ty {
        syn::Type::Path(ref typath) => {
            if let Some(segment) = typath.path.segments.last() {
                match segment.value().ident.to_string().as_str() {
                    // check for PyResult<T>
                    "PyResult" => match segment.value().arguments {
                        syn::PathArguments::AngleBracketed(ref data) => {
                            result = true;
                            succ = data.args[0].clone();

                            // check for PyResult<Option<T>>
                            if let syn::GenericArgument::Type(syn::Type::Path(ref typath)) =
                                data.args[0]
                            {
                                if let Some(segment) = typath.path.segments.last() {
                                    if "Option" == segment.value().ident.to_string().as_str() {
                                        // get T from Option<T>
                                        if let syn::PathArguments::AngleBracketed(ref data) =
                                            segment.value().arguments
                                        {
                                            result = false;
                                            succ = data.args[0].clone();
                                        }
                                    }
                                }
                            }
                        }
                        _ => panic!("fn result type is not supported"),
                    },
                    _ => panic!(
                        "fn result type has to be PyResult or (), got {:?}",
                        segment.value().ident
                    ),
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
        _ => panic!(),
    }
}

// modify method signature
fn modify_arg_ty(sig: &mut syn::MethodSig, idx: usize, decl1: &syn::FnDecl, decl2: &syn::FnDecl) {
    let arg = sig.decl.inputs[idx].clone();
    match arg {
        syn::FnArg::Captured(ref cap) => match cap.ty {
            syn::Type::Path(ref typath) => {
                let seg = *typath.path.segments.last().unwrap().value();
                if seg.ident == "Option" {
                    sig.decl.inputs[idx] = fix_name(&cap.pat, &decl2.inputs[idx]);
                } else {
                    sig.decl.inputs[idx] = fix_name(&cap.pat, &decl1.inputs[idx]);
                }
            }
            _ => {
                sig.decl.inputs[idx] = fix_name(&cap.pat, &decl1.inputs[idx]);
            }
        },
        _ => panic!("not supported"),
    }

    sig.decl.output = decl1.output.clone();
}

fn modify_self_ty(sig: &mut syn::MethodSig) {
    match sig.decl.inputs[0] {
        syn::FnArg::SelfRef(ref mut slf) => {
            slf.lifetime = Some(syn::parse_quote! {'p});
        }
        syn::FnArg::Captured(_) => {}
        _ => panic!("not supported"),
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
