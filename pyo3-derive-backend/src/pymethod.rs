// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::method::{FnArg, FnSpec, FnType};
use crate::utils;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::ext::IdentExt;

pub enum PropertyType<'a> {
    Descriptor(&'a syn::Field),
    Function(&'a FnSpec<'a>),
}

pub fn gen_py_method(
    cls: &syn::Type,
    sig: &mut syn::Signature,
    meth_attrs: &mut Vec<syn::Attribute>,
) -> syn::Result<TokenStream> {
    check_generic(sig)?;
    let spec = FnSpec::parse(sig, &mut *meth_attrs, true)?;

    Ok(match spec.tp {
        FnType::Fn => impl_py_method_def(&spec, &impl_wrap(cls, &spec, true)),
        FnType::PySelfRef(ref self_ty) => {
            impl_py_method_def(&spec, &impl_wrap_pyslf(cls, &spec, self_ty, true))
        }
        FnType::PySelfPath(ref self_ty) => {
            impl_py_method_def(&spec, &impl_wrap_pyslf(cls, &spec, self_ty, true))
        }
        FnType::FnNew => impl_py_method_def_new(&spec, &impl_wrap_new(cls, &spec)),
        FnType::FnCall => impl_py_method_def_call(&spec, &impl_wrap(cls, &spec, false)),
        FnType::FnClass => impl_py_method_def_class(&spec, &impl_wrap_class(cls, &spec)),
        FnType::FnStatic => impl_py_method_def_static(&spec, &impl_wrap_static(cls, &spec)),
        FnType::Getter => impl_py_getter_def(
            &spec.python_name,
            &spec.doc,
            &impl_wrap_getter(cls, PropertyType::Function(&spec))?,
        ),
        FnType::Setter => impl_py_setter_def(
            &spec.python_name,
            &spec.doc,
            &impl_wrap_setter(cls, PropertyType::Function(&spec))?,
        ),
    })
}

fn check_generic(sig: &syn::Signature) -> syn::Result<()> {
    let err_msg = |typ| format!("A Python method can't have a generic {} parameter", typ);
    for param in &sig.generics.params {
        match param {
            syn::GenericParam::Lifetime(_) => {}
            syn::GenericParam::Type(_) => {
                return Err(syn::Error::new_spanned(param, err_msg("type")));
            }
            syn::GenericParam::Const(_) => {
                return Err(syn::Error::new_spanned(param, err_msg("const")));
            }
        }
    }
    Ok(())
}

/// Generate function wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap(cls: &syn::Type, spec: &FnSpec<'_>, noargs: bool) -> TokenStream {
    let body = impl_call(cls, &spec);
    let borrow_self = spec.borrow_self();
    let slf = quote! {
        let _slf = _py.from_borrowed_ptr::<pyo3::PyCell<#cls>>(_slf);
        #borrow_self
    };
    impl_wrap_common(cls, spec, noargs, slf, body)
}

pub fn impl_wrap_pyslf(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    self_ty: impl quote::ToTokens,
    noargs: bool,
) -> TokenStream {
    let names = get_arg_names(spec);
    let name = &spec.name;
    let body = quote! {
        #cls::#name(_slf, #(#names),*)
    };
    let slf = quote! {
        let _cell = _py.from_borrowed_ptr::<pyo3::PyCell<#cls>>(_slf);
        let _slf: #self_ty = std::convert::TryFrom::try_from(_cell)?;
    };
    impl_wrap_common(cls, spec, noargs, slf, body)
}

fn impl_wrap_common(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    noargs: bool,
    slf: TokenStream,
    body: TokenStream,
) -> TokenStream {
    let python_name = &spec.python_name;
    if spec.args.is_empty() && noargs {
        quote! {
            unsafe extern "C" fn __wrap(
                _slf: *mut pyo3::ffi::PyObject,
                _args: *mut pyo3::ffi::PyObject,
            ) -> *mut pyo3::ffi::PyObject
            {
                const _LOCATION: &'static str = concat!(
                    stringify!(#cls), ".", stringify!(#python_name), "()");
                let _pool = pyo3::GILPool::new();
                let _py = _pool.python();
                pyo3::run_callback(_py, || {
                    #slf
                    pyo3::callback::convert(_py, #body)
                })
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
                    stringify!(#cls), ".", stringify!(#python_name), "()");
                let _pool = pyo3::GILPool::new();
                let _py = _pool.python();
                pyo3::run_callback(_py, || {
                    #slf
                    let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                    let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                    #body

                    pyo3::callback::convert(_py, _result)
                })
            }
        }
    }
}

/// Generate function wrapper for protocol method (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_proto_wrap(cls: &syn::Type, spec: &FnSpec<'_>) -> TokenStream {
    let python_name = &spec.python_name;
    let cb = impl_call(cls, &spec);
    let body = impl_arg_params(&spec, cb);
    let borrow_self = spec.borrow_self();

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = _pool.python();
            pyo3::run_callback(_py, || {
                let _slf = _py.from_borrowed_ptr::<pyo3::PyCell<#cls>>(_slf);
                #borrow_self
                let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                #body

                pyo3::callback::convert(_py, _result)
            })
        }
    }
}

