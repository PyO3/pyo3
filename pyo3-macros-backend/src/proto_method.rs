// Copyright (c) 2017-present PyO3 Project and Contributors
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Token};

// TODO:
//   Add lifetime support for args with Rptr
#[derive(Debug)]
pub struct MethodProto {
    pub name: &'static str,
    pub args: &'static [&'static str],
    pub proto: &'static str,
    pub with_self: bool,
    pub with_result: bool,
}

impl MethodProto {
    pub const fn new(name: &'static str, proto: &'static str) -> Self {
        MethodProto {
            name,
            proto,
            args: &[],
            with_self: false,
            with_result: true,
        }
    }
    pub const fn args(mut self, args: &'static [&'static str]) -> MethodProto {
        self.args = args;
        self
    }
    pub const fn has_self(mut self) -> MethodProto {
        self.with_self = true;
        self
    }
    pub const fn no_result(mut self) -> MethodProto {
        self.with_result = false;
        self
    }
}

pub(crate) fn impl_method_proto(
    cls: &syn::Type,
    sig: &mut syn::Signature,
    module: &syn::Path,
    meth: &MethodProto,
) -> syn::Result<TokenStream> {
    let proto: syn::Path = syn::parse_str(meth.proto).unwrap();

    let mut impl_types = Vec::new();
    for (i, arg) in meth.args.iter().enumerate() {
        let idx = if meth.with_self { i + 1 } else { i };
        let arg_name = syn::Ident::new(arg, Span::call_site());
        let input = match &mut sig.inputs[idx] {
            syn::FnArg::Typed(input) => input,
            receiver @ syn::FnArg::Receiver(_) => {
                bail_spanned!(receiver.span() => "unexpected receiver in #[pyproto]")
            }
        };
        // replace signature in trait with the parametrised one, which is identical to the declared
        // function signature.
        let decl = syn::parse_quote! { <#cls as #module::#proto<'p>>::#arg_name };
        let mut arg_ty = match crate::utils::option_type_argument(&input.ty) {
            Some(arg_ty) => {
                let arg_ty = arg_ty.clone();
                *input.ty = syn::parse_quote! { Option<#decl> };
                arg_ty
            }
            None => std::mem::replace(&mut *input.ty, decl),
        };
        // ensure the type has all lifetimes so it can be used in the protocol trait associated type
        insert_lifetime(&mut arg_ty);
        impl_types.push(quote! {type #arg_name = #arg_ty;});
    }

    if meth.with_self {
        modify_self_ty(sig);
    }

    let res_type_def = if meth.with_result {
        let ret_ty = match &sig.output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, ty) => {
                let mut ty = ty.clone();
                insert_lifetime(&mut ty);
                ty.to_token_stream()
            }
        };

        sig.output = syn::parse_quote! { -> <#cls as #module::#proto<'p>>::Result };
        quote! { type Result = #ret_ty; }
    } else {
        proc_macro2::TokenStream::new()
    };

    Ok(quote! {
        impl<'p> #module::#proto<'p> for #cls {
            #(#impl_types)*
            #res_type_def
        }
    })
}

/// Insert lifetime `'p` to `PyRef<Self>` or references (e.g., `&PyType`).
fn insert_lifetime(ty: &mut syn::Type) {
    fn insert_lifetime_for_path(path: &mut syn::TypePath) {
        if let Some(seg) = path.path.segments.last_mut() {
            if let syn::PathArguments::AngleBracketed(ref mut args) = seg.arguments {
                let mut has_lifetime = false;
                for arg in &mut args.args {
                    match arg {
                        // Insert `'p` recursively for `Option<PyRef<Self>>` or so.
                        syn::GenericArgument::Type(ref mut ty) => insert_lifetime(ty),
                        syn::GenericArgument::Lifetime(_) => has_lifetime = true,
                        _ => {}
                    }
                }
                // Insert lifetime to PyRef (i.e., PyRef<Self> -> PyRef<'p, Self>)
                if !has_lifetime && (seg.ident == "PyRef" || seg.ident == "PyRefMut") {
                    args.args.insert(0, syn::parse_quote! {'p});
                }
            }
        }
    }

    match ty {
        syn::Type::Reference(ref mut r) => {
            r.lifetime.get_or_insert(syn::parse_quote! {'p});
            insert_lifetime(&mut r.elem);
        }
        syn::Type::Path(ref mut path) => insert_lifetime_for_path(path),
        _ => {}
    }
}

fn modify_self_ty(sig: &mut syn::Signature) {
    match sig.inputs[0] {
        syn::FnArg::Receiver(ref mut slf) => {
            slf.reference = Some((Token![&](Span::call_site()), syn::parse_quote! {'p}));
        }
        syn::FnArg::Typed(_) => {}
    }
}
