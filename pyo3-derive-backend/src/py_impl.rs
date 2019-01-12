// Copyright (c) 2017-present PyO3 Project and Contributors

use method::FnSpec;
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
    use syn::PathArguments::AngleBracketed;

    // If there are generics, we expect a `variants` directive.
    let variants = if !generics.params.is_empty() {
        if let Some(syn::Expr::Call(ref call)) = attrs.first() {
            utils::parse_variants(call)
                .into_iter()
                .map(|(_, x)| AngleBracketed(x))
                .collect()
        } else {
            panic!("`variants` annotation is required when using generics");
        }
    } else {
        vec![syn::PathArguments::None]
    };

    // Parse method info.
    let untouched_impls = impls.clone();
    let fn_specs: Vec<_> = impls
        .iter_mut()
        .filter_map(|x| match x {
            syn::ImplItem::Method(meth) => {
                Some(FnSpec::parse(&meth.sig.ident, &meth.sig, &mut meth.attrs))
            }
            _ => None,
        })
        .collect();

    // Emit one `PyMethodsProtocolImpl` impl for each variant.
    let impls = variants.into_iter().map(|variant_args| {
        // Replace generic path arguments with concrete variant type arguments and generate
        // `type T1 = ConcreteT1` statements for use in the wrapper methods.
        //
        // Why do aliasing instead of just replacing the types in the arg and return types, you may
        // ask. I originally wrote a function recursively traversing and replacing generic types in
        // all arguments and the return val, however it turned out to be a pretty complex beast
        // that would also be guaranteed to be a burden in maintenance to keep up with all Rust
        // syntax additions. This simple aliasing approach doesn't have these problems.
        let mut variant_ty = ty.clone();
        let ty_map = if let syn::Type::Path(syn::TypePath { ref mut path, .. }) = variant_ty {
            let tail = path.segments.iter_mut().last().unwrap();
            let generic_args = std::mem::replace(&mut tail.arguments, variant_args);

            match (&generic_args, &mut tail.arguments) {
                (AngleBracketed(generic), AngleBracketed(variant)) => {
                    // Some generated methods require the type in turbo-fish syntax.
                    variant.colon2_token = parse_quote! { :: };

                    generic
                        .args
                        .iter()
                        .zip(variant.args.iter())
                        .map(|(a, b)| {
                            quote! {
                                type #a = #b;
                            }
                        })
                        .collect()
                }
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };

        // Generate wrappers for Python methods.
        let mut methods = Vec::new();
        for (iimpl, fn_spec) in untouched_impls.iter().zip(&fn_specs) {
            if let syn::ImplItem::Method(meth) = iimpl {
                methods.push(py_method::gen_py_method(
                    &variant_ty,
                    &meth.sig.ident,
                    &meth.sig,
                    &meth.attrs,
                    fn_spec,
                ));
            }
        }

        // Emit the `PyMethodsProtocolImpl` impl for this struct variant.
        quote! {
            impl ::pyo3::class::methods::PyMethodsProtocolImpl for #variant_ty {
                fn py_methods() -> &'static [::pyo3::class::PyMethodDefType] {
                    #(#ty_map)*
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
