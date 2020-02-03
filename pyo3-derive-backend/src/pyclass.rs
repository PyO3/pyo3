// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::method::{FnArg, FnSpec, FnType};
use crate::pymethod::{impl_py_getter_def, impl_py_setter_def, impl_wrap_getter, impl_wrap_setter};
use crate::utils;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_quote, Expr, Token};

/// The parsed arguments of the pyclass macro
pub struct PyClassArgs {
    pub freelist: Option<syn::Expr>,
    pub name: Option<syn::Expr>,
    pub flags: Vec<syn::Expr>,
    pub base: syn::TypePath,
    pub has_extends: bool,
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
            flags: vec![parse_quote! { 0 }],
            base: parse_quote! { pyo3::types::PyAny },
            has_extends: false,
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
                exp.path.segments.first().unwrap().ident.to_string()
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
                    self.has_extends = true;
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
        let flag = exp.path.segments.first().unwrap().ident.to_string();
        let path = match flag.as_str() {
            "gc" => {
                parse_quote! {pyo3::type_flags::GC}
            }
            "weakref" => {
                parse_quote! {pyo3::type_flags::WEAKREF}
            }
            "subclass" => {
                parse_quote! {pyo3::type_flags::BASETYPE}
            }
            "dict" => {
                parse_quote! {pyo3::type_flags::DICT}
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
    let text_signature = utils::parse_text_signature_attrs(
        &mut class.attrs,
        &get_class_python_name(&class.ident, attr),
    )?;
    let doc = utils::get_doc(&class.attrs, text_signature, true)?;
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

    impl_class(&class.ident, &attr, doc, descriptors)
}

/// Parses `#[pyo3(get, set)]`
fn parse_descriptors(item: &mut syn::Field) -> syn::Result<Vec<FnType>> {
    let mut descs = Vec::new();
    let mut new_attrs = Vec::new();
    for attr in item.attrs.iter() {
        if let Ok(syn::Meta::List(ref list)) = attr.parse_meta() {
            if list.path.is_ident("pyo3") {
                for meta in list.nested.iter() {
                    if let syn::NestedMeta::Meta(ref metaitem) = meta {
                        if metaitem.path().is_ident("get") {
                            descs.push(FnType::Getter);
                        } else if metaitem.path().is_ident("set") {
                            descs.push(FnType::Setter);
                        } else {
                            return Err(syn::Error::new_spanned(
                                metaitem,
                                "Only get and set are supported",
                            ));
                        }
                    }
                }
            } else {
                new_attrs.push(attr.clone())
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

fn get_class_python_name(cls: &syn::Ident, attr: &PyClassArgs) -> TokenStream {
    match &attr.name {
        Some(name) => quote! { #name },
        None => quote! { #cls },
    }
}

fn impl_class(
    cls: &syn::Ident,
    attr: &PyClassArgs,
    doc: syn::LitStr,
    descriptors: Vec<(syn::Field, Vec<FnType>)>,
) -> syn::Result<TokenStream> {
    let cls_name = get_class_python_name(cls, attr).to_string();

    let extra = {
        if let Some(freelist) = &attr.freelist {
            quote! {
                impl pyo3::freelist::PyClassWithFreeList for #cls {
                    #[inline]
                    fn get_free_list() -> &'static mut pyo3::freelist::FreeList<*mut pyo3::ffi::PyObject> {
                        static mut FREELIST: *mut pyo3::freelist::FreeList<*mut pyo3::ffi::PyObject> = 0 as *mut _;
                        unsafe {
                            if FREELIST.is_null() {
                                FREELIST = Box::into_raw(Box::new(
                                    pyo3::freelist::FreeList::with_capacity(#freelist)));
                            }
                            &mut *FREELIST
                        }
                    }
                }
            }
        } else {
            quote! {
                impl pyo3::pyclass::PyClassAlloc for #cls {}
            }
        }
    };

    let extra = if !descriptors.is_empty() {
        let path = syn::Path::from(syn::PathSegment::from(cls.clone()));
        let ty = syn::Type::from(syn::TypePath { path, qself: None });
        let desc_impls = impl_descriptors(&ty, descriptors)?;
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
    let mut has_gc = false;
    for f in attr.flags.iter() {
        if let syn::Expr::Path(ref epath) = f {
            if epath.path == parse_quote! { pyo3::type_flags::WEAKREF } {
                has_weakref = true;
            } else if epath.path == parse_quote! { pyo3::type_flags::DICT } {
                has_dict = true;
            } else if epath.path == parse_quote! { pyo3::type_flags::GC } {
                has_gc = true;
            }
        }
    }

    let weakref = if has_weakref {
        quote! { type WeakRef = pyo3::pyclass_slots::PyClassWeakRefSlot; }
    } else {
        quote! { type WeakRef = pyo3::pyclass_slots::PyClassDummySlot; }
    };
    let dict = if has_dict {
        quote! { type Dict = pyo3::pyclass_slots::PyClassDictSlot; }
    } else {
        quote! { type Dict = pyo3::pyclass_slots::PyClassDummySlot; }
    };
    let module = if let Some(m) = &attr.module {
        quote! { Some(#m) }
    } else {
        quote! { None }
    };

    // Enforce at compile time that PyGCProtocol is implemented
    let gc_impl = if has_gc {
        let closure_name = format!("__assertion_closure_{}", cls.to_string());
        let closure_token = syn::Ident::new(&closure_name, Span::call_site());
        quote! {
            fn #closure_token() {
                use pyo3::class;

                fn _assert_implements_protocol<'p, T: pyo3::class::PyGCProtocol<'p>>() {}
                _assert_implements_protocol::<#cls>();
            }
        }
    } else {
        quote! {}
    };

    let inventory_impl = impl_inventory(&cls);

    let base = &attr.base;
    let flags = &attr.flags;
    let extended = if attr.has_extends {
        quote! { pyo3::type_flags::EXTENDED }
    } else {
        quote! { 0 }
    };

    // If #cls is not extended type, we allow Self->PyObject conversion
    let into_pyobject = if !attr.has_extends {
        quote! {
            impl pyo3::IntoPy<PyObject> for #cls {
                fn into_py(self, py: pyo3::Python) -> pyo3::PyObject {
                    pyo3::IntoPy::into_py(pyo3::Py::new(py, self).unwrap(), py)
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        unsafe impl pyo3::type_object::PyTypeInfo for #cls {
            type Type = #cls;
            type BaseType = #base;
            type ConcreteLayout = pyo3::pyclass::PyClassShell<Self>;
            type Initializer = pyo3::pyclass_init::PyClassInitializer<Self>;

            const NAME: &'static str = #cls_name;
            const MODULE: Option<&'static str> = #module;
            const DESCRIPTION: &'static str = #doc;
            const FLAGS: usize = #(#flags)|* | #extended;

            #[inline]
            fn type_object() -> std::ptr::NonNull<pyo3::ffi::PyTypeObject> {
                use pyo3::type_object::LazyTypeObject;
                static TYPE_OBJECT: LazyTypeObject = LazyTypeObject::new();
                TYPE_OBJECT.get_pyclass_type::<Self>()
            }
        }

        impl pyo3::PyClass for #cls {
            #dict
            #weakref
        }

        impl pyo3::conversion::FromPyObjectImpl for #cls {
            type Impl = pyo3::conversion::extract_impl::Cloned;
        }

        impl pyo3::conversion::FromPyObjectImpl for &'_ #cls {
            type Impl = pyo3::conversion::extract_impl::Reference;
        }

        impl pyo3::conversion::FromPyObjectImpl for &'_ mut #cls {
            type Impl = pyo3::conversion::extract_impl::MutReference;
        }

        #into_pyobject

        #inventory_impl

        #extra

        #gc_impl

    })
}

fn impl_descriptors(
    cls: &syn::Type,
    descriptors: Vec<(syn::Field, Vec<FnType>)>,
) -> syn::Result<TokenStream> {
    let methods: Vec<TokenStream> = descriptors
        .iter()
        .flat_map(|&(ref field, ref fns)| {
            fns.iter()
                .map(|desc| {
                    let name = field.ident.as_ref().unwrap();
                    let field_ty = &field.ty;
                    match *desc {
                        FnType::Getter => {
                            quote! {
                                impl #cls {
                                    fn #name(&self) -> pyo3::PyResult<#field_ty> {
                                        Ok(self.#name.clone())
                                    }
                                }
                            }
                        }
                        FnType::Setter => {
                            let setter_name =
                                syn::Ident::new(&format!("set_{}", name.unraw()), Span::call_site());
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
                    let name = field.ident.as_ref().unwrap();

                    let doc = utils::get_doc(&field.attrs, None, true)
                        .unwrap_or_else(|_| syn::LitStr::new(&name.to_string(), name.span()));

                    let field_ty = &field.ty;
                    match *desc {
                        FnType::Getter => {
                            let spec = FnSpec {
                                tp: FnType::Getter,
                                name: &name,
                                python_name: name.unraw(),
                                attrs: Vec::new(),
                                args: Vec::new(),
                                output: parse_quote!(PyResult<#field_ty>),
                                doc,
                            };
                            Ok(impl_py_getter_def(&spec, &impl_wrap_getter(&cls, &spec)?))
                        }
                        FnType::Setter => {
                            let setter_name = syn::Ident::new(
                                &format!("set_{}", name.unraw()),
                                Span::call_site(),
                            );
                            let spec = FnSpec {
                                tp: FnType::Setter,
                                name: &setter_name,
                                python_name: name.unraw(),
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
                                doc,
                            };
                            Ok(impl_py_setter_def(&spec, &impl_wrap_setter(&cls, &spec)?))
                        }
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<syn::Result<TokenStream>>>()
        })
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        #(#methods)*

        pyo3::inventory::submit! {
            #![crate = pyo3] {
                type ClsInventory = <#cls as pyo3::class::methods::PyMethodsInventoryDispatch>::InventoryType;
                <ClsInventory as pyo3::class::methods::PyMethodsInventory>::new(&[#(#py_methods),*])
            }
        }
    })
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
