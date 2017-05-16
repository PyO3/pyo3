// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};

use py_method;


struct Methods {
    methods: &'static [&'static str],
    non_pyobj_result: &'static [&'static str],
    no_adjust: bool,
}

static DEFAULT_METHODS: Methods = Methods {
    methods: &[],
    non_pyobj_result: &[],
    no_adjust: true,
};

static BUFFER_METHODS: Methods = Methods {
    methods: &[],
    non_pyobj_result: &["bf_getbuffer", "bf_releasebuffer"],
    no_adjust: false,
};

static GC_METHODS: Methods = Methods {
    methods: &[],
    non_pyobj_result: &["__traverse__", "__clear__"],
    no_adjust: false,
};

static CONTEXT_METHODS: Methods = Methods {
    methods: &["__enter__", "__exit__"],
    non_pyobj_result: &[],
    no_adjust: false,
};

static MAPPING_METHODS: Methods = Methods {
    methods: &[],
    non_pyobj_result: &["__setitem__", "__len__"],
    no_adjust: false,
};

static NUM_METHODS: Methods = Methods {
    methods: &[
        "__radd__", "__rsub__", "__rmul__", "__rmatmul__", "__rtruediv__",
        "__rfloordiv__", "__rmod__", "__rdivmod__", "__rpow__", "__rlshift__",
        "__rrshift__", "__rand__", "__rxor__", "__ror__", "__complex__",
        "__round__"
    ],
    non_pyobj_result: &[],
    no_adjust: true,
};


enum ImplType {
    Async,
    Buffer,
    Context,
    GC,
    Mapping,
    Sequence,
    Number,
}

pub fn build_py_proto(ast: &mut syn::Item) -> Tokens {
    match ast.node {
        syn::ItemKind::Impl(_, _, _, ref path, ref ty, ref mut impl_items) => {
            if let &Some(ref path) = path {
                match process_path(path) {
                    ImplType::Async => {
                        impl_protocol("pyo3::class::async::PyAsyncProtocolImpl",
                                      path.clone(), ty, impl_items, &DEFAULT_METHODS)
                    }
                    ImplType::Buffer => {
                        impl_protocol("pyo3::class::buffer::PyBufferProtocolImpl",
                                      path.clone(), ty, impl_items, &BUFFER_METHODS)
                    }
                    ImplType::Context => {
                        impl_protocol("pyo3::class::context::PyContextProtocolImpl",
                                      path.clone(), ty, impl_items, &CONTEXT_METHODS)
                    }
                    ImplType::GC => {
                        impl_protocol("pyo3::class::gc::PyGCProtocolImpl",
                                      path.clone(), ty, impl_items, &GC_METHODS)
                    }
                    ImplType::Mapping => {
                        impl_protocol("pyo3::class::mapping::PyMappingProtocolImpl",
                                      path.clone(), ty, impl_items, &MAPPING_METHODS)
                    },
                    ImplType::Sequence => {
                        impl_protocol("pyo3::class::mapping::PySequenceProtocolImpl",
                                      path.clone(), ty, impl_items, &DEFAULT_METHODS)
                    },
                    ImplType::Number => {
                        impl_protocol("pyo3::class::number::PyNumberProtocolImpl",
                                      path.clone(), ty, impl_items, &NUM_METHODS)
                    }
                }
            } else {
                panic!("#[py_proto] can only be used with protocol trait implementations")
            }
        },
        _ => panic!("#[py_proto] can only be used with Impl blocks"),
    }
}

fn process_path(path: &syn::Path) -> ImplType {
    if let Some(segment) = path.segments.last() {
            match segment.ident.as_ref() {
                "PyAsyncProtocol" => ImplType::Async,
                "PyBufferProtocol" => ImplType::Buffer,
                "PyContextProtocol" => ImplType::Context,
                "PyGCProtocol" => ImplType::GC,
                "PyMappingProtocol" => ImplType::Mapping,
                "PySequenceProtocol" => ImplType::Sequence,
                "PyNumberProtocol" => ImplType::Number,
                _ => panic!("#[py_proto] can not be used with this block"),
            }
    } else {
        panic!("#[py_proto] can not be used with this block");
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
            syn::ImplItemKind::Method(ref mut sig, ref mut block) => {
                if methods.methods.contains(&iimpl.ident.as_ref()) {
                    py_methods.push(py_method::gen_py_method(
                        ty, &iimpl.ident, sig, block, &iimpl.attrs));
                } else {
                    meth.push(String::from(iimpl.ident.as_ref()));

                    // adjust return type
                    if !methods.non_pyobj_result.contains(&iimpl.ident.as_ref()) &&
                        !methods.no_adjust {
                        impl_adjust_result(sig, block);
                    }
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

                fn py_methods() -> &'static [pyo3::class::PyMethodDef] {
                    static METHODS: &'static [pyo3::class::PyMethodDef] = &[
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
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate pyo3;
            use pyo3::ffi;

            #tokens
        };
    }
}

fn impl_adjust_result(sig: &mut syn::MethodSig, block: &mut syn::Block) {
    match sig.decl.output {
        syn::FunctionRetTy::Ty(ref mut ty) => match *ty {
            syn::Ty::Path(_, ref mut path) => {
                // check if function returns PyResult
                if let Some(segment) = path.segments.last_mut() {
                    match segment.ident.as_ref() {
                        // check result type
                        "PyResult" => match segment.parameters {
                            syn::PathParameters::AngleBracketed(ref mut data) => {
                                if rewrite_pyobject(&mut data.types) {
                                    let expr = {
                                        let s = block as &ToTokens;
                                        quote! {
                                            match #s {
                                                Ok(res) => Ok(res.to_py_object(py)),
                                                Err(err) => Err(err)
                                            }
                                        }
                                    };
                                    let expr = syn::parse_expr(&expr.as_str()).unwrap();
                                    let expr = syn::Stmt::Expr(Box::new(expr));
                                    block.stmts = vec![expr];
                                }
                            },
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            _ => (),
        },
        syn::FunctionRetTy::Default => (),
    }
}

fn rewrite_pyobject(path: &mut Vec<syn::Ty>) -> bool {
    if path.len() != 1 {
        false
    } else {
        if let &mut syn::Ty::Path(_, ref mut path) = path.first_mut().unwrap() {
            if let Some(segment) = path.segments.last_mut() {
                if segment.ident.as_ref() == "PyObject" {
                    return false
                } else {
                    segment.ident = syn::Ident::from("PyObject");
                    return true
                }
            }
        }
        let ty = syn::Ty::Path(
            None, syn::Path{
                global: false,
                segments: vec![
                    syn::PathSegment {
                        ident: syn::Ident::from("PyObject"),
                        parameters: syn::PathParameters::AngleBracketed(
                            syn::AngleBracketedParameterData {
                                lifetimes: vec![], types: vec![], bindings: vec![] }) }]});
        let _ = path.pop();
        let _ = path.push(ty);
        true
    }
}
