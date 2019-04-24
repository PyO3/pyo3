// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::method::{FnArg, FnSpec, FnType};
use crate::utils;
use proc_macro2::{Span, TokenStream};
use quote::quote;

pub fn gen_py_method(
    cls: &syn::Type,
    name: &syn::Ident,
    sig: &mut syn::MethodSig,
    meth_attrs: &mut Vec<syn::Attribute>,
) -> TokenStream {
    check_generic(name, sig);

    let doc = utils::get_doc(&meth_attrs, true);
    let spec = FnSpec::parse(name, sig, meth_attrs).unwrap();

    match spec.tp {
        FnType::Fn => impl_py_method_def(name, doc, &spec, &impl_wrap(cls, name, &spec, true)),
        FnType::PySelf(ref self_ty) => impl_py_method_def(
            name,
            doc,
            &spec,
            &impl_wrap_pyslf(cls, name, &spec, self_ty, true),
        ),
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
    for param in &sig.decl.generics.params {
        match param {
            syn::GenericParam::Lifetime(_) => {}
            syn::GenericParam::Type(_) => panic!(
                "A Python method can't have a generic type parameter: {}",
                name
            ),
            syn::GenericParam::Const(_) => panic!(
                "A Python method can't have a const generic parameter: {}",
                name
            ),
        }
    }
}

/// Generate function wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap(
    cls: &syn::Type,
    name: &syn::Ident,
    spec: &FnSpec<'_>,
    noargs: bool,
) -> TokenStream {
    let body = impl_call(cls, name, &spec);
    let slf = impl_self(&quote! { &mut #cls });
    impl_wrap_common(cls, name, spec, noargs, slf, body)
}

pub fn impl_wrap_pyslf(
    cls: &syn::Type,
    name: &syn::Ident,
    spec: &FnSpec<'_>,
    self_ty: &syn::TypePath,
    noargs: bool,
) -> TokenStream {
    let names = get_arg_names(spec);
    let body = quote! {
        #cls::#name(_slf, #(#names),*)
    };
    let slf = impl_self(self_ty);
    impl_wrap_common(cls, name, spec, noargs, slf, body)
}

fn impl_wrap_common(
    cls: &syn::Type,
    name: &syn::Ident,
    spec: &FnSpec<'_>,
    noargs: bool,
    slf: TokenStream,
    body: TokenStream,
) -> TokenStream {
    if spec.args.is_empty() && noargs {
        quote! {
            unsafe extern "C" fn __wrap(
                _slf: *mut pyo3::ffi::PyObject
            ) -> *mut pyo3::ffi::PyObject
            {
                const _LOCATION: &'static str = concat!(
                    stringify!(#cls), ".", stringify!(#name), "()");
                let _pool = pyo3::GILPool::new();
                let _py = pyo3::Python::assume_gil_acquired();
                #slf
                let _result = {
                    pyo3::derive_utils::IntoPyResult::into_py_result(#body)
                };

                pyo3::callback::cb_convert(
                    pyo3::callback::PyObjectCallbackConverter, _py, _result)
            }
        }
    } else {
        let body = impl_arg_params(&spec, body);

        quote! {
            unsafe extern "C" fn __wrap(
                _slf: *mut pyo3::ffi::PyObject,
                _args: *mut pyo3::ffi::PyObject,
                _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
            {
                const _LOCATION: &'static str = concat!(
                    stringify!(#cls), ".", stringify!(#name), "()");
                let _pool = pyo3::GILPool::new();
                let _py = pyo3::Python::assume_gil_acquired();
                #slf
                let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                #body

                pyo3::callback::cb_convert(
                    pyo3::callback::PyObjectCallbackConverter, _py, _result)
            }
        }
    }
}

/// Generate function wrapper for protocol method (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_proto_wrap(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec<'_>) -> TokenStream {
    let cb = impl_call(cls, name, &spec);
    let body = impl_arg_params(&spec, cb);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = pyo3::Python::assume_gil_acquired();
            let _slf = _py.mut_from_borrowed_ptr::<#cls>(_slf);
            let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
            let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

            #body

            pyo3::callback::cb_convert(
                pyo3::callback::PyObjectCallbackConverter, _py, _result)
        }
    }
}

/// Generate class method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_new(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec<'_>) -> TokenStream {
    let names: Vec<syn::Ident> = get_arg_names(&spec);
    let cb = quote! { #cls::#name(&_obj, #(#names),*) };

    let body = impl_arg_params(spec, cb);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _cls: *mut pyo3::ffi::PyTypeObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            use pyo3::type_object::PyTypeInfo;

            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = pyo3::Python::assume_gil_acquired();
            match pyo3::type_object::PyRawObject::new(_py, #cls::type_object(), _cls) {
                Ok(_obj) => {
                    let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                    let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                    #body

                    match _result {
                        Ok(_) => pyo3::IntoPyPointer::into_ptr(_obj),
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
fn impl_wrap_init(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec<'_>) -> TokenStream {
    let cb = impl_call(cls, name, &spec);
    let output = &spec.output;
    let result_empty: syn::Type = syn::parse_quote!(PyResult<()>);
    let empty: syn::Type = syn::parse_quote!(());
    if output != &result_empty || output != &empty {
        panic!("Constructor must return PyResult<()> or a ()");
    }

    let body = impl_arg_params(&spec, cb);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> pyo3::libc::c_int
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = pyo3::Python::assume_gil_acquired();
            let _slf = _py.mut_from_borrowed_ptr::<#cls>(_slf);
            let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
            let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

            #body

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
pub fn impl_wrap_class(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec<'_>) -> TokenStream {
    let names: Vec<syn::Ident> = get_arg_names(&spec);
    let cb = quote! { #cls::#name(&_cls, #(#names),*) };

    let body = impl_arg_params(spec, cb);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _cls: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = pyo3::Python::assume_gil_acquired();
            let _cls = pyo3::types::PyType::from_type_ptr(_py, _cls as *mut pyo3::ffi::PyTypeObject);
            let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
            let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

            #body

            pyo3::callback::cb_convert(
                pyo3::callback::PyObjectCallbackConverter, _py, _result)
        }
    }
}

/// Generate static method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_static(cls: &syn::Type, name: &syn::Ident, spec: &FnSpec<'_>) -> TokenStream {
    let names: Vec<syn::Ident> = get_arg_names(&spec);
    let cb = quote! { #cls::#name(#(#names),*) };

    let body = impl_arg_params(spec, cb);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = pyo3::Python::assume_gil_acquired();
            let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
            let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

            #body

            pyo3::callback::cb_convert(
                pyo3::callback::PyObjectCallbackConverter, _py, _result)
        }
    }
}

/// Generate functiona wrapper (PyCFunction, PyCFunctionWithKeywords)
pub(crate) fn impl_wrap_getter(cls: &syn::Type, name: &syn::Ident) -> TokenStream {
    quote! {
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject, _: *mut ::std::os::raw::c_void) -> *mut pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");

            let _pool = pyo3::GILPool::new();
            let _py = pyo3::Python::assume_gil_acquired();
            let _slf = _py.mut_from_borrowed_ptr::<#cls>(_slf);

            let result = pyo3::derive_utils::IntoPyResult::into_py_result(_slf.#name());

            match result {
                Ok(val) => {
                    pyo3::IntoPyPointer::into_ptr(val.into_object(_py))
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
pub(crate) fn impl_wrap_setter(
    cls: &syn::Type,
    name: &syn::Ident,
    spec: &FnSpec<'_>,
) -> TokenStream {
    if spec.args.is_empty() {
        println!(
            "Not enough arguments for setter {}::{}",
            quote! {#cls},
            name
        );
    }
    let val_ty = spec.args[0].ty;

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _value: *mut pyo3::ffi::PyObject, _: *mut ::std::os::raw::c_void) -> pyo3::libc::c_int
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = pyo3::Python::assume_gil_acquired();
            let _slf = _py.mut_from_borrowed_ptr::<#cls>(_slf);
            let _value = _py.from_borrowed_ptr(_value);

            let _result = match <#val_ty as pyo3::FromPyObject>::extract(_value) {
                Ok(_val) => {
                    pyo3::derive_utils::IntoPyResult::into_py_result(_slf.#name(_val))
                }
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

/// This function abstracts away some copied code and can propably be simplified itself
pub fn get_arg_names(spec: &FnSpec) -> Vec<syn::Ident> {
    spec.args
        .iter()
        .enumerate()
        .map(|(pos, _)| syn::Ident::new(&format!("arg{}", pos), Span::call_site()))
        .collect()
}

fn impl_call(_cls: &syn::Type, fname: &syn::Ident, spec: &FnSpec<'_>) -> TokenStream {
    let names = get_arg_names(spec);
    quote! { _slf.#fname(#(#names),*) }
}

fn impl_self<T: quote::ToTokens>(self_ty: &T) -> TokenStream {
    quote! {
        let _slf: #self_ty = pyo3::FromPyPointer::from_borrowed_ptr(_py, _slf);
    }
}

/// Converts a bool to "true" or "false"
fn bool_to_ident(condition: bool) -> syn::Ident {
    if condition {
        syn::Ident::new("true", Span::call_site())
    } else {
        syn::Ident::new("false", Span::call_site())
    }
}

pub fn impl_arg_params(spec: &FnSpec<'_>, body: TokenStream) -> TokenStream {
    if spec.args.is_empty() {
        return quote! {
            let _result = {
                pyo3::derive_utils::IntoPyResult::into_py_result(#body)
            };
        };
    }

    let mut params = Vec::new();

    for arg in spec.args.iter() {
        if arg.py || spec.is_args(&arg.name) || spec.is_kwargs(&arg.name) {
            continue;
        }
        let name = arg.name;
        let kwonly = bool_to_ident(spec.is_kw_only(&arg.name));
        let opt = bool_to_ident(arg.optional.is_some() || spec.default_value(&arg.name).is_some());

        params.push(quote! {
            pyo3::derive_utils::ParamDescription {
                name: stringify!(#name),
                is_optional: #opt,
                kw_only: #kwonly
            }
        });
    }
    let placeholders: Vec<syn::Ident> = params
        .iter()
        .map(|_| syn::Ident::new("None", Span::call_site()))
        .collect();

    let mut param_conversion = Vec::new();
    let mut option_pos = 0;
    for (idx, arg) in spec.args.iter().enumerate() {
        param_conversion.push(impl_arg_param(&arg, &spec, idx, &mut option_pos));
    }

    let accept_args = bool_to_ident(spec.accept_args());
    let accept_kwargs = bool_to_ident(spec.accept_kwargs());

    // create array of arguments, and then parse
    quote! {
        use pyo3::ObjectProtocol;
        const PARAMS: &'static [pyo3::derive_utils::ParamDescription] = &[
            #(#params),*
        ];

        let mut output = [#(#placeholders),*];

        // Workaround to use the question mark operator without rewriting everything
        let _result = (|| {
            pyo3::derive_utils::parse_fn_args(
                Some(_LOCATION),
                PARAMS,
                &_args,
                _kwargs,
                #accept_args,
                #accept_kwargs,
                &mut output
            )?;

            #(#param_conversion)*

            pyo3::derive_utils::IntoPyResult::into_py_result(#body)
        })();
    }
}

/// Re option_pos: The option slice doesn't contain the py: Python argument, so the argument
/// index and the index in option diverge when using py: Python
fn impl_arg_param(
    arg: &FnArg<'_>,
    spec: &FnSpec<'_>,
    idx: usize,
    option_pos: &mut usize,
) -> TokenStream {
    let arg_name = syn::Ident::new(&format!("arg{}", idx), Span::call_site());

    if arg.py {
        return quote! {
            let #arg_name = _py;
        };
    }
    let arg_value = quote!(output[#option_pos]);
    *option_pos += 1;

    let ty = arg.ty;
    let name = arg.name;

    if spec.is_args(&name) {
        quote! {
            let #arg_name = <#ty as pyo3::FromPyObject>::extract(_args.as_ref())?;
        }
    } else if spec.is_kwargs(&name) {
        quote! {
            let #arg_name = _kwargs;
        }
    } else if arg.optional.is_some() {
        let default = if let Some(d) = spec.default_value(name) {
            quote! { Some(#d) }
        } else {
            quote! { None }
        };

        quote! {
            let #arg_name = match #arg_value.as_ref() {
                Some(_obj) => {
                    if _obj.is_none() {
                        #default
                    } else {
                        Some(_obj.extract()?)
                    }
                },
                None => #default
            };
        }
    } else if let Some(default) = spec.default_value(name) {
        quote! {
            let #arg_name = match #arg_value.as_ref() {
                Some(_obj) => {
                    if _obj.is_none() {
                        #default
                    } else {
                        _obj.extract()?
                    }
                },
                None => #default
            };
        }
    } else {
        quote! {
            let #arg_name = #arg_value.unwrap().extract()?;
        }
    }
}

pub fn impl_py_method_def(
    name: &syn::Ident,
    doc: syn::Lit,
    spec: &FnSpec<'_>,
    wrapper: &TokenStream,
) -> TokenStream {
    if spec.args.is_empty() {
        quote! {
            pyo3::class::PyMethodDefType::Method({
                #wrapper

                pyo3::class::PyMethodDef {
                    ml_name: stringify!(#name),
                    ml_meth: pyo3::class::PyMethodType::PyNoArgsFunction(__wrap),
                    ml_flags: pyo3::ffi::METH_NOARGS,
                    ml_doc: #doc,
                }
            })
        }
    } else {
        quote! {
            pyo3::class::PyMethodDefType::Method({
                #wrapper

                pyo3::class::PyMethodDef {
                    ml_name: stringify!(#name),
                    ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                    ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
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
        pyo3::class::PyMethodDefType::New({
            #wrapper

            pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: pyo3::class::PyMethodType::PyNewFunc(__wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
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
        pyo3::class::PyMethodDefType::Init({
            #wrapper

            pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: pyo3::class::PyMethodType::PyInitFunc(__wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
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
        pyo3::class::PyMethodDefType::Class({
            #wrapper

            pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS |
                pyo3::ffi::METH_CLASS,
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
        pyo3::class::PyMethodDefType::Static({
            #wrapper

            pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS | pyo3::ffi::METH_STATIC,
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
        pyo3::class::PyMethodDefType::Call({
            #wrapper

            pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
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
    let n = if let Some(ref name) = setter {
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
        pyo3::class::PyMethodDefType::Setter({
            #wrapper

            pyo3::class::PySetterDef {
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
    let n = if let Some(ref name) = getter {
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
        pyo3::class::PyMethodDefType::Getter({
            #wrapper

            pyo3::class::PyGetterDef {
                name: #n,
                meth: __wrap,
                doc: #doc,
            }
        })
    }
}
