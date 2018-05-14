// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;

use py_method;


pub fn build_py_methods(ast: &mut syn::ItemImpl) -> Tokens {
    if ast.trait_.is_some() {
        panic!("#[methods] can not be used only with trait impl block");
    } else {
        impl_methods(&iimpl.self_ty, &mut iimpl.items)
    }
}

pub fn impl_methods(ty: &syn::Type, impls: &mut Vec<syn::ImplItem>) -> Tokens {

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
        typath.path.segments.last().as_ref().unwrap().value().ident.as_ref()
    } else {
        "CLS_METHODS"
    };

    let dummy_const = syn::Ident::from(format!("_IMPL_PYO3_METHODS_{}", n));
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
