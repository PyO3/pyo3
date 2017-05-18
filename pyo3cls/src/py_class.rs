// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};


pub fn build_py_class(ast: &mut syn::DeriveInput) -> Tokens {
    let base = syn::Ident::from("_pyo3::PyObject");

    let mut tokens = Tokens::new();

    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref mut data)) => {
            impl_storage(&ast.ident, &base, data).to_tokens(&mut tokens);

            let tt = quote! {
                struct Test {
                    _unsafe_inner: PyObject
                }
            };
            let t = syn::parse_item(tt.as_str()).unwrap();
            match t.node {
                syn::ItemKind::Struct(syn::VariantData::Struct(fields), _) => {
                    data.clear();
                    data.extend(fields);
                }
                _ => panic!("something is worng"),
            }
        },
        _ =>
            panic!("#[class] can only be used with notmal structs"),
    }

    impl_to_py_object(&ast.ident).to_tokens(&mut tokens);
    impl_from_py_object(&ast.ident).to_tokens(&mut tokens);
    impl_python_object(&ast.ident).to_tokens(&mut tokens);
    impl_checked_downcast(&ast.ident).to_tokens(&mut tokens);

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_CLS_{}", ast.ident));
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

fn impl_storage(cls: &syn::Ident, base: &syn::Ident, fields: &Vec<syn::Field>) -> Tokens {
    let names: Vec<syn::Ident> = fields.iter()
        .map(|f| f.ident.as_ref().unwrap().clone()).collect();
    let values: Vec<syn::Ident> = fields.iter()
        .map(|f| f.ident.as_ref().unwrap().clone()).collect();
    //let types: Vec<syn::Ty> = fields.iter().map(|f| f.ty.clone()).collect();

    let storage = syn::Ident::from(format!("{}_Storage", cls).as_str());

    let mut accessors = Tokens::new();
    for field in fields.iter() {
        let name = &field.ident.as_ref().unwrap();
        let name_mut = syn::Ident::from(format!("{}_mut", name.as_ref()));
        let ty = &field.ty;

        let accessor = quote!{
            impl #cls {
                fn #name<'a>(&'a self, py: _pyo3::Python<'a>) -> &'a #ty {
                    unsafe {
                        let ptr = (self._unsafe_inner.as_ptr() as *const u8)
                            .offset(base_offset() as isize) as *const #storage;
                        &(*ptr).#name
                    }
                }
                fn #name_mut<'a>(&'a self, py: _pyo3::Python<'a>) -> &'a mut #ty {
                    unsafe {
                        let ptr = (self._unsafe_inner.as_ptr() as *const u8)
                            .offset(base_offset() as isize) as *mut #storage;
                        &mut (*ptr).#name
                    }
                }
            }
        };
        accessor.to_tokens(&mut accessors);
    }

    let cls_name = quote! { #cls }.as_str().to_string();

    quote! {
        pub struct #storage {
            #(#fields),*
        }

        impl #cls {
            fn create_instance(py: _pyo3::Python, #(#fields),*) -> _pyo3::PyResult<#cls> {
                let obj = try!(unsafe {
                    <#cls as _pyo3::class::BaseObject>::alloc(
                        py, &py.get_type::<#cls>(),
                        #storage { #(#names: #values),*})});

                return Ok(#cls { _unsafe_inner: obj });
            }
        }

        #accessors

        impl _pyo3::PythonObjectWithTypeObject for #cls {
            #[inline]
            fn type_name() -> &'static str { #cls_name }

            #[inline]
            fn type_object(py: _pyo3::Python) -> _pyo3::PyType {
                unsafe { <#cls as _pyo3::class::PyTypeObject>::initialized(py, None) }
            }
        }

        impl _pyo3::class::PyTypeObject for #cls {

            fn add_to_module(py: _pyo3::Python, module: &_pyo3::PyModule) -> _pyo3::PyResult<()> {
                let ty = unsafe { #cls::initialized(py, module.name(py).ok()) };
                module.add(py, stringify!(#cls), ty)
            }

            #[inline]
            unsafe fn type_obj() -> &'static mut _pyo3::ffi::PyTypeObject {
                static mut TYPE_OBJECT: _pyo3::ffi::PyTypeObject = _pyo3::ffi::PyTypeObject_INIT;
                &mut TYPE_OBJECT
            }

            unsafe fn initialized(py: _pyo3::Python, module_name: Option<&str>) -> _pyo3::PyType {
                let mut ty = #cls::type_obj();

                if (ty.tp_flags & _pyo3::ffi::Py_TPFLAGS_READY) != 0 {
                    _pyo3::PyType::from_type_ptr(py, ty)
                } else {
                    // automatically initialize the class on-demand
                    _pyo3::class::typeob::initialize_type::<#cls>(
                        py, module_name, #cls_name, ty).expect(
                        concat!("An error occurred while initializing class ",
                                stringify!(#cls)));
                    _pyo3::PyType::from_type_ptr(py, ty)
                }
            }
        }

        #[inline]
        fn base_offset() -> usize {
            let align = std::mem::align_of::<#storage>();
            let bs = <#base as _pyo3::class::BaseObject>::size();

            // round base_size up to next multiple of align
            (bs + align - 1) / align * align
        }

        impl _pyo3::class::BaseObject for #cls {
            type Type = #storage;

            #[inline]
            fn size() -> usize {
                base_offset() + std::mem::size_of::<Self::Type>()
            }

            unsafe fn alloc(py: _pyo3::Python, ty: &_pyo3::PyType,
                            value: Self::Type) -> _pyo3::PyResult<_pyo3::PyObject>
            {
                let obj = try!(<#base as _pyo3::class::BaseObject>::alloc(py, ty, ()));

                let ptr = (obj.as_ptr() as *mut u8)
                    .offset(base_offset() as isize) as *mut Self::Type;
                std::ptr::write(ptr, value);

                Ok(obj)
            }

            unsafe fn dealloc(py: _pyo3::Python, obj: *mut _pyo3::ffi::PyObject) {
                let ptr = (obj as *mut u8).offset(base_offset() as isize) as *mut Self::Type;
                std::ptr::drop_in_place(ptr);

                <#base as _pyo3::class::BaseObject>::dealloc(py, obj)
            }
        }
    }
}

fn impl_to_py_object(cls: &syn::Ident) -> Tokens {
    quote! {
        /// Identity conversion: allows using existing `PyObject` instances where
        /// `T: ToPyObject` is expected.
        impl _pyo3::ToPyObject for #cls where #cls: _pyo3::PythonObject {
            #[inline]
            fn to_py_object(&self, py: _pyo3::Python) -> _pyo3::PyObject {
                _pyo3::PyClone::clone_ref(self, py).into_object()
            }

            #[inline]
            fn into_py_object(self, _py: _pyo3::Python) -> _pyo3::PyObject {
                self.into_object()
            }

            #[inline]
            fn with_borrowed_ptr<F, R>(&self, _py: _pyo3::Python, f: F) -> R
                where F: FnOnce(*mut _pyo3::ffi::PyObject) -> R
            {
                f(_pyo3::PythonObject::as_object(self).as_ptr())
            }
        }
    }
}

fn impl_from_py_object(cls: &syn::Ident) -> Tokens {
    quote! {
        impl <'source> _pyo3::FromPyObject<'source> for #cls {
            #[inline]
            fn extract(py: _pyo3::Python, obj: &'source _pyo3::PyObject)
                       -> _pyo3::PyResult<#cls> {
                Ok(obj.clone_ref(py).cast_into::<#cls>(py)?)
            }
        }

        impl <'source> _pyo3::FromPyObject<'source> for &'source #cls {
            #[inline]
            fn extract(py: _pyo3::Python, obj: &'source _pyo3::PyObject)
                       -> _pyo3::PyResult<&'source #cls> {
                Ok(obj.cast_as::<#cls>(py)?)
            }
        }
    }
}

fn impl_python_object(cls: &syn::Ident) -> Tokens {
    quote! {
        impl _pyo3::PythonObject for #cls {
            #[inline]
            fn as_object(&self) -> &_pyo3::PyObject {
                &self._unsafe_inner
            }

            #[inline]
            fn into_object(self) -> _pyo3::PyObject {
                self._unsafe_inner
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_from(obj: _pyo3::PyObject) -> Self {
                #cls { _unsafe_inner: obj }
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a _pyo3::PyObject) -> &'a Self {
                std::mem::transmute(obj)
            }
        }
    }
}

fn impl_checked_downcast(cls: &syn::Ident) -> Tokens {
    quote! {
        impl _pyo3::PythonObjectWithCheckedDowncast for #cls {
            #[inline]
            fn downcast_from<'p>(py: _pyo3::Python<'p>, obj: _pyo3::PyObject)
                                 -> Result<#cls, _pyo3::PythonObjectDowncastError<'p>> {
                if py.get_type::<#cls>().is_instance(py, &obj) {
                    Ok(#cls { _unsafe_inner: obj })
                } else {
                    Err(_pyo3::PythonObjectDowncastError(py))
                }
            }

            #[inline]
            fn downcast_borrow_from<'a, 'p>(py: _pyo3::Python<'p>, obj: &'a _pyo3::PyObject)
                                            -> Result<&'a #cls, _pyo3::PythonObjectDowncastError<'p>> {
                if py.get_type::<#cls>().is_instance(py, obj) {
                    unsafe { Ok(std::mem::transmute(obj)) }
                } else {
                    Err(_pyo3::PythonObjectDowncastError(py))
                }
            }
        }
    }
}
