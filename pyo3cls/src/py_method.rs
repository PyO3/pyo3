// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};
use method::{FnArg, FnSpec, FnType};


pub fn gen_py_method<'a>(cls: &Box<syn::Ty>, name: &syn::Ident,
                         sig: &mut syn::MethodSig, meth_attrs: &mut Vec<syn::Attribute>) -> Tokens
{
    check_generic(name, sig);

    let spec = FnSpec::parse(name, sig, meth_attrs);

    match spec.tp {
        FnType::Fn =>
            impl_py_method_def(name, &impl_wrap(cls, name, &spec)),
        FnType::FnNew =>
            impl_py_method_def_new(name, &impl_wrap_new(cls, name, &spec)),
        FnType::FnCall =>
            impl_py_method_def_call(name, &impl_wrap(cls, name, &spec)),
        FnType::Getter(ref getter) =>
            impl_py_getter_def(name, getter, &impl_wrap_getter(cls, name, &spec)),
        FnType::Setter(ref setter) =>
            impl_py_setter_def(name, setter, &impl_wrap_setter(cls, name, &spec)),
    }
}


fn check_generic(name: &syn::Ident, sig: &syn::MethodSig) {
    if !sig.generics.ty_params.is_empty() {
        panic!("python method can not be generic: {:?}", name);
    }
}


/// Generate function wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap(cls: &Box<syn::Ty>, name: &syn::Ident, spec: &FnSpec) -> Tokens {
    let cb = impl_call(cls, name, &spec);
    let body = impl_arg_params(&spec, cb);

    quote! {
        unsafe extern "C" fn wrap(slf: *mut _pyo3::ffi::PyObject,
                                  args: *mut _pyo3::ffi::PyObject,
                                  kwargs: *mut _pyo3::ffi::PyObject) -> *mut _pyo3::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            _pyo3::callback::handle(LOCATION, _pyo3::callback::PyObjectCallbackConverter, |py| {
                let args: _pyo3::Py<_pyo3::PyTuple> = _pyo3::Py::from_borrowed_ptr(py, args);
                let kwargs: Option<_pyo3::Py<_pyo3::PyDict>> =
                    _pyo3::argparse::get_kwargs(py, kwargs);

                #body
            })
        }
    }
}

/// Generate function wrapper for protocol method (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_proto_wrap(cls: &Box<syn::Ty>, name: &syn::Ident, spec: &FnSpec) -> Tokens {
    let cb = impl_call(cls, name, &spec);
    let body = impl_arg_params(&spec, cb);

    quote! {
        unsafe extern "C" fn wrap<'p>(
            slf: *mut _pyo3::ffi::PyObject,
            args: *mut _pyo3::ffi::PyObject,
            kwargs: *mut _pyo3::ffi::PyObject) -> *mut _pyo3::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            _pyo3::callback::handle(LOCATION, _pyo3::callback::PyObjectCallbackConverter, |py| {
                let args: _pyo3::Py<_pyo3::PyTuple> = _pyo3::Py::from_borrowed_ptr(py, args);
                let kwargs: Option<_pyo3::Py<_pyo3::PyDict>> =
                    _pyo3::argparse::get_kwargs(py, kwargs);

                #body
            })
        }
    }
}

/// Generate function wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_new(cls: &Box<syn::Ty>, name: &syn::Ident, spec: &FnSpec) -> Tokens {
    let cb = impl_class_new(cls, name, spec);
    let body = impl_arg_params(spec, cb);

    quote! {
        unsafe extern "C" fn wrap(cls: *mut _pyo3::ffi::PyTypeObject,
                                  args: *mut _pyo3::ffi::PyObject,
                                  kwargs: *mut _pyo3::ffi::PyObject) -> *mut _pyo3::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(
                stringify!(#cls), ".", stringify!(#name), "()");
            _pyo3::callback::handle(LOCATION, _pyo3::callback::PyObjectCallbackConverter, |py| {
                let args: _pyo3::Py<_pyo3::PyTuple> = _pyo3::Py::from_borrowed_ptr(py, args);
                let kwargs: Option<_pyo3::Py<_pyo3::PyDict>> =
                    _pyo3::argparse::get_kwargs(py, kwargs);

                #body
            })
        }
    }
}


