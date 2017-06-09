// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::collections::HashMap;

use syn;
use quote::Tokens;


pub fn build_py_class(ast: &mut syn::DeriveInput, attr: String) -> Tokens {
    let params = parse_attribute(attr);

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
    let tokens = impl_class(&ast.ident, &base, token, params);

    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables, non_camel_case_types)]
        const #dummy_const: () = {
            use std;
            extern crate pyo3 as _pyo3;

            #tokens
        };
    }
}

fn impl_class(cls: &syn::Ident, base: &syn::Ident,
              token: Option<syn::Ident>, params: HashMap<&'static str, syn::Ident>) -> Tokens {
    let cls_name = match params.get("name") {
        Some(name) => quote! { #name }.as_str().to_string(),
        None => quote! { #cls }.as_str().to_string()
    };

    let extra = if let Some(token) = token {
        Some(quote! {
            impl _pyo3::PyObjectWithToken for #cls {
                fn token<'p>(&'p self) -> _pyo3::python::Python<'p> {
                    self.#token.token()
                }
            }

            impl std::fmt::Debug for #cls {
                fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                    let py = _pyo3::PyObjectWithToken::token(self);
                    let ptr = <#cls as _pyo3::python::ToPyPointer>::as_ptr(self);
                    let repr = unsafe {
                        _pyo3::PyString::downcast_from_ptr(
                            py, _pyo3::ffi::PyObject_Repr(ptr)).map_err(|_| std::fmt::Error)?
                    };
                    let result = f.write_str(&repr.to_string_lossy(py));
                    py.release(repr);
                    result
                }
            }

            impl std::fmt::Display for #cls {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                    let py = _pyo3::PyObjectWithToken::token(self);
                    let ptr = <#cls as _pyo3::python::ToPyPointer>::as_ptr(self);
                    let str_obj = unsafe {
                        _pyo3::PyString::downcast_from_ptr(
                            py, _pyo3::ffi::PyObject_Str(ptr)).map_err(|_| std::fmt::Error)?
                    };
                    let result = f.write_str(&str_obj.to_string_lossy(py));
                    py.release(str_obj);
                    result
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

        impl _pyo3::typeob::PyTypeObject for #cls {
            #[inline(always)]
            fn init_type(py: Python) {
                static START: std::sync::Once = std::sync::ONCE_INIT;
                START.call_once(|| {
                    let mut ty = <#cls as _pyo3::typeob::PyTypeInfo>::type_object();

                    if (ty.tp_flags & _pyo3::ffi::Py_TPFLAGS_READY) == 0 {
                        // automatically initialize the class on-demand
                        let to = _pyo3::typeob::initialize_type::<#cls>(
                            py, None, <#cls as _pyo3::typeob::PyTypeInfo>::type_name(), ty)
                            .expect(
                                format!("An error occurred while initializing class {}",
                                        <#cls as _pyo3::typeob::PyTypeInfo>::type_name())
                                    .as_ref());
                        py.release(to);
                    }
                });
            }
        }

        impl _pyo3::python::PyDowncastFrom for #cls
        {
            fn downcast_from<'a, 'p>(py: Python<'p>, ob: &'a _pyo3::PyObject)
                                     -> Result<&'a #cls, _pyo3::PyDowncastError<'p>>
            {
                unsafe {
                    let checked = ffi::PyObject_TypeCheck(
                        ob.as_ptr(), <#cls as _pyo3::typeob::PyTypeInfo>::type_object()) != 0;

                    if checked {
                        let offset = <#cls as _pyo3::typeob::PyTypeInfo>::offset();
                        let ptr = (ob.as_ptr() as *mut u8).offset(offset) as *mut #cls;
                        Ok(ptr.as_ref().unwrap())
                    } else {
                        Err(_pyo3::PyDowncastError(py, None))
                    }
                }
            }
        }
        impl _pyo3::python::PyMutDowncastFrom for #cls
        {
            fn downcast_mut_from<'a, 'p>(py: Python<'p>, ob: &'a mut _pyo3::PyObject)
                                         -> Result<&'a mut #cls, _pyo3::PyDowncastError<'p>>
            {
                unsafe {
                    let checked = ffi::PyObject_TypeCheck(
                        ob.as_ptr(), <#cls as _pyo3::typeob::PyTypeInfo>::type_object()) != 0;

                    if checked {
                        let offset = <#cls as _pyo3::typeob::PyTypeInfo>::offset();
                        let ptr = (ob.as_ptr() as *mut u8).offset(offset) as *mut #cls;
                        Ok(ptr.as_mut().unwrap())
                    } else {
                        Err(_pyo3::PyDowncastError(py, None))
                    }
                }
            }
        }
        impl _pyo3::ToPyObject for #cls
        {
            #[inline]
            fn to_object<'p>(&self, py: _pyo3::Python<'p>) -> _pyo3::PyObject {
                _pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr())
            }

            #[inline]
            fn with_borrowed_ptr<F, R>(&self, _py: _pyo3::Python, f: F) -> R
                where F: FnOnce(*mut ffi::PyObject) -> R
            {
                f(self.as_ptr())
            }
        }
        impl std::convert::AsRef<PyObject> for #cls {
            fn as_ref(&self) -> &_pyo3::PyObject {
                unsafe{std::mem::transmute(self.as_ptr())}
            }
        }
        impl _pyo3::python::ToPyPointer for #cls {
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
                return segment.ident.as_ref() == "PyToken"
            }
        }
        _ => (),
    }
    return false
}

