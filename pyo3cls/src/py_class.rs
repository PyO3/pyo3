// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;


pub fn build_py_class(ast: &mut syn::DeriveInput) -> Tokens {
    let base = syn::Ident::from("_pyo3::PyObject");
    let mut token: Option<syn::Ident> = None;

    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref mut fields)) => {
            for field in fields.iter() {
                if is_python_token(field) {
                    token = field.ident.clone();
                    break
                }
            }
        },
        _ => panic!("#[class] can only be used with notmal structs"),
    }

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_CLS_{}", ast.ident));
    let tokens = impl_class(&ast.ident, &base, token);

    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables, non_camel_case_types)]
        const #dummy_const: () = {
            extern crate pyo3 as _pyo3;

            #tokens
        };
    }
}

fn impl_class(cls: &syn::Ident, base: &syn::Ident, token: Option<syn::Ident>) -> Tokens {
    let cls_name = quote! { #cls }.as_str().to_string();

    let extra = if let Some(token) = token {
        Some(quote! {
            impl _pyo3::PythonObjectWithToken for #cls {
                fn token<'p>(&'p self) -> _pyo3::python::Python<'p> {
                    self.#token.token()
                }
            }

            impl std::fmt::Debug for #cls {
                fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                    let ptr = <#cls as _pyo3::python::ToPythonPointer>::as_ptr(self);
                    let repr = unsafe {
                        PyString::downcast_from_owned_ptr(
                            self.token(), _pyo3::ffi::PyObject_Repr(ptr))
                            .map_err(|_| std::fmt::Error)?
                    };
                    f.write_str(&repr.to_string_lossy())
                }
            }

            impl std::fmt::Display for #cls {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                    let ptr = <#cls as _pyo3::python::ToPythonPointer>::as_ptr(self);
                    let str_obj = unsafe {
                        PyString::downcast_from_owned_ptr(
                            self.token(), _pyo3::ffi::PyObject_Str(ptr))
                            .map_err(|_| std::fmt::Error)?
                    };
                    f.write_str(&str_obj.to_string_lossy())
                }
            }
        })
    } else {
        None
    };

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

        impl<'p> _pyo3::python::PyDowncastFrom<'p> for #cls
        {
            fn downcast_from(py: &'p _pyo3::PyObject<'p>)
                             -> Result<&'p #cls, _pyo3::PyDowncastError<'p>>
            {
                unsafe {
                    let checked = ffi::PyObject_TypeCheck(
                        py.as_ptr(), <#cls as _pyo3::typeob::PyTypeInfo>::type_object()) != 0;

                    if checked {
                        let offset = <#cls as _pyo3::typeob::PyTypeInfo>::offset();
                        let ptr = (py.as_ptr() as *mut u8).offset(offset) as *mut #cls;
                        Ok(ptr.as_ref().unwrap())
                    } else {
                        Err(_pyo3::PyDowncastError(py.gil(), None))
                    }
                }
            }
        }

        impl _pyo3::python::ToPythonPointer for #cls {
            #[inline]
            fn as_ptr(&self) -> *mut ffi::PyObject {
                let offset = <#cls as _pyo3::typeob::PyTypeInfo>::offset();
                unsafe {
                    {self as *const _ as *mut u8}.offset(-offset) as *mut ffi::PyObject
                }
            }
        }

        #extra
    }
}

fn is_python_token(field: &syn::Field) -> bool {
    match field.ty {
        syn::Ty::Path(_, ref path) => {
            if let Some(segment) = path.segments.last() {
                return segment.ident.as_ref() == "PythonToken"
            }
        }
        _ => (),
    }
    return false
}
