// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::method::{FnArg, FnSpec, FnType};
use crate::pymethod::{impl_py_getter_def, impl_py_setter_def, impl_wrap_getter, impl_wrap_setter};
use crate::utils;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_quote, Expr, Token};

/// The parsed arguments of the pyclass macro
pub struct PyClassArgs {
    pub freelist: Option<syn::Expr>,
    pub name: Option<syn::Expr>,
    pub flags: Vec<syn::Expr>,
    pub base: syn::TypePath,
    pub module: Option<syn::LitStr>,
}

impl Parse for PyClassArgs {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let mut slf = PyClassArgs::default();

        let vars = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        for expr in vars {
            slf.add_expr(&expr)?;
        }
        Ok(slf)
    }
}

impl Default for PyClassArgs {
    fn default() -> Self {
        PyClassArgs {
            freelist: None,
            name: None,
            module: None,
            // We need the 0 as value for the constant we're later building using quote for when there
            // are no other flags
            flags: vec![parse_quote! {0}],
            base: parse_quote! {pyo3::types::PyAny},
        }
    }
}

impl PyClassArgs {
    /// Adda single expression from the comma separated list in the attribute, which is
    /// either a single word or an assignment expression
    fn add_expr(&mut self, expr: &Expr) -> syn::parse::Result<()> {
        match expr {
            syn::Expr::Path(ref exp) if exp.path.segments.len() == 1 => self.add_path(exp),
            syn::Expr::Assign(ref assign) => self.add_assign(assign),
            _ => Err(syn::Error::new_spanned(expr, "Could not parse arguments")),
        }
    }

    /// Match a single flag
    fn add_assign(&mut self, assign: &syn::ExprAssign) -> syn::Result<()> {
        let key = match *assign.left {
            syn::Expr::Path(ref exp) if exp.path.segments.len() == 1 => {
                exp.path.segments.first().unwrap().value().ident.to_string()
            }
            _ => {
                return Err(syn::Error::new_spanned(assign, "could not parse argument"));
            }
        };

        match key.as_str() {
            "freelist" => {
                // We allow arbitrary expressions here so you can e.g. use `8*64`
                self.freelist = Some(*assign.right.clone());
            }
            "name" => match *assign.right {
                syn::Expr::Path(ref exp) if exp.path.segments.len() == 1 => {
                    self.name = Some(exp.clone().into());
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        *assign.right.clone(),
                        "Wrong 'name' format",
                    ));
                }
            },
            "extends" => match *assign.right {
                syn::Expr::Path(ref exp) => {
                    self.base = syn::TypePath {
                        path: exp.path.clone(),
                        qself: None,
                    };
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        *assign.right.clone(),
                        "Wrong format for extends",
                    ));
                }
            },
            "module" => match *assign.right {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(ref lit),
                    ..
                }) => {
                    self.module = Some(lit.clone());
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        *assign.right.clone(),
                        "Wrong format for module",
                    ));
                }
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    *assign.left.clone(),
                    "Unsupported parameter",
                ));
            }
        };

        Ok(())
    }

    /// Match a key/value flag
    fn add_path(&mut self, exp: &syn::ExprPath) -> syn::Result<()> {
        let flag = exp.path.segments.first().unwrap().value().ident.to_string();
        let path = match flag.as_str() {
            "gc" => {
                parse_quote! {pyo3::type_object::PY_TYPE_FLAG_GC}
            }
            "weakref" => {
                parse_quote! {pyo3::type_object::PY_TYPE_FLAG_WEAKREF}
            }
            "subclass" => {
                parse_quote! {pyo3::type_object::PY_TYPE_FLAG_BASETYPE}
            }
            "dict" => {
                parse_quote! {pyo3::type_object::PY_TYPE_FLAG_DICT}
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    exp.path.clone(),
                    "Unsupported parameter",
                ));
            }
        };

        self.flags.push(syn::Expr::Path(path));
        Ok(())
    }
}

pub fn build_py_class(class: &mut syn::ItemStruct, attr: &PyClassArgs) -> syn::Result<TokenStream> {
    let doc = utils::get_doc(&class.attrs, true);
    let mut descriptors = Vec::new();

    check_generics(class)?;
    if let syn::Fields::Named(ref mut fields) = class.fields {
        for field in fields.named.iter_mut() {
            let field_descs = parse_descriptors(field)?;
            if !field_descs.is_empty() {
                descriptors.push((field.clone(), field_descs));
            }
        }
    } else {
        return Err(syn::Error::new_spanned(
            &class.fields,
            "#[pyclass] can only be used with C-style structs",
        ));
    }

    Ok(impl_class(&class.ident, &attr, doc, descriptors))
}

