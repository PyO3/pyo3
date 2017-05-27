// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;


pub fn build_py_class(ast: &mut syn::DeriveInput) -> Tokens {
    let base = syn::Ident::from("_pyo3::PyObject");

    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(_)) => {
        },
        _ => panic!("#[class] can only be used with notmal structs"),
    }

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_CLS_{}", ast.ident));
    let tokens = impl_class(&ast.ident, &base);

    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables, non_camel_case_types)]
        const #dummy_const: () = {
            extern crate pyo3 as _pyo3;
            use std;

            #tokens
        };
    }
}

fn impl_class(cls: &syn::Ident, base: &syn::Ident) -> Tokens {
    let token_name = syn::Ident::from("__py_token");
    let cls_name = quote! { #cls }.as_str().to_string();

    quote! {
        impl _pyo3::typeob::PyTypeInfo for #cls {
            type Type = #cls;

            #[inline]
            fn size() -> usize {
                Self::offset() as usize + std::mem::size_of::<#cls>()
            }

            #[inline]
            fn offset() -> isize {
                let align = std::mem::align_of::<#cls>();
                let bs = <#base as _pyo3::typeob::PyTypeInfo>::size();

                // round base_size up to next multiple of align
                ((bs + align - 1) / align * align) as isize
            }

            #[inline]
            fn type_name() -> &'static str { #cls_name }

            #[inline]
            fn type_object() -> &'static mut _pyo3::ffi::PyTypeObject {
                static mut TYPE_OBJECT: _pyo3::ffi::PyTypeObject = _pyo3::ffi::PyTypeObject_INIT;
                unsafe { &mut TYPE_OBJECT }
            }
        }

        impl _pyo3::python::ToPythonPointer for #cls {
            #[inline]
            fn as_ptr(&self) -> *mut ffi::PyObject {
                let offset = <#cls as _pyo3::typeob::PyTypeInfo>::offset();
                unsafe {
                    {self as *const _ as *mut u8}.offset(-offset) as *mut _pyo3::ffi::PyObject
                }
            }
        }

        impl _pyo3::python::PyClone for #cls {
            fn clone_ref(&self) -> PyPtr<#cls> {
                unsafe {
                    let ptr = <#cls as _pyo3::python::ToPythonPointer>::as_ptr(self);
                    _pyo3::PyPtr::from_borrowed_ptr(ptr)
                }
            }
        }
    }
}