/// Generate class method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_new(cls: &syn::Type, spec: &FnSpec<'_>) -> TokenStream {
    let name = &spec.name;
    let python_name = &spec.python_name;
    let names: Vec<syn::Ident> = get_arg_names(&spec);
    let cb = quote! { #cls::#name(#(#names),*) };
    let body = impl_arg_params_(
        spec,
        cb,
        quote! { pyo3::derive_utils::IntoPyNewResult::into_pynew_result },
    );

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _cls: *mut pyo3::ffi::PyTypeObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            use pyo3::type_object::PyTypeInfo;

            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = _pool.python();
            pyo3::run_callback(_py, || {
                let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                #body

                let cell = pyo3::PyClassInitializer::from(_result?).create_cell(_py)?;
                Ok(cell as *mut pyo3::ffi::PyObject)
            })
        }
    }
}

/// Generate class method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_class(cls: &syn::Type, spec: &FnSpec<'_>) -> TokenStream {
    let name = &spec.name;
    let python_name = &spec.python_name;
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
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = _pool.python();
            pyo3::run_callback(_py, || {
                let _cls = pyo3::types::PyType::from_type_ptr(_py, _cls as *mut pyo3::ffi::PyTypeObject);
                let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                #body

                pyo3::callback::convert(_py, _result)
            })
        }
    }
}

/// Generate static method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_static(cls: &syn::Type, spec: &FnSpec<'_>) -> TokenStream {
    let name = &spec.name;
    let python_name = &spec.python_name;
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
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = _pool.python();
            pyo3::run_callback(_py, || {
                let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                #body

                pyo3::callback::convert(_py, _result)
            })
        }
    }
}

