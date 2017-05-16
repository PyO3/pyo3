// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};


pub fn build_py_class(ast: &mut syn::DeriveInput) -> Tokens {
    if let syn::Body::Enum(_) = ast.body {
        panic!("#[py_class] can only be used with structs")
    }

    let mut tokens = Tokens::new();
    impl_to_py_object(&ast.ident).to_tokens(&mut tokens);
    impl_from_py_object(&ast.ident).to_tokens(&mut tokens);
    impl_python_object(&ast.ident).to_tokens(&mut tokens);
    impl_checked_downcast(&ast.ident).to_tokens(&mut tokens);
    impl_class_init(&ast.ident).to_tokens(&mut tokens);

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_CLS_{}", ast.ident));
    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate pyo3;
            use std;
            use pyo3::ffi;

            #tokens
        };
    }
}

fn impl_to_py_object(cls: &syn::Ident) -> Tokens {
    quote! {
        /// Identity conversion: allows using existing `PyObject` instances where
        /// `T: ToPyObject` is expected.
        impl pyo3::ToPyObject for #cls where #cls: pyo3::PythonObject {
            #[inline]
            fn to_py_object(&self, py: pyo3::Python) -> pyo3::PyObject {
                pyo3::PyClone::clone_ref(self, py).into_object()
            }

            #[inline]
            fn into_py_object(self, _py: pyo3::Python) -> pyo3::PyObject {
                self.into_object()
            }

            #[inline]
            fn with_borrowed_ptr<F, R>(&self, _py: pyo3::Python, f: F) -> R
                where F: FnOnce(*mut pyo3::ffi::PyObject) -> R
            {
                f(pyo3::PythonObject::as_object(self).as_ptr())
            }
        }
    }
}

fn impl_from_py_object(cls: &syn::Ident) -> Tokens {
    quote! {
        impl <'source> pyo3::FromPyObject<'source> for #cls {
            #[inline]
            fn extract(py: pyo3::Python, obj: &'source pyo3::PyObject)
                       -> pyo3::PyResult<#cls> {
                Ok(obj.clone_ref(py).cast_into::<#cls>(py)?)
            }
        }

        impl <'source> pyo3::FromPyObject<'source> for &'source #cls {
            #[inline]
            fn extract(py: pyo3::Python, obj: &'source pyo3::PyObject)
                       -> pyo3::PyResult<&'source #cls> {
                Ok(obj.cast_as::<#cls>(py)?)
            }
        }
    }
}

fn impl_python_object(cls: &syn::Ident) -> Tokens {
    quote! {
        impl pyo3::PythonObject for #cls {
            #[inline]
            fn as_object(&self) -> &pyo3::PyObject {
                &self._unsafe_inner
            }

            #[inline]
            fn into_object(self) -> pyo3::PyObject {
                self._unsafe_inner
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_from(obj: pyo3::PyObject) -> Self {
                #cls { _unsafe_inner: obj }
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a pyo3::PyObject) -> &'a Self {
                std::mem::transmute(obj)
            }
        }
    }
}

fn impl_checked_downcast(cls: &syn::Ident) -> Tokens {
    quote! {
        impl pyo3::PythonObjectWithCheckedDowncast for #cls {
            #[inline]
            fn downcast_from<'p>(py: pyo3::Python<'p>, obj: pyo3::PyObject)
                                 -> Result<#cls, pyo3::PythonObjectDowncastError<'p>> {
                if py.get_type::<#cls>().is_instance(py, &obj) {
                    Ok(#cls { _unsafe_inner: obj })
                } else {
                    Err(pyo3::PythonObjectDowncastError(py))
                }
            }

            #[inline]
            fn downcast_borrow_from<'a, 'p>(py: pyo3::Python<'p>, obj: &'a pyo3::PyObject)
                                            -> Result<&'a #cls, pyo3::PythonObjectDowncastError<'p>> {
                if py.get_type::<#cls>().is_instance(py, obj) {
                    unsafe { Ok(std::mem::transmute(obj)) }
                } else {
                    Err(pyo3::PythonObjectDowncastError(py))
                }
            }
        }
    }
}

fn impl_class_init(cls: &syn::Ident) -> Tokens {
    quote! {
        impl pyo3::class::typeob::PyClassInit for #cls {
            fn init() -> bool {
                true
            }
        }
    }
}
