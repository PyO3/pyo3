// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;

use args;
use method;
use py_method;
use utils;


pub fn build_py3_module_init(ast: &mut syn::Item, attr: String) -> Tokens {
    let modname = &attr.to_string()[1..attr.to_string().len()-1].to_string();

    match ast.node {
        syn::ItemKind::Fn(_, _, _, _, _, ref mut block) => {
            let mut stmts = Vec::new();
            for stmt in block.stmts.iter_mut() {
                match stmt {
                    &mut syn::Stmt::Item(ref mut item) => {
                        if let Some(block) = wrap_fn(item) {
                            for stmt in block.stmts.iter() {
                                stmts.push(stmt.clone());
                            }
                            continue
                        }
                    }
                    _ => (),
                }
                stmts.push(stmt.clone());
            }
            block.stmts = stmts;

            py3_init(&ast.ident, &modname, utils::get_doc(&ast.attrs, false))
        },
        _ => panic!("#[modinit] can only be used with fn block"),
    }
}

pub fn py3_init(fnname: &syn::Ident, name: &String, doc: syn::Lit) -> Tokens {
    let m_name = syn::Ident::from(name.trim().as_ref());
    let cb_name = syn::Ident::from(format!("PyInit_{}", name.trim()).as_ref());
    quote! {
        #[no_mangle]
        #[allow(non_snake_case, unused_imports)]
        pub unsafe extern "C" fn #cb_name() -> *mut ::pyo3::ffi::PyObject {
            extern crate pyo3;
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

pub fn build_py2_module_init(ast: &mut syn::Item, attr: String) -> Tokens {
    let modname = &attr.to_string()[1..attr.to_string().len()-1].to_string();

    match ast.node {
        syn::ItemKind::Fn(_, _, _, _, _, ref mut block) => {
            let mut stmts = Vec::new();
            for stmt in block.stmts.iter_mut() {
                match stmt {
                    &mut syn::Stmt::Item(ref mut item) => {
                        if let Some(block) = wrap_fn(item) {
                            for stmt in block.stmts.iter() {
                                stmts.push(stmt.clone());
                            }
                            continue
                        }
                    }
                    _ => (),
                }
                stmts.push(stmt.clone());
            }
            block.stmts = stmts;

            py2_init(&ast.ident, &modname, utils::get_doc(&ast.attrs, false))
        },
        _ => panic!("#[modinit] can only be used with fn block"),
    }
}

pub fn py2_init(fnname: &syn::Ident, name: &String, doc: syn::Lit) -> Tokens {
    let m_name = syn::Ident::from(name.trim().as_ref());
    let cb_name = syn::Ident::from(format!("init{}", name.trim()).as_ref());

    quote! {
        #[no_mangle]
        #[allow(non_snake_case, unused_imports)]
        pub unsafe extern "C" fn #cb_name() {
            extern crate pyo3;
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

fn wrap_fn(item: &mut syn::Item) -> Option<Box<syn::Block>> {
    let name = item.ident.clone();
    let mut new_attrs = Vec::new();
    let mut fnname = None;
    let mut modname = None;
    let mut fn_attrs = Vec::new();

    for attr in item.attrs.iter() {
        match attr.value {
            syn::MetaItem::List(ref name, ref meta) => {
                match name.as_ref() {
                    "pyfn" => {
                        if meta.len() >= 2 {
                            match meta[0] {
                                syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref ident)) => {
                                    modname = Some(ident.clone());
                                }
                                _ => modname = None
                            }
                            match meta[1] {
                                syn::NestedMetaItem::Literal(syn::Lit::Str(ref s, _)) => {
                                    fnname = Some(syn::Ident::from(s.as_str()));
                                }
                                _ => fnname = None
                            }
                        } else {
                            println!("can not parse 'pyfn' params {:?}", attr);
                            modname = None
                        }
                        if meta.len() >= 3 {
                            fn_attrs = args::parse_arguments(&meta[2..meta.len()]);
                        }
                        continue;
                    }
                    _ => (),
                }
            }
            _ => (),
        };
        new_attrs.push(attr.clone())
    }
    item.attrs.clear();
    item.attrs.extend(new_attrs);

    if let None = fnname {
        return None
    }
    if let None = modname {
        return None
    }

    match item.node {
        syn::ItemKind::Fn(ref decl, _, _, _, _, _) => {
            let mut arguments = Vec::new();

            for input in decl.inputs.iter() {
                match input {
                    &syn::FnArg::SelfRef(_, _) | &syn::FnArg::SelfValue(_) => (),
                    &syn::FnArg::Captured(ref pat, ref ty) => {
                        let (mode, ident) = match pat {
                            &syn::Pat::Ident(ref mode, ref ident, _) =>
                                (mode, ident),
                            _ =>
                                panic!("unsupported argument: {:?}", pat),
                        };

                        let py = match ty {
                            &syn::Ty::Path(_, ref path) =>
                                if let Some(segment) = path.segments.last() {
                                    segment.ident.as_ref() == "Python"
                                } else {
                                    false
                                },
                            _ => false
                        };

                        let opt = method::check_arg_ty_and_optional(&name, ty);
                        arguments.push(method::FnArg {name: ident,
                                                      mode: mode,
                                                      ty: ty,
                                                      optional: opt,
                                                      py: py,
                                                      reference: method::is_ref(&name, ty)});
                    }
                    &syn::FnArg::Ignored(_) =>
                        panic!("ignored argument: {:?}", name),
                }
            }

            let ty = match decl.output {
                syn::FunctionRetTy::Default => syn::Ty::Infer,
                syn::FunctionRetTy::Ty(ref ty) => ty.clone()
            };

            let spec = method::FnSpec {
                tp: method::FnType::Fn,
                attrs: fn_attrs,
                args: arguments,
                output: ty,
            };

            let m = modname.unwrap();
            let fnname = fnname.unwrap();
            let wrapper = impl_wrap(&name, &spec);
            let item2 = item.clone();
            let doc = utils::get_doc(&item.attrs, true);

            let tokens = quote! {
                fn test() {
                    #item2

                    #[allow(unused_imports)]
                    {
                        use std;
                        use pyo3 as _pyo3;
                        use pyo3::ObjectProtocol;

                        #wrapper

                        let _def = pyo3::class::PyMethodDef {
                            ml_name: stringify!(#fnname),
                            ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                            ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
                            ml_doc: #doc,
                        };

                        unsafe {
                            let func = pyo3::PyObject::from_owned_ptr_or_panic(
                                py, pyo3::ffi::PyCFunction_New(
                                    Box::into_raw(Box::new(_def.as_method_def())),
                                    std::ptr::null_mut()));

                            #m.add(stringify!(#fnname), func)?
                        }
                    }
                }
            }.to_string();

            let item = syn::parse_item(tokens.as_str()).unwrap();
            match item.node {
                syn::ItemKind::Fn(_, _, _, _, _, ref block) => {
                    return Some(block.clone())
                },
                _ => ()
            }
        },
        _ => (),
    }

    None
}


/// Generate static method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap(name: &syn::Ident, spec: &method::FnSpec) -> Tokens {
    let names: Vec<syn::Ident> = spec.args.iter().enumerate().map(
        |item| if item.1.py {syn::Ident::from("_py")} else {
            syn::Ident::from(format!("arg{}", item.0))}).collect();
    let cb = quote! {{
        #name(#(#names),*)
    }};

    let body = py_method::impl_arg_params(spec, cb);
    let output = &spec.output;

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

            let _result: #output = {
                #body
            };
            _pyo3::callback::cb_convert(
                _pyo3::callback::PyObjectCallbackConverter, _py, _result)
        }
    }
}