fn parse_attribute(attr: String) -> HashMap<&'static str, syn::Ident> {
    let mut params = HashMap::new();

    if let Ok(tts) = syn::parse_token_trees(&attr) {
        let mut elem = Vec::new();
        let mut elems = Vec::new();

        for tt in tts.iter() {
            match tt {
                &syn::TokenTree::Token(_) => {
                    println!("Wrong format: {:?}", attr.to_string());
                }
                &syn::TokenTree::Delimited(ref delimited) => {
                    for tt in delimited.tts.iter() {
                        match tt {
                            &syn::TokenTree::Token(syn::Token::Comma) => {
                                let el = std::mem::replace(&mut elem, Vec::new());
                                elems.push(el);
                            },
                            _ => elem.push(tt.clone())
                        }
                    }
                }
            }
        }
        if !elem.is_empty() {
            elems.push(elem);
        }

        for elem in elems {
            if elem.len() < 3 {
                println!("Wrong format: {:?}", elem);
                continue
            }

            let key = match elem[0] {
                syn::TokenTree::Token(syn::Token::Ident(ref ident)) => {
                    ident.as_ref().to_owned().to_lowercase()
                },
                _ => {
                    println!("Wrong format: {:?}", attr.to_string());
                    continue
                }
            };

            match elem[1] {
                syn::TokenTree::Token(syn::Token::Eq) => (),
                _ => {
                    println!("Wrong format: {:?}", attr.to_string());
                    continue
                }
            }

            match key.as_ref() {
                "freelist" => {
                    if elem.len() != 3 {
                        println!("Wrong 'freelist' format: {:?}", elem);
                    } else {
                        match elem[2] {
                            syn::TokenTree::Token(
                                syn::Token::Literal(
                                    syn::Lit::Int(val, _))) => {
                                params.insert("freelist", syn::Ident::from(val.to_string()));
                            }
                            _ => println!("Wrong 'freelist' format: {:?}", elem)
                        }
                    }
                },
                "name" => {
                    if elem.len() != 3 {
                        println!("Wrong 'name' format: {:?}", elem);
                    } else {
                        match elem[2] {
                            syn::TokenTree::Token(syn::Token::Ident(ref ident)) => {
                                params.insert("name", ident.clone());
                            },
                            _ => println!("Wrong 'name' format: {:?}", elem)
                        }
                    }
                },
                "base" => {

                }
                _ => {
                    println!("Unsupported parameter: {:?}", key);
                }
            }
        }
    }
    params
}
