// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::defs;
use crate::method::{FnSpec, FnType};
use crate::proto_method::impl_method_proto;
use crate::pyfunction::PyFunctionOptions;
use crate::pymethod;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use std::collections::HashSet;
use syn::spanned::Spanned;

pub fn build_py_proto(ast: &mut syn::ItemImpl) -> syn::Result<TokenStream> {
    let (path, proto) = if let Some((_, path, _)) = &mut ast.trait_ {
        let proto = match path.segments.last() {
            Some(segment) if segment.ident == "PyObjectProtocol" => &defs::OBJECT,
            Some(segment) if segment.ident == "PyAsyncProtocol" => &defs::ASYNC,
            Some(segment) if segment.ident == "PyMappingProtocol" => &defs::MAPPING,
            Some(segment) if segment.ident == "PyIterProtocol" => &defs::ITER,
            Some(segment) if segment.ident == "PySequenceProtocol" => &defs::SEQ,
            Some(segment) if segment.ident == "PyNumberProtocol" => &defs::NUM,
            Some(segment) if segment.ident == "PyDescrProtocol" => &defs::DESCR,
            Some(segment) if segment.ident == "PyBufferProtocol" => &defs::BUFFER,
            Some(segment) if segment.ident == "PyGCProtocol" => &defs::GC,
            _ => bail_spanned!(path.span() => "unrecognised trait for #[pyproto]"),
        };
        (path, proto)
    } else {
        bail_spanned!(
            ast.span() => "#[pyproto] can only be used with protocol trait implementations"
        );
    };

    let tokens = impl_proto_impl(&ast.self_ty, &mut ast.items, proto)?;

    // attach lifetime
    let mut seg = path.segments.pop().unwrap().into_value();
    seg.arguments = syn::PathArguments::AngleBracketed(syn::parse_quote! {<'p>});
    path.segments.push(seg);
    ast.generics.params = syn::parse_quote! {'p};
    Ok(tokens)
}

fn impl_proto_impl(
    ty: &syn::Type,
    impls: &mut [syn::ImplItem],
    proto: &defs::Proto,
) -> syn::Result<TokenStream> {
    let mut trait_impls = TokenStream::new();
    let mut py_methods = Vec::new();
    let mut method_names = HashSet::new();
    let module = proto.module();

    for iimpl in impls.iter_mut() {
        if let syn::ImplItem::Method(met) = iimpl {
            // impl Py~Protocol<'p> { type = ... }
            if let Some(m) = proto.get_proto(&met.sig.ident) {
                impl_method_proto(ty, &mut met.sig, &module, m)?.to_tokens(&mut trait_impls);
                // Insert the method to the HashSet
                method_names.insert(met.sig.ident.to_string());
            }
            // Add non-slot methods to inventory like `#[pymethods]`
            if let Some(m) = proto.get_method(&met.sig.ident) {
                let fn_spec =
                    FnSpec::parse(&mut met.sig, &mut met.attrs, PyFunctionOptions::default())?;

                let flags = if m.can_coexist {
                    // We need METH_COEXIST here to prevent __add__  from overriding __radd__
                    Some(quote!(_pyo3::ffi::METH_COEXIST))
                } else {
                    None
                };

                let method = if let FnType::Fn(_) = &fn_spec.tp {
                    pymethod::impl_py_method_def(ty, &fn_spec, flags)?
                } else {
                    bail_spanned!(
                        met.sig.span() => "expected method with receiver for #[pyproto] method"
                    );
                };

                py_methods.push(method);
            }
        }
    }
    let items = impl_proto_items(method_names, py_methods, ty, proto);

    Ok(quote! {
        const _: () = {
            use ::pyo3 as _pyo3; // pyproto doesn't support specifying #[pyo3(crate)]
            #trait_impls
            #items
        };
    })
}

fn impl_proto_items(
    method_names: HashSet<String>,
    py_methods: Vec<TokenStream>,
    ty: &syn::Type,
    proto: &defs::Proto,
) -> TokenStream {
    if method_names.is_empty() && py_methods.is_empty() {
        return TokenStream::default();
    }

    let module = proto.module();
    let items_trait = proto.items_trait();
    let items_trait_items = proto.items_trait_items();

    let mut tokens = proto
        .slot_defs(method_names)
        .map(|def| {
            let slot = syn::Ident::new(def.slot, Span::call_site());
            let slot_impl = syn::Ident::new(def.slot_impl, Span::call_site());
            quote! {{
                _pyo3::ffi::PyType_Slot {
                    slot: _pyo3::ffi::#slot,
                    pfunc: #module::#slot_impl::<#ty> as _
                }
            }}
        })
        .peekable();

    if tokens.peek().is_none() {
        return TokenStream::default();
    }

    quote! {
        impl _pyo3::impl_::pyclass::#items_trait<#ty>
            for _pyo3::impl_::pyclass::PyClassImplCollector<#ty>
        {
            fn #items_trait_items(self) -> &'static _pyo3::impl_::pyclass::PyClassItems {
                static ITEMS: _pyo3::impl_::pyclass::PyClassItems = _pyo3::impl_::pyclass::PyClassItems {
                    methods: &[#(#py_methods),*],
                    slots: &[#(#tokens),*]
                };
                &ITEMS
            }
        }
    }
}
