// Copyright (c) 2017-present PyO3 Project and Contributors

use method::{FnArg, FnSpec, FnType};
use quote::ToTokens;
use syn;

use proc_macro2::{Span, TokenStream};
use utils;

pub fn gen_py_method<'a>(
    cls: &syn::Type,
    name: &syn::Ident,
    sig: &mut syn::MethodSig,
    meth_attrs: &mut Vec<syn::Attribute>,
) -> TokenStream {
    check_generic(name, sig);

    let doc = utils::get_doc(&meth_attrs, true);
    let spec = FnSpec::parse(name, sig, meth_attrs);

    match spec.tp {
        FnType::Fn => impl_py_method_def(name, doc, &spec, &impl_wrap(cls, name, &spec, true)),
        FnType::FnNew => impl_py_method_def_new(name, doc, &impl_wrap_new(cls, name, &spec)),
        FnType::FnInit => impl_py_method_def_init(name, doc, &impl_wrap_init(cls, name, &spec)),
        FnType::FnCall => impl_py_method_def_call(name, doc, &impl_wrap(cls, name, &spec, false)),
        FnType::FnClass => impl_py_method_def_class(name, doc, &impl_wrap_class(cls, name, &spec)),
        FnType::FnStatic => {
            impl_py_method_def_static(name, doc, &impl_wrap_static(cls, name, &spec))
        }
        FnType::Getter(ref getter) => {
            impl_py_getter_def(name, doc, getter, &impl_wrap_getter(cls, name))
        }
        FnType::Setter(ref setter) => {
            impl_py_setter_def(name, doc, setter, &impl_wrap_setter(cls, name, &spec))
        }
    }
}

fn check_generic(name: &syn::Ident, sig: &syn::MethodSig) {
    if !sig.decl.generics.params.is_empty() {
        panic!("python method can not be generic: {:?}", name);
    }
}

fn get_class_name(cls: &syn::Type) -> &syn::Ident {
    if let &syn::Type::Path(ref type_path) = cls {
        let &syn::TypePath { ref path, .. } = type_path;
        let &syn::Path { ref segments, .. } = path;
        let last = segments.last().expect("class path should not be empty").into_value();
        &last.ident
    } else {
        panic!("class should be a path");
    }
}

pub fn body_to_result(body: &TokenStream, spec: &FnSpec) -> TokenStream {
    let output = &spec.output;
    quote! {
        let _result: ::pyo3::PyResult<<#output as ::pyo3::ReturnTypeIntoPyResult>::Inner> = {
            #body
        };
    }
}

