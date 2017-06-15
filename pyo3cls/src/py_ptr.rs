// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;

pub fn build_ptr(cls: syn::Ident, ast: &mut syn::DeriveInput) -> Tokens {
    let ptr = &ast.ident;
    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_CLS_PTR_{}", ast.ident));

    quote! {
        #[feature(specialization)]
        #[allow(non_upper_case_globals, unused_attributes,
                unused_qualifications, unused_variables, non_camel_case_types)]
        const #dummy_const: () = {
            use std;
            extern crate pyo3 as _pyo3;

            // thread-safe, because any python related operations require a Python<'p> token.
            unsafe impl Send for #ptr {}
            unsafe impl Sync for #ptr {}

            impl _pyo3::ToInstancePtr<#cls> for #cls {
                type Target = #ptr;

                fn to_inst_ptr(&self) -> #ptr {
                    #ptr(unsafe{_pyo3::PyPtr::from_borrowed_ptr(self.as_ptr())})
                }
                unsafe fn from_owned_ptr(ptr: *mut _pyo3::ffi::PyObject) -> #ptr {
                    #ptr(_pyo3::PyPtr::from_owned_ptr(ptr))
                }
                unsafe fn from_borrowed_ptr(ptr: *mut _pyo3::ffi::PyObject) -> #ptr {
                    #ptr(_pyo3::PyPtr::from_borrowed_ptr(ptr))
                }
            }

            impl _pyo3::InstancePtr<#cls> for #ptr {

                #[inline]
                fn as_ref(&self, _py: Python) -> &#cls {
                    let offset = <#cls as _pyo3::typeob::PyTypeInfo>::offset();
                    unsafe {
                        let ptr = (self.as_ptr() as *mut u8).offset(offset) as *mut #cls;
                        ptr.as_ref().unwrap()
                    }
                }
                #[inline]
                fn as_mut(&self, _py: Python) -> &mut #cls {
                    let offset = <#cls as _pyo3::typeob::PyTypeInfo>::offset();
                    unsafe {
                        let ptr = (self.as_ptr() as *mut u8).offset(offset) as *mut #cls;
                        ptr.as_mut().unwrap()
                    }
                }
            }

            impl std::ops::Deref for #ptr {
                type Target = _pyo3::PyPtr;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
            impl std::ops::DerefMut for #ptr {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
            impl _pyo3::PyClone for #ptr {
                fn clone_ref(&self, _py: _pyo3::Python) -> #ptr {
                    #ptr(unsafe{ _pyo3::PyPtr::from_borrowed_ptr(self.as_ptr()) })
                }
            }
            impl _pyo3::ToPyObject for #ptr {
                fn to_object(&self, py: Python) -> _pyo3::PyObject {
                    _pyo3::PyObject::from_borrowed_ptr(py, self.as_ptr())
                }
            }
            impl _pyo3::IntoPyObject for #ptr {
                fn into_object(self, _py: Python) -> _pyo3::PyObject {
                    unsafe {std::mem::transmute(self)}
                }
            }
            impl _pyo3::IntoPyPointer for #ptr {
                /// Gets the underlying FFI pointer, returns a owned pointer.
                #[inline]
                #[must_use]
                fn into_ptr(self) -> *mut ffi::PyObject {
                    self.0.into_ptr()
                }
            }
            impl _pyo3::PyDowncastInto for #ptr
            {
                fn downcast_into<'p, I>(py: _pyo3::Python<'p>, ob: I)
                                        -> Result<Self, _pyo3::PyDowncastError<'p>>
                    where I: _pyo3::IntoPyPointer
                {
                    <#ptr as _pyo3::PyDowncastInto>::downcast_from_ptr(py, ob.into_ptr())
                }

                fn downcast_from_ptr<'p>(py: _pyo3::Python<'p>, ptr: *mut _pyo3::ffi::PyObject)
                                         -> Result<#ptr, _pyo3::PyDowncastError<'p>>
                {
                    unsafe{
                        let checked = ffi::PyObject_TypeCheck(
                            ptr, <#cls as _pyo3::typeob::PyTypeInfo>::type_object()) != 0;

                        if checked {
                            Ok(#ptr(PyPtr::from_owned_ptr(ptr)))
                        } else {
                            _pyo3::ffi::Py_DECREF(ptr);
                            Err(_pyo3::PyDowncastError(py, None))
                        }
                    }
                }

                fn unchecked_downcast_into<'p, I>(ob: I) -> Self where I: _pyo3::IntoPyPointer
                {
                    unsafe{
                        #ptr(_pyo3::PyPtr::from_owned_ptr(ob.into_ptr()))
                    }
                }
            }

            impl std::convert::From<#ptr> for _pyo3::PyObject {
                fn from(ob: #ptr) -> Self {
                    unsafe{std::mem::transmute(ob)}
                }
            }
        };
    }
}
