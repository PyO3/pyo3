// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};


pub fn build_py_class(ast: &mut syn::DeriveInput) -> Tokens {
    let base = syn::Ident::from("pyo3::PyObject");

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
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate pyo3;
            use std;
            use pyo3::class::BaseObject;
            use pyo3::{ffi, Python, PyObject, PyType, PyResult, PyModule};

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

    let mut accessors = Tokens::new();
    for field in fields.iter() {
        let name = &field.ident.as_ref().unwrap();
        let name_mut = syn::Ident::from(format!("{}_mut", name.as_ref()));
        let ty = &field.ty;

        let accessor = quote!{
            impl #cls {
                fn #name<'a>(&'a self, py: Python<'a>) -> &'a #ty {
                    unsafe {
                        let ptr = (self._unsafe_inner.as_ptr() as *const u8)
                            .offset(base_offset() as isize) as *const Storage;
                        &(*ptr).#name
                    }
                }
                fn #name_mut<'a>(&'a self, py: Python<'a>) -> &'a mut #ty {
                    unsafe {
                        let ptr = (self._unsafe_inner.as_ptr() as *const u8)
                            .offset(base_offset() as isize) as *mut Storage;
                        &mut (*ptr).#name
                    }
                }
            }
        };
        accessor.to_tokens(&mut accessors);
    }

    quote! {
        struct Storage {
            #(#fields),*
        }

        impl #cls {
            fn create_instance(py: Python, #(#fields),*) -> PyResult<#cls> {
                let obj = try!(unsafe {
                    <#cls as BaseObject>::alloc(
                        py, &py.get_type::<#cls>(),
                        Storage { #(#names: #values),*})});

                return Ok(#cls { _unsafe_inner: obj });
            }
        }

        #accessors

        impl pyo3::PythonObjectWithTypeObject for #cls {
            #[inline]
            fn type_object(py: Python) -> PyType {
                unsafe { #cls::initialized(py, None) }
            }
        }

        impl pyo3::class::PyTypeObject for #cls {

            fn add_to_module(py: Python, module: &PyModule) -> PyResult<()> {
                let ty = unsafe { #cls::initialized(py, module.name(py).ok()) };
                module.add(py, stringify!(#cls), ty)
            }

            #[inline]
            unsafe fn type_obj() -> &'static mut ffi::PyTypeObject {
                static mut TYPE_OBJECT: ffi::PyTypeObject = ffi::PyTypeObject_INIT;
                &mut TYPE_OBJECT
            }

            unsafe fn initialized(py: Python, module_name: Option<&str>) -> PyType {
                let mut ty = #cls::type_obj();

                if (ty.tp_flags & ffi::Py_TPFLAGS_READY) != 0 {
                    PyType::from_type_ptr(py, ty)
                } else {
                    // automatically initialize the class on-demand
                    pyo3::class::typeob::initialize_type::<#cls>(
                        py, module_name, ty).expect(
                        concat!("An error occurred while initializing class ",
                                stringify!(#cls)));
                    PyType::from_type_ptr(py, ty)
                }
            }
        }

        #[inline]
        fn base_offset() -> usize {
            let align = std::mem::align_of::<Storage>();
            let bs = <#base as BaseObject>::size();

            // round base_size up to next multiple of align
            (bs + align - 1) / align * align
        }

        impl BaseObject for #cls {
            type Type = Storage;

            #[inline]
            fn size() -> usize {
                base_offset() + std::mem::size_of::<Self::Type>()
            }

            unsafe fn alloc(py: Python, ty: &PyType, value: Self::Type) -> PyResult<PyObject>
            {
                let obj = try!(<#base as BaseObject>::alloc(py, ty, ()));

                let ptr = (obj.as_ptr() as *mut u8)
                    .offset(base_offset() as isize) as *mut Self::Type;
                std::ptr::write(ptr, value);

                Ok(obj)
            }

            unsafe fn dealloc(py: Python, obj: *mut ffi::PyObject) {
                let ptr = (obj as *mut u8).offset(base_offset() as isize) as *mut Self::Type;
                std::ptr::drop_in_place(ptr);

                <#base as BaseObject>::dealloc(py, obj)
            }
        }
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
