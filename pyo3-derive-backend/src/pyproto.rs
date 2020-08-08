// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::defs;
use crate::method::{FnSpec, FnType};
use crate::proto_method::impl_method_proto;
use crate::pymethod;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use std::collections::HashSet;

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
    let mut method_names = HashSet::new();

    for iimpl in impls.iter_mut() {
        if let syn::ImplItem::Method(ref mut met) = iimpl {
            // impl Py~Protocol<'p> { type = ... }
            if let Some(m) = proto.get_proto(&met.sig.ident) {
                impl_method_proto(ty, &mut met.sig, m).to_tokens(&mut trait_impls);
                // Insert the method to the HashSet
                method_names.insert(met.sig.ident.to_string());
            }
            // Add non-slot methods to inventory like `#[pymethods]`
            if let Some(m) = proto.get_method(&met.sig.ident) {
                let name = &met.sig.ident;
                let fn_spec = FnSpec::parse(&met.sig, &mut met.attrs, false)?;

                let method = if let FnType::Fn(self_ty) = &fn_spec.tp {
                    pymethod::impl_proto_wrap(ty, &fn_spec, &self_ty)
                } else {
                    return Err(syn::Error::new_spanned(
                        &met.sig,
                        "Expected method with receiver for #[pyproto] method",
                    ));
                };

                let coexist = if m.can_coexist {
                    // We need METH_COEXIST here to prevent __add__  from overriding __radd__
                    quote!(pyo3::ffi::METH_COEXIST)
                } else {
                    quote!(0)
                };
                // TODO(kngwyu): Set ml_doc
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
    let inventory_submission = inventory_submission(py_methods, ty);
    let slot_initialization = slot_initialization(method_names, ty, proto)?;
    Ok(quote! {
        #trait_impls
        #inventory_submission
        #slot_initialization
    })
}

fn inventory_submission(py_methods: Vec<TokenStream>, ty: &syn::Type) -> TokenStream {
    if py_methods.is_empty() {
        return quote! {};
    }
    quote! {
        pyo3::inventory::submit! {
            #![crate = pyo3] {
                type Inventory = <#ty as pyo3::class::methods::HasMethodsInventory>::Methods;
                <Inventory as pyo3::class::methods::PyMethodsInventory>::new(&[#(#py_methods),*])
            }
        }
    }
}

fn slot_initialization(
    method_names: HashSet<String>,
    ty: &syn::Type,
    proto: &defs::Proto,
) -> syn::Result<TokenStream> {
    // Some setters cannot coexist.
    // E.g., if we have `__add__`, we need to skip `set_radd`.
    let mut skipped_setters = Vec::new();
    // Collect initializers
    let mut initializers: Vec<TokenStream> = vec![];
    'outer_loop: for m in proto.slot_setters {
        if skipped_setters.contains(&m.set_function) {
            continue;
        }
        for name in m.proto_names {
            // If this `#[pyproto]` block doesn't provide all required methods,
            // let's skip implementing this method.
            if !method_names.contains(*name) {
                continue 'outer_loop;
            }
        }
        skipped_setters.extend_from_slice(m.skipped_setters);
        // Add slot methods to PyProtoRegistry
        let set = syn::Ident::new(m.set_function, Span::call_site());
        initializers.push(quote! { table.#set::<#ty>(); });
    }
    if initializers.is_empty() {
        return Ok(quote! {});
    }
    let table: syn::Path = syn::parse_str(proto.slot_table)?;
    let set = syn::Ident::new(proto.set_slot_table, Span::call_site());
    let ty_hash = typename_hash(ty);
    let init = syn::Ident::new(
        &format!("__init_{}_{}", proto.name, ty_hash),
        Span::call_site(),
    );
    Ok(quote! {
        #[pyo3::ctor::ctor]
        fn #init() {
            let mut table = #table::default();
            #(#initializers)*
            <#ty as pyo3::class::proto_methods::HasProtoRegistry>::registry().#set(table);
        }
    })
}

fn typename_hash(ty: &syn::Type) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    ty.hash(&mut hasher);
    hasher.finish()
}
