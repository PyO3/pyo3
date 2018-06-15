// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;

use py_method;
use proc_macro2::{TokenStream, Span};


pub fn build_py_methods(ast: &mut syn::ItemImpl) -> TokenStream {
    if ast.trait_.is_some() {
        panic!("#[methods] can not be used only with trait impl block");
    } else {
        impl_methods(&ast.self_ty, &mut ast.items)
    }
}

pub fn impl_methods(ty: &syn::Type, impls: &mut Vec<syn::ImplItem>) -> TokenStream {

    // get method names in impl block
    let mut methods = Vec::new();
    for iimpl in impls.iter_mut() {
        if let syn::ImplItem::Method(ref mut meth) = iimpl {
            let name = meth.sig.ident.clone();
            methods.push(py_method::gen_py_method(ty, &name, &mut meth.sig, &mut meth.attrs));
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

    let n = if let &syn::Type::Path(ref typath) = ty {
        typath.path.segments.last().as_ref().unwrap().value().ident.to_string()
    } else {
        "CLS_METHODS".to_string()
    };

    let dummy_const = syn::Ident::new(&format!("_IMPL_PYO3_METHODS_{}", n), Span::call_site());
    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables, unused_imports)]
        const #dummy_const: () = {
            use pyo3 as _pyo3;

            #tokens
        };
    }
}
