// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};

use py_method;
use method::FnSpec;
use func::{MethodProto, impl_method_proto};


struct Methods {
    methods: &'static [&'static str],
}

struct PyMethod {
    name: &'static str,
    proto: &'static str,
}

struct Proto {
    name: &'static str,
    methods: &'static [MethodProto],
    py_methods: &'static [PyMethod],
}

static DEFAULT_METHODS: Methods = Methods {
    methods: &[],
};

static DESCR_METHODS: Methods = Methods {
    methods: &["__delete__", "__set_name__"],
};

static NUM_METHODS: Methods = Methods {
    methods: &[
        "__radd__", "__rsub__", "__rmul__", "__rmatmul__", "__rtruediv__",
        "__rfloordiv__", "__rmod__", "__rdivmod__", "__rpow__", "__rlshift__",
        "__rrshift__", "__rand__", "__rxor__", "__ror__", "__complex__",
        "__round__"
    ],
};

static ASYNC: Proto = Proto {
    name: "Async",
    methods: &[
        MethodProto::Unary {
            name: "__await__",
            pyres: true,
            proto: "_pyo3::class::async::PyAsyncAwaitProtocol"},
        MethodProto::Unary{
            name: "__aiter__",
            pyres: true,
            proto: "_pyo3::class::async::PyAsyncAiterProtocol"},
        MethodProto::Unary{
            name: "__anext__",
            pyres: true,
            proto: "_pyo3::class::async::PyAsyncAnextProtocol"},
        MethodProto::Unary{
            name: "__aenter__",
            pyres: true,
            proto: "_pyo3::class::async::PyAsyncAenterProtocol"},
        MethodProto::Quaternary {
            name: "__aexit__",
            arg1: "ExcType", arg2: "ExcValue", arg3: "Traceback",
            proto: "_pyo3::class::async::PyAsyncAexitProtocol"},
    ],
    py_methods: &[
        PyMethod {
            name: "__aenter__",
            proto: "_pyo3::class::async::PyAsyncAenterProtocolImpl",
        },
        PyMethod {
            name: "__aexit__",
            proto: "_pyo3::class::async::PyAsyncAexitProtocolImpl",
        },
    ],
};

static CONTEXT: Proto = Proto {
    name: "Context",
    methods: &[
        MethodProto::Unary{
            name: "__enter__",
            pyres: true,
            proto: "_pyo3::class::context::PyContextEnterProtocol"},
        MethodProto::Quaternary {
            name: "__exit__",
            arg1: "ExcType", arg2: "ExcValue", arg3: "Traceback",
            proto: "_pyo3::class::context::PyContextExitProtocol"},
    ],
    py_methods: &[
        PyMethod {
            name: "__enter__",
            proto: "_pyo3::class::context::PyContextEnterProtocolImpl",
        },
        PyMethod {
            name: "__exit__",
            proto: "_pyo3::class::context::PyContextExitProtocolImpl",
        },
    ],
};

static ITER: Proto = Proto {
    name: "Iter",
    py_methods: &[],
    methods: &[
        MethodProto::Unary{
            name: "__iter__",
            pyres: true,
            proto: "_pyo3::class::iter::PyIterIterProtocol"},
        MethodProto::Unary{
            name: "__next__",
            pyres: true,
            proto: "_pyo3::class::iter::PyIterNextProtocol"},
    ],
};


static MAPPING: Proto = Proto {
    name: "Mapping",
    methods: &[
        MethodProto::Unary{
            name: "__len__",
            pyres: false,
            proto: "_pyo3::class::mapping::PyMappingLenProtocol"},
        MethodProto::Binary{
            name: "__getitem__",
            arg: "Key",
            pyres: true,
            proto: "_pyo3::class::mapping::PyMappingGetItemProtocol"},
        MethodProto::Ternary{
            name: "__setitem__",
            arg1: "Key",
            arg2: "Value",
            pyres: false,
            proto: "_pyo3::class::mapping::PyMappingSetItemProtocol"},
        MethodProto::Binary{
            name: "__delitem__",
            arg: "Key",
            pyres: false,
            proto: "_pyo3::class::mapping::PyMappingDelItemProtocol"},
        MethodProto::Binary{
            name: "__contains__",
            arg: "Value",
            pyres: false,
            proto: "_pyo3::class::mapping::PyMappingContainsProtocol"},
        MethodProto::Unary{
            name: "__reversed__",
            pyres: true,
            proto: "_pyo3::class::mapping::PyMappingReversedProtocol"},
        MethodProto::Unary{
            name: "__iter__",
            pyres: true,
            proto: "_pyo3::class::mapping::PyMappingIterProtocol"},
    ],
    py_methods: &[
        PyMethod {
            name: "__iter__",
            proto: "_pyo3::class::mapping::PyMappingIterProtocolImpl",
        },
        PyMethod {
            name: "__contains__",
            proto: "_pyo3::class::mapping::PyMappingContainsProtocolImpl",
        },
        PyMethod {
            name: "__reversed__",
            proto: "_pyo3::class::mapping::PyMappingReversedProtocolImpl",
        },
    ],
};

