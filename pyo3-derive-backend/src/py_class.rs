// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::method::{FnArg, FnSpec, FnType};
use crate::py_method::{
    impl_py_getter_def, impl_py_setter_def, impl_wrap_getter, impl_wrap_setter,
};
use crate::utils;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn;

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

/// The orphan rule disallows using a generic inventory struct, so we create the whole boilerplate
/// once per class
fn impl_inventory(cls: &syn::Ident) -> TokenStream {
    // Try to build a unique type that gives a hint about it's function when
    // it comes up in error messages
    let name = cls.to_string() + "GeneratedPyo3Inventory";
    let inventory_cls = syn::Ident::new(&name, Span::call_site());

    quote! {
        #[doc(hidden)]
        pub struct #inventory_cls {
            methods: &'static [::pyo3::class::PyMethodDefType],
        }

        impl ::pyo3::class::methods::PyMethodsInventory for #inventory_cls {
            fn new(methods: &'static [::pyo3::class::PyMethodDefType]) -> Self {
                Self {
                    methods
                }
            }

            fn get_methods(&self) -> &'static [::pyo3::class::PyMethodDefType] {
                self.methods
            }
        }

        impl ::pyo3::class::methods::PyMethodsInventoryDispatch for #cls {
            type InventoryType = #inventory_cls;
        }

        ::pyo3::inventory::collect!(#inventory_cls);
    }
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
            quote! {
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
            }
        } else {
            quote! {
                impl ::pyo3::typeob::PyObjectAlloc for #cls {}
            }
        }
    };

    let extra = if !descriptors.is_empty() {
        let ty = syn::parse_str(&cls.to_string()).expect("no name");
        let desc_impls = impl_descriptors(&ty, descriptors);
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
    for f in flags.iter() {
        if let syn::Expr::Path(ref epath) = f {
            if epath.path == syn::parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_WEAKREF} {
                has_weakref = true;
            } else if epath.path == syn::parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_DICT} {
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

    let inventory_impl = impl_inventory(&cls);

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

        impl ::pyo3::IntoPyObject for #cls {
            fn into_object(self, py: ::pyo3::Python) -> ::pyo3::PyObject {
                ::pyo3::Py::new(py, self).unwrap().into_object(py)
            }
        }

        #inventory_impl

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
                                output: syn::parse_quote!(PyResult<()>),
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

        ::pyo3::inventory::submit! {
            #![crate = pyo3] {
                type ClsInventory = <#cls as ::pyo3::class::methods::PyMethodsInventoryDispatch>::InventoryType;
                <ClsInventory as ::pyo3::class::methods::PyMethodsInventory>::new(&[#(#py_methods),*])
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
    let mut flags = vec![syn::parse_quote! {0}];
    let mut base: syn::TypePath = syn::parse_quote! {::pyo3::types::PyObjectRef};

    for expr in args.iter() {
        match expr {
            // Match a single flag
            syn::Expr::Path(ref exp) if exp.path.segments.len() == 1 => {
                let flag = exp.path.segments.first().unwrap().value().ident.to_string();
                let path = match flag.as_str() {
                    "gc" => {
                        syn::parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_GC}
                    }
                    "weakref" => {
                        syn::parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_WEAKREF}
                    }
                    "subclass" => {
                        syn::parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_BASETYPE}
                    }
                    "dict" => {
                        syn::parse_quote! {::pyo3::typeob::PY_TYPE_FLAG_DICT}
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
