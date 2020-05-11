// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::defs;
use crate::func::impl_method_proto;
use crate::method::FnSpec;
use crate::pymethod;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

pub fn build_py_proto(ast: &mut syn::ItemImpl) -> syn::Result<TokenStream> {
    if let Some((_, ref mut path, _)) = ast.trait_ {
        let proto = if let Some(ref mut segment) = path.segments.last() {
            match segment.ident.to_string().as_str() {
                "PyObjectProtocol" => &defs::OBJECT,
                "PyAsyncProtocol" => &defs::ASYNC,
                "PyMappingProtocol" => &defs::MAPPING,
                "PyIterProtocol" => &defs::ITER,
                "PyContextProtocol" => &defs::CONTEXT,
                "PySequenceProtocol" => &defs::SEQ,
                "PyNumberProtocol" => &defs::NUM,
                "PyDescrProtocol" => &defs::DESCR,
                "PyBufferProtocol" => &defs::BUFFER,
                "PyGCProtocol" => &defs::GC,
                _ => {
                    return Err(syn::Error::new_spanned(
                        path,
                        "#[pyproto] can not be used with this block",
                    ));
                }
            }
        } else {
            return Err(syn::Error::new_spanned(
                path,
                "#[pyproto] can only be used with protocol trait implementations",
            ));
        };

        let tokens = impl_proto_impl(&ast.self_ty, &mut ast.items, proto)?;

        // attach lifetime
        let mut seg = path.segments.pop().unwrap().into_value();
        seg.arguments = syn::PathArguments::AngleBracketed(syn::parse_quote! {<'p>});
        path.segments.push(seg);
        ast.generics.params = syn::parse_quote! {'p};

        Ok(tokens)
    } else {
        Err(syn::Error::new_spanned(
            ast,
            "#[pyproto] can only be used with protocol trait implementations",
        ))
    }
}

fn impl_proto_impl(
    ty: &syn::Type,
    impls: &mut Vec<syn::ImplItem>,
    proto: &defs::Proto,
) -> syn::Result<TokenStream> {
    let mut trait_impls = TokenStream::new();
    let mut py_methods = Vec::new();

    for iimpl in impls.iter_mut() {
        if let syn::ImplItem::Method(ref mut met) = iimpl {
            if let Some(m) = proto.get_proto(&met.sig.ident) {
                impl_method_proto(ty, &mut met.sig, m).to_tokens(&mut trait_impls);
            }
            if let Some(m) = proto.get_method(&met.sig.ident) {
                let name = &met.sig.ident;
                let fn_spec = FnSpec::parse(&met.sig, &mut met.attrs, false)?;
                let method = pymethod::impl_proto_wrap(ty, &fn_spec);
                let coexist = if m.can_coexist {
                    // We need METH_COEXIST here to prevent __add__  from overriding __radd__
                    quote!(pyo3::ffi::METH_COEXIST)
                } else {
                    quote!(0)
                };
                // TODO(kngwyu): doc
                py_methods.push(quote! {
                    pyo3::class::PyMethodDefType::Method({
                        #method
                        pyo3::class::PyMethodDef {
                            ml_name: stringify!(#name),
                            ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                            ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS | #coexist,
                            ml_doc: ""
                        }
                    })
                });
            }
        }
    }

    if py_methods.is_empty() {
        return Ok(quote! { #trait_impls });
    }
    let inventory_submission = quote! {
        pyo3::inventory::submit! {
            #![crate = pyo3] {
                type ProtoInventory = <#ty as pyo3::class::methods::PyMethodsImpl>::Methods;
                <ProtoInventory as pyo3::class::methods::PyMethodsInventory>::new(&[#(#py_methods),*])
            }
        }
    };
    Ok(quote! {
        #trait_impls
        #inventory_submission
    })
}
