// Copyright (c) 2017-present PyO3 Project and Contributors

use args;
use method;
use py_method;
use quote::Tokens;
use syn;
use utils;

/// Generates the function that is called by the python interpreter to initialize the native
/// module
pub fn py3_init(fnname: &syn::Ident, name: &String, doc: syn::Lit) -> Tokens {
    let m_name = syn::Ident::from(name.trim().as_ref());
    let cb_name = syn::Ident::from(format!("PyInit_{}", name.trim()).as_ref());
    quote! {
        #[no_mangle]
        #[allow(non_snake_case, unused_imports)]
        pub unsafe extern "C" fn #cb_name() -> *mut ::pyo3::ffi::PyObject {
            use std;
            use pyo3::{IntoPyPointer, ObjectProtocol};

            // initialize pyo3
            pyo3::prepare_pyo3_library();

            static mut MODULE_DEF: pyo3::ffi::PyModuleDef = pyo3::ffi::PyModuleDef_INIT;
            // We can't convert &'static str to *const c_char within a static initializer,
            // so we'll do it here in the module initialization:
            MODULE_DEF.m_name = concat!(stringify!(#m_name), "\0").as_ptr() as *const _;

            #[cfg(py_sys_config = "WITH_THREAD")]
            pyo3::ffi::PyEval_InitThreads();

            let _module = pyo3::ffi::PyModule_Create(&mut MODULE_DEF);
            if _module.is_null() {
                return _module;
            }

            let _pool = pyo3::GILPool::new();
            let _py = pyo3::Python::assume_gil_acquired();
            let _module = match _py.from_owned_ptr_or_err::<pyo3::PyModule>(_module) {
                Ok(m) => m,
                Err(e) => {
                    pyo3::PyErr::from(e).restore(_py);
                    return std::ptr::null_mut();
                }
            };
            _module.add("__doc__", #doc).expect("Failed to add doc for module");
            match #fnname(_py, _module) {
                Ok(_) => _module.into_ptr(),
                Err(e) => {
                    e.restore(_py);
                    std::ptr::null_mut()
                }
            }
        }
    }
}

pub fn py2_init(fnname: &syn::Ident, name: &String, doc: syn::Lit) -> Tokens {
    let m_name = syn::Ident::from(name.trim().as_ref());
    let cb_name = syn::Ident::from(format!("init{}", name.trim()).as_ref());

    quote! {
        #[no_mangle]
        #[allow(non_snake_case, unused_imports)]
        pub unsafe extern "C" fn #cb_name() {
            use std;

            // initialize python
            pyo3::prepare_pyo3_library();
            pyo3::ffi::PyEval_InitThreads();

            let _name = concat!(stringify!(#m_name), "\0").as_ptr() as *const _;
            let _pool = pyo3::GILPool::new();
            let _py = pyo3::Python::assume_gil_acquired();
            let _module = pyo3::ffi::Py_InitModule(_name, std::ptr::null_mut());
            if _module.is_null() {
                return
            }

            let _module = match _py.from_borrowed_ptr_or_err::<pyo3::PyModule>(_module) {
                Ok(m) => m,
                Err(e) => {
                    pyo3::PyErr::from(e).restore(_py);
                    return
                }
            };

            _module.add("__doc__", #doc).expect("Failed to add doc for module");
            if let Err(e) = #fnname(_py, _module) {
                e.restore(_py)
            }
        }
    }
}

/// Finds and takes care of the #[pyfn(...)] in #[modinit(...)]
pub fn process_functions_in_module(ast: &mut syn::Item) {
    if let syn::ItemKind::Fn(_, _, _, _, _, ref mut block) = ast.node {
        let mut stmts: Vec<syn::Stmt> = Vec::new();
        for stmt in block.stmts.iter_mut() {
            if let &mut syn::Stmt::Item(ref mut item) = stmt {
                if let Some((module_name, python_name, pyfn_attrs)) =
                    extract_pyfn_attrs(&mut item.attrs)
                {
                    let function_to_python = add_fn_to_module(item, &python_name, pyfn_attrs);
                    let function_wrapper_ident = function_wrapper_ident(&item.ident);
                    let tokens = quote! {
                        fn block_wrapper() {
                            #function_to_python

                            #module_name.add_function(&#function_wrapper_ident);
                        }
                    }.to_string();

                    let item = syn::parse_item(tokens.as_str()).unwrap();
                    let block = match item.node {
                        syn::ItemKind::Fn(_, _, _, _, _, ref block) => block.clone(),
                        _ => unreachable!(),
                    };

                    stmts.extend(block.stmts.into_iter());
                }
            };
            stmts.push(stmt.clone());
        }
        block.stmts = stmts;
    } else {
        panic!("#[modinit] can only be used with fn block");
    }
}

/// Transforms a rust fn arg parsed with syn into a method::FnArg
fn wrap_fn_argument<'a>(input: &'a syn::FnArg, name: &'a syn::Ident) -> Option<method::FnArg<'a>> {
    match input {
        &syn::FnArg::SelfRef(_, _) | &syn::FnArg::SelfValue(_) => None,
        &syn::FnArg::Captured(ref pat, ref ty) => {
            let (mode, ident) = match pat {
                &syn::Pat::Ident(ref mode, ref ident, _) => (mode, ident),
                _ => panic!("unsupported argument: {:?}", pat),
            };

            let py = match ty {
                &syn::Ty::Path(_, ref path) => if let Some(segment) = path.segments.last() {
                    segment.ident.as_ref() == "Python"
                } else {
                    false
                },
                _ => false,
            };

            let opt = method::check_arg_ty_and_optional(&name, ty);
            Some(method::FnArg {
                name: ident,
                mode,
                ty,
                optional: opt,
                py,
                reference: method::is_ref(&name, ty),
            })
        }
        &syn::FnArg::Ignored(_) => panic!("ignored argument: {:?}", name),
    }
}

/// Extracts the data from the #[pyfn(...)] attribute of a function
fn extract_pyfn_attrs(
    attrs: &mut Vec<syn::Attribute>,
) -> Option<(syn::Ident, syn::Ident, Vec<args::Argument>)> {
    let mut new_attrs = Vec::new();
    let mut fnname = None;
    let mut modname = None;
    let mut fn_attrs = Vec::new();

    for attr in attrs.iter() {
        if let syn::MetaItem::List(ref name, ref meta) = attr.value {
            if name.as_ref() == "pyfn" {
                if meta.len() >= 2 {
                    match meta[0] {
                        syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref ident)) => {
                            modname = Some(ident.clone());
                        }
                        _ => panic!("The first parameter of pyfn must be a MetaItem"),
                    }
                    match meta[1] {
                        syn::NestedMetaItem::Literal(syn::Lit::Str(ref s, _)) => {
                            fnname = Some(syn::Ident::from(s.as_str()));
                        }
                        _ => panic!("The second parameter of pyfn must be a Literal"),
                    }
                    if meta.len() >= 3 {
                        fn_attrs = args::parse_arguments(&meta[2..meta.len()]);
                    }
                } else {
                    panic!("can not parse 'pyfn' params {:?}", attr);
                }
                continue;
            }
        };
        new_attrs.push(attr.clone())
    }
    attrs.clear();
    attrs.extend(new_attrs);
    Some((modname?, fnname?, fn_attrs))
}