/// Generate function wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec, noargs: bool) -> TokenStream {
    let body = impl_call(cls, name, &spec);

    if spec.args.is_empty() && noargs {
        let body_to_result = body_to_result(&body, spec);

        quote! {
            unsafe extern "C" fn __wrap(
                _slf: *mut ::pyo3::ffi::PyObject) -> *mut ::pyo3::ffi::PyObject
            {
                const _LOCATION: &'static str = concat!(
                    stringify!(#cls), ".", stringify!(#name), "()");
                let _pool = ::pyo3::GILPool::new();
                let _py = ::pyo3::Python::assume_gil_acquired();
                let _slf = _py.mut_from_borrowed_ptr::<#cls>(_slf);

                #body_to_result
                ::pyo3::callback::cb_convert(
                    ::pyo3::callback::PyObjectCallbackConverter, _py, _result)
            }
        }
    } else {
        let body = impl_arg_params(&spec, body);
        let body_to_result = body_to_result(&body, spec);

        quote! {
            unsafe extern "C" fn __wrap(
                _slf: *mut ::pyo3::ffi::PyObject,
                _args: *mut ::pyo3::ffi::PyObject,
                _kwargs: *mut ::pyo3::ffi::PyObject) -> *mut ::pyo3::ffi::PyObject
            {
                const _LOCATION: &'static str = concat!(
                    stringify!(#cls), ".", stringify!(#name), "()");
                let _pool = ::pyo3::GILPool::new();
                let _py = ::pyo3::Python::assume_gil_acquired();
                let _slf = _py.mut_from_borrowed_ptr::<#cls>(_slf);
                let _args = _py.from_borrowed_ptr::<::pyo3::PyTuple>(_args);
                let _kwargs = ::pyo3::argparse::get_kwargs(_py, _kwargs);

                #body_to_result
                ::pyo3::callback::cb_convert(
                    ::pyo3::callback::PyObjectCallbackConverter, _py, _result)
            }
        }
    }
}

/// Generate function wrapper for protocol method (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_proto_wrap(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec) -> TokenStream {
    let cb = impl_call(cls, name, &spec);
    let body = impl_arg_params(&spec, cb);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut ::pyo3::ffi::PyObject,
            _args: *mut ::pyo3::ffi::PyObject,
            _kwargs: *mut ::pyo3::ffi::PyObject) -> *mut ::pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = ::pyo3::GILPool::new();
            let _py = ::pyo3::Python::assume_gil_acquired();
            let _slf = _py.mut_from_borrowed_ptr::<#cls>(_slf);
            let _args = _py.from_borrowed_ptr::<::pyo3::PyTuple>(_args);
            let _kwargs = ::pyo3::argparse::get_kwargs(_py, _kwargs);

            let _result = {
                #body
            };
            ::pyo3::callback::cb_convert(
                ::pyo3::callback::PyObjectCallbackConverter, _py, _result)
        }
    }
}

/// Generate class method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_new(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec) -> TokenStream {
    let cls_name = get_class_name(cls);
    let names: Vec<syn::Ident> = spec
        .args
        .iter()
        .enumerate()
        .map(|item| {
            if item.1.py {
                syn::Ident::new("_py", Span::call_site())
            } else {
                syn::Ident::new(&format!("arg{}", item.0), Span::call_site())
            }
        })
        .collect();
    let cb = quote! {
        ::pyo3::ReturnTypeIntoPyResult::return_type_into_py_result(#cls_name::#name(&_obj, #(#names),*))
    };

    let body = impl_arg_params(spec, cb);
    let body_to_result = body_to_result(&body, spec);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _cls: *mut ::pyo3::ffi::PyTypeObject,
            _args: *mut ::pyo3::ffi::PyObject,
            _kwargs: *mut ::pyo3::ffi::PyObject) -> *mut ::pyo3::ffi::PyObject
        {
            use pyo3::typeob::PyTypeInfo;

            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = ::pyo3::GILPool::new();
            let _py = ::pyo3::Python::assume_gil_acquired();
            match ::pyo3::typeob::PyRawObject::new(_py, #cls_name::type_object(), _cls) {
                Ok(_obj) => {
                    let _args = _py.from_borrowed_ptr::<::pyo3::PyTuple>(_args);
                    let _kwargs = ::pyo3::argparse::get_kwargs(_py, _kwargs);

                    #body_to_result

                    match _result {
                        Ok(_) => _obj.into_ptr(),
                        Err(e) => {
                            e.restore(_py);
                            ::std::ptr::null_mut()
                        }
                    }
                }
                Err(e) => {
                    e.restore(_py);
                    ::std::ptr::null_mut()
                }
            }
        }
    }
}

