// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::defs;
use crate::func::impl_method_proto;
use crate::method::FnSpec;
use crate::pymethod;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

pub fn build_py_proto(ast: &mut syn::ItemImpl) -> syn::Result<TokenStream> {
    if let Some((_, ref mut path, _)) = ast.trait_ {
        let proto = if let Some(ref mut segment) = path.segments.last() {
            match segment.value().ident.to_string().as_str() {
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

        let tokens = impl_proto_impl(&ast.self_ty, &mut ast.items, proto);

        // attach lifetime
        let mut seg = path.segments.pop().unwrap().into_value();
        seg.arguments = syn::PathArguments::AngleBracketed(syn::parse_quote! {<'p>});
        path.segments.push(seg);
        ast.generics.params = syn::parse_quote! {'p};

        Ok(tokens)
    } else {
        return Err(syn::Error::new_spanned(
            ast,
            "#[pyproto] can only be used with protocol trait implementations",
        ));
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
        if let syn::ImplItem::Method(ref mut met) = iimpl {
            for m in proto.methods {
                if m == met.sig.ident.to_string().as_str() {
                    impl_method_proto(ty, &mut met.sig, m).to_tokens(&mut tokens);
                }
            }
            for m in proto.py_methods {
                let ident = met.sig.ident.clone();
                if m.name == ident.to_string().as_str() {
                    let name = syn::Ident::new(m.name, Span::call_site());
                    let proto: syn::Path = syn::parse_str(m.proto).unwrap();

                    let fn_spec = match FnSpec::parse(&ident, &met.sig, &mut met.attrs) {
                        Ok(fn_spec) => fn_spec,
                        Err(err) => return err.to_compile_error(),
                    };
                    let meth = pymethod::impl_proto_wrap(ty, &ident, &fn_spec);

                    py_methods.push(quote! {
                        impl #proto for #ty
                        {
                            #[inline]
                            fn #name() -> Option<pyo3::class::methods::PyMethodDef> {
                                #meth

                                Some(pyo3::class::PyMethodDef {
                                    ml_name: stringify!(#name),
                                    ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                                    ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
                                    ml_doc: ""
                                })
                            }
                        }
                    });
                }
            }
        }
    }

    quote! {
        #tokens

        #(#py_methods)*
    }
}
