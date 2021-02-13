// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::pymethod;
use proc_macro2::TokenStream;
use pymethod::GeneratedPyMethod;
use quote::quote;
use syn::spanned::Spanned;

pub fn build_py_methods(ast: &mut syn::ItemImpl) -> syn::Result<TokenStream> {
    if let Some((_, path, _)) = &ast.trait_ {
        bail_spanned!(path.span() => "#[pymethods] cannot be used on trait impl blocks");
    } else if ast.generics != Default::default() {
        bail_spanned!(
            ast.generics.span() =>
            "#[pymethods] cannot be used with lifetime parameters or generics"
        );
    } else {
        impl_methods(&ast.self_ty, &mut ast.items)
    }
}

pub fn impl_methods(ty: &syn::Type, impls: &mut Vec<syn::ImplItem>) -> syn::Result<TokenStream> {
    let mut new_impls = Vec::new();
    let mut call_impls = Vec::new();
    let mut methods = Vec::new();
    for iimpl in impls.iter_mut() {
        match iimpl {
            syn::ImplItem::Method(meth) => {
                match pymethod::gen_py_method(ty, &mut meth.sig, &mut meth.attrs)? {
                    GeneratedPyMethod::Method(token_stream) => {
                        let attrs = get_cfg_attributes(&meth.attrs);
                        methods.push(quote!(#(#attrs)* #token_stream));
                    }
                    GeneratedPyMethod::New(token_stream) => {
                        let attrs = get_cfg_attributes(&meth.attrs);
                        new_impls.push(quote!(#(#attrs)* #token_stream));
                    }
                    GeneratedPyMethod::Call(token_stream) => {
                        let attrs = get_cfg_attributes(&meth.attrs);
                        call_impls.push(quote!(#(#attrs)* #token_stream));
                    }
                }
            }
            syn::ImplItem::Const(konst) => {
                if let Some(meth) = pymethod::gen_py_const(ty, &konst.ident, &mut konst.attrs)? {
                    let attrs = get_cfg_attributes(&konst.attrs);
                    methods.push(quote!(#(#attrs)* #meth));
                }
            }
            _ => (),
        }
    }

    Ok(quote! {
        #(#new_impls)*

        #(#call_impls)*

        pyo3::inventory::submit! {
            #![crate = pyo3] {
                type Inventory = <#ty as pyo3::class::methods::HasMethodsInventory>::Methods;
                <Inventory as pyo3::class::methods::PyMethodsInventory>::new(vec![#(#methods),*])
            }
        }
    })
}

fn get_cfg_attributes(attrs: &[syn::Attribute]) -> Vec<&syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("cfg"))
        .collect()
}
