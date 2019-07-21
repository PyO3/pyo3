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
        impl_methods(&ast.self_ty, &mut ast.items)
    }
}

fn binary_func_protocol_wrap(ty: &syn::Type, name: &syn::Ident) -> TokenStream {
    quote! {{
        #[allow(unused_mut)]
        unsafe extern "C" fn wrap(
            lhs: *mut pyo3::ffi::PyObject,
            rhs: *mut pyo3::ffi::PyObject,
        ) -> *mut pyo3::ffi::PyObject {
            use pyo3::ObjectProtocol;
            let _pool = pyo3::GILPool::new();
            let py = pyo3::Python::assume_gil_acquired();
            let lhs = py.from_borrowed_ptr::<pyo3::types::PyAny>(lhs);
            let rhs = py.from_borrowed_ptr::<pyo3::types::PyAny>(rhs);

            let result = match lhs.extract() {
                Ok(lhs) => match rhs.extract() {
                    Ok(rhs) => #ty::#name(lhs, rhs).into(),
                    Err(e) => Err(e.into()),
                },
                Err(e) => Err(e.into()),
            };
            pyo3::callback::cb_convert(pyo3::callback::PyObjectCallbackConverter, py, result)
        }
        pyo3::class::methods::protocols::PyProcotolMethodWrapped::Add(wrap)
    }}
}

pub fn impl_methods(ty: &syn::Type, impls: &mut Vec<syn::ImplItem>) -> syn::Result<TokenStream> {
    // get method names in impl block
    let mut methods = Vec::new();
    let mut protocol_methods = Vec::new();
    for iimpl in impls.iter_mut() {
        if let syn::ImplItem::Method(ref mut meth) = iimpl {
            let name = meth.sig.ident.clone();

            if name.to_string().starts_with("__") && name.to_string().ends_with("__") {
                #[allow(clippy::single_match)]
                {
                    match name.to_string().as_str() {
                        "__add__" => {
                            protocol_methods.push(binary_func_protocol_wrap(&ty, &name));
                        }
                        _ => {
                            // This currently breaks the tests
                            /*
                                return Err(syn::Error::new_spanned(
                                    meth.sig.ident.clone(),
                                    "Unknown dunder method",
                                ))
                            */
                        }
                    }
                }
            }

            methods.push(pymethod::gen_py_method(
                ty,
                &name,
                &mut meth.sig,
                &mut meth.attrs,
            )?);
        }
    }

    Ok(quote! {
        pyo3::inventory::submit! {
            #![crate = pyo3] {
                type TyInventory = <#ty as pyo3::class::methods::PyMethodsInventoryDispatch>::InventoryType;
                <TyInventory as pyo3::class::methods::PyMethodsInventory>::new(&[#(#methods),*])
            }
        }

        pyo3::inventory::submit! {
            #![crate = pyo3] {
                type ProtocolInventory = <#ty as pyo3::class::methods::protocols::PyProtocolInventoryDispatch>::ProtocolInventoryType;
                <ProtocolInventory as pyo3::class::methods::protocols::PyProtocolInventory>::new(&[#(#protocol_methods),*])
            }
        }
    })
}