fn impl_call_getter(spec: &FnSpec) -> syn::Result<TokenStream> {
    let (py_arg, args) = split_off_python_arg(&spec.args);
    if !args.is_empty() {
        return Err(syn::Error::new_spanned(
            args[0].ty,
            "Getter function can only have one argument of type pyo3::Python",
        ));
    }

    let name = &spec.name;
    let fncall = if py_arg.is_some() {
        quote! { _slf.#name(_py) }
    } else {
        quote! { _slf.#name() }
    };
    Ok(fncall)
}

/// Generate a function wrapper called `__wrap` for a property getter
pub(crate) fn impl_wrap_getter(
    cls: &syn::Type,
    property_type: PropertyType,
) -> syn::Result<TokenStream> {
    let (python_name, getter_impl) = match property_type {
        PropertyType::Descriptor(field) => {
            let name = field.ident.as_ref().unwrap();
            (
                name.unraw(),
                quote!({
                    use pyo3::derive_utils::GetPropertyValue;
                    (&_slf.#name).get_property_value(_py)
                }),
            )
        }
        PropertyType::Function(spec) => (spec.python_name.clone(), impl_call_getter(&spec)?),
    };

    let borrow_self = crate::utils::borrow_self(false);
    Ok(quote! {
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject, _: *mut ::std::os::raw::c_void) -> *mut pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");

            let _pool = pyo3::GILPool::new();
            let _py = _pool.python();
            pyo3::run_callback(_py, || {
                let _slf = _py.from_borrowed_ptr::<pyo3::PyCell<#cls>>(_slf);
                #borrow_self
                pyo3::callback::convert(_py, #getter_impl)
            })
        }
    })
}

fn impl_call_setter(spec: &FnSpec) -> syn::Result<TokenStream> {
    let (py_arg, args) = split_off_python_arg(&spec.args);

    if args.is_empty() {
        return Err(syn::Error::new_spanned(
            &spec.name,
            "Setter function expected to have one argument",
        ));
    } else if args.len() > 1 {
        return Err(syn::Error::new_spanned(
            &args[1].ty,
            "Setter function can have at most two arguments: one of pyo3::Python, and one other",
        ));
    }

    let name = &spec.name;
    let fncall = if py_arg.is_some() {
        quote!(pyo3::derive_utils::IntoPyResult::into_py_result(_slf.#name(_py, _val))?;)
    } else {
        quote!(pyo3::derive_utils::IntoPyResult::into_py_result(_slf.#name(_val))?;)
    };

    Ok(fncall)
}

/// Generate a function wrapper called `__wrap` for a property setter
pub(crate) fn impl_wrap_setter(
    cls: &syn::Type,
    property_type: PropertyType,
) -> syn::Result<TokenStream> {
    let (python_name, setter_impl) = match property_type {
        PropertyType::Descriptor(field) => {
            let name = field.ident.as_ref().unwrap();
            (name.unraw(), quote!(_slf.#name = _val;))
        }
        PropertyType::Function(spec) => (spec.python_name.clone(), impl_call_setter(&spec)?),
    };

    let borrow_self = crate::utils::borrow_self(true);
    Ok(quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _value: *mut pyo3::ffi::PyObject, _: *mut ::std::os::raw::c_void) -> pyo3::libc::c_int
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            let _pool = pyo3::GILPool::new();
            let _py = _pool.python();
            pyo3::run_callback(_py, || {
                let _slf = _py.from_borrowed_ptr::<pyo3::PyCell<#cls>>(_slf);
                #borrow_self
                let _value = _py.from_borrowed_ptr::<pyo3::types::PyAny>(_value);
                let _val = pyo3::FromPyObject::extract(_value)?;
                pyo3::callback::convert(_py, {#setter_impl})
            })
        }
    })
}

/// This function abstracts away some copied code and can propably be simplified itself
pub fn get_arg_names(spec: &FnSpec) -> Vec<syn::Ident> {
    (0..spec.args.len())
        .map(|pos| syn::Ident::new(&format!("arg{}", pos), Span::call_site()))
        .collect()
}

fn impl_call(_cls: &syn::Type, spec: &FnSpec<'_>) -> TokenStream {
    let fname = &spec.name;
    let names = get_arg_names(spec);
    quote! { _slf.#fname(#(#names),*) }
}

fn impl_arg_params_(spec: &FnSpec<'_>, body: TokenStream, into_result: TokenStream) -> TokenStream {
    if spec.args.is_empty() {
        return quote! {
            let _result = {
                #into_result (#body)
            };
        };
    }

    let mut params = Vec::new();

    for arg in spec.args.iter() {
        if arg.py || spec.is_args(&arg.name) || spec.is_kwargs(&arg.name) {
            continue;
        }
        let name = arg.name;
        let kwonly = spec.is_kw_only(&arg.name);
        let opt = arg.optional.is_some() || spec.default_value(&arg.name).is_some();

        params.push(quote! {
            pyo3::derive_utils::ParamDescription {
                name: stringify!(#name),
                is_optional: #opt,
                kw_only: #kwonly
            }
        });
    }

    let mut param_conversion = Vec::new();
    let mut option_pos = 0;
    for (idx, arg) in spec.args.iter().enumerate() {
        param_conversion.push(impl_arg_param(&arg, &spec, idx, &mut option_pos));
    }

    let (mut accept_args, mut accept_kwargs) = (false, false);

    for s in spec.attrs.iter() {
        use crate::pyfunction::Argument;
        match s {
            Argument::VarArgs(_) => accept_args = true,
            Argument::KeywordArgs(_) => accept_kwargs = true,
            _ => continue,
        }
    }
    let num_normal_params = params.len();
    // create array of arguments, and then parse
    quote! {
        use pyo3::ObjectProtocol;
        const PARAMS: &'static [pyo3::derive_utils::ParamDescription] = &[
            #(#params),*
        ];

        let mut output = [None; #num_normal_params];
        let mut _args = _args;
        let mut _kwargs = _kwargs;

        let (_args, _kwargs) = pyo3::derive_utils::parse_fn_args(
            Some(_LOCATION),
            PARAMS,
            _args,
            _kwargs,
            #accept_args,
            #accept_kwargs,
            &mut output
        )?;

        #(#param_conversion)*

        let _result = #into_result(#body);
    }
}

pub fn impl_arg_params(spec: &FnSpec<'_>, body: TokenStream) -> TokenStream {
    impl_arg_params_(
        spec,
        body,
        quote! { pyo3::derive_utils::IntoPyResult::into_py_result },
    )
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

    let ty = arg.ty;
    let name = arg.name;

    if spec.is_args(&name) {
        return quote! {
            let #arg_name = <#ty as pyo3::FromPyObject>::extract(_args.as_ref())?;
        };
    } else if spec.is_kwargs(&name) {
        return quote! {
            let #arg_name = _kwargs;
        };
    }
    let arg_value = quote!(output[#option_pos]);
    *option_pos += 1;

    return if let Some(ty) = arg.optional.as_ref() {
        let default = if let Some(d) = spec.default_value(name).filter(|d| d.to_string() != "None")
        {
            quote! { Some(#d) }
        } else {
            quote! { None }
        };
        if let syn::Type::Reference(tref) = ty {
            let (tref, mut_) = tref_preprocess(tref);
            let as_deref = if mut_.is_some() {
                quote! { as_deref_mut }
            } else {
                quote! { as_deref }
            };
            // Get Option<&T> from Option<PyRef<T>>
            quote! {
                let #mut_ _tmp = match #arg_value.as_ref().filter(|obj| !obj.is_none()) {
                    Some(_obj) => {
                        Some(_obj.extract::<<#tref as pyo3::derive_utils::ExtractExt>::Target>()?)
                    },
                    None => #default,
                };
                let #arg_name = _tmp.#as_deref();
            }
        } else {
            quote! {
                let #arg_name = match #arg_value.as_ref().filter(|obj| !obj.is_none()) {
                    Some(_obj) => Some(_obj.extract()?),
                    None => #default,
                };
            }
        }
    } else if let Some(default) = spec.default_value(name) {
        quote! {
            let #arg_name = match #arg_value.as_ref().filter(|obj| !obj.is_none()) {
                Some(_obj) => _obj.extract()?,
                None => #default,
            };
        }
    } else if let syn::Type::Reference(tref) = arg.ty {
        let (tref, mut_) = tref_preprocess(tref);
        // Get &T from PyRef<T>
        quote! {
            let #mut_ _tmp: <#tref as pyo3::derive_utils::ExtractExt>::Target
                = #arg_value.unwrap().extract()?;
            let #arg_name = &#mut_ *_tmp;
        }
    } else {
        quote! {
            let #arg_name = #arg_value.unwrap().extract()?;
        }
    };

    fn tref_preprocess(tref: &syn::TypeReference) -> (syn::TypeReference, Option<syn::token::Mut>) {
        let mut tref = tref.to_owned();
        tref.lifetime = None;
        let mut_ = tref.mutability;
        (tref, mut_)
    }
}

pub fn impl_py_method_def(spec: &FnSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    let doc = &spec.doc;
    if spec.args.is_empty() {
        quote! {
            pyo3::class::PyMethodDefType::Method({
                #wrapper

                pyo3::class::PyMethodDef {
                    ml_name: stringify!(#python_name),
                    ml_meth: pyo3::class::PyMethodType::PyCFunction(__wrap),
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
                    ml_name: stringify!(#python_name),
                    ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                    ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
                    ml_doc: #doc,
                }
            })
        }
    }
}

pub fn impl_py_method_def_new(spec: &FnSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    let doc = &spec.doc;
    quote! {
        pyo3::class::PyMethodDefType::New({
            #wrapper

            pyo3::class::PyMethodDef {
                ml_name: stringify!(#python_name),
                ml_meth: pyo3::class::PyMethodType::PyNewFunc(__wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
                ml_doc: #doc,
            }
        })
    }
}

pub fn impl_py_method_def_class(spec: &FnSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    let doc = &spec.doc;
    quote! {
        pyo3::class::PyMethodDefType::Class({
            #wrapper

            pyo3::class::PyMethodDef {
                ml_name: stringify!(#python_name),
                ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS |
                pyo3::ffi::METH_CLASS,
                ml_doc: #doc,
            }
        })
    }
}

pub fn impl_py_method_def_static(spec: &FnSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    let doc = &spec.doc;
    quote! {
        pyo3::class::PyMethodDefType::Static({
            #wrapper

            pyo3::class::PyMethodDef {
                ml_name: stringify!(#python_name),
                ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS | pyo3::ffi::METH_STATIC,
                ml_doc: #doc,
            }
        })
    }
}

pub fn impl_py_method_def_call(spec: &FnSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    let doc = &spec.doc;
    quote! {
        pyo3::class::PyMethodDefType::Call({
            #wrapper

            pyo3::class::PyMethodDef {
                ml_name: stringify!(#python_name),
                ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(__wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
                ml_doc: #doc,
            }
        })
    }
}

pub(crate) fn impl_py_setter_def(
    python_name: &syn::Ident,
    doc: &syn::LitStr,
    wrapper: &TokenStream,
) -> TokenStream {
    quote! {
        pyo3::class::PyMethodDefType::Setter({
            #wrapper

            pyo3::class::PySetterDef {
                name: stringify!(#python_name),
                meth: __wrap,
                doc: #doc,
            }
        })
    }
}

pub(crate) fn impl_py_getter_def(
    python_name: &syn::Ident,
    doc: &syn::LitStr,
    wrapper: &TokenStream,
) -> TokenStream {
    quote! {
        pyo3::class::PyMethodDefType::Getter({
            #wrapper

            pyo3::class::PyGetterDef {
                name: stringify!(#python_name),
                meth: __wrap,
                doc: #doc,
            }
        })
    }
}

/// Split an argument of pyo3::Python from the front of the arg list, if present
fn split_off_python_arg<'a>(args: &'a [FnArg<'a>]) -> (Option<&FnArg>, &[FnArg]) {
    match args {
        [py, rest @ ..] if utils::if_type_is_python(&py.ty) => (Some(py), rest),
        rest => (None, rest),
    }
}
