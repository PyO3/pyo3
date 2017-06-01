// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;

pub fn build_ptr(cls: syn::Ident, ast: &mut syn::DeriveInput) -> Tokens {
    let ptr = &ast.ident;
    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_CLS_PTR_{}", ast.ident));

    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables, non_camel_case_types)]
        const #dummy_const: () = {
            use std;
            extern crate pyo3 as _pyo3;

            // thread-safe, because any python related operations require a Python<'p> token.
            unsafe impl Send for #ptr {}
            unsafe impl Sync for #ptr {}

            impl _pyo3::python::ParkRef for #cls {
                type Target = #ptr;

                fn park(&self) -> #ptr {
                    let token = _pyo3::PythonObjectWithToken::token(self);
                    let ptr = self.clone_ref(token).into_ptr();

                    #ptr(unsafe{_pyo3::PyPtr::from_owned_ptr(ptr)})
                }
            }

            impl<'p> _pyo3::python::Unpark<'p> for #ptr {
                type Target = Py<'p, #cls>;

                fn unpark(self, _py: Python<'p>) -> Py<'p, #cls> {
                    unsafe {std::mem::transmute(self)}
                }
            }

            impl std::ops::Deref for #ptr {
                type Target = _pyo3::pointers::PyPtr;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl _pyo3::IntoPyObject for #ptr {

                fn into_object(self, _py: Python) -> _pyo3::PyObjectPtr {
                    unsafe {std::mem::transmute(self)}
                }
            }
        };
    }
}
