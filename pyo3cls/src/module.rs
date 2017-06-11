// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;


pub fn build_py3_module_init(ast: &mut syn::Item, attr: String) -> Tokens {
    let modname = &attr.to_string()[1..attr.to_string().len()-1].to_string();
    let name = syn::Ident::from(modname.as_ref());

    println!("MOD: {:?}", modname);

    let tokens = match ast.node {
        syn::ItemKind::Fn(_, _, _, _, _, _) => {
            py3_init(&ast.ident, &name);
        },
        _ => panic!("#[modinit] can only be used with fn block"),
    };

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_MODINIT_{}", modname.trim()));

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

pub fn py3_init(fnname: &syn::Ident, name: &syn::Ident) -> Tokens {
    quote! {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn PyInit_#name() -> *mut _pyo3::ffi::PyObject {
            static mut MODULE_DEF: $crate::ffi::PyModuleDef = $crate::ffi::PyModuleDef_INIT;
            // We can't convert &'static str to *const c_char within a static initializer,
            // so we'll do it here in the module initialization:
            MODULE_DEF.m_name = concat!(stringify!($name), "\0").as_ptr() as *const _;

            let guard = _pyo3::callback::AbortOnDrop("py_module_init");
            let py = _pyo3::Python::assume_gil_acquired();
            _pyo3::ffi::PyEval_InitThreads();
            let module = _pyo3::ffi::PyModule_Create(def);
            if module.is_null() {
                mem::forget(guard);
                return module;
            }

            let module = match _pyo3::PyObject::from_owned_ptr(
                py, module).cast_into::<PyModule>(py)
            {
                Ok(m) => m,
                Err(e) => {
                    _pyo3::PyErr::from(e).restore(py);
                    mem::forget(guard);
                    return ptr::null_mut();
                }
            };
            let ret = match #fnname(py, &module) {
                Ok(()) => module.into_ptr(),
                Err(e) => {
                    e.restore(py);
                    ptr::null_mut()
                }
            };
            mem::forget(guard);
            ret
        }
    }
}

pub fn build_py2_module_init(ast: &mut syn::Item, attr: String) -> Tokens {
    let modname = &attr.to_string()[1..attr.to_string().len()-1].to_string();
    let name = syn::Ident::from(modname.as_ref());

    let tokens = match ast.node {
        syn::ItemKind::Fn(_, _, _, _, _, _) => {
            py2_init(&ast.ident, &name);
        },
        _ => panic!("#[modinit] can only be used with fn block"),
    };

    let dummy_const = syn::Ident::new(format!("_IMPL_PYO3_MODINIT_{}", modname.trim()));

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

pub fn py2_init(fnname: &syn::Ident, name: &syn::Ident) -> Tokens {
    quote! {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn init#name() -> *mut _pyo3::ffi::PyObject {
            use pyo3::ffi;

            let name = concat!(stringify!($name), "\0").as_ptr() as *const _;
            let guard = function::AbortOnDrop("py_module_initializer");
            let py = Python::assume_gil_acquired();
            ffi::PyEval_InitThreads();
            let module = ffi::Py_InitModule(name, ptr::null_mut());
            if module.is_null() {
                mem::forget(guard);
                return;
            }

            let module = match PyObject::from_borrowed_ptr(py, module).cast_into::<PyModule>(py) {
                Ok(m) => m,
                Err(e) => {
                    _pyo3::PyErr::from(e).restore(py);
                    mem::forget(guard);
                    return;
                }
            };
            let ret = match #fnname(py, &module) {
                Ok(()) => (),
                Err(e) => e.restore(py)
            };
            mem::forget(guard);
            ret
        }
    }
}
