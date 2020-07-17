// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::pyclass::impl_methods_inventory;
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
    _variants: Vec<(syn::Ident, syn::ExprLit)>,
) -> syn::Result<TokenStream> {
    let inventory = impl_methods_inventory(enum_);

    let enum_name = enum_.to_string();

    Ok(quote! {
        unsafe impl pyo3::type_object::PyTypeInfo for #enum_ {
            type Type = #enum_;
            type BaseType = pyo3::PyAny;
            type Layout = pyo3::PyCell<Self>;
            type BaseLayout = pyo3::pycell::PyCellBase<pyo3::PyAny>;

            type Initializer = pyo3::pyclass_init::PyClassInitializer<Self>;
            type AsRefTarget = pyo3::PyCell<Self>;

            const NAME: &'static str = #enum_name;
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

        impl pyo3::PyClass for #enum_ {
            type Dict =  pyo3::pyclass_slots::PyClassDummySlot ;
            type WeakRef = pyo3::pyclass_slots::PyClassDummySlot;
            type BaseNativeType = pyo3::PyAny;
        }

        impl<'a> pyo3::derive_utils::ExtractExt<'a> for &'a #enum_
        {
            type Target = pyo3::PyRef<'a, #enum_>;
        }

        impl<'a> pyo3::derive_utils::ExtractExt<'a> for &'a mut #enum_
        {
            type Target = pyo3::PyRefMut<'a, #enum_>;
        }

        impl pyo3::class::proto_methods::HasProtoRegistry for #enum_ {
            fn registry() -> &'static pyo3::class::proto_methods::PyProtoRegistry {
                static REGISTRY: pyo3::class::proto_methods::PyProtoRegistry
                    = pyo3::class::proto_methods::PyProtoRegistry::new();
                &REGISTRY
            }
        }

        impl pyo3::pyclass::PyClassAlloc for #enum_ {}

        #inventory
    })
}
