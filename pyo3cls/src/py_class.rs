// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;


pub fn build_py_class(ast: &mut syn::DeriveInput) -> Tokens {
    let base = syn::Ident::from("_pyo3::PyObject");

    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(_)) => (),
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
            use pyo3::python::IntoPythonPointer;

            #tokens
        };
    }
}

fn impl_class(cls: &syn::Ident, base: &syn::Ident) -> Tokens {
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

        impl _pyo3::IntoPyObject for #cls {
            #[inline]
            fn into_object<'p>(self, py: Python<'p>) -> Py<'p, PyObject> where Self: Sized
            {
                let ptr = py.init(self).into_ptr();
                _pyo3::PyObject::from_owned_ptr(py, ptr)
            }
        }

        impl<'p> _pyo3::python::AsPy<'p> for &'p #cls {
            #[inline]
            fn py<'a>(&'a self) -> _pyo3::Python<'p> {
                unsafe { _pyo3::python::Python::assume_gil_acquired() }
            }
        }

        impl<'p> _pyo3::python::AsPy<'p> for &'p mut #cls {
            #[inline]
            fn py<'a>(&'a self) -> _pyo3::Python<'p> {
                unsafe { _pyo3::python::Python::assume_gil_acquired() }
            }
        }
    }
}
