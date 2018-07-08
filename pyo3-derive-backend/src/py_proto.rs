// Copyright (c) 2017-present PyO3 Project and Contributors

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn;

use defs;
use func::impl_method_proto;
use method::FnSpec;
use py_method;

pub fn build_py_proto(ast: &mut syn::ItemImpl) -> TokenStream {
    if let Some((_, ref mut path, _)) = ast.trait_ {
        let tokens = if let Some(ref mut segment) = path.segments.last() {
            let ty = &ast.self_ty;
            let items = &mut ast.items;
            match segment.value().ident.to_string().as_str() {
                "PyObjectProtocol" => impl_proto_impl(ty, items, &defs::OBJECT),
                "PyAsyncProtocol" => impl_proto_impl(ty, items, &defs::ASYNC),
                "PyMappingProtocol" => impl_proto_impl(ty, items, &defs::MAPPING),
                "PyIterProtocol" => impl_proto_impl(ty, items, &defs::ITER),
                "PyContextProtocol" => impl_proto_impl(ty, items, &defs::CONTEXT),
                "PySequenceProtocol" => impl_proto_impl(ty, items, &defs::SEQ),
                "PyNumberProtocol" => impl_proto_impl(ty, items, &defs::NUM),
                "PyDescrProtocol" => impl_proto_impl(ty, items, &defs::DESCR),
                "PyBufferProtocol" => impl_proto_impl(ty, items, &defs::BUFFER),
                "PyGCProtocol" => impl_proto_impl(ty, items, &defs::GC),
                _ => {
                    warn!("#[pyproto] can not be used with this block");
                    return TokenStream::new();
                }
            }
        } else {
            panic!("#[pyproto] can only be used with protocol trait implementations")
        };

        // attach lifetime
        let mut seg = path.segments.pop().unwrap().into_value();
        seg.arguments = syn::PathArguments::AngleBracketed(parse_quote!{<'p>});
        path.segments.push(seg);
        ast.generics.params = parse_quote!{'p};

        tokens
    } else {
        panic!("#[pyproto] can only be used with protocol trait implementations")
    }
}

fn impl_proto_impl(
    ty: &syn::Type,
    impls: &mut Vec<syn::ImplItem>,
    proto: &defs::Proto,
) -> TokenStream {
    let mut tokens = TokenStream::new();
    let mut py_methods = Vec::new();

    for iimpl in impls.iter_mut() {
        match iimpl {
            syn::ImplItem::Method(ref mut met) => {
                for m in proto.methods {
                    if m.eq(met.sig.ident.to_string().as_str()) {
                        impl_method_proto(ty, &mut met.sig, m).to_tokens(&mut tokens);
                    }
                }
                for m in proto.py_methods {
                    let ident = met.sig.ident.clone();
                    if m.name == ident.to_string().as_str() {
                        let name: syn::Ident = syn::parse_str(m.name).unwrap();
                        let proto: syn::Path = syn::parse_str(m.proto).unwrap();

                        let fn_spec = FnSpec::parse(&ident, &mut met.sig, &mut met.attrs);
                        let meth = py_method::impl_proto_wrap(ty, &ident, &fn_spec);

                        py_methods.push(quote! {
                            impl #proto for #ty
                            {
                                #[inline]
                                fn #name() -> Option<::pyo3::class::methods::PyMethodDef> {
                                    #meth

                                    Some(::pyo3::class::PyMethodDef {
                                        ml_name: stringify!(#name),
                                        ml_meth: ::pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                                        ml_flags: ::pyo3::ffi::METH_VARARGS | ::pyo3::ffi::METH_KEYWORDS,
                                        ml_doc: ""})
                                }
                            }
                        });
                    }
                }
            }
            _ => (),
        }
    }

    quote! {
        #tokens

        #(#py_methods)*
    }
}