/// Generate functiona wrapper (PyCFunction, PyCFunctionWithKeywords)
fn impl_wrap_getter(cls: &Box<syn::Ty>, name: &syn::Ident, _spec: &FnSpec) -> Tokens {
    quote! {
        unsafe extern "C" fn wrap(slf: *mut _pyo3::ffi::PyObject,
                                  _: *mut _pyo3::c_void)
                                  -> *mut _pyo3::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(
                stringify!(#cls), ".getter_", stringify!(#name), "()");
            _pyo3::callback::handle(LOCATION, _pyo3::callback::PyObjectCallbackConverter, |py| {
                let slf: _pyo3::Py<#cls> = _pyo3::Py::from_borrowed_ptr(py, slf);
                slf.#name(py)
            })
        }
    }
}

/// Generate functiona wrapper (PyCFunction, PyCFunctionWithKeywords)
fn impl_wrap_setter(cls: &Box<syn::Ty>, name: &syn::Ident, spec: &FnSpec) -> Tokens {
    let val_ty = spec.args[0].ty;

    quote! {
        unsafe extern "C" fn wrap(slf: *mut _pyo3::ffi::PyObject,
                                  value: *mut _pyo3::ffi::PyObject,
                                  _: *mut _pyo3::c_void) -> _pyo3::c_int
        {
            const LOCATION: &'static str = concat!(
                stringify!(#cls), ".setter", stringify!(#name), "()");
            _pyo3::callback::handle(LOCATION, _pyo3::callback::UnitCallbackConverter, |py| {
                let slf: _pyo3::Py<#cls> = _pyo3::Py::from_borrowed_ptr(py, slf);
                let value = _pyo3::PyObject::from_borrowed_ptr(py, value);

                match <#val_ty as _pyo3::FromPyObject>::extract(&value) {
                    Ok(val) => {
                        let ret = slf.#name(val);
                        ret.map(|o| ())
                    }
                    Err(e) => Err(e)
                }
            })
        }
    }
}


fn impl_call(cls: &Box<syn::Ty>, fname: &syn::Ident, spec: &FnSpec) -> Tokens {
    let names: Vec<&syn::Ident> = spec.args.iter().map(|item| item.name).collect();
    quote! {{
        let slf: _pyo3::Py<#cls> = _pyo3::Py::from_borrowed_ptr(py, slf);
        slf.as_mut().#fname(#(#names),*)
    }}
}

fn impl_class_new(cls: &Box<syn::Ty>, fname: &syn::Ident, spec: &FnSpec) -> Tokens {
    let names: Vec<&syn::Ident> = spec.args.iter().map(|item| item.name).collect();
    quote! {{
        let cls: _pyo3::Py<_pyo3::PyType> = _pyo3::Py::from_borrowed_ptr(
            py, cls as *mut _pyo3::ffi::PyObject);
        #cls::#fname(&cls, #(#names),*)
    }}
}

fn impl_arg_params(spec: &FnSpec, body: Tokens) -> Tokens {
    if spec.args.is_empty() {
        return body
    }
    
    let mut params = Vec::new();

    for arg in spec.args.iter() {
        if ! (spec.is_args(&arg.name) || spec.is_kwargs(&arg.name)) {
            let name = arg.name.as_ref();
            let opt = if let Some(_) = arg.optional {
                syn::Ident::from("true")
            } else {
                if let Some(_) = spec.default_value(&arg.name) {
                    syn::Ident::from("true")
                } else {
                    syn::Ident::from("false")
                }
            };
            params.push(
                quote! {
                    _pyo3::argparse::ParamDescription{name: #name, is_optional: #opt,}
                }
            );
        }
    }
    let placeholders: Vec<syn::Ident> = params.iter().map(
        |_| syn::Ident::from("None")).collect();

    // generate extrat args
    let mut rargs = spec.args.clone();
    rargs.reverse();
    let mut body = body;
    for arg in rargs.iter() {
        body = impl_arg_param(&arg, &spec, &body);
    }

    let accept_args = syn::Ident::from(
        if spec.accept_args() { "true" } else { "false" });
    let accept_kwargs = syn::Ident::from(
        if spec.accept_kwargs() { "true" } else { "false" });

    // create array of arguments, and then parse
    quote! {
        const PARAMS: &'static [_pyo3::argparse::ParamDescription<'static>] = &[
            #(#params),*
        ];

        let mut output = [#(#placeholders),*];
        match _pyo3::argparse::parse_args(py, Some(LOCATION), PARAMS, &args,
                                          kwargs.as_ref(), #accept_args, #accept_kwargs,
                                          &mut output) {
            Ok(_) => {
                let mut _iter = output.iter();

                #body
            },
            Err(err) => Err(err)
        }
    }
}

fn impl_arg_param(arg: &FnArg, spec: &FnSpec, body: &Tokens) -> Tokens {
    let ty = arg.ty;
    let name = arg.name;

    // First unwrap() asserts the iterated sequence is long enough (which should be guaranteed);
    // second unwrap() asserts the parameter was not missing (which fn
    // parse_args already checked for).

    if spec.is_args(&name) {
        quote! {
            match <#ty as _pyo3::FromPyObject>::extract(&args)
            {
                Ok(#name) => {
                    #body
                }
                Err(e) => Err(e)
            }
        }
    }
    else if spec.is_kwargs(&name) {
        quote! {
            let #name = kwargs.as_ref();
            #body
        }
    }
    else {
        if let Some(_) = arg.optional {
            // default value
            let mut default = Tokens::new();
            if let Some(d) = spec.default_value(name) {
                let dt = quote!{ Some(#d) };
                dt.to_tokens(&mut default);
            } else {
                syn::Ident::from("None").to_tokens(&mut default);
            }
            
            quote! {
                match
                    match _iter.next().unwrap().as_ref() {
                        Some(obj) => {
                            if obj == &py.None() {
                                Ok(#default)
                            } else {
                                match _pyo3::callback::unref_r(obj).extract() {
                                    Ok(obj) => Ok(Some(obj)),
                                    Err(e) => Err(e)
                                }
                            }
                        },
                        None => Ok(#default)
                    }
                {
                    Ok(#name) => #body,
                    Err(e) => Err(e)
                }
            }
        } else if let Some(default) = spec.default_value(name) {
            quote! {
                match match _iter.next().unwrap().as_ref() {
                    Some(obj) => {
                        if obj == &py.None() {
                            Ok(#default)
                        } else {
                            match _pyo3::callback::unref_r(obj).extract() {
                                Ok(obj) => Ok(obj),
                                Err(e) => Err(e),
                            }
                        }
                    },
                    None => Ok(#default)
                } {
                    Ok(#name) => #body,
                    Err(e) => Err(e)
                }
            }
        }
        else {
            quote! {
                match _pyo3::callback::unref_r(
                    _iter.next().unwrap().as_ref().unwrap()).extract()
                {
                    Ok(#name) => {
                        #body
                    }
                    Err(e) => Err(e)
                }
            }
        }
    }
}

pub fn impl_py_method_def(name: &syn::Ident, wrapper: &Tokens) -> Tokens {
    quote! {
        _pyo3::class::PyMethodDefType::Method({
            #wrapper

            _pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: _pyo3::class::PyMethodType::PyCFunctionWithKeywords(wrap),
                ml_flags: _pyo3::ffi::METH_VARARGS | _pyo3::ffi::METH_KEYWORDS,
                ml_doc: "",
            }
        })
    }
}

pub fn impl_py_method_def_new(name: &syn::Ident, wrapper: &Tokens) -> Tokens {
    quote! {
        _pyo3::class::PyMethodDefType::New({
            #wrapper

            _pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: _pyo3::class::PyMethodType::PyNewFunc(wrap),
                ml_flags: _pyo3::ffi::METH_VARARGS | _pyo3::ffi::METH_KEYWORDS,
                ml_doc: "",
            }
        })
    }
}

pub fn impl_py_method_def_call(name: &syn::Ident, wrapper: &Tokens) -> Tokens {
    quote! {
        _pyo3::class::PyMethodDefType::Call({
            #wrapper

            _pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: _pyo3::class::PyMethodType::PyCFunctionWithKeywords(wrap),
                ml_flags: _pyo3::ffi::METH_VARARGS | _pyo3::ffi::METH_KEYWORDS,
                ml_doc: "",
            }
        })
    }
}

fn impl_py_setter_def(name: &syn::Ident, setter: &Option<String>, wrapper: &Tokens) -> Tokens {
    let n = if let &Some(ref name) = setter {
        name.to_string()
    } else {
        let n = String::from(name.as_ref());
        if n.starts_with("set_") {
            n[4..].to_string()
        } else {
            n
        }
    };

    quote! {
        _pyo3::class::PyMethodDefType::Setter({
            #wrapper

            _pyo3::class::PySetterDef {
                name: #n,
                meth: wrap,
                doc: "",
            }
        })
    }
}

fn impl_py_getter_def(name: &syn::Ident, getter: &Option<String>, wrapper: &Tokens) -> Tokens {
    let n = if let &Some(ref name) = getter {
        name.to_string()
    } else {
        let n = String::from(name.as_ref());
        if n.starts_with("get_") {
            n[4..].to_string()
        } else {
            n
        }
    };

    quote! {
        _pyo3::class::PyMethodDefType::Getter({
            #wrapper

            _pyo3::class::PyGetterDef {
                name: #n,
                meth: wrap,
                doc: "",
            }
        })
    }
}
