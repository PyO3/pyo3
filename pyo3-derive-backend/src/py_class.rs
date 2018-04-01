// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::collections::HashMap;

use syn;
use quote::{Tokens, ToTokens};

use utils;
use method::{FnType, FnSpec, FnArg};
use py_method::{impl_wrap_getter, impl_wrap_setter, impl_py_getter_def, impl_py_setter_def};


pub fn build_py_class(ast: &mut syn::DeriveInput, attr: String) -> Tokens {
    let (params, flags, base) = parse_attribute(attr);
    let doc = utils::get_doc(&ast.attrs, true);
    let mut token: Option<syn::Ident> = None;
    let mut descriptors = Vec::new();
    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref mut fields)) => {
            for field in fields.iter_mut() {
                if is_python_token(field) {
                    token = field.ident.clone();
                    break
                } else {
                    let field_descs = parse_descriptors(field);
                    if !field_descs.is_empty() {
                        descriptors.push((field.clone(), field_descs));
                    }
                }
            }
        },
        _ => panic!("#[class] can only be used with normal structs"),
    }

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_CLS_{}", ast.ident));
    let tokens = impl_class(&ast.ident, &base, token, doc, params, flags, descriptors);

    quote! {
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables, non_camel_case_types)]
        const #dummy_const: () = {
            use std;
            extern crate pyo3 as _pyo3;

            #tokens
        };
    }
}

fn parse_descriptors(item: &mut syn::Field) -> Vec<FnType> {
    let mut descs = Vec::new();
    let mut new_attrs = Vec::new();
    for attr in item.attrs.iter() {
        match attr.value {
            syn::MetaItem::List(ref name, ref metas) => {
                match name.as_ref() {
                    "prop" => {
                        for meta in metas.iter() {
                            match *meta {
                                syn::NestedMetaItem::MetaItem(ref metaitem) => {
                                    match metaitem.name() {
                                        "get" => {
                                            descs.push(FnType::Getter(None));
                                        }
                                        "set" => {
                                            descs.push(FnType::Setter(None));
                                        }
                                        _ => {
                                            panic!("Only getter and setter supported");
                                        }
                                    }
                                }
                                _ => ()
                            }
                        }
                    }
                    _ => {
                        new_attrs.push(attr.clone());
                    }
                }
            }
            _ => {
                new_attrs.push(attr.clone());
            }
        }
    }
    item.attrs.clear();
    item.attrs.extend(new_attrs);
    descs
}

