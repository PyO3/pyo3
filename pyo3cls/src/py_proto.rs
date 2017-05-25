// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};

use defs;
use py_method;
use method::FnSpec;
use func::impl_method_proto;


struct Methods {
    methods: &'static [&'static str],
}

static DEFAULT_METHODS: Methods = Methods {
    methods: &[],
};


pub fn build_py_proto(ast: &mut syn::Item) -> Tokens {
    match ast.node {
        syn::ItemKind::Impl(_, _, ref mut gen, ref mut path, ref ty, ref mut impl_items) => {
            if let &mut Some(ref mut path) = path {
                let tokens = if let Some(ref mut segment) = path.segments.last() {
                    match segment.ident.as_ref() {
                        "PyObjectProtocol" =>
                            impl_proto_impl(ty, impl_items, &defs::OBJECT),
                        "PyAsyncProtocol" =>
                            impl_proto_impl(ty, impl_items, &defs::ASYNC),
                        "PyMappingProtocol" =>
                            impl_proto_impl(ty, impl_items, &defs::MAPPING),
                        "PyIterProtocol" =>
                            impl_proto_impl(ty, impl_items, &defs::ITER),
                        "PyContextProtocol" =>
                            impl_proto_impl(ty, impl_items, &defs::CONTEXT),
                        "PySequenceProtocol" =>
                            impl_proto_impl(ty, impl_items, &defs::SEQ),
                        "PyNumberProtocol" =>
                            impl_proto_impl(ty, impl_items, &defs::NUM),
                        "PyDescrProtocol" =>
                            impl_proto_impl(ty, impl_items, &defs::DESCR),
                        "PyBufferProtocol" =>
                            impl_protocol("_pyo3::class::buffer::PyBufferProtocolImpl",
                                          path.clone(), ty, impl_items, &DEFAULT_METHODS),
                        "PyGCProtocol" =>
                            impl_protocol("_pyo3::class::gc::PyGCProtocolImpl",
                                          path.clone(), ty, impl_items, &DEFAULT_METHODS),
                        _ => {
                            warn!("#[proto] can not be used with this block");
                            return Tokens::new()
                        }
                    }
                } else {
                    panic!("#[proto] can only be used with protocol trait implementations")
                };

                // attach lifetime
                gen.lifetimes = vec![syn::LifetimeDef {
                    attrs: vec![], bounds: vec![],
                    lifetime: syn::Lifetime { ident: syn::Ident::from("\'p") },
                }];

                let seg = path.segments.pop().unwrap();
                path.segments.push(syn::PathSegment{
                    ident: seg.ident.clone(),
                    parameters: syn::PathParameters::AngleBracketed(
                        syn::AngleBracketedParameterData {
                            lifetimes: vec![syn::Lifetime { ident: syn::Ident::from("\'p") }],
                            types: vec![], bindings: vec![] })});

                tokens
            } else {
                panic!("#[proto] can only be used with protocol trait implementations")
            }
        },
        _ => panic!("#[proto] can only be used with Impl blocks"),
    }
}

fn impl_proto_impl(ty: &Box<syn::Ty>, impls: &mut Vec<syn::ImplItem>, proto: &defs::Proto) -> Tokens {
    let mut tokens = Tokens::new();
    let mut py_methods = Vec::new();

    for iimpl in impls.iter_mut() {
        match iimpl.node {
            syn::ImplItemKind::Method(ref mut sig, _) => {
                for m in proto.methods {
                    if m.eq(iimpl.ident.as_ref()) {
                        impl_method_proto(ty, sig, m).to_tokens(&mut tokens);
                    }
                }
                for m in proto.py_methods {
                    if m.name == iimpl.ident.as_ref() {
                        let name = syn::Ident::from(m.name);
                        let proto = syn::Ident::from(m.proto);

                        let fn_spec = FnSpec::parse(
                            &iimpl.ident, sig, &mut iimpl.attrs);
                        let meth = py_method::impl_proto_wrap(ty, &iimpl.ident, &fn_spec);

                        py_methods.push(
                            quote! {
                                impl #proto for #ty
                                {
                                    #[inline]
                                    fn #name() -> Option<_pyo3::class::methods::PyMethodDef> {
                                        #meth

                                        Some(_pyo3::class::PyMethodDef {
                                            ml_name: stringify!(#name),
                                            ml_meth: _pyo3::class::PyMethodType::PyCFunctionWithKeywords(wrap),
                                            ml_flags: _pyo3::ffi::METH_VARARGS | _pyo3::ffi::METH_KEYWORDS,
                                            ml_doc: ""})
                                    }
                                }
                            }
                        );
                    }
                }
            },
            _ => (),
        }
    }

    // unique mod name
    let p = proto.name;
    let n = match ty.as_ref() {
        &syn::Ty::Path(_, ref p) => {
        p.segments.last().as_ref().unwrap().ident.as_ref()
    }
    _ => "PROTO_METHODS"
    };

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_{}_{}", n, p));
    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables)]
        const #dummy_const: () = {
            extern crate pyo3 as _pyo3;

            #tokens

            #(#py_methods)*
        };
    }
}

fn impl_protocol(name: &'static str,
                 path: syn::Path, ty: &Box<syn::Ty>,
                 impls: &mut Vec<syn::ImplItem>, methods: &Methods) -> Tokens {
    let mut py_methods = Vec::new();

    // get method names in impl block
    let mut meth = Vec::new();
    for iimpl in impls.iter_mut() {
        match iimpl.node {
            syn::ImplItemKind::Method(ref mut sig, _) => {
                if methods.methods.contains(&iimpl.ident.as_ref()) {
                    py_methods.push(py_method::gen_py_method(
                        ty, &iimpl.ident, sig, &mut iimpl.attrs));
                } else {
                    meth.push(String::from(iimpl.ident.as_ref()));
                }
            },
            _ => (),
        }
    }

    // set trait name
    let mut path = path;
    {
        let mut last = path.segments.last_mut().unwrap();
        last.ident = syn::Ident::from(name);
    }

    let i = syn::Ident::from(name);
    let tokens = if py_methods.is_empty() {
        quote! {
            impl #i for #ty {
                fn methods() -> &'static [&'static str] {
                    static METHODS: &'static [&'static str] = &[#(#meth),*];
                    METHODS
                }
            }
        }
    } else {
        quote! {
            impl #i for #ty {
                fn methods() -> &'static [&'static str] {
                    static METHODS: &'static [&'static str] = &[#(#meth,),*];
                    METHODS
                }

                fn py_methods() -> &'static [pyo3::class::PyMethodDefType] {
                    static METHODS: &'static [pyo3::class::PyMethodDefType] = &[
                        #(#py_methods),*
                    ];
                    METHODS
                }
            }
        }
    };
    let name = name.split("::").last().unwrap();
    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_{}", name));
    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables)]
        const #dummy_const: () = {
            extern crate pyo3 as _pyo3;

            #tokens
        };
    }
}
