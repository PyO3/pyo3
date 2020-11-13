// Copyright (c) 2017-present PyO3 Project and Contributors
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Token;

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
    // TODO: workaround for no unsized casts in const fn on Rust 1.45 (stable in 1.46)
    const EMPTY_ARGS: &'static [&'static str] = &[];
    pub const fn new(name: &'static str, proto: &'static str) -> Self {
        MethodProto {
            name,
            proto,
            args: MethodProto::EMPTY_ARGS,
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
    meth: &MethodProto,
) -> syn::Result<TokenStream> {
    let p: syn::Path = syn::parse_str(meth.proto).unwrap();

    let mut impl_types = Vec::new();
    for (i, arg) in meth.args.iter().enumerate() {
        let idx = if meth.with_self { i + 1 } else { i };
        let arg_name = syn::Ident::new(arg, Span::call_site());
        let arg_ty = get_arg_ty(sig, idx)?;

        impl_types.push(quote! {type #arg_name = #arg_ty;});

        let type1 = syn::parse_quote! { arg: <#cls as #p<'p>>::#arg_name};
        let type2 = syn::parse_quote! { arg: Option<<#cls as #p<'p>>::#arg_name>};
        modify_arg_ty(sig, idx, &type1, &type2)?;
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

        sig.output = syn::parse_quote! { -> <#cls as #p<'p>>::Result };
        quote! { type Result = #ret_ty; }
    } else {
        proc_macro2::TokenStream::new()
    };

    Ok(quote! {
        impl<'p> #p<'p> for #cls {
            #(#impl_types)*
            #res_type_def
        }
    })
}

/// Some hacks for arguments: get `T` from `Option<T>` and insert lifetime
fn get_arg_ty(sig: &syn::Signature, idx: usize) -> syn::Result<syn::Type> {
    fn get_option_ty(path: &syn::Path) -> Option<syn::Type> {
        let seg = path.segments.last()?;
        if seg.ident == "Option" {
            if let syn::PathArguments::AngleBracketed(ref data) = seg.arguments {
                if let Some(syn::GenericArgument::Type(ref ty)) = data.args.last() {
                    return Some(ty.to_owned());
                }
            }
        }
        None
    }

    let mut ty = match &sig.inputs[idx] {
        syn::FnArg::Typed(ref cap) => match &*cap.ty {
            // For `Option<T>`, we use `T` as an associated type for the protocol.
            syn::Type::Path(ref ty) => get_option_ty(&ty.path).unwrap_or_else(|| *cap.ty.clone()),
            _ => *cap.ty.clone(),
        },
        ty => {
            return Err(syn::Error::new_spanned(
                ty,
                format!("Unsupported argument type: {:?}", ty),
            ))
        }
    };
    insert_lifetime(&mut ty);
    Ok(ty)
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
            insert_lifetime(&mut *r.elem);
        }
        syn::Type::Path(ref mut path) => insert_lifetime_for_path(path),
        _ => {}
    }
}

fn modify_arg_ty(
    sig: &mut syn::Signature,
    idx: usize,
    decl1: &syn::FnArg,
    decl2: &syn::FnArg,
) -> syn::Result<()> {
    let arg = sig.inputs[idx].clone();
    match arg {
        syn::FnArg::Typed(ref cap) if crate::utils::option_type_argument(&*cap.ty).is_some() => {
            sig.inputs[idx] = fix_name(&cap.pat, &decl2)?;
        }
        syn::FnArg::Typed(ref cap) => {
            sig.inputs[idx] = fix_name(&cap.pat, &decl1)?;
        }
        _ => return Err(syn::Error::new_spanned(arg, "not supported")),
    }

    Ok(())
}

fn modify_self_ty(sig: &mut syn::Signature) {
    match sig.inputs[0] {
        syn::FnArg::Receiver(ref mut slf) => {
            slf.reference = Some((Token![&](Span::call_site()), syn::parse_quote! {'p}));
        }
        syn::FnArg::Typed(_) => {}
    }
}

fn fix_name(pat: &syn::Pat, arg: &syn::FnArg) -> syn::Result<syn::FnArg> {
    if let syn::FnArg::Typed(ref cap) = arg {
        Ok(syn::FnArg::Typed(syn::PatType {
            attrs: cap.attrs.clone(),
            pat: Box::new(pat.clone()),
            colon_token: cap.colon_token,
            ty: cap.ty.clone(),
        }))
    } else {
        Err(syn::Error::new_spanned(arg, "Expected a typed argument"))
    }
}