/// Generate function wrapper for ffi::initproc
fn impl_wrap_init(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec) -> TokenStream {
    let cb = impl_call(cls, name, &spec);
    let output = &spec.output;
    let result_empty: syn::Type = parse_quote!(PyResult<()>);
    let empty: syn::Type = parse_quote!(());
    if output != &result_empty || output != &empty {
        panic!("Constructor must return PyResult<()> or a ()");
    }

    let body = impl_arg_params(&spec, cb);
    let body_to_result = body_to_result(&body, spec);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut ::pyo3::ffi::PyObject,
            _args: *mut ::pyo3::ffi::PyObject,
            _kwargs: *mut ::pyo3::ffi::PyObject) -> ::pyo3::c_int
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = ::pyo3::GILPool::new();
            let _py = ::pyo3::Python::assume_gil_acquired();
            let _slf = _py.mut_from_borrowed_ptr::<#cls>(_slf);
            let _args = _py.from_borrowed_ptr::<::pyo3::PyTuple>(_args);
            let _kwargs = ::pyo3::argparse::get_kwargs(_py, _kwargs);

            #body_to_result
            match _result {
                Ok(_) => 0,
                Err(e) => {
                    e.restore(_py);
                    -1
                }
            }
        }
    }
}

/// Generate class method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_class(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec) -> TokenStream {
    let names: Vec<syn::Ident> = spec
        .args
        .iter()
        .enumerate()
        .map(|item| {
            if item.1.py {
                syn::Ident::new("_py", Span::call_site())
            } else {
                syn::Ident::new(&format!("arg{}", item.0), Span::call_site())
            }
        })
        .collect();
    let cb = quote! {
        ::pyo3::ReturnTypeIntoPyResult::return_type_into_py_result(#cls::#name(&_cls, #(#names),*))
    };

    let body = impl_arg_params(spec, cb);
    let body_to_result = body_to_result(&body, spec);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _cls: *mut ::pyo3::ffi::PyObject,
            _args: *mut ::pyo3::ffi::PyObject,
            _kwargs: *mut ::pyo3::ffi::PyObject) -> *mut ::pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = ::pyo3::GILPool::new();
            let _py = ::pyo3::Python::assume_gil_acquired();
            let _cls = ::pyo3::PyType::from_type_ptr(_py, _cls as *mut ::pyo3::ffi::PyTypeObject);
            let _args = _py.from_borrowed_ptr::<::pyo3::PyTuple>(_args);
            let _kwargs = ::pyo3::argparse::get_kwargs(_py, _kwargs);

            #body_to_result
            ::pyo3::callback::cb_convert(
                ::pyo3::callback::PyObjectCallbackConverter, _py, _result)
        }
    }
}

/// Generate static method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_static(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec) -> TokenStream {
    let names: Vec<syn::Ident> = spec
        .args
        .iter()
        .enumerate()
        .map(|item| {
            if item.1.py {
                syn::Ident::new("_py", Span::call_site())
            } else {
                syn::Ident::new(&format!("arg{}", item.0), Span::call_site())
            }
        })
        .collect();
    let cb = quote! {
        ::pyo3::ReturnTypeIntoPyResult::return_type_into_py_result(#cls::#name(#(#names),*))
    };

    let body = impl_arg_params(spec, cb);
    let body_to_result = body_to_result(&body, spec);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut ::pyo3::ffi::PyObject,
            _args: *mut ::pyo3::ffi::PyObject,
            _kwargs: *mut ::pyo3::ffi::PyObject) -> *mut ::pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = ::pyo3::GILPool::new();
            let _py = ::pyo3::Python::assume_gil_acquired();
            let _args = _py.from_borrowed_ptr::<::pyo3::PyTuple>(_args);
            let _kwargs = ::pyo3::argparse::get_kwargs(_py, _kwargs);

            #body_to_result
            ::pyo3::callback::cb_convert(
                ::pyo3::callback::PyObjectCallbackConverter, _py, _result)
        }
    }
}

/// Generate functiona wrapper (PyCFunction, PyCFunctionWithKeywords)
pub(crate) fn impl_wrap_getter(cls: &syn::Type, name: &syn::Ident) -> TokenStream {
    quote! {
        unsafe extern "C" fn __wrap(
            _slf: *mut ::pyo3::ffi::PyObject, _: *mut ::pyo3::c_void) -> *mut ::pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");

            let _pool = ::pyo3::GILPool::new();
            let _py = ::pyo3::Python::assume_gil_acquired();
            let _slf = _py.mut_from_borrowed_ptr::<#cls>(_slf);

            match _slf.#name() {
                Ok(val) => {
                    val.into_object(_py).into_ptr()
                }
                Err(e) => {
                    e.restore(_py);
                    ::std::ptr::null_mut()
                }
            }
        }
    }
}

