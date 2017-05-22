// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};


pub fn build_py_class(ast: &mut syn::DeriveInput) -> Tokens {
    let base = syn::Ident::from("_pyo3::PyObject");

    let mut tokens = Tokens::new();

    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(_)) => {
            impl_storage(&ast.ident, &base).to_tokens(&mut tokens);
        },
        _ =>
            panic!("#[class] can only be used with notmal structs"),
    }

    impl_to_py_object(&ast.ident).to_tokens(&mut tokens);
    impl_from_py_object(&ast.ident).to_tokens(&mut tokens);
    impl_python_object(&ast.ident).to_tokens(&mut tokens);

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

fn impl_storage(cls: &syn::Ident, base: &syn::Ident) -> Tokens {
    let cls_name = quote! { #cls }.as_str().to_string();

    quote! {
        impl _pyo3::class::typeob::PyTypeObjectInfo for #cls {
            #[inline]
            fn size() -> usize {
                Self::offset() + std::mem::size_of::<#cls>()
            }

            #[inline]
            fn offset() -> usize {
                let align = std::mem::align_of::<#cls>();
                let bs = <#base as _pyo3::class::BaseObject>::size();

                // round base_size up to next multiple of align
                (bs + align - 1) / align * align
            }

            #[inline]
            fn type_name() -> &'static str { #cls_name }

            #[inline]
            fn type_object() -> &'static mut _pyo3::ffi::PyTypeObject {
                static mut TYPE_OBJECT: _pyo3::ffi::PyTypeObject = _pyo3::ffi::PyTypeObject_INIT;
                unsafe { &mut TYPE_OBJECT }
            }
        }

        #[inline]
        fn offset() -> usize {
            let align = std::mem::align_of::<#cls>();
            let bs = <#base as _pyo3::class::BaseObject>::size();

            // round base_size up to next multiple of align
            (bs + align - 1) / align * align
        }

        impl _pyo3::class::BaseObject for #cls {
            type Type = #cls;

            #[inline]
            fn size() -> usize {
                offset() + std::mem::size_of::<Self::Type>()
            }

            unsafe fn alloc(py: _pyo3::Python,
                            value: Self::Type) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject>
            {
                let ty = py.get_type::<Self::Type>();
                let obj = ffi::PyType_GenericAlloc(ty.as_type_ptr(), 0);

                if obj.is_null() {
                    return Err(PyErr::fetch(py))
                }

                let ptr = (obj as *mut u8).offset(offset() as isize) as *mut Self::Type;
                std::ptr::write(ptr, value);

                Ok(obj)
            }

            unsafe fn dealloc(py: _pyo3::Python, obj: *mut _pyo3::ffi::PyObject) {
                let ptr = (obj as *mut u8).offset(offset() as isize) as *mut Self::Type;
                std::ptr::drop_in_place(ptr);

                // Unfortunately, there is no PyType_GenericFree, so
                // we have to manually un-do the work of PyType_GenericAlloc:
                let ty = _pyo3::ffi::Py_TYPE(obj);
                if _pyo3::ffi::PyType_IS_GC(ty) != 0 {
                    _pyo3::ffi::PyObject_GC_Del(obj as *mut c_void);
                } else {
                    _pyo3::ffi::PyObject_Free(obj as *mut c_void);
                }
                // For heap types, PyType_GenericAlloc calls INCREF on the type objects,
                // so we need to call DECREF here:
                if _pyo3::ffi::PyType_HasFeature(ty, _pyo3::ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
                    _pyo3::ffi::Py_DECREF(ty as *mut _pyo3::ffi::PyObject);
                }
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
        impl <'source> _pyo3::FromPyObj<'source> for &'source #cls {
            #[inline]
            fn extr<S>(py: &'source _pyo3::Py<'source, S>) -> _pyo3::PyResult<&'source #cls>
                where S: _pyo3::class::typeob::PyTypeObjectInfo
            {
                Ok(py.cast_as::<#cls>()?)
            }
        }
    }
}

fn impl_python_object(cls: &syn::Ident) -> Tokens {
    quote! {
        impl _pyo3::PythonObject for #cls {
            #[inline]
            fn as_object(&self) -> &_pyo3::PyObject {
                unimplemented!();
                /*unsafe {
                    let py = Python::assume_gil_acquired();

                    let ty = cls::type_object(py);
                    let align = std::mem::align_of::<T>();
                    let bs = <T as BaseObject>::size();

                    // round base_size up to next multiple of align
                    let offset = (bs + align - 1) / align * align;

                    let ptr = (self as *mut u8).offset(-1(offset as isize)) as *mut ffi::PyObject;

                    Ok(PyObject::from_owned_ptr(py, ptr))
                }*/
            }

            #[inline]
            fn into_object(self) -> _pyo3::PyObject {
                unsafe {
                    let py = Python::assume_gil_acquired();

                    let ty = #cls::type_object(py);
                    let align = std::mem::align_of::<#cls>();
                    let bs = <#cls as BaseObject>::size();

                    // round base_size up to next multiple of align
                    let offset = (bs + align - 1) / align * align;

                    let ptr = (&self as *const _ as *mut u8).offset(
                        -(offset as isize)) as *mut ffi::PyObject;

                    PyObject::from_borrowed_ptr(py, ptr)
                }
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_from(obj: _pyo3::PyObject) -> Self {
                unimplemented!();
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_borrow_from<'b>(obj: &'b _pyo3::PyObject) -> &'b Self {
                unimplemented!();
            }
        }
    }
}