fn impl_class(cls: &syn::Ident, base: &syn::Ident,
              token: Option<syn::Ident>, doc: syn::Lit,
              params: HashMap<&'static str, syn::Ident>,
              flags: Vec<syn::Ident>,
              descriptors: Vec<(syn::Field, Vec<FnType>)>) -> Tokens {
    let cls_name = match params.get("name") {
        Some(name) => quote! { #name }.as_str().to_string(),
        None => quote! { #cls }.as_str().to_string()
    };

    let extra = if let Some(token) = token {
        Some(quote! {
            impl _pyo3::PyObjectWithToken for #cls {
                #[inline(always)]
                fn py<'p>(&'p self) -> _pyo3::Python<'p> {
                    self.#token.py()
                }
            }
            impl _pyo3::ToPyObject for #cls {
                #[inline]
                fn to_object<'p>(&self, py: _pyo3::Python<'p>) -> _pyo3::PyObject {
                    unsafe { _pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
                }
            }
            impl _pyo3::ToBorrowedObject for #cls {
                #[inline]
                fn with_borrowed_ptr<F, R>(&self, _py: _pyo3::Python, f: F) -> R
                    where F: FnOnce(*mut _pyo3::ffi::PyObject) -> R
                {
                    f(self.as_ptr())
                }
            }
            impl<'a> _pyo3::ToPyObject for &'a mut #cls {
                #[inline]
                fn to_object<'p>(&self, py: _pyo3::Python<'p>) -> _pyo3::PyObject {
                    unsafe { _pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
                }
            }
            impl<'a> _pyo3::ToBorrowedObject for &'a mut #cls {
                #[inline]
                fn with_borrowed_ptr<F, R>(&self, _py: _pyo3::Python, f: F) -> R
                    where F: FnOnce(*mut _pyo3::ffi::PyObject) -> R
                {
                    f(self.as_ptr())
                }
            }
            impl<'a> std::convert::From<&'a mut #cls> for &'a #cls
            {
                fn from(ob: &'a mut #cls) -> Self {
                    unsafe{std::mem::transmute(ob)}
                }
            }
            impl _pyo3::ToPyPointer for #cls {
                #[inline]
                fn as_ptr(&self) -> *mut _pyo3::ffi::PyObject {
                    unsafe {
                        {self as *const _ as *mut u8}
                        .offset(-<#cls as _pyo3::typeob::PyTypeInfo>::OFFSET) as *mut _pyo3::ffi::PyObject
                    }
                }
            }
            impl std::fmt::Debug for #cls {
                fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                    use pyo3::ObjectProtocol;
                    let s = try!(self.repr().map_err(|_| std::fmt::Error));
                    f.write_str(&s.to_string_lossy())
                }
            }
            impl std::fmt::Display for #cls {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                    use pyo3::ObjectProtocol;
                    let s = try!(self.str().map_err(|_| std::fmt::Error));
                    f.write_str(&s.to_string_lossy())
                }
            }
        })
    } else {
        None
    };

    let extra = {
        if let Some(freelist) = params.get("freelist") {
            Some(quote! {
                impl _pyo3::freelist::PyObjectWithFreeList for #cls {
                    #[inline]
                    fn get_free_list() -> &'static mut _pyo3::freelist::FreeList<*mut _pyo3::ffi::PyObject> {
                        static mut FREELIST: *mut _pyo3::freelist::FreeList<*mut _pyo3::ffi::PyObject> = 0 as *mut _;
                        unsafe {
                            if FREELIST.is_null() {
                                FREELIST = Box::into_raw(Box::new(
                                    _pyo3::freelist::FreeList::with_capacity(#freelist)));

                                <#cls as _pyo3::typeob::PyTypeObject>::init_type();
                            }
                            &mut *FREELIST
                        }
                    }
                }

                #extra
            })
        } else {
            extra
        }
    };

    let extra = if !descriptors.is_empty() {
        let ty = syn::parse::ty(cls.as_ref()).expect("no name");
        let desc_impls = impl_descriptors(&ty, descriptors);
        Some(quote! {
            #desc_impls
            #extra
        })
    } else {
        extra
    };

    // insert space for weak ref
    let mut has_weakref = false;
    let mut has_dict = false;
    for f in flags.iter() {
        if *f == syn::Ident::from("_pyo3::typeob::PY_TYPE_FLAG_WEAKREF") {
            has_weakref = true;
        } else if *f == syn::Ident::from("_pyo3::typeob::PY_TYPE_FLAG_DICT") {
            has_dict = true;
        }
    }
    let weakref = if has_weakref {
        syn::Ident::from("std::mem::size_of::<*const _pyo3::ffi::PyObject>()")
    } else {
        syn::Ident::from("0")
    };
    let dict = if has_dict {
        syn::Ident::from("std::mem::size_of::<*const _pyo3::ffi::PyObject>()")
    } else {
        syn::Ident::from("0")
    };

    quote! {
        impl _pyo3::typeob::PyTypeInfo for #cls {
            type Type = #cls;
            type BaseType = #base;

            const NAME: &'static str = #cls_name;
            const DESCRIPTION: &'static str = #doc;
            const FLAGS: usize = #(#flags)|*;

            const SIZE: usize = {
                Self::OFFSET as usize +
                    std::mem::size_of::<#cls>() + #weakref + #dict
            };
            const OFFSET: isize = {
                // round base_size up to next multiple of align
                (
                    (<#base as _pyo3::typeob::PyTypeInfo>::SIZE +
                     std::mem::align_of::<#cls>() - 1)  /
                        std::mem::align_of::<#cls>() * std::mem::align_of::<#cls>()
                ) as isize
            };

            #[inline]
            unsafe fn type_object() -> &'static mut _pyo3::ffi::PyTypeObject {
                static mut TYPE_OBJECT: _pyo3::ffi::PyTypeObject = _pyo3::ffi::PyTypeObject_INIT;
                &mut TYPE_OBJECT
            }
        }

        impl _pyo3::typeob::PyTypeObject for #cls {
            #[inline(always)]
            fn init_type() {
                static START: std::sync::Once = std::sync::ONCE_INIT;
                START.call_once(|| {
                    let ty = unsafe{<#cls as _pyo3::typeob::PyTypeInfo>::type_object()};

                    if (ty.tp_flags & _pyo3::ffi::Py_TPFLAGS_READY) == 0 {
                        let gil = _pyo3::Python::acquire_gil();
                        let py = gil.python();

                        // automatically initialize the class on-demand
                        _pyo3::typeob::initialize_type::<#cls>(py, None)
                            .map_err(|e| e.print(py))
                            .expect(format!("An error occurred while initializing class {}",
                                            <#cls as _pyo3::typeob::PyTypeInfo>::NAME).as_ref());
                    }
                });
            }
        }

        #extra
    }
}

fn impl_descriptors(cls: &syn::Ty, descriptors: Vec<(syn::Field, Vec<FnType>)>) -> Tokens {
    let methods: Vec<Tokens> = descriptors.iter().flat_map(|&(ref field, ref fns)| {
        fns.iter().map(|desc| {
            let name = field.ident.clone().unwrap();
            let field_ty = &field.ty;
            match *desc {
                FnType::Getter(_) => {
                    quote! {
                        impl #cls {
                            fn #name(&self) -> _pyo3::PyResult<#field_ty> {
                                Ok(self.#name)
                            }
                        }
                    }
                }
                FnType::Setter(_) => {
                    let setter_name = syn::Ident::from(format!("set_{}", name));
                    quote! {
                        impl #cls {
                            fn #setter_name(&mut self, value: #field_ty) -> _pyo3::PyResult<()> {
                                self.#name = value;
                                Ok(())
                            }
                        }
                    }
                },
                _ => unreachable!()
            }
        }).collect::<Vec<Tokens>>()
    }).collect();

    let py_methods: Vec<Tokens> = descriptors.iter().flat_map(|&(ref field, ref fns)| {
        fns.iter().map(|desc| {
            let name = field.ident.clone().unwrap();
            // FIXME better doc?
            let doc = syn::Lit::from(name.as_ref());
            let field_ty = &field.ty;
            match *desc {
                FnType::Getter(ref getter) => {
                    impl_py_getter_def(&name, doc, getter, &impl_wrap_getter(&Box::new(cls.clone()), &name))
                }
                FnType::Setter(ref setter) => {
                    let mode = syn::BindingMode::ByValue(syn::Mutability::Immutable);
                    let setter_name = syn::Ident::from(format!("set_{}", name));
                    let spec = FnSpec {
                        tp: FnType::Setter(None),
                        attrs: Vec::new(),
                        args: vec![FnArg {
                            name: &name,
                            mode: &mode,
                            ty: field_ty,
                            optional: None,
                            py: true,
                            reference: false
                        }],
                        output: syn::parse::ty("PyResult<()>").expect("error parse PyResult<()>")
                    };
                    impl_py_setter_def(&name, doc, setter, &impl_wrap_setter(&Box::new(cls.clone()), &setter_name, &spec))
                },
                _ => unreachable!()
            }
        }).collect::<Vec<Tokens>>()
    }).collect();

    let tokens = quote! {
        #(#methods)*

        impl _pyo3::class::methods::PyPropMethodsProtocolImpl for #cls {
            fn py_methods() -> &'static [_pyo3::class::PyMethodDefType] {
                static METHODS: &'static [_pyo3::class::PyMethodDefType] = &[
                    #(#py_methods),*
                ];
                METHODS
            }
        }
    };

    let n = match cls {
        &syn::Ty::Path(_, ref p) => {
            p.segments.last().as_ref().unwrap().ident.as_ref()
        }
        _ => "CLS_METHODS"
    };

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_DESCRIPTORS_{}", n));
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

