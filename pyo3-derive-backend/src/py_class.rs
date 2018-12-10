// Copyright (c) 2017-present PyO3 Project and Contributors

use method::{FnArg, FnSpec, FnType};
use proc_macro2::{Span, TokenStream};
use py_method::{impl_py_getter_def, impl_py_setter_def, impl_wrap_getter, impl_wrap_setter};
use syn;
use utils;

#[derive(Default, Debug)]
struct PyClassAttributes {
    flags: Vec<syn::Expr>,
    freelist: Option<syn::Expr>,
    name: Option<syn::Expr>,
    base: Option<syn::TypePath>,
    variants: Option<Vec<(String, syn::AngleBracketedGenericArguments)>>,
}

pub fn build_py_class(class: &mut syn::ItemStruct, attr: &Vec<syn::Expr>) -> TokenStream {
    let attrs = parse_attribute(attr);
    let doc = utils::get_doc(&class.attrs, true);
    let mut descriptors = Vec::new();

    if let syn::Fields::Named(ref mut fields) = class.fields {
        for field in fields.named.iter_mut() {
            let field_descs = parse_descriptors(field);
            if !field_descs.is_empty() {
                descriptors.push((field.clone(), field_descs));
            }
        }
    } else {
        panic!("#[pyclass] can only be used with C-style structs")
    }

    impl_class(
        &class.ident,
        &attrs,
        doc,
        descriptors,
        class.generics.clone(),
    )
}

