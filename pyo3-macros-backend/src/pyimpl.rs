// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::pymethod;
use proc_macro2::TokenStream;
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
    let mut methods = Vec::new();
    let mut cfg_attributes = Vec::new();
    for iimpl in impls.iter_mut() {
        match iimpl {
            syn::ImplItem::Method(meth) => {
                methods.push(pymethod::gen_py_method(ty, &mut meth.sig, &mut meth.attrs)?);
                cfg_attributes.push(get_cfg_attributes(&meth.attrs));
            }
            syn::ImplItem::Const(konst) => {
                if let Some(meth) = pymethod::gen_py_const(ty, &konst.ident, &mut konst.attrs)? {
                    methods.push(meth);
                }
                cfg_attributes.push(get_cfg_attributes(&konst.attrs));
            }
            _ => (),
        }
    }

    Ok(quote! {
       pyo3::inventory::submit! {
            #![crate = pyo3] {
                type Inventory = <#ty as pyo3::class::methods::HasMethodsInventory>::Methods;
                <Inventory as pyo3::class::methods::PyMethodsInventory>::new(vec![#(
                    #(#cfg_attributes)*
                    #methods
                ),*])
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
