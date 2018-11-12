// Copyright (c) 2017-present PyO3 Project and Contributors

use method::{FnArg, FnSpec, FnType};
use proc_macro2::{Span, TokenStream};
use py_method::{impl_py_getter_def, impl_py_setter_def, impl_wrap_getter, impl_wrap_setter};
use std::collections::HashMap;
use syn;
use utils;

pub fn build_py_class(class: &mut syn::ItemStruct, attr: &Vec<syn::Expr>) -> TokenStream {
    let (params, flags, base) = parse_attribute(attr);
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

    impl_class(&class.ident, &base, doc, params, flags, descriptors)
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
    base: &syn::TypePath,
    doc: syn::Lit,
    params: HashMap<&'static str, syn::Expr>,
    flags: Vec<syn::Expr>,
    descriptors: Vec<(syn::Field, Vec<FnType>)>,
) -> TokenStream {
    let cls_name = match params.get("name") {
        Some(name) => quote! { #name }.to_string(),
        None => quote! { #cls }.to_string(),
    };

    let extra = {
        if let Some(freelist) = params.get("freelist") {
            Some(quote! {
                impl ::pyo3::freelist::PyObjectWithFreeList for #cls {
                    #[inline]
                    fn get_free_list() -> &'static mut ::pyo3::freelist::FreeList<*mut ::pyo3::ffi::PyObject> {
                        static mut FREELIST: *mut ::pyo3::freelist::FreeList<*mut ::pyo3::ffi::PyObject> = 0 as *mut _;
                        unsafe {
                            if FREELIST.is_null() {
                                FREELIST = Box::into_raw(Box::new(
                                    ::pyo3::freelist::FreeList::with_capacity(#freelist)));

                                <#cls as ::pyo3::typeob::PyTypeCreate>::init_type();
                            }
                            &mut *FREELIST
                        }
                    }
                }
            })
        } else {
            None
        }
    };

    let extra = if !descriptors.is_empty() {
        let ty = syn::parse_str(&cls.to_string()).expect("no name");
        let desc_impls = impl_descriptors(&ty, descriptors);
        Some(quote! {
            #desc_impls
            #extra
        })
    } else {
        extra
    };

    // insert space for weak ref
    let mut has_weakref = false;
    let mut has_dict = false;
    for f in flags.iter() {
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

    quote! {
        impl ::pyo3::typeob::PyTypeInfo for #cls {
            type Type = #cls;
            type BaseType = #base;

            const NAME: &'static str = #cls_name;
            const DESCRIPTION: &'static str = #doc;
            const FLAGS: usize = #(#flags)|*;

            const SIZE: usize = {
                Self::OFFSET as usize +
                ::std::mem::size_of::<#cls>() + #weakref + #dict
            };
            const OFFSET: isize = {
                // round base_size up to next multiple of align
                (
                    (<#base as ::pyo3::typeob::PyTypeInfo>::SIZE +
                     ::std::mem::align_of::<#cls>() - 1)  /
                        ::std::mem::align_of::<#cls>() * ::std::mem::align_of::<#cls>()
                ) as isize
            };

            #[inline]
            unsafe fn type_object() -> &'static mut ::pyo3::ffi::PyTypeObject {
                static mut TYPE_OBJECT: ::pyo3::ffi::PyTypeObject = ::pyo3::ffi::PyTypeObject_INIT;
                &mut TYPE_OBJECT
            }
        }

        // TBH I'm not sure what exactely this does and I'm sure there's a better way,
        // but for now it works and it only safe code and it is required to return custom
        // objects, so for now I'm keeping it
        impl ::pyo3::IntoPyObject for #cls {
            fn into_object(self, py: ::pyo3::Python) -> ::pyo3::PyObject {
                ::pyo3::Py::new(py, || self).unwrap().into_object(py)
            }
        }

        impl ::pyo3::ToPyObject for #cls {
            fn to_object(&self, py: ::pyo3::Python) -> ::pyo3::PyObject {
                use ::pyo3::python::ToPyPointer;
                unsafe { ::pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        impl ::pyo3::ToPyPointer for #cls {
            fn as_ptr(&self) -> *mut ::pyo3::ffi::PyObject {
                unsafe {
                    {self as *const _ as *mut u8}
                    .offset(-<#cls as ::pyo3::typeob::PyTypeInfo>::OFFSET) as *mut ::pyo3::ffi::PyObject
                }
            }
        }

        impl<'a> ::pyo3::ToPyObject for &'a mut #cls {
            fn to_object(&self, py: ::pyo3::Python) -> ::pyo3::PyObject {
                use ::pyo3::python::ToPyPointer;
                unsafe { ::pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
            }
        }

        #extra
    }
}

fn impl_descriptors(cls: &syn::Type, descriptors: Vec<(syn::Field, Vec<FnType>)>) -> TokenStream {
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
                                impl #cls {
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
                                impl #cls {
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

        impl ::pyo3::class::methods::PyPropMethodsProtocolImpl for #cls {
            fn py_methods() -> &'static [::pyo3::class::PyMethodDefType] {
                static METHODS: &'static [::pyo3::class::PyMethodDefType] = &[
                    #(#py_methods),*
                ];
                METHODS
            }
        }
    }
}

fn parse_attribute(
    args: &Vec<syn::Expr>,
) -> (
    HashMap<&'static str, syn::Expr>,
    Vec<syn::Expr>,
    syn::TypePath,
) {
    let mut params = HashMap::new();
    // We need the 0 as value for the constant we're later building using quote for when there
    // are no other flags
    let mut flags = vec![parse_quote! {0}];
    let mut base: syn::TypePath = parse_quote! {::pyo3::types::PyObjectRef};

    for expr in args.iter() {
        match expr {
            // Match a single flag
            syn::Expr::Path(ref exp) if exp.path.segments.len() == 1 => {
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

                flags.push(syn::Expr::Path(path));
            }

            // Match a key/value flag
            syn::Expr::Assign(ref ass) => {
                let key = match *ass.left {
                    syn::Expr::Path(ref exp) if exp.path.segments.len() == 1 => {
                        exp.path.segments.first().unwrap().value().ident.to_string()
                    }
                    _ => panic!("could not parse argument: {:?}", ass),
                };

                match key.as_str() {
                    "freelist" => {
                        // TODO: check if int literal
                        params.insert("freelist", *ass.right.clone());
                    }
                    "name" => match *ass.right {
                        syn::Expr::Path(ref exp) if exp.path.segments.len() == 1 => {
                            params.insert("name", exp.clone().into());
                        }
                        _ => panic!("Wrong 'name' format: {:?}", *ass.right),
                    },
                    "extends" => match *ass.right {
                        syn::Expr::Path(ref exp) => {
                            base = syn::TypePath {
                                path: exp.path.clone(),
                                qself: None,
                            };
                        }
                        _ => panic!("Wrong 'base' format: {:?}", *ass.right),
                    },
                    _ => {
                        panic!("Unsupported parameter: {:?}", key);
                    }
                }
            }

            _ => panic!("could not parse arguments"),
        }
    }

    (params, flags, base)
}
