use syn;
use quote::Tokens;

// TODO:
//   Add lifetime support for args with Rptr

#[derive(Debug)]
pub enum MethodProto {
    Unary{name: &'static str, pyres: bool, proto: &'static str, },
    Binary{name: &'static str, arg: &'static str, pyres: bool, proto: &'static str},
    Ternary{name: &'static str,
            arg1: &'static str,
            arg2: &'static str,
            pyres: bool, proto: &'static str},
    Quaternary{name: &'static str,
               arg1: &'static str,
               arg2: &'static str,
               arg3: &'static str, proto: &'static str},
}

impl MethodProto {

    pub fn eq(&self, name: &str) -> bool {
        match *self {
            MethodProto::Unary{name: n, pyres: _, proto: _} => n == name,
            MethodProto::Binary{name: n, arg: _, pyres: _, proto: _} => n == name,
            MethodProto::Ternary{name: n, arg1: _, arg2: _, pyres: _, proto: _} => n == name,
            MethodProto::Quaternary{name: n, arg1: _, arg2: _, arg3: _, proto: _} => n == name,
        }
    }
}


pub fn impl_method_proto(cls: &Box<syn::Ty>,
                         sig: &mut syn::MethodSig,
                         meth: &MethodProto) -> Tokens {
    let decl = sig.decl.clone();

    match decl.output {
        syn::FunctionRetTy::Ty(ref ty) => {
            match *meth {
                MethodProto::Unary{name: _, pyres, proto} => {
                    let p = syn::Ident::from(proto);
                    let succ = get_res_success(ty);

                    let tmp = extract_decl(syn::parse_item(
                        quote! {fn test(&self)
                                        -> <#cls as #p<'a>>::Result {}}.as_str()).unwrap());
                    sig.decl.output = tmp.output.clone();

                    if pyres {
                        quote! {
                            impl<'a> #p<'a> for #cls {
                                type Success = #succ;
                                type Result = #ty;
                            }
                        }
                    } else {
                        quote! {
                            impl<'a> #p<'a> for #cls {
                                type Result = #ty;
                            }
                        }
                    }
                },
                MethodProto::Binary{name: _, arg, pyres, proto} => {
                    let p = syn::Ident::from(proto);
                    let arg_name = syn::Ident::from(arg);
                    let arg_ty = get_arg_ty(sig, 1);
                    let succ = get_res_success(ty);

                    let tmp = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg: <#cls as #p<'a>>::#arg_name)
                                -> <#cls as #p<'a>>::Result {}}.as_str()).unwrap());
                    let tmp2 = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg: Option<<#cls as #p<'a>>::#arg_name>)
                                -> <#cls as #p<'a>>::Result {}}.as_str()).unwrap());
                    modify_arg_ty(sig, 1, &tmp, &tmp2);

                    if pyres {
                        quote! {
                            impl<'a> #p<'a> for #cls {
                                type #arg_name = #arg_ty;
                                type Success = #succ;
                                type Result = #ty;
                            }
                        }
                    } else {
                        quote! {
                            impl<'a> #p<'a> for #cls {
                                type #arg_name = #arg_ty;
                                type Result = #ty;
                            }
                        }
                    }
                },
                MethodProto::Ternary{name: _, arg1, arg2, pyres, proto} => {
                    let p = syn::Ident::from(proto);
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 1);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 2);
                    let succ = get_res_success(ty);

                    // rewrite ty
                    let tmp = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg1: <#cls as #p<'a>>::#arg1_name,
                            arg2: <#cls as #p<'a>>::#arg2_name)
                                -> <#cls as #p<'a>>::Result {}}.as_str()).unwrap());
                    let tmp2 = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg1: Option<<#cls as #p<'a>>::#arg1_name>,
                            arg2: Option<<#cls as #p<'a>>::#arg2_name>)
                                -> <#cls as #p<'a>>::Result {}}.as_str()).unwrap());
                    modify_arg_ty(sig, 1, &tmp, &tmp2);
                    modify_arg_ty(sig, 2, &tmp, &tmp2);

                    if pyres {
                        quote! {
                            impl<'a> #p<'a> for #cls {
                                type #arg1_name = #arg1_ty;
                                type #arg2_name = #arg2_ty;
                                type Success = #succ;
                                type Result = #ty;
                            }
                        }
                    } else {
                        quote! {
                            impl<'a> #p<'a> for #cls {
                                type #arg1_name = #arg1_ty;
                                type #arg2_name = #arg2_ty;
                                type Result = #ty;
                            }
                        }
                    }
                },
                MethodProto::Quaternary{name: _, arg1, arg2, arg3, proto} => {
                    let p = syn::Ident::from(proto);
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 2);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 3);
                    let arg3_name = syn::Ident::from(arg3);
                    let arg3_ty = get_arg_ty(sig, 4);
                    let succ = get_res_success(ty);

                    // rewrite ty
                    let tmp = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg1: <#cls as #p<'a>>::#arg1_name,
                            arg2: <#cls as #p<'a>>::#arg2_name,
                            arg3: <#cls as #p<'a>>::#arg3_name)
                                -> <#cls as #p<'a>>::Result {}}.as_str()).unwrap());
                    let tmp2 = extract_decl(syn::parse_item(
                        quote! {fn test(
                            &self,
                            arg1: Option<<#cls as #p<'a>>::#arg1_name>,
                            arg2: Option<<#cls as #p<'a>>::#arg2_name>,
                            arg3: Option<<#cls as #p<'a>>::#arg3_name>)
                                -> <#cls as #p<'a>>::Result {}}.as_str()).unwrap());
                    modify_arg_ty(sig, 1, &tmp, &tmp2);
                    modify_arg_ty(sig, 2, &tmp, &tmp2);
                    modify_arg_ty(sig, 3, &tmp, &tmp2);

                    quote! {
                        impl<'a> #p<'a> for #cls {
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
        _ => panic!("not supported"),
    }
}


// TODO: better arg ty detection
fn get_arg_ty(sig: &syn::MethodSig, idx: usize) -> syn::Ty {
    match sig.decl.inputs[idx] {
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
                _ => {
                    arg_ty.clone()
                }
            }
        },
        _ =>
            panic!("not supported"),
    }
}

// Success
fn get_res_success(ty: &syn::Ty) -> syn::Ty {
    match ty {
        &syn::Ty::Path(_, ref path) => {
            if let Some(segment) = path.segments.last() {
                match segment.ident.as_ref() {
                    // check result type
                    "PyResult" => match segment.parameters {
                        syn::PathParameters::AngleBracketed(ref data) => {
                            data.types[0].clone()
                        },
                        _ => panic!("not supported"),
                    },
                    _ => panic!("not supported"),
                }
            } else {
                panic!("not supported")
            }
        }
        _ => panic!("not supported"),
    }
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
        syn::FnArg::Captured(_, ref arg_ty) => {
            match arg_ty {
                &syn::Ty::Path(_, ref path) => {
                    let seg = path.segments.last().unwrap().clone();
                    if seg.ident.as_ref() == "Option" {
                        sig.decl.inputs[idx] = decl2.inputs[idx].clone();
                    } else {
                        sig.decl.inputs[idx] = decl1.inputs[idx].clone();
                    }
                },
                _ => {
                    sig.decl.inputs[idx] = decl1.inputs[idx].clone();
                }
            }
        },
        _ =>
            panic!("not supported"),
    }

    sig.decl.output = decl1.output.clone();
}
