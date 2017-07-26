// Copyright (c) 2017-present PyO3 Project and Contributors

use std;
use std::collections::HashMap;

use syn;
use quote::Tokens;

use utils;


pub fn build_py_class(ast: &mut syn::DeriveInput, attr: String) -> Tokens {
    let (params, flags) = parse_attribute(attr);
    let doc = utils::get_doc(&ast.attrs, true);

    let base = syn::Ident::from("_pyo3::PyObjectRef");
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
        _ => panic!("#[class] can only be used with normal structs"),
    }

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_CLS_{}", ast.ident));
    let tokens = impl_class(&ast.ident, &base, token, doc, params, flags);

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

fn impl_class(cls: &syn::Ident, base: &syn::Ident,
              token: Option<syn::Ident>, doc: syn::Lit,
              params: HashMap<&'static str, syn::Ident>, flags: Vec<syn::Ident>) -> Tokens {
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
                #[inline]
                fn with_borrowed_ptr<F, R>(&self, _py: _pyo3::Python, f: F) -> R
                    where F: FnOnce(*mut ffi::PyObject) -> R
                {
                    f(self.as_ptr())
                }
            }
            impl<'a> _pyo3::ToPyObject for &'a mut #cls {
                #[inline]
                fn to_object<'p>(&self, py: _pyo3::Python<'p>) -> _pyo3::PyObject {
                    unsafe { _pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
                }
                #[inline]
                fn with_borrowed_ptr<F, R>(&self, _py: _pyo3::Python, f: F) -> R
                    where F: FnOnce(*mut ffi::PyObject) -> R
                {
                    f(self.as_ptr())
                }
            }
            impl<'a> _pyo3::IntoPyObject for &'a #cls
            {
                #[inline]
                fn into_object<'p>(self, py: _pyo3::Python) -> _pyo3::PyObject {
                    unsafe { _pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
                }
            }
            impl<'a> _pyo3::IntoPyObject for &'a mut #cls
            {
                #[inline]
                fn into_object<'p>(self, py: _pyo3::Python) -> _pyo3::PyObject {
                    unsafe { _pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr()) }
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
                fn as_ptr(&self) -> *mut ffi::PyObject {
                    unsafe {
                        {self as *const _ as *mut u8}
                        .offset(-<#cls as _pyo3::typeob::PyTypeInfo>::OFFSET) as *mut ffi::PyObject
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

    // insert space for weak ref
    let mut has_weakref = false;
    for f in flags.iter() {
        if *f == syn::Ident::from("_pyo3::typeob::PY_TYPE_FLAG_WEAKREF") {
            has_weakref = true;
        }
    }
    let weakref = if has_weakref {
        syn::Ident::from("std::mem::size_of::<*const _pyo3::ffi::PyObject>()")
    } else {
        syn::Ident::from("0")
    };

    quote! {
        impl _pyo3::typeob::PyTypeInfo for #cls {
            type Type = #cls;
            const NAME: &'static str = #cls_name;
            const DESCRIPTION: &'static str = #doc;

            const SIZE: usize = Self::OFFSET as usize + std::mem::size_of::<#cls>() + #weakref;
            const OFFSET: isize = {
                // round base_size up to next multiple of align
                ((<#base as _pyo3::typeob::PyTypeInfo>::SIZE + std::mem::align_of::<#cls>()-1) /
                 std::mem::align_of::<#cls>() * std::mem::align_of::<#cls>()) as isize
            };

            const FLAGS: usize = #(#flags)|*;

            #[inline]
            unsafe fn type_object() -> &'static mut _pyo3::ffi::PyTypeObject {
                static mut TYPE_OBJECT: _pyo3::ffi::PyTypeObject = _pyo3::ffi::PyTypeObject_INIT;
                &mut TYPE_OBJECT
            }

            #[inline]
            fn is_instance(ptr: *mut _pyo3::ffi::PyObject) -> bool {
                unsafe {_pyo3::ffi::PyObject_TypeCheck(
                    ptr, <#cls as _pyo3::typeob::PyTypeInfo>::type_object()) != 0}
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
                        _pyo3::typeob::initialize_type::<#cls>(py, None).expect(
                            format!("An error occurred while initializing class {}",
                                    <#cls as _pyo3::typeob::PyTypeInfo>::NAME).as_ref());
                    }
                });
            }
        }

        impl _pyo3::PyDowncastFrom for #cls
        {
            fn try_downcast_from(ob: &_pyo3::PyObjectRef) -> Option<&#cls>
            {
                unsafe {
                    let ptr = ob.as_ptr();
                    let checked = ffi::PyObject_TypeCheck(
                        ptr, <#cls as _pyo3::typeob::PyTypeInfo>::type_object()) != 0;

                    if checked {
                        let ptr = (ptr as *mut u8)
                            .offset(<#cls as _pyo3::typeob::PyTypeInfo>::OFFSET) as *mut #cls;
                        Some(ptr.as_ref().unwrap())
                    } else {
                        None
                    }
                }
            }

            fn try_exact_downcast_from(ob: &_pyo3::PyObjectRef) -> Option<&#cls>
            {
                unsafe {
                    let ptr = ob.as_ptr();
                    if (*ptr).ob_type == <#cls as _pyo3::typeob::PyTypeInfo>::type_object()
                    {
                        let ptr = (ptr as *mut u8)
                            .offset(<#cls as _pyo3::typeob::PyTypeInfo>::OFFSET) as *mut #cls;
                        Some(ptr.as_ref().unwrap())
                    } else {
                        None
                    }
                }
            }

            #[inline]
            unsafe fn unchecked_downcast_from(ob: &_pyo3::PyObjectRef) -> &Self
            {
                let ptr = (ob.as_ptr() as *mut u8)
                    .offset(<#cls as _pyo3::typeob::PyTypeInfo>::OFFSET) as *mut #cls;
                &*ptr
            }
            #[inline]
            unsafe fn unchecked_mut_downcast_from(ob: &_pyo3::PyObjectRef) -> &mut Self
            {
                let ptr = (ob.as_ptr() as *mut u8)
                    .offset(<#cls as _pyo3::typeob::PyTypeInfo>::OFFSET) as *mut #cls;
                &mut *ptr
            }
        }
        impl _pyo3::PyMutDowncastFrom for #cls
        {
            fn try_mut_downcast_from(ob: &mut _pyo3::PyObjectRef) -> Option<&mut #cls>
            {
                unsafe {
                    let ptr = ob.as_ptr();
                    let checked = ffi::PyObject_TypeCheck(
                        ptr, <#cls as _pyo3::typeob::PyTypeInfo>::type_object()) != 0;

                    if checked {
                        let ptr = (ptr as *mut u8)
                            .offset(<#cls as _pyo3::typeob::PyTypeInfo>::OFFSET) as *mut #cls;
                        Some(ptr.as_mut().unwrap())
                    } else {
                        None
                    }
                }
            }
            fn try_mut_exact_downcast_from(ob: &mut _pyo3::PyObjectRef) -> Option<&mut #cls>
            {
                unsafe {
                    let ptr = ob.as_ptr();
                    if (*ptr).ob_type == <#cls as _pyo3::typeob::PyTypeInfo>::type_object()
                    {
                        let ptr = (ptr as *mut u8)
                            .offset(<#cls as _pyo3::typeob::PyTypeInfo>::OFFSET) as *mut #cls;
                        Some(ptr.as_mut().unwrap())
                    } else {
                        None
                    }
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

fn parse_attribute(attr: String) -> (HashMap<&'static str, syn::Ident>, Vec<syn::Ident>) {
    let mut params = HashMap::new();
    let mut flags = vec![syn::Ident::from("0")];

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

                },
                _ => {
                    println!("Unsupported parameter: {:?}", key);
                }
            }
        }
    }

    (params, flags)
}