/// Parses `#[pyo3(get, set)]`
fn parse_descriptors(item: &mut syn::Field) -> syn::Result<Vec<FnType>> {
    let mut descs = Vec::new();
    let mut new_attrs = Vec::new();
    for attr in item.attrs.iter() {
        if let Ok(syn::Meta::List(ref list)) = attr.parse_meta() {
            match list.ident.to_string().as_str() {
                "pyo3" => {
                    for meta in list.nested.iter() {
                        if let syn::NestedMeta::Meta(ref metaitem) = meta {
                            match metaitem.name().to_string().as_str() {
                                "get" => {
                                    descs.push(FnType::Getter(None));
                                }
                                "set" => {
                                    descs.push(FnType::Setter(None));
                                }
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        metaitem,
                                        "Only get and set are supported",
                                    ));
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
    Ok(descs)
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
            methods: &'static [pyo3::class::PyMethodDefType],
        }

        impl pyo3::class::methods::PyMethodsInventory for #inventory_cls {
            fn new(methods: &'static [pyo3::class::PyMethodDefType]) -> Self {
                Self {
                    methods
                }
            }

            fn get_methods(&self) -> &'static [pyo3::class::PyMethodDefType] {
                self.methods
            }
        }

        impl pyo3::class::methods::PyMethodsInventoryDispatch for #cls {
            type InventoryType = #inventory_cls;
        }

        pyo3::inventory::collect!(#inventory_cls);
    }
}

fn impl_class(
    cls: &syn::Ident,
    attr: &PyClassArgs,
    doc: syn::Lit,
    descriptors: Vec<(syn::Field, Vec<FnType>)>,
) -> TokenStream {
    let cls_name = match &attr.name {
        Some(name) => quote! { #name }.to_string(),
        None => cls.to_string(),
    };

    let extra = {
        if let Some(freelist) = &attr.freelist {
            quote! {
                impl pyo3::freelist::PyObjectWithFreeList for #cls {
                    #[inline]
                    fn get_free_list() -> &'static mut pyo3::freelist::FreeList<*mut pyo3::ffi::PyObject> {
                        static mut FREELIST: *mut pyo3::freelist::FreeList<*mut pyo3::ffi::PyObject> = 0 as *mut _;
                        unsafe {
                            if FREELIST.is_null() {
                                FREELIST = Box::into_raw(Box::new(
                                    pyo3::freelist::FreeList::with_capacity(#freelist)));

                                <#cls as pyo3::type_object::PyTypeObject>::init_type();
                            }
                            &mut *FREELIST
                        }
                    }
                }
            }
        } else {
            quote! {
                impl pyo3::type_object::PyObjectAlloc for #cls {}
            }
        }
    };

    let extra = if !descriptors.is_empty() {
        let path = syn::Path::from(syn::PathSegment::from(cls.clone()));
        let ty = syn::Type::from(syn::TypePath { path, qself: None });
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
    for f in attr.flags.iter() {
        if let syn::Expr::Path(ref epath) = f {
            if epath.path == parse_quote! {pyo3::type_object::PY_TYPE_FLAG_WEAKREF} {
                has_weakref = true;
            } else if epath.path == parse_quote! {pyo3::type_object::PY_TYPE_FLAG_DICT} {
                has_dict = true;
            }
        }
    }
    let weakref = if has_weakref {
        quote! {std::mem::size_of::<*const pyo3::ffi::PyObject>()}
    } else {
        quote! {0}
    };
    let dict = if has_dict {
        quote! {std::mem::size_of::<*const pyo3::ffi::PyObject>()}
    } else {
        quote! {0}
    };
    let module = if let Some(m) = &attr.module {
        quote! { Some(#m) }
    } else {
        quote! { None }
    };

    let inventory_impl = impl_inventory(&cls);

    let base = &attr.base;
    let flags = &attr.flags;

    quote! {
        impl pyo3::type_object::PyTypeInfo for #cls {
            type Type = #cls;
            type BaseType = #base;

            const NAME: &'static str = #cls_name;
            const MODULE: Option<&'static str> = #module;
            const DESCRIPTION: &'static str = #doc;
            const FLAGS: usize = #(#flags)|*;

            const SIZE: usize = {
                Self::OFFSET as usize +
                ::std::mem::size_of::<#cls>() + #weakref + #dict
            };
            const OFFSET: isize = {
                // round base_size up to next multiple of align
                (
                    (<#base as pyo3::type_object::PyTypeInfo>::SIZE +
                     ::std::mem::align_of::<#cls>() - 1)  /
                        ::std::mem::align_of::<#cls>() * ::std::mem::align_of::<#cls>()
                ) as isize
            };

            #[inline]
            unsafe fn type_object() -> &'static mut pyo3::ffi::PyTypeObject {
                static mut TYPE_OBJECT: pyo3::ffi::PyTypeObject = pyo3::ffi::PyTypeObject_INIT;
                &mut TYPE_OBJECT
            }
        }

        impl pyo3::IntoPyObject for #cls {
            fn into_object(self, py: pyo3::Python) -> pyo3::PyObject {
                pyo3::Py::new(py, self).unwrap().into_object(py)
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
                                    fn #name(&self) -> pyo3::PyResult<#field_ty> {
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
                                    fn #setter_name(&mut self, value: #field_ty) -> pyo3::PyResult<()> {
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
                    let doc = syn::Lit::from(syn::LitStr::new(&name.to_string(), name.span()));

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

        pyo3::inventory::submit! {
            #![crate = pyo3] {
                type ClsInventory = <#cls as pyo3::class::methods::PyMethodsInventoryDispatch>::InventoryType;
                <ClsInventory as pyo3::class::methods::PyMethodsInventory>::new(&[#(#py_methods),*])
            }
        }
    }
}

fn check_generics(class: &mut syn::ItemStruct) -> syn::Result<()> {
    if class.generics.params.is_empty() {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            &class.generics,
            "#[pyclass] cannot have generic parameters",
        ))
    }
}
