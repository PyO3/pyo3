// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;

use py_method;


pub fn build_py_methods(ast: &mut syn::Item) -> Tokens {
    match ast.node {
        syn::ItemKind::Impl(_, _, _, ref path, ref ty, ref mut impl_items) => {
            if let &Some(_) = path {
                panic!("#[methods] can not be used only with trait impl block");
            } else {
                impl_methods(ty, impl_items)
            }
        },
        _ => panic!("#[methods] can only be used with Impl blocks"),
    }
}

fn impl_methods(ty: &Box<syn::Ty>, impls: &mut Vec<syn::ImplItem>) -> Tokens {

    // get method names in impl block
    let mut methods = Vec::new();
    for iimpl in impls.iter_mut() {
        match iimpl.node {
            syn::ImplItemKind::Method(ref mut sig, _) => {
                methods.push(py_method::gen_py_method(
                    ty, &iimpl.ident, sig, &mut iimpl.attrs));
            },
            _ => (),
        }
    }

    let tokens = quote! {
        impl _pyo3::class::methods::PyMethodsProtocolImpl for #ty {
            fn py_methods() -> &'static [_pyo3::class::PyMethodDefType] {
                static METHODS: &'static [_pyo3::class::PyMethodDefType] = &[
                    #(#methods),*
                ];
                METHODS
            }
        }
    };

    let n = match ty.as_ref() {
        &syn::Ty::Path(_, ref p) => {
            p.segments.last().as_ref().unwrap().ident.as_ref()
        }
        _ => "CLS_METHODS"
    };

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_METHODS_{}", n));
    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables, unused_imports)]
        const #dummy_const: () = {
            extern crate pyo3 as _pyo3;

            #tokens
        };
    }
}
