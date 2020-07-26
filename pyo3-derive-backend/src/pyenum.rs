// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::common::{impl_extractext, impl_methods_inventory, impl_proto_registry};
use proc_macro2::TokenStream;
use quote::quote;

pub fn build_py_enum(enum_: &syn::ItemEnum) -> syn::Result<TokenStream> {
    let mut variants = Vec::new();

    for variant in enum_.variants.iter() {
        if !variant.fields.is_empty() {
            return Err(syn::Error::new_spanned(
                variant,
                "#[pyenum] only supports unit enums",
            ));
        }
        if let Some((_, syn::Expr::Lit(lit))) = &variant.discriminant {
            variants.push((variant.ident.clone(), lit.clone()))
        } else {
            return Err(syn::Error::new_spanned(
                variant,
                "#[pyenum] requires explicit discriminant (MyVal = 4)",
            ));
        }
    }

    impl_enum(&enum_.ident, variants)
}

fn impl_enum(
    enum_: &syn::Ident,
    variants: Vec<(syn::Ident, syn::ExprLit)>,
) -> syn::Result<TokenStream> {
    let enum_cls = impl_class(enum_)?;
    let variant_cls = variants
        .iter()
        .map(|(ident, _)| impl_class(ident))
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        #enum_cls
        #(#variant_cls)*
    })
}

fn impl_class(cls: &syn::Ident) -> syn::Result<TokenStream> {
    let inventory = impl_methods_inventory(cls);
    let extractext = impl_extractext(cls);
    let protoregistry = impl_proto_registry(cls);

    let clsname = cls.to_string();

    Ok(quote! {
        unsafe impl pyo3::type_object::PyTypeInfo for #cls {
            type Type = #cls;
            type BaseType = pyo3::PyAny;
            type Layout = pyo3::PyCell<Self>;
            type BaseLayout = pyo3::pycell::PyCellBase<pyo3::PyAny>;

            type Initializer = pyo3::pyclass_init::PyClassInitializer<Self>;
            type AsRefTarget = pyo3::PyCell<Self>;

            const NAME: &'static str = #clsname;
            const MODULE: Option<&'static str> = None;
            const DESCRIPTION: &'static str = "y'know, an enum\0"; // TODO
            const FLAGS: usize = 0;

            #[inline]
            fn type_object_raw(py: pyo3::Python) -> *mut pyo3::ffi::PyTypeObject {
                use pyo3::type_object::LazyStaticType;
                static TYPE_OBJECT: LazyStaticType = LazyStaticType::new();
                TYPE_OBJECT.get_or_init::<Self>(py)
            }

        }

        impl pyo3::PyClass for #cls {
            type Dict =  pyo3::pyclass_slots::PyClassDummySlot ;
            type WeakRef = pyo3::pyclass_slots::PyClassDummySlot;
            type BaseNativeType = pyo3::PyAny;
        }

        #protoregistry

        #extractext

        impl pyo3::pyclass::PyClassAlloc for #cls {}

        // TODO: handle not in send
        impl pyo3::pyclass::PyClassSend for #cls {
            type ThreadChecker = pyo3::pyclass::ThreadCheckerStub<#cls>;
        }

        #inventory
    })
}