static SEQ: Proto = Proto {
    name: "Sequence",
    methods: &[
        MethodProto::Unary{
            name: "__len__",
            pyres: false,
            proto: "_pyo3::class::sequence::PySequenceLenProtocol"},
        MethodProto::Unary{
            name: "__getitem__",
            pyres: true,
            proto: "_pyo3::class::sequence::PySequenceGetItemProtocol"},
        MethodProto::Binary{
            name: "__setitem__",
            arg: "Value",
            pyres: false,
            proto: "_pyo3::class::sequence::PyMappingSetItemProtocol"},
        MethodProto::Binary{
            name: "__delitem__",
            arg: "Key",
            pyres: false,
            proto: "_pyo3::class::mapping::PyMappingDelItemProtocol"},
        MethodProto::Binary{
            name: "__contains__",
            arg: "Item",
            pyres: false,
            proto: "_pyo3::class::sequence::PySequenceContainsProtocol"},
        MethodProto::Binary{
            name: "__concat__",
            arg: "Other",
            pyres: true,
            proto: "_pyo3::class::sequence::PySequenceConcatProtocol"},
        MethodProto::Unary{
            name: "__repeat__",
            pyres: true,
            proto: "_pyo3::class::sequence::PySequenceRepeatProtocol"},
        MethodProto::Binary{
            name: "__inplace_concat__",
            arg: "Other",
            pyres: true,
            proto: "_pyo3::class::sequence::PySequenceInplaceConcatProtocol"},
        MethodProto::Unary{
            name: "__inplace_repeat__",
            pyres: true,
            proto: "_pyo3::class::sequence::PySequenceInplaceRepeatProtocol"},
    ],
    py_methods: &[],
};


pub fn build_py_proto(ast: &mut syn::Item) -> Tokens {
    match ast.node {
        syn::ItemKind::Impl(_, _, _, ref path, ref ty, ref mut impl_items) => {
            if let &Some(ref path) = path {
                if let Some(segment) = path.segments.last() {
                    match segment.ident.as_ref() {
                        "PyObjectProtocol" =>
                            impl_protocol("_pyo3::class::basic::PyObjectProtocolImpl",
                                          path.clone(), ty, impl_items, &DEFAULT_METHODS),
                        "PyAsyncProtocol" =>
                            impl_proto_impl(ty, impl_items, &ASYNC),
                        "PyMappingProtocol" =>
                            impl_proto_impl(ty, impl_items, &MAPPING),
                        "PyIterProtocol" =>
                            impl_proto_impl(ty, impl_items, &ITER),
                        "PyContextProtocol" =>
                            impl_proto_impl(ty, impl_items, &CONTEXT),
                        "PySequenceProtocol" =>
                            impl_proto_impl(ty, impl_items, &SEQ),
                        "PyBufferProtocol" =>
                            impl_protocol("_pyo3::class::buffer::PyBufferProtocolImpl",
                                          path.clone(), ty, impl_items, &DEFAULT_METHODS),
                        "PyDescrProtocol" =>
                            impl_protocol("_pyo3::class::descr::PyDescrProtocolImpl",
                                          path.clone(), ty, impl_items, &DESCR_METHODS),
                        "PyGCProtocol" =>
                            impl_protocol("_pyo3::class::gc::PyGCProtocolImpl",
                                          path.clone(), ty, impl_items, &DEFAULT_METHODS),
                        "PyNumberProtocol" =>
                            impl_protocol("_pyo3::class::number::PyNumberProtocolImpl",
                                          path.clone(), ty, impl_items, &NUM_METHODS),
                        _ => {
                            warn!("#[proto] can not be used with this block");
                            Tokens::new()
                        }
                    }
                } else {
                    panic!("#[proto] can only be used with protocol trait implementations")
                }
            } else {
                panic!("#[proto] can only be used with protocol trait implementations")
            }
        },
        _ => panic!("#[proto] can only be used with Impl blocks"),
    }
}

fn impl_proto_impl(ty: &Box<syn::Ty>, impls: &mut Vec<syn::ImplItem>, proto: &Proto) -> Tokens {
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
                        let meth = py_method::impl_wrap(ty, &iimpl.ident, &fn_spec);

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
