use proc_macro2::{Span, TokenStream};
use quote::quote;

/// To allow multiple #[pymethods]/#[pyproto] block, we define inventory types.
pub fn impl_methods_inventory(cls: &syn::Ident) -> TokenStream {
    // Try to build a unique type for better error messages
    let name = format!("Pyo3MethodsInventoryFor{}", cls);
    let inventory_cls = syn::Ident::new(&name, Span::call_site());

    quote! {
        #[doc(hidden)]
        pub struct #inventory_cls {
            methods: &'static [pyo3::class::PyMethodDefType],
        }
        impl pyo3::class::methods::PyMethodsInventory for #inventory_cls {
            fn new(methods: &'static [pyo3::class::PyMethodDefType]) -> Self {
                Self { methods }
            }
            fn get(&self) -> &'static [pyo3::class::PyMethodDefType] {
                self.methods
            }
        }

        impl pyo3::class::methods::HasMethodsInventory for #cls {
            type Methods = #inventory_cls;
        }

        pyo3::inventory::collect!(#inventory_cls);
    }
}

pub fn impl_extractext(cls: &syn::Ident) -> TokenStream {
    quote! {
        impl<'a> pyo3::derive_utils::ExtractExt<'a> for &'a #cls
        {
            type Target = pyo3::PyRef<'a, #cls>;
        }

        impl<'a> pyo3::derive_utils::ExtractExt<'a> for &'a mut #cls
        {
            type Target = pyo3::PyRefMut<'a, #cls>;
        }
    }
}

/// Implement `HasProtoRegistry` for the class for lazy protocol initialization.
pub fn impl_proto_registry(cls: &syn::Ident) -> TokenStream {
    quote! {
        impl pyo3::class::proto_methods::HasProtoRegistry for #cls {
            fn registry() -> &'static pyo3::class::proto_methods::PyProtoRegistry {
                static REGISTRY: pyo3::class::proto_methods::PyProtoRegistry
                    = pyo3::class::proto_methods::PyProtoRegistry::new();
                &REGISTRY
            }
        }
    }
}
