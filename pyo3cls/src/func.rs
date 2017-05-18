use syn;
use quote::Tokens;

// TODO:
//   Add lifetime support for args with Rptr

pub enum MethodProto {
    Len{name: &'static str, proto: &'static str},
    Unary{name: &'static str, proto: &'static str},
    Binary{name: &'static str, arg: &'static str, proto: &'static str},
    Ternary{name: &'static str, arg1: &'static str, arg2: &'static str, proto: &'static str},
}

impl MethodProto {

    pub fn eq(&self, name: &str) -> bool {
        match *self {
            MethodProto::Len{name: n, proto: _} => n == name,
            MethodProto::Unary{name: n, proto: _} => n == name,
            MethodProto::Binary{name: n, arg: _, proto: _} => n == name,
            MethodProto::Ternary{name: n, arg1: _, arg2: _, proto: _} => n == name,
        }
    }
}


pub fn impl_method_proto(cls: &Box<syn::Ty>,
                         sig: &syn::MethodSig,
                         meth: &MethodProto) -> Tokens {
    match sig.decl.output {
        syn::FunctionRetTy::Ty(ref ty) => {
            match *meth {
                MethodProto::Len{name: _, proto} => {
                    let p = syn::Ident::from(proto);
                    quote! {
                        impl #p for #cls {
                            type Result = #ty;
                        }
                    }
                },
                MethodProto::Unary{name: _, proto} => {
                    let p = syn::Ident::from(proto);
                    let succ = get_res_success(ty);

                    quote! {
                        impl #p for #cls {
                            type Success = #succ;
                            type Result = #ty;
                        }
                    }
                },
                MethodProto::Binary{name: _, arg, proto} => {
                    let p = syn::Ident::from(proto);
                    let arg_name = syn::Ident::from(arg);
                    let arg_ty = get_arg_ty(sig, 2);
                    let succ = get_res_success(ty);

                    quote! {
                        impl #p for #cls {
                            type #arg_name = #arg_ty;
                            type Success = #succ;
                            type Result = #ty;
                        }
                    }
                },
                MethodProto::Ternary{name: _, arg1, arg2, proto} => {
                    let p = syn::Ident::from(proto);
                    let arg1_name = syn::Ident::from(arg1);
                    let arg1_ty = get_arg_ty(sig, 2);
                    let arg2_name = syn::Ident::from(arg2);
                    let arg2_ty = get_arg_ty(sig, 3);
                    let succ = get_res_success(ty);

                    quote! {
                        impl #p for #cls {
                            type #arg1_name = #arg1_ty;
                            type #arg2_name = #arg2_ty;
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


fn get_arg_ty(sig: &syn::MethodSig, idx: usize) -> syn::Ty {
    match sig.decl.inputs[idx] {
        syn::FnArg::Captured(_, ref arg_ty) => {
            arg_ty.clone()
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
