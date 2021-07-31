// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::{
    konst::{ConstAttributes, ConstSpec},
    pyfunction::PyFunctionOptions,
    pymethod,
};
use proc_macro2::TokenStream;
use pymethod::GeneratedPyMethod;
use quote::quote;
use syn::spanned::Spanned;

/// The mechanism used to collect `#[pymethods]` into the type object
pub enum PyClassMethodsType {
    Specialization,
    Inventory,
}

pub fn build_py_methods(
    ast: &mut syn::ItemImpl,
    methods_type: PyClassMethodsType,
) -> syn::Result<TokenStream> {
    if let Some((_, path, _)) = &ast.trait_ {
        bail_spanned!(path.span() => "#[pymethods] cannot be used on trait impl blocks");
    } else if ast.generics != Default::default() {
        bail_spanned!(
            ast.generics.span() =>
            "#[pymethods] cannot be used with lifetime parameters or generics"
        );
    } else {
        impl_methods(&ast.self_ty, &mut ast.items, methods_type)
    }
}

pub fn impl_methods(
    ty: &syn::Type,
    impls: &mut Vec<syn::ImplItem>,
    methods_type: PyClassMethodsType,
) -> syn::Result<TokenStream> {
    let mut new_impls = Vec::new();
    let mut call_impls = Vec::new();
    let mut methods = Vec::new();
    for iimpl in impls.iter_mut() {
        match iimpl {
            syn::ImplItem::Method(meth) => {
                let options = PyFunctionOptions::from_attrs(&mut meth.attrs)?;
                match pymethod::gen_py_method(ty, &mut meth.sig, &mut meth.attrs, options)? {
                    GeneratedPyMethod::Method(token_stream) => {
                        let attrs = get_cfg_attributes(&meth.attrs);
                        methods.push(quote!(#(#attrs)* #token_stream));
                    }
                    GeneratedPyMethod::New(token_stream) => {
                        let attrs = get_cfg_attributes(&meth.attrs);
                        new_impls.push(quote!(#(#attrs)* #token_stream));
                    }
                    GeneratedPyMethod::Call(token_stream) => {
                        let attrs = get_cfg_attributes(&meth.attrs);
                        call_impls.push(quote!(#(#attrs)* #token_stream));
                    }
                }
            }
            syn::ImplItem::Const(konst) => {
                let attributes = ConstAttributes::from_attrs(&mut konst.attrs)?;
                if attributes.is_class_attr {
                    let spec = ConstSpec {
                        rust_ident: konst.ident.clone(),
                        attributes,
                    };
                    let attrs = get_cfg_attributes(&konst.attrs);
                    let meth = gen_py_const(ty, &spec);
                    methods.push(quote!(#(#attrs)* #meth));
                }
            }
            _ => (),
        }
    }

    let methods_registration = match methods_type {
        PyClassMethodsType::Specialization => impl_py_methods(ty, methods),
        PyClassMethodsType::Inventory => submit_methods_inventory(ty, methods),
    };

    Ok(quote! {
        #(#new_impls)*

        #(#call_impls)*

        #methods_registration
    })
}

fn gen_py_const(cls: &syn::Type, spec: &ConstSpec) -> TokenStream {
    let member = &spec.rust_ident;
    let deprecations = &spec.attributes.deprecations;
    let python_name = &spec.null_terminated_python_name();
    quote! {
        pyo3::class::PyMethodDefType::ClassAttribute({
            pyo3::class::PyClassAttributeDef::new(
                #python_name,
                pyo3::class::methods::PyClassAttributeFactory({
                    fn __wrap(py: pyo3::Python<'_>) -> pyo3::PyObject {
                        #deprecations
                        pyo3::IntoPy::into_py(#cls::#member, py)
                    }
                    __wrap
                })
            )
        })
    }
}

fn impl_py_methods(ty: &syn::Type, methods: Vec<TokenStream>) -> TokenStream {
    quote! {
        impl pyo3::class::impl_::PyMethods<#ty>
            for pyo3::class::impl_::PyClassImplCollector<#ty>
        {
            fn py_methods(self) -> &'static [pyo3::class::methods::PyMethodDefType] {
                static METHODS: &[pyo3::class::methods::PyMethodDefType] = &[#(#methods),*];
                METHODS
            }
        }
    }
}

fn submit_methods_inventory(ty: &syn::Type, methods: Vec<TokenStream>) -> TokenStream {
    if methods.is_empty() {
        return TokenStream::default();
    }

    quote! {
        pyo3::inventory::submit! {
            #![crate = pyo3] {
                type Inventory = <#ty as pyo3::class::impl_::HasMethodsInventory>::Methods;
                <Inventory as pyo3::class::impl_::PyMethodsInventory>::new(vec![#(#methods),*])
            }
        }
    }
}

fn get_cfg_attributes(attrs: &[syn::Attribute]) -> Vec<&syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("cfg"))
        .collect()
}