/// Generate functiona wrapper (PyCFunction, PyCFunctionWithKeywords)
pub(crate) fn impl_wrap_setter(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec) -> TokenStream {
    if spec.args.len() < 1 {
        println!("Not enough arguments for setter {}::{}", quote!{#cls}, name);
    }
    let val_ty = spec.args[0].ty;

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut ::pyo3::ffi::PyObject,
            _value: *mut ::pyo3::ffi::PyObject, _: *mut ::pyo3::c_void) -> ::pyo3::c_int
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = ::pyo3::GILPool::new();
            let _py = ::pyo3::Python::assume_gil_acquired();
            let _slf = _py.mut_from_borrowed_ptr::<#cls>(_slf);
            let _value = _py.from_borrowed_ptr(_value);

            let _result = match <#val_ty as ::pyo3::FromPyObject>::extract(_value) {
                Ok(_val) => _slf.#name(_val),
                Err(e) => Err(e)
            };
            match _result {
                Ok(_) => 0,
                Err(e) => {
                    e.restore(_py);
                    -1
                }
            }
        }
    }
}

fn impl_call(_cls: &syn::Type, fname: &syn::Ident, spec: &FnSpec) -> TokenStream {
    let names: Vec<syn::Ident> = spec
        .args
        .iter()
        .enumerate()
        .map(|item| {
            if item.1.py {
                syn::Ident::new("_py", Span::call_site())
            } else {
                syn::Ident::new(&format!("arg{}", item.0), Span::call_site())
            }
        })
        .collect();
    quote! {
        ::pyo3::ReturnTypeIntoPyResult::return_type_into_py_result(_slf.#fname(#(#names),*))
    }
}

pub fn impl_arg_params(spec: &FnSpec, body: TokenStream) -> TokenStream {
    let args: Vec<FnArg> = spec
        .args
        .iter()
        .filter(|item| !item.py)
        .map(|item| item.clone())
        .collect();
    if args.is_empty() {
        return body;
    }

    let mut params = Vec::new();

    for arg in spec.args.iter() {
        if arg.py {
            continue;
        }
        if !(spec.is_args(&arg.name) || spec.is_kwargs(&arg.name)) {
            let name = arg.name;
            let kwonly = if spec.is_kw_only(&arg.name) {
                syn::Ident::new("true", Span::call_site())
            } else {
                syn::Ident::new("false", Span::call_site())
            };

            let opt = if let Some(_) = arg.optional {
                syn::Ident::new("true", Span::call_site())
            } else if let Some(_) = spec.default_value(&arg.name) {
                syn::Ident::new("true", Span::call_site())
            } else {
                syn::Ident::new("false", Span::call_site())
            };

            params.push(quote! {
                ::pyo3::argparse::ParamDescription{
                    name: stringify!(#name), is_optional: #opt, kw_only: #kwonly}
            });
        }
    }
    let placeholders: Vec<syn::Ident> = params
        .iter()
        .map(|_| syn::Ident::new("None", Span::call_site()))
        .collect();

    // generate extrat args
    let len = spec.args.len();
    let mut rargs = spec.args.clone();
    rargs.reverse();
    let mut body = body;

    for (idx, arg) in rargs.iter().enumerate() {
        body = impl_arg_param(&arg, &spec, &body, len - idx - 1);
    }

    let accept_args = syn::Ident::new(
        if spec.accept_args() { "true" } else { "false" },
        Span::call_site(),
    );
    let accept_kwargs = syn::Ident::new(
        if spec.accept_kwargs() {
            "true"
        } else {
            "false"
        },
        Span::call_site(),
    );

    // create array of arguments, and then parse
    quote! {
        const _PARAMS: &'static [::pyo3::argparse::ParamDescription<'static>] = &[
            #(#params),*
        ];

        let mut _output = [#(#placeholders),*];
        match ::pyo3::argparse::parse_args(Some(_LOCATION), _PARAMS, &_args,
            _kwargs, #accept_args, #accept_kwargs, &mut _output)
        {
            Ok(_) => {
                let mut _iter = _output.iter();

                #body
            },
            Err(err) => Err(err)
        }
    }
}

fn impl_arg_param(arg: &FnArg, spec: &FnSpec, body: &TokenStream, idx: usize) -> TokenStream {
    if arg.py {
        return body.clone();
    }
    let ty = arg.ty;
    let name = arg.name;
    let arg_name = syn::Ident::new(&format!("arg{}", idx), Span::call_site());

    // First unwrap() asserts the iterated sequence is long enough (which should be guaranteed);
    // second unwrap() asserts the parameter was not missing (which fn
    // parse_args already checked for).

    if spec.is_args(&name) {
        quote! {
            <#ty as ::pyo3::FromPyObject>::extract(_args.as_ref())
                .and_then(|#arg_name| {
                    #body
                })
        }
    } else if spec.is_kwargs(&name) {
        quote! {{
            let #arg_name = _kwargs;
            #body
        }}
    } else {
        if let Some(_) = arg.optional {
            // default value
            let mut default = TokenStream::new();
            if let Some(d) = spec.default_value(name) {
                let dt = quote! { Some(#d) };
                dt.to_tokens(&mut default);
            } else {
                syn::Ident::new("None", Span::call_site()).to_tokens(&mut default);
            }

            quote! {
                match
                    match _iter.next().unwrap().as_ref() {
                        Some(_obj) => {
                            if _obj.is_none() {
                                Ok(#default)
                            } else {
                                match _obj.extract() {
                                    Ok(_obj) => Ok(Some(_obj)),
                                    Err(e) => Err(e)
                                }
                            }
                        },
                        None => Ok(#default)
                    }
                {
                    Ok(#arg_name) => #body,
                    Err(e) => Err(e)
                }
            }
        } else if let Some(default) = spec.default_value(name) {
            quote! {
                match match _iter.next().unwrap().as_ref() {
                    Some(_obj) => {
                        if _obj.is_none() {
                            Ok(#default)
                        } else {
                            match _obj.extract() {
                                Ok(_obj) => Ok(_obj),
                                Err(e) => Err(e),
                            }
                        }
                    },
                    None => Ok(#default)
                } {
                    Ok(#arg_name) => #body,
                    Err(e) => Err(e)
                }
            }
        } else {
            quote! {
                ::pyo3::ObjectProtocol::extract(_iter.next().unwrap().unwrap())
                    .and_then(|#arg_name| {
                        #body
                    })
            }
        }
    }
}

pub fn impl_py_method_def(
    name: &syn::Ident,
    doc: syn::Lit,
    spec: &FnSpec,
    wrapper: &TokenStream,
) -> TokenStream {
    if spec.args.is_empty() {
        quote! {
            ::pyo3::class::PyMethodDefType::Method({
                #wrapper

                ::pyo3::class::PyMethodDef {
                    ml_name: stringify!(#name),
                    ml_meth: ::pyo3::class::PyMethodType::PyNoArgsFunction(__wrap),
                    ml_flags: ::pyo3::ffi::METH_NOARGS,
                    ml_doc: #doc,
                }
            })
        }
    } else {
        quote! {
            ::pyo3::class::PyMethodDefType::Method({
                #wrapper

                ::pyo3::class::PyMethodDef {
                    ml_name: stringify!(#name),
                    ml_meth: ::pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                    ml_flags: ::pyo3::ffi::METH_VARARGS | ::pyo3::ffi::METH_KEYWORDS,
                    ml_doc: #doc,
                }
            })
        }
    }
}

pub fn impl_py_method_def_new(
    name: &syn::Ident,
    doc: syn::Lit,
    wrapper: &TokenStream,
) -> TokenStream {
    quote! {
        ::pyo3::class::PyMethodDefType::New({
            #wrapper

            ::pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: ::pyo3::class::PyMethodType::PyNewFunc(__wrap),
                ml_flags: ::pyo3::ffi::METH_VARARGS | ::pyo3::ffi::METH_KEYWORDS,
                ml_doc: #doc,
            }
        })
    }
}

pub fn impl_py_method_def_init(
    name: &syn::Ident,
    doc: syn::Lit,
    wrapper: &TokenStream,
) -> TokenStream {
    quote! {
        ::pyo3::class::PyMethodDefType::Init({
            #wrapper

            ::pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: ::pyo3::class::PyMethodType::PyInitFunc(__wrap),
                ml_flags: ::pyo3::ffi::METH_VARARGS | ::pyo3::ffi::METH_KEYWORDS,
                ml_doc: #doc,
            }
        })
    }
}

pub fn impl_py_method_def_class(
    name: &syn::Ident,
    doc: syn::Lit,
    wrapper: &TokenStream,
) -> TokenStream {
    quote! {
        ::pyo3::class::PyMethodDefType::Class({
            #wrapper

            ::pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: ::pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                ml_flags: ::pyo3::ffi::METH_VARARGS | ::pyo3::ffi::METH_KEYWORDS |
                ::pyo3::ffi::METH_CLASS,
                ml_doc: #doc,
            }
        })
    }
}

pub fn impl_py_method_def_static(
    name: &syn::Ident,
    doc: syn::Lit,
    wrapper: &TokenStream,
) -> TokenStream {
    quote! {
        ::pyo3::class::PyMethodDefType::Static({
            #wrapper

            ::pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: ::pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                ml_flags: ::pyo3::ffi::METH_VARARGS | ::pyo3::ffi::METH_KEYWORDS | ::pyo3::ffi::METH_STATIC,
                ml_doc: #doc,
            }
        })
    }
}

pub fn impl_py_method_def_call(
    name: &syn::Ident,
    doc: syn::Lit,
    wrapper: &TokenStream,
) -> TokenStream {
    quote! {
        ::pyo3::class::PyMethodDefType::Call({
            #wrapper

            ::pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: ::pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                ml_flags: ::pyo3::ffi::METH_VARARGS | ::pyo3::ffi::METH_KEYWORDS,
                ml_doc: #doc,
            }
        })
    }
}

pub(crate) fn impl_py_setter_def(
    name: &syn::Ident,
    doc: syn::Lit,
    setter: &Option<String>,
    wrapper: &TokenStream,
) -> TokenStream {
    let n = if let &Some(ref name) = setter {
        name.to_string()
    } else {
        let n = name.to_string();
        if n.starts_with("set_") {
            n[4..].to_string()
        } else {
            n
        }
    };

    quote! {
        ::pyo3::class::PyMethodDefType::Setter({
            #wrapper

            ::pyo3::class::PySetterDef {
                name: #n,
                meth: __wrap,
                doc: #doc,
            }
        })
    }
}

pub(crate) fn impl_py_getter_def(
    name: &syn::Ident,
    doc: syn::Lit,
    getter: &Option<String>,
    wrapper: &TokenStream,
) -> TokenStream {
    let n = if let &Some(ref name) = getter {
        name.to_string()
    } else {
        let n = name.to_string();
        if n.starts_with("get_") {
            n[4..].to_string()
        } else {
            n
        }
    };

    quote! {
        ::pyo3::class::PyMethodDefType::Getter({
            #wrapper

            ::pyo3::class::PyGetterDef {
                name: #n,
                meth: __wrap,
                doc: #doc,
            }
        })
    }
}
