// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::utils::print_err;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Token;

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
        proto: &'static str,
    },
    UnaryS {
        name: &'static str,
        arg: &'static str,
        proto: &'static str,
    },
    Binary {
        name: &'static str,
        arg: &'static str,
        proto: &'static str,
    },
    BinaryS {
        name: &'static str,
        arg1: &'static str,
        arg2: &'static str,
        proto: &'static str,
    },
    Ternary {
        name: &'static str,
        arg1: &'static str,
        arg2: &'static str,
        proto: &'static str,
    },
    TernaryS {
        name: &'static str,
        arg1: &'static str,
        arg2: &'static str,
        arg3: &'static str,
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

impl MethodProto {
    pub fn name(&self) -> &str {
        match *self {
            MethodProto::Free { ref name, .. } => name,
            MethodProto::Unary { ref name, .. } => name,
            MethodProto::UnaryS { ref name, .. } => name,
            MethodProto::Binary { ref name, .. } => name,
            MethodProto::BinaryS { ref name, .. } => name,
            MethodProto::Ternary { ref name, .. } => name,
            MethodProto::TernaryS { ref name, .. } => name,
            MethodProto::Quaternary { ref name, .. } => name,
        }
    }
}

pub(crate) fn impl_method_proto(
    cls: &syn::Type,
    sig: &mut syn::Signature,
    meth: &MethodProto,
) -> TokenStream {
    if let MethodProto::Free { proto, .. } = meth {
        let p: syn::Path = syn::parse_str(proto).unwrap();
        return quote! {
            impl<'p> #p<'p> for #cls {}
        };
    }

    let ret_ty = &*if let syn::ReturnType::Type(_, ref ty) = sig.output {
        ty.clone()
    } else {
        panic!("fn return type is not supported")
    };

    match *meth {
        MethodProto::Free { .. } => unreachable!(),
        MethodProto::Unary { proto, .. } => {
            let p: syn::Path = syn::parse_str(proto).unwrap();

            let tmp: syn::ItemFn = syn::parse_quote! {
                fn test(&self) -> <#cls as #p<'p>>::Result {}
            };
            sig.output = tmp.sig.output;
            modify_self_ty(sig);

            quote! {
                impl<'p> #p<'p> for #cls {
                    type Result = #ret_ty;
                }
            }
        }
        MethodProto::UnaryS { proto, arg, .. } => {
            let p: syn::Path = syn::parse_str(proto).unwrap();

            let slf_name = syn::Ident::new(arg, Span::call_site());
            let mut slf_ty = get_arg_ty(sig, 0);

            // update the type if no lifetime was given:
            // PyRef<Self> --> PyRef<'p, Self>
            if let syn::Type::Path(ref mut path) = slf_ty {
                if let syn::PathArguments::AngleBracketed(ref mut args) =
                    path.path.segments[0].arguments
                {
                    if let syn::GenericArgument::Lifetime(_) = args.args[0] {
                    } else {
                        let lt = syn::parse_quote! {'p};
                        args.args.insert(0, lt);
                    }
                }
            }

            let tmp: syn::ItemFn = syn::parse_quote! {
                fn test(&self) -> <#cls as #p<'p>>::Result {}
            };
            sig.output = tmp.sig.output;
            modify_self_ty(sig);

            if let syn::FnArg::Typed(ref mut arg) = sig.inputs[0] {
                arg.ty = Box::new(syn::parse_quote! {
                    <#cls as #p<'p>>::#slf_name
                });
            }

            quote! {
                impl<'p> #p<'p> for #cls {
                    type #slf_name = #slf_ty;
                    type Result = #ret_ty;
                }
            }
        }
        MethodProto::Binary { name, arg, proto } => {
            if sig.inputs.len() <= 1 {
                println!("Not enough arguments for {}", name);
                return TokenStream::new();
            }

            let p: syn::Path = syn::parse_str(proto).unwrap();
            let arg_name = syn::Ident::new(arg, Span::call_site());
            let arg_ty = get_arg_ty(sig, 1);

            let tmp = extract_decl(syn::parse_quote! {
                fn test(&self,arg: <#cls as #p<'p>>::#arg_name)-> <#cls as #p<'p>>::Result {}
            });

            let tmp2 = extract_decl(syn::parse_quote! {
                fn test(&self, arg: Option<<#cls as #p<'p>>::#arg_name>) -> <#cls as #p<'p>>::Result {}
            });

            modify_arg_ty(sig, 1, &tmp, &tmp2);
            modify_self_ty(sig);

            quote! {
                impl<'p> #p<'p> for #cls {
                    type #arg_name = #arg_ty;
                    type Result = #ret_ty;
                }
            }
        }
        MethodProto::BinaryS {
            name,
            arg1,
            arg2,
            proto,
        } => {
            if sig.inputs.len() <= 1 {
                print_err(format!("Not enough arguments {}", name), quote!(sig));
                return TokenStream::new();
            }
            let p: syn::Path = syn::parse_str(proto).unwrap();
            let arg1_name = syn::Ident::new(arg1, Span::call_site());
            let arg1_ty = get_arg_ty(sig, 0);
            let arg2_name = syn::Ident::new(arg2, Span::call_site());
            let arg2_ty = get_arg_ty(sig, 1);

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

            quote! {
                impl<'p> #p<'p> for #cls {
                    type #arg1_name = #arg1_ty;
                    type #arg2_name = #arg2_ty;
                    type Result = #ret_ty;
                }
            }
        }
        MethodProto::Ternary {
            name,
            arg1,
            arg2,
            proto,
        } => {
            if sig.inputs.len() <= 2 {
                print_err(format!("Not enough arguments {}", name), quote!(sig));
                return TokenStream::new();
            }
            let p: syn::Path = syn::parse_str(proto).unwrap();
            let arg1_name = syn::Ident::new(arg1, Span::call_site());
            let arg1_ty = get_arg_ty(sig, 1);
            let arg2_name = syn::Ident::new(arg2, Span::call_site());
            let arg2_ty = get_arg_ty(sig, 2);

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

            quote! {
                impl<'p> #p<'p> for #cls {
                    type #arg1_name = #arg1_ty;
                    type #arg2_name = #arg2_ty;
                    type Result = #ret_ty;
                }
            }
        }
        MethodProto::TernaryS {
            name,
            arg1,
            arg2,
            arg3,
            proto,
        } => {
            if sig.inputs.len() <= 2 {
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

            quote! {
                impl<'p> #p<'p> for #cls {
                    type #arg1_name = #arg1_ty;
                    type #arg2_name = #arg2_ty;
                    type #arg3_name = #arg3_ty;
                    type Result = #ret_ty;
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
            if sig.inputs.len() <= 3 {
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
                    type Result = #ret_ty;
                }
            }
        }
    }
}

