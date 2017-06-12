// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;


pub fn build_py3_module_init(ast: &mut syn::Item, attr: String) -> Tokens {
    let modname = &attr.to_string()[1..attr.to_string().len()-1].to_string();

    match ast.node {
        syn::ItemKind::Fn(_, _, _, _, _, _) => {
            py3_init(&ast.ident, &modname)
        },
        _ => panic!("#[modinit] can only be used with fn block"),
    }
}

pub fn py3_init(fnname: &syn::Ident, name: &String) -> Tokens {
    let cb_name = syn::Ident::from(format!("PyInit_{}", name.trim()).as_ref());
    quote! {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #cb_name() -> *mut ::pyo3::ffi::PyObject {
            use std;
            extern crate pyo3 as _pyo3;

            static mut MODULE_DEF: _pyo3::ffi::PyModuleDef = _pyo3::ffi::PyModuleDef_INIT;
            // We can't convert &'static str to *const c_char within a static initializer,
            // so we'll do it here in the module initialization:
            MODULE_DEF.m_name = concat!(stringify!(#cb_name), "\0").as_ptr() as *const _;

            let guard = _pyo3::callback::AbortOnDrop("py_module_init");
            let py = _pyo3::Python::assume_gil_acquired();
            _pyo3::ffi::PyEval_InitThreads();
            let module = _pyo3::ffi::PyModule_Create(&mut MODULE_DEF);
            if module.is_null() {
                std::mem::forget(guard);
                return module;
            }

            let module = match _pyo3::PyObject::from_owned_ptr(
                py, module).cast_into::<PyModule>(py)
            {
                Ok(m) => m,
                Err(e) => {
                    _pyo3::PyErr::from(e).restore(py);
                    std::mem::forget(guard);
                    return std::ptr::null_mut();
                }
            };
            let ret = match #fnname(py, &module) {
                Ok(_) => module.into_ptr(),
                Err(e) => {
                    e.restore(py);
                    std::ptr::null_mut()
                }
            };
            std::mem::forget(guard);
            ret
        }
    }
}

pub fn build_py2_module_init(ast: &mut syn::Item, attr: String) -> Tokens {
    let modname = &attr.to_string()[1..attr.to_string().len()-1].to_string();

    match ast.node {
        syn::ItemKind::Fn(_, _, _, _, _, _) => {
            py2_init(&ast.ident, &modname)
        },
        _ => panic!("#[modinit] can only be used with fn block"),
    }
}

pub fn py2_init(fnname: &syn::Ident, name: &String) -> Tokens {
    let cb_name = syn::Ident::from(format!("init{}", name.trim()).as_ref());

    quote! {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #cb_name() {
            extern crate pyo3 as _pyo3;
            use std;

            let name = concat!(stringify!(#cb_name), "\0").as_ptr() as *const _;
            let guard = _pyo3::callback::AbortOnDrop("py_module_initializer");
            let py = pyo3::Python::assume_gil_acquired();
            pyo3::ffi::PyEval_InitThreads();
            let module = pyo3::ffi::Py_InitModule(name, std::ptr::null_mut());
            if module.is_null() {
                std::mem::forget(guard);
                return
            }

            let module = match pyo3::PyObject::from_borrowed_ptr(
                py, module).cast_into::<pyo3::PyModule>(py)
            {
                Ok(m) => m,
                Err(e) => {
                    _pyo3::PyErr::from(e).restore(py);
                    std::mem::forget(guard);
                    return
                }
            };
            let ret = match #fnname(py, &module) {
                Ok(()) => (),
                Err(e) => e.restore(py)
            };
            std::mem::forget(guard);
            ret
        }
    }
}