/// Coordinates the naming of a the add-function-to-python-module function
fn function_wrapper_ident(name: &syn::Ident) -> syn::Ident {
    // Make sure this ident matches the one of wrap_function
    syn::Ident::new("__pyo3_get_function_".to_string() + &name.to_string())
}

/// Generates python wrapper over a function that allows adding it to a python module as a python
/// function
pub fn add_fn_to_module(
    item: &mut syn::Item,
    python_name: &syn::Ident,
    pyfn_attrs: Vec<args::Argument>,
) -> Tokens {
    let name = item.ident.clone();

    let decl = if let syn::ItemKind::Fn(ref decl, _, _, _, _, _) = item.node {
        decl.clone()
    } else {
        panic!("Expected a function")
    };

    let mut arguments = Vec::new();

    for input in decl.inputs.iter() {
        if let Some(fn_arg) = wrap_fn_argument(input, &name) {
            arguments.push(fn_arg);
        }
    }

    let ty = method::get_return_info(&decl.output);

    let spec = method::FnSpec {
        tp: method::FnType::Fn,
        attrs: pyfn_attrs,
        args: arguments,
        output: ty,
    };

    let function_wrapper_ident = function_wrapper_ident(&name);

    let wrapper = function_c_wrapper(&name, &spec);
    let doc = utils::get_doc(&item.attrs, true);

    let tokens = quote! (
        fn #function_wrapper_ident(py: ::pyo3::Python) -> ::pyo3::PyObject {
            use std;
            use pyo3 as _pyo3;
            use pyo3::ObjectProtocol;

            #wrapper

            let _def = pyo3::class::PyMethodDef {
                ml_name: stringify!(#python_name),
                ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
                ml_doc: #doc,
            };

            let function = unsafe {
                pyo3::PyObject::from_owned_ptr_or_panic(
                    py,
                    pyo3::ffi::PyCFunction_New(
                        Box::into_raw(Box::new(_def.as_method_def())),
                        std::ptr::null_mut()
                    )
                )
            };

            function
        }
    );

    tokens
}

/// Generate static function wrapper (PyCFunction, PyCFunctionWithKeywords)
fn function_c_wrapper(name: &syn::Ident, spec: &method::FnSpec) -> Tokens {
    let names: Vec<syn::Ident> = spec.args
        .iter()
        .enumerate()
        .map(|item| {
            if item.1.py {
                syn::Ident::from("_py")
            } else {
                syn::Ident::from(format!("arg{}", item.0))
            }
        })
        .collect();
    let cb = quote! {
        ::pyo3::ReturnTypeIntoPyResult::return_type_into_py_result(#name(#(#names),*))
    };

    let body = py_method::impl_arg_params(spec, cb);
    let body_to_result = py_method::body_to_result(&body, spec);

    quote! {
        #[allow(unused_variables, unused_imports)]
        unsafe extern "C" fn __wrap(
            _slf: *mut _pyo3::ffi::PyObject,
            _args: *mut _pyo3::ffi::PyObject,
            _kwargs: *mut _pyo3::ffi::PyObject) -> *mut _pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#name), "()");

            let _pool = _pyo3::GILPool::new();
            let _py = _pyo3::Python::assume_gil_acquired();
            let _args = _py.from_borrowed_ptr::<_pyo3::PyTuple>(_args);
            let _kwargs = _pyo3::argparse::get_kwargs(_py, _kwargs);

            #body_to_result
            _pyo3::callback::cb_convert(
                _pyo3::callback::PyObjectCallbackConverter, _py, _result)
        }
    }
}