fn parse_descriptors(item: &mut syn::Field) -> Vec<FnType> {
    let mut descs = Vec::new();
    let mut new_attrs = Vec::new();
    for attr in item.attrs.iter() {
        if let Some(syn::Meta::List(ref list)) = attr.interpret_meta() {
            match list.ident.to_string().as_str() {
                "prop" => {
                    for meta in list.nested.iter() {
                        if let &syn::NestedMeta::Meta(ref metaitem) = meta {
                            match metaitem.name().to_string().as_str() {
                                "get" => {
                                    descs.push(FnType::Getter(None));
                                }
                                "set" => {
                                    descs.push(FnType::Setter(None));
                                }
                                x => {
                                    panic!(r#"Only "get" and "set" supported are, not "{}""#, x);
                                }
                            }
                        }
                    }
                }
                _ => new_attrs.push(attr.clone()),
            }
        } else {
            new_attrs.push(attr.clone());
        }
    }
    item.attrs.clear();
    item.attrs.extend(new_attrs);
    descs
}

fn impl_class(
    cls: &syn::Ident,
    attrs: &PyClassAttributes,
    doc: syn::Lit,
    descriptors: Vec<(syn::Field, Vec<FnType>)>,
    mut generics: syn::Generics,
) -> TokenStream {
    let cls_name = match attrs.name {
        Some(ref name) => quote! { #name }.to_string(),
        None => quote! { #cls }.to_string(),
    };

    if attrs.variants.is_none() && generics.params.len() != 0 {
        panic!(
            "The `variants` parameter is required when using generic structs, \
             e.g. `#[pyclass(variants(\"{}U32<u32>\", \"{}F32<f32>\"))]`.",
            cls_name, cls_name,
        );
    }

    // Split generics into pieces for impls using them.
    generics.make_where_clause();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mut where_clause = where_clause.unwrap().clone();

    // Insert `MyStruct<T>: PyTypeInfo` bound.
    where_clause.predicates.push(parse_quote! {
        #cls #ty_generics: ::pyo3::typeob::PyTypeInfo
    });

    let extra = {
        if let Some(ref freelist) = attrs.freelist {
            quote! {
                impl #impl_generics ::pyo3::freelist::PyObjectWithFreeList
                    for #cls #ty_generics #where_clause
                {
                    #[inline]
                    fn get_free_list() -> &'static mut ::pyo3::freelist::FreeList<*mut ::pyo3::ffi::PyObject> {
                        static mut FREELIST: *mut ::pyo3::freelist::FreeList<*mut ::pyo3::ffi::PyObject> = 0 as *mut _;
                        unsafe {
                            if FREELIST.is_null() {
                                FREELIST = Box::into_raw(Box::new(
                                    ::pyo3::freelist::FreeList::with_capacity(#freelist)));

                                <#cls #ty_generics as ::pyo3::typeob::PyTypeCreate>::init_type();
                            }
                            &mut *FREELIST
                        }
                    }
                }
            }
        } else {
            quote! {
                impl #impl_generics ::pyo3::typeob::PyObjectAlloc for #cls #ty_generics #where_clause {}
            }
        }
    };

    let extra = if !descriptors.is_empty() {
        let ty = syn::parse_str(&cls.to_string()).expect("no name");
        let desc_impls = impl_descriptors(
            &ty,
            &impl_generics,
            &ty_generics,
            &where_clause,
            descriptors,
        );
        quote! {
            #desc_impls
            #extra
        }
    } else {
        extra
    };

    // insert space for weak ref
    let mut has_weakref = false;
    let mut has_dict = false;
    for f in attrs.flags.iter() {
        if let syn::Expr::Path(ref epath) = f {
            if epath.path == parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_WEAKREF} {
                has_weakref = true;
            } else if epath.path == parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_DICT} {
                has_dict = true;
            }
        }
    }
    let weakref = if has_weakref {
        quote! {std::mem::size_of::<*const ::pyo3::ffi::PyObject>()}
    } else {
        quote! {0}
    };
    let dict = if has_dict {
        quote! {std::mem::size_of::<*const ::pyo3::ffi::PyObject>()}
    } else {
        quote! {0}
    };

    // Create a variant of our generics with lifetime 'a prepended.
    let mut gen_with_a = generics.clone();
    gen_with_a.params.insert(0, parse_quote! { 'a });
    let impl_with_a = gen_with_a.split_for_impl().0;

    // Generate one PyTypeInfo per generic variant.
    use quote::ToTokens;
    let variant_iter: Box<dyn Iterator<Item = (String, TokenStream)>> = match attrs.variants {
        Some(ref x) => Box::new(
            x.clone()
                .into_iter()
                .map(|(a, b)| (a, b.into_token_stream())),
        ),
        None => Box::new(std::iter::once((cls_name, TokenStream::new()))),
    };

    let base = &attrs.base;
    let flags = &attrs.flags;
    let type_info_impls: Vec<_> = variant_iter.map(|(name, for_ty)| quote! {
        impl ::pyo3::typeob::PyTypeInfo for #cls #for_ty {
            type Type = #cls #for_ty;
            type BaseType = #base;

            const NAME: &'static str = #name;
            const DESCRIPTION: &'static str = #doc;
            const FLAGS: usize = #(#flags)|*;

            const SIZE: usize = {
                Self::OFFSET as usize +
                ::std::mem::size_of::<Self>() + #weakref + #dict
            };
            const OFFSET: isize = {
                // round base_size up to next multiple of align
                (
                    (<#base as ::pyo3::typeob::PyTypeInfo>::SIZE +
                     ::std::mem::align_of::<Self>() - 1) /
                        ::std::mem::align_of::<Self>() *
                        ::std::mem::align_of::<Self>()
                ) as isize
            };

            #[inline]
            unsafe fn type_object() -> &'static mut ::pyo3::ffi::PyTypeObject {
                static mut TYPE_OBJECT: ::pyo3::ffi::PyTypeObject = ::pyo3::ffi::PyTypeObject_INIT;
                &mut TYPE_OBJECT
            }
        }
    }).collect();

    quote! {
        #(#type_info_impls)*

        // TBH I'm not sure what exactely this does and I'm sure there's a better way,
        // but for now it works and it only safe code and it is required to return custom
        // objects, so for now I'm keeping it
        impl #impl_generics ::pyo3::IntoPyObject for #cls #ty_generics #where_clause {
            fn into_object(self, py: ::pyo3::Python) -> ::pyo3::PyObject {
                ::pyo3::Py::new(py, || self).unwrap().into_object(py)
            }
        }

        impl #impl_generics ::pyo3::ToPyObject for #cls #ty_generics #where_clause {
            fn to_object(&self, py: ::pyo3::Python) -> ::pyo3::PyObject {
                use ::pyo3::python::ToPyPointer;
                unsafe { ::pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        impl #impl_generics ::pyo3::ToPyPointer for #cls #ty_generics #where_clause {
            fn as_ptr(&self) -> *mut ::pyo3::ffi::PyObject {
                unsafe {
                    (self as *const _ as *mut u8).offset(
                        -<Self as ::pyo3::typeob::PyTypeInfo>::OFFSET
                    ) as *mut ::pyo3::ffi::PyObject
                }
            }
        }

        impl #impl_with_a ::pyo3::ToPyObject for &'a mut #cls #ty_generics #where_clause {
            fn to_object(&self, py: ::pyo3::Python) -> ::pyo3::PyObject {
                use ::pyo3::python::ToPyPointer;
                unsafe { ::pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        #extra
    }
}

fn impl_descriptors(
    cls: &syn::Type,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: &syn::WhereClause,
    descriptors: Vec<(syn::Field, Vec<FnType>)>,
) -> TokenStream {
    let methods: Vec<TokenStream> = descriptors
        .iter()
        .flat_map(|&(ref field, ref fns)| {
            fns.iter()
                .map(|desc| {
                    let name = field.ident.clone().unwrap();
                    let field_ty = &field.ty;
                    match *desc {
                        FnType::Getter(_) => {
                            quote! {
                                impl #impl_generics #cls #ty_generics #where_clause {
                                    fn #name(&self) -> ::pyo3::PyResult<#field_ty> {
                                        Ok(self.#name.clone())
                                    }
                                }
                            }
                        }
                        FnType::Setter(_) => {
                            let setter_name =
                                syn::Ident::new(&format!("set_{}", name), Span::call_site());
                            quote! {
                                impl #impl_generics #cls #ty_generics #where_clause {
                                    fn #setter_name(&mut self, value: #field_ty) -> ::pyo3::PyResult<()> {
                                        self.#name = value;
                                        Ok(())
                                    }
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<TokenStream>>()
        })
        .collect();

    let py_methods: Vec<TokenStream> = descriptors
        .iter()
        .flat_map(|&(ref field, ref fns)| {
            fns.iter()
                .map(|desc| {
                    let name = field.ident.clone().unwrap();

                    // FIXME better doc?
                    let doc: syn::Lit = syn::parse_str(&format!("\"{}\"", name)).unwrap();

                    let field_ty = &field.ty;
                    match *desc {
                        FnType::Getter(ref getter) => {
                            impl_py_getter_def(&name, doc, getter, &impl_wrap_getter(&cls, &name))
                        }
                        FnType::Setter(ref setter) => {
                            let setter_name =
                                syn::Ident::new(&format!("set_{}", name), Span::call_site());
                            let spec = FnSpec {
                                tp: FnType::Setter(None),
                                attrs: Vec::new(),
                                args: vec![FnArg {
                                    name: &name,
                                    mutability: &None,
                                    by_ref: &None,
                                    ty: field_ty,
                                    optional: None,
                                    py: true,
                                    reference: false,
                                }],
                                output: parse_quote!(PyResult<()>),
                            };
                            impl_py_setter_def(
                                &name,
                                doc,
                                setter,
                                &impl_wrap_setter(&cls, &setter_name, &spec),
                            )
                        }
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<TokenStream>>()
        })
        .collect();

    quote! {
        #(#methods)*

        impl #impl_generics ::pyo3::class::methods::PyPropMethodsProtocolImpl
            for #cls #ty_generics #where_clause
        {
            fn py_methods() -> &'static [::pyo3::class::PyMethodDefType] {
                static METHODS: &'static [::pyo3::class::PyMethodDefType] = &[
                    #(#py_methods),*
                ];
                METHODS
            }
        }
    }
}

fn parse_attribute(args: &Vec<syn::Expr>) -> PyClassAttributes {
    use syn::Expr::*;

    let mut attrs = PyClassAttributes {
        // We need the 0 as value for the constant we're later building using
        // quote for when there are no other flags
        flags: vec![parse_quote! {0}],
        base: Some(parse_quote! {::pyo3::types::PyObjectRef}),
        ..Default::default()
    };

    for expr in args.iter() {
        match expr {
            // Match a single flag
            Path(ref exp) if exp.path.segments.len() == 1 => {
                let flag = exp.path.segments.first().unwrap().value().ident.to_string();
                let path = match flag.as_str() {
                    "gc" => {
                        parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_GC}
                    }
                    "weakref" => {
                        parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_WEAKREF}
                    }
                    "subclass" => {
                        parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_BASETYPE}
                    }
                    "dict" => {
                        parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_DICT}
                    }
                    param => panic!("Unsupported parameter: {}", param),
                };

                attrs.flags.push(Path(path));
            }

            // Match a key/value flag
            Assign(ref ass) => {
                let key = match *ass.left {
                    Path(ref exp) if exp.path.segments.len() == 1 => {
                        exp.path.segments.first().unwrap().value().ident.to_string()
                    }
                    _ => panic!("could not parse argument: {:?}", ass),
                };

                match key.as_str() {
                    "freelist" => {
                        // TODO: check if int literal
                        attrs.freelist = Some(*ass.right.clone());
                    }
                    "name" => match *ass.right {
                        Path(ref exp) if exp.path.segments.len() == 1 => {
                            attrs.name = Some(exp.clone().into());
                        }
                        _ => panic!("Wrong 'name' format: {:?}", *ass.right),
                    },
                    "extends" => match *ass.right {
                        Path(ref exp) => {
                            attrs.base = Some(syn::TypePath {
                                path: exp.path.clone(),
                                qself: None,
                            });
                        }
                        _ => panic!("Wrong 'base' format: {:?}", *ass.right),
                    },
                    _ => {
                        panic!("Unsupported parameter: {:?}", key);
                    }
                }
            }

            // Match variants (e.g. `variants("MyTypeU32<u32>", "MyTypeF32<f32>")`)
            Call(ref call) => {
                let path = match *call.func {
                    Path(ref expr_path) => expr_path,
                    _ => panic!("Unsupported argument syntax"),
                };
                let path_segments = &path.path.segments;

                if path_segments.len() != 1
                    || path_segments.first().unwrap().value().ident.to_string() != "variants"
                {
                    panic!("Unsupported argument syntax");
                }

                attrs.variants = Some(
                    call.args
                        .iter()
                        .map(|x| {
                            // Extract string argument.
                            let lit = match x {
                                Lit(syn::ExprLit {
                                    lit: syn::Lit::Str(ref lit),
                                    ..
                                }) => lit.value(),
                                _ => panic!("Unsupported argument syntax"),
                            };

                            // Parse string as type.
                            let ty: syn::Type =
                                syn::parse_str(&lit).expect("Invalid type definition");

                            let path_segs = match ty {
                                syn::Type::Path(syn::TypePath { ref path, .. }) => {
                                    path.segments.clone()
                                }
                                _ => panic!("Unsupported type syntax"),
                            };

                            if path_segs.len() != 1 {
                                panic!("Type path is expected to have exactly one segment.");
                            }

                            let seg = path_segs.iter().nth(0).unwrap();
                            let args = match seg.arguments {
                                syn::PathArguments::AngleBracketed(ref args) => args.clone(),
                                _ => panic!("Expected angle bracketed type arguments"),
                            };

                            (seg.ident.to_string(), args)
                        })
                        .collect(),
                );
            }

            _ => panic!("Could not parse arguments"),
        }
    }

    attrs
}
