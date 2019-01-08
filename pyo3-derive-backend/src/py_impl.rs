// Copyright (c) 2017-present PyO3 Project and Contributors

use proc_macro2::TokenStream;
use py_method;
use syn;
use utils;

pub fn build_py_methods(ast: &mut syn::ItemImpl, attrs: &Vec<syn::Expr>) -> TokenStream {
    if ast.trait_.is_some() {
        panic!("#[pymethods] can not be used only with trait impl block");
    }

    impl_methods(&ast.self_ty, &mut ast.items, attrs, &ast.generics)
}

pub fn impl_methods(
    ty: &syn::Type,
    impls: &mut Vec<syn::ImplItem>,
    attrs: &Vec<syn::Expr>,
    generics: &syn::Generics,
) -> TokenStream {
    // If there are generics, we expect a `variants` directive.
    let variants = if !generics.params.is_empty() {
        if let Some(syn::Expr::Call(ref call)) = attrs.first() {
            utils::parse_variants(call)
                .into_iter()
                .map(|(_, x)| syn::PathArguments::AngleBracketed(x))
                .collect()
        } else {
            panic!("`variants` annotation is required when using generics");
        }
    } else {
        vec![syn::PathArguments::None]
    };

    // Emit one `PyMethodsProtocolImpl` impl for each variant.
    let impls = variants.into_iter().map(|ty_args| {
        // Replace generic path arguments with concrete variant type arguments.
        let mut variant_ty = ty.clone();
        if let syn::Type::Path(syn::TypePath { ref mut path, .. }) = variant_ty {
            let tail = path.segments.iter_mut().last().unwrap();
            tail.arguments = ty_args;
        }

        // Generate wrappers for Python methods.
        let mut methods = Vec::new();
        for iimpl in impls.iter_mut() {
            if let syn::ImplItem::Method(ref mut meth) = iimpl {
                let name = meth.sig.ident.clone();
                methods.push(py_method::gen_py_method(
                    &variant_ty,
                    &name,
                    &mut meth.sig,
                    &mut meth.attrs,
                ));
            }
        }

        // Emit the `PyMethodsProtocolImpl` impl for this struct variant.
        quote! {
            impl ::pyo3::class::methods::PyMethodsProtocolImpl for #variant_ty {
                fn py_methods() -> &'static [::pyo3::class::PyMethodDefType] {
                    static METHODS: &'static [::pyo3::class::PyMethodDefType] = &[
                        #(#methods),*
                    ];
                    METHODS
                }
            }
        }
    });

    // Merge everything.
    quote! { #(#impls)* }
}
