// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::pymethod;
use proc_macro2::TokenStream;
use quote::quote;

pub fn build_py_methods(ast: &mut syn::ItemImpl) -> syn::Result<TokenStream> {
    if let Some((_, ref path, _)) = ast.trait_ {
        Err(syn::Error::new_spanned(
            path,
            "#[pymethods] can not be used only with trait impl block",
        ))
    } else if ast.generics != Default::default() {
        Err(syn::Error::new_spanned(
            ast.generics.clone(),
            "#[pymethods] can not be used with lifetime parameters or generics",
        ))
    } else {
        Ok(impl_methods(&ast.self_ty, &mut ast.items))
    }
}

pub fn impl_methods(ty: &syn::Type, impls: &mut Vec<syn::ImplItem>) -> TokenStream {
    // get method names in impl block
    let mut methods = Vec::new();
    for iimpl in impls.iter_mut() {
        if let syn::ImplItem::Method(ref mut meth) = iimpl {
            let name = meth.sig.ident.clone();
            methods.push(pymethod::gen_py_method(
                ty,
                &name,
                &mut meth.sig,
                &mut meth.attrs,
            ));
        }
    }

    quote! {
       pyo3::inventory::submit! {
            #![crate = pyo3] {
                type TyInventory = <#ty as pyo3::class::methods::PyMethodsInventoryDispatch>::InventoryType;
                <TyInventory as pyo3::class::methods::PyMethodsInventory>::new(&[#(#methods),*])
            }
        }
    }
}