// TODO: better arg ty detection
fn get_arg_ty(sig: &syn::Signature, idx: usize) -> syn::Type {
    let mut ty = match sig.inputs[idx] {
        syn::FnArg::Typed(ref cap) => {
            match *cap.ty {
                syn::Type::Path(ref ty) => {
                    // use only last path segment for Option<>
                    let seg = ty.path.segments.last().unwrap().clone();
                    if seg.ident == "Option" {
                        if let syn::PathArguments::AngleBracketed(ref data) = seg.arguments {
                            if let Some(pair) = data.args.last() {
                                match pair {
                                    syn::GenericArgument::Type(ref ty) => return ty.clone(),
                                    _ => panic!("Option only accepted for concrete types"),
                                }
                            };
                        }
                    }
                    *cap.ty.clone()
                }
                _ => *cap.ty.clone(),
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

fn extract_decl(spec: syn::Item) -> syn::Signature {
    match spec {
        syn::Item::Fn(f) => f.sig,
        _ => panic!(),
    }
}

// modify method signature
fn modify_arg_ty(
    sig: &mut syn::Signature,
    idx: usize,
    decl1: &syn::Signature,
    decl2: &syn::Signature,
) {
    let arg = sig.inputs[idx].clone();
    match arg {
        syn::FnArg::Typed(ref cap) => match *cap.ty {
            syn::Type::Path(ref typath) => {
                let seg = typath.path.segments.last().unwrap().clone();
                if seg.ident == "Option" {
                    sig.inputs[idx] = fix_name(&cap.pat, &decl2.inputs[idx]);
                } else {
                    sig.inputs[idx] = fix_name(&cap.pat, &decl1.inputs[idx]);
                }
            }
            _ => {
                sig.inputs[idx] = fix_name(&cap.pat, &decl1.inputs[idx]);
            }
        },
        _ => panic!("not supported"),
    }

    sig.output = decl1.output.clone();
}

fn modify_self_ty(sig: &mut syn::Signature) {
    match sig.inputs[0] {
        syn::FnArg::Receiver(ref mut slf) => {
            slf.reference = Some((Token![&](Span::call_site()), syn::parse_quote! {'p}));
        }
        syn::FnArg::Typed(_) => {}
    }
}

fn fix_name(pat: &syn::Pat, arg: &syn::FnArg) -> syn::FnArg {
    if let syn::FnArg::Typed(ref cap) = arg {
        syn::FnArg::Typed(syn::PatType {
            attrs: cap.attrs.clone(),
            pat: Box::new(pat.clone()),
            colon_token: cap.colon_token,
            ty: cap.ty.clone(),
        })
    } else {
        panic!("func.rs::296")
    }
}