fn parse_attribute(attr: String) -> (HashMap<&'static str, syn::Ident>,
                                     Vec<syn::Ident>, syn::Ident) {
    let mut params = HashMap::new();
    let mut flags = vec![syn::Ident::from("0")];
    let mut base = syn::Ident::from("_pyo3::PyObjectRef");

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
            let key = match elem[0] {
                syn::TokenTree::Token(syn::Token::Ident(ref ident)) => {
                    ident.as_ref().to_owned().to_lowercase()
                },
                _ => {
                    println!("Wrong format: {:?}", attr.to_string());
                    continue
                }
            };

            if elem.len() == 1 {
                match key.as_ref() {
                    "gc" => {
                        flags.push(syn::Ident::from("_pyo3::typeob::PY_TYPE_FLAG_GC"));
                        continue
                    }
                    "weakref" => {
                        flags.push(syn::Ident::from("_pyo3::typeob::PY_TYPE_FLAG_WEAKREF"));
                        continue
                    }
                    "subclass" => {
                        flags.push(syn::Ident::from("_pyo3::typeob::PY_TYPE_FLAG_BASETYPE"));
                        continue
                    }
                    "dict" => {
                        flags.push(syn::Ident::from("_pyo3::typeob::PY_TYPE_FLAG_DICT"));
                        continue
                    }
                    _ => {
                        println!("Unsupported parameter: {:?}", key);
                    }
                }
            }

            if elem.len() < 3 {
                println!("Wrong format: {:?}", elem);
                continue
            }

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
                    let mut m = String::new();
                    for el in elem[2..elem.len()].iter() {
                        let mut t = Tokens::new();
                        el.to_tokens(&mut t);
                        m += t.as_str().trim();
                    }
                    base = syn::Ident::from(m.as_str());
                },
                _ => {
                    println!("Unsupported parameter: {:?}", key);
                }
            }
        }
    }

    (params, flags, base)
}
