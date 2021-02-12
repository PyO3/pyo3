// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::konst::ConstSpec;
use crate::method::{FnArg, FnSpec, FnType, SelfType};
use crate::utils;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{ext::IdentExt, spanned::Spanned};

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

    Ok(match &spec.tp {
        FnType::Fn(self_ty) => impl_py_method_def(&spec, &impl_wrap(cls, &spec, self_ty, true)),
        FnType::FnNew => impl_py_method_def_new(&spec, &impl_wrap_new(cls, &spec)),
        FnType::FnCall(self_ty) => {
            impl_py_method_def_call(&spec, &impl_wrap(cls, &spec, self_ty, false))
        }
        FnType::FnClass => impl_py_method_def_class(&spec, &impl_wrap_class(cls, &spec)),
        FnType::FnStatic => impl_py_method_def_static(&spec, &impl_wrap_static(cls, &spec)),
        FnType::ClassAttribute => {
            impl_py_method_class_attribute(&spec, &impl_wrap_class_attribute(cls, &spec))
        }
        FnType::Getter(self_ty) => impl_py_getter_def(
            &spec.python_name,
            &spec.doc,
            &impl_wrap_getter(cls, PropertyType::Function(&spec), self_ty)?,
        ),
        FnType::Setter(self_ty) => impl_py_setter_def(
            &spec.python_name,
            &spec.doc,
            &impl_wrap_setter(cls, PropertyType::Function(&spec), self_ty)?,
        ),
    })
}

fn check_generic(sig: &syn::Signature) -> syn::Result<()> {
    let err_msg = |typ| format!("a Python method can't have a generic {} parameter", typ);
    for param in &sig.generics.params {
        match param {
            syn::GenericParam::Lifetime(_) => {}
            syn::GenericParam::Type(_) => bail_spanned!(param.span() => err_msg("type")),
            syn::GenericParam::Const(_) => bail_spanned!(param.span() => err_msg("const")),
        }
    }
    Ok(())
}

pub fn gen_py_const(
    cls: &syn::Type,
    name: &syn::Ident,
    attrs: &mut Vec<syn::Attribute>,
) -> syn::Result<Option<TokenStream>> {
    let spec = ConstSpec::parse(name, attrs)?;
    if spec.is_class_attr {
        let wrapper = quote! {
            fn __wrap(py: pyo3::Python<'_>) -> pyo3::PyObject {
                pyo3::IntoPy::into_py(#cls::#name, py)
            }
        };
        return Ok(Some(impl_py_const_class_attribute(&spec, &wrapper)));
    }
    Ok(None)
}

/// Generate function wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    self_ty: &SelfType,
    noargs: bool,
) -> TokenStream {
    let body = impl_call(cls, &spec);
    let slf = self_ty.receiver(cls);
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
                pyo3::callback_body_without_convert!(_py, {
                    #slf
                    pyo3::callback::convert(_py, #body)
                })
            }
        }
    } else {
        let body = impl_arg_params(&spec, Some(cls), body);

        quote! {
            unsafe extern "C" fn __wrap(
                _slf: *mut pyo3::ffi::PyObject,
                _args: *mut pyo3::ffi::PyObject,
                _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
            {
                const _LOCATION: &'static str = concat!(
                    stringify!(#cls), ".", stringify!(#python_name), "()");
                pyo3::callback_body_without_convert!(_py, {
                    #slf
                    let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                    let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                    pyo3::callback::convert(_py, #body)
                })
            }
        }
    }
}

/// Generate function wrapper for protocol method (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_proto_wrap(cls: &syn::Type, spec: &FnSpec<'_>, self_ty: &SelfType) -> TokenStream {
    let python_name = &spec.python_name;
    let cb = impl_call(cls, &spec);
    let body = impl_arg_params(&spec, Some(cls), cb);
    let slf = self_ty.receiver(cls);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            pyo3::callback_body_without_convert!(_py, {
                #slf
                let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                pyo3::callback::convert(_py, #body)
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
    let body = impl_arg_params(spec, Some(cls), cb);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            subtype: *mut pyo3::ffi::PyTypeObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            use pyo3::type_object::PyTypeInfo;
            use pyo3::callback::IntoPyCallbackOutput;
            use std::convert::TryFrom;

            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            pyo3::callback_body_without_convert!(_py, {
                let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                let initializer: pyo3::PyClassInitializer::<#cls> = #body.convert(_py)?;
                let cell = initializer.create_cell_from_subtype(_py, subtype)?;
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

    let body = impl_arg_params(spec, Some(cls), cb);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _cls: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            pyo3::callback_body_without_convert!(_py, {
                let _cls = pyo3::types::PyType::from_type_ptr(_py, _cls as *mut pyo3::ffi::PyTypeObject);
                let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                pyo3::callback::convert(_py, #body)
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

    let body = impl_arg_params(spec, Some(cls), cb);

    quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            pyo3::callback_body_without_convert!(_py, {
                let _args = _py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = _py.from_borrowed_ptr_or_opt(_kwargs);

                pyo3::callback::convert(_py, #body)
            })
        }
    }
}

/// Generate a wrapper for initialization of a class attribute from a method
/// annotated with `#[classattr]`.
/// To be called in `pyo3::pyclass::initialize_type_object`.
pub fn impl_wrap_class_attribute(cls: &syn::Type, spec: &FnSpec<'_>) -> TokenStream {
    let name = &spec.name;
    let cb = quote! { #cls::#name() };

    quote! {
        fn __wrap(py: pyo3::Python<'_>) -> pyo3::PyObject {
            pyo3::IntoPy::into_py(#cb, py)
        }
    }
}

fn impl_call_getter(cls: &syn::Type, spec: &FnSpec) -> syn::Result<TokenStream> {
    let (py_arg, args) = split_off_python_arg(&spec.args);
    ensure_spanned!(
        args.is_empty(),
        args[0].ty.span() => "getter function can only have one argument (of type pyo3::Python)"
    );

    let name = &spec.name;
    let fncall = if py_arg.is_some() {
        quote!(#cls::#name(_slf, _py))
    } else {
        quote!(#cls::#name(_slf))
    };

    Ok(fncall)
}

/// Generate a function wrapper called `__wrap` for a property getter
pub(crate) fn impl_wrap_getter(
    cls: &syn::Type,
    property_type: PropertyType,
    self_ty: &SelfType,
) -> syn::Result<TokenStream> {
    let (python_name, getter_impl) = match property_type {
        PropertyType::Descriptor(field) => {
            let name = field.ident.as_ref().unwrap();
            (
                name.unraw(),
                quote!({
                    _slf.#name.clone()
                }),
            )
        }
        PropertyType::Function(spec) => (spec.python_name.clone(), impl_call_getter(cls, spec)?),
    };

    let slf = self_ty.receiver(cls);
    Ok(quote! {
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject, _: *mut std::os::raw::c_void) -> *mut pyo3::ffi::PyObject
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            pyo3::callback_body_without_convert!(_py, {
                #slf
                pyo3::callback::convert(_py, #getter_impl)
            })
        }
    })
}

fn impl_call_setter(cls: &syn::Type, spec: &FnSpec) -> syn::Result<TokenStream> {
    let (py_arg, args) = split_off_python_arg(&spec.args);

    if args.is_empty() {
        bail_spanned!(spec.name.span() => "setter function expected to have one argument");
    } else if args.len() > 1 {
        bail_spanned!(
            args[1].ty.span() =>
            "setter function can have at most two arguments ([pyo3::Python,] and value)"
        );
    }

    let name = &spec.name;
    let fncall = if py_arg.is_some() {
        quote!(#cls::#name(_slf, _py, _val))
    } else {
        quote!(#cls::#name(_slf, _val))
    };

    Ok(fncall)
}

/// Generate a function wrapper called `__wrap` for a property setter
pub(crate) fn impl_wrap_setter(
    cls: &syn::Type,
    property_type: PropertyType,
    self_ty: &SelfType,
) -> syn::Result<TokenStream> {
    let (python_name, setter_impl) = match property_type {
        PropertyType::Descriptor(field) => {
            let name = field.ident.as_ref().unwrap();
            (name.unraw(), quote!({ _slf.#name = _val; }))
        }
        PropertyType::Function(spec) => (spec.python_name.clone(), impl_call_setter(cls, spec)?),
    };

    let slf = self_ty.receiver(cls);
    Ok(quote! {
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _value: *mut pyo3::ffi::PyObject, _: *mut std::os::raw::c_void) -> std::os::raw::c_int
        {
            const _LOCATION: &'static str = concat!(stringify!(#cls),".",stringify!(#python_name),"()");
            pyo3::callback_body_without_convert!(_py, {
                #slf
                let _value = _py.from_borrowed_ptr::<pyo3::types::PyAny>(_value);
                let _val = pyo3::FromPyObject::extract(_value)?;

                pyo3::callback::convert(_py, #setter_impl)
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

fn impl_call(cls: &syn::Type, spec: &FnSpec<'_>) -> TokenStream {
    let fname = &spec.name;
    let names = get_arg_names(spec);
    quote! { #cls::#fname(_slf, #(#names),*) }
}

pub fn impl_arg_params(
    spec: &FnSpec<'_>,
    self_: Option<&syn::Type>,
    body: TokenStream,
) -> TokenStream {
    if spec.args.is_empty() {
        return quote! {
            #body
        };
    }

    let mut params = Vec::new();

    for arg in spec.args.iter() {
        if arg.py || spec.is_args(&arg.name) || spec.is_kwargs(&arg.name) {
            continue;
        }
        let name = arg.name.unraw().to_string();
        let kwonly = spec.is_kw_only(&arg.name);
        let opt = arg.optional.is_some() || spec.default_value(&arg.name).is_some();

        params.push(quote! {
            pyo3::derive_utils::ParamDescription {
                name: #name,
                is_optional: #opt,
                kw_only: #kwonly
            }
        });
    }

    let mut param_conversion = Vec::new();
    let mut option_pos = 0;
    for (idx, arg) in spec.args.iter().enumerate() {
        param_conversion.push(impl_arg_param(&arg, &spec, idx, self_, &mut option_pos));
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
    quote! {{
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

        #body
    }}
}

/// Re option_pos: The option slice doesn't contain the py: Python argument, so the argument
/// index and the index in option diverge when using py: Python
fn impl_arg_param(
    arg: &FnArg<'_>,
    spec: &FnSpec<'_>,
    idx: usize,
    self_: Option<&syn::Type>,
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
    let transform_error = quote! {
        |e| pyo3::derive_utils::argument_extraction_error(_py, stringify!(#name), e)
    };

    if spec.is_args(&name) {
        return quote! {
            let #arg_name = <#ty as pyo3::FromPyObject>::extract(_args.as_ref())
                .map_err(#transform_error)?;
        };
    } else if spec.is_kwargs(&name) {
        return quote! {
            let #arg_name = _kwargs;
        };
    }
    let arg_value = quote!(output[#option_pos]);
    *option_pos += 1;

    let default = match (spec.default_value(name), arg.optional.is_some()) {
        (Some(default), true) if default.to_string() != "None" => quote! { Some(#default) },
        (Some(default), _) => quote! { #default },
        (None, true) => quote! { None },
        (None, false) => quote! { panic!("Failed to extract required method argument") },
    };

    return if let syn::Type::Reference(tref) = arg.optional.as_ref().unwrap_or(&ty) {
        let (tref, mut_) = preprocess_tref(tref, self_);
        let (target_ty, borrow_tmp) = if arg.optional.is_some() {
            // Get Option<&T> from Option<PyRef<T>>
            (
                quote! { Option<<#tref as pyo3::derive_utils::ExtractExt>::Target> },
                if mut_.is_some() {
                    quote! { _tmp.as_deref_mut() }
                } else {
                    quote! { _tmp.as_deref() }
                },
            )
        } else {
            // Get &T from PyRef<T>
            (
                quote! { <#tref as pyo3::derive_utils::ExtractExt>::Target },
                quote! { &#mut_ *_tmp },
            )
        };

        quote! {
            let #mut_ _tmp: #target_ty = match #arg_value {
                Some(_obj) => _obj.extract().map_err(#transform_error)?,
                None => #default,
            };
            let #arg_name = #borrow_tmp;
        }
    } else {
        quote! {
            let #arg_name = match #arg_value {
                Some(_obj) => _obj.extract().map_err(#transform_error)?,
                None => #default,
            };
        }
    };

    /// Replace `Self`, remove lifetime and get mutability from the type
    fn preprocess_tref(
        tref: &syn::TypeReference,
        self_: Option<&syn::Type>,
    ) -> (syn::TypeReference, Option<syn::token::Mut>) {
        let mut tref = tref.to_owned();
        if let Some(syn::Type::Path(tpath)) = self_ {
            replace_self(&mut tref, &tpath.path);
        }
        tref.lifetime = None;
        let mut_ = tref.mutability;
        (tref, mut_)
    }

    /// Replace `Self` with the exact type name since it is used out of the impl block
    fn replace_self(tref: &mut syn::TypeReference, self_path: &syn::Path) {
        match &mut *tref.elem {
            syn::Type::Reference(tref_inner) => replace_self(tref_inner, self_path),
            syn::Type::Path(tpath) => {
                if let Some(ident) = tpath.path.get_ident() {
                    if ident == "Self" {
                        tpath.path = self_path.to_owned();
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn impl_py_method_def(spec: &FnSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    let doc = &spec.doc;
    if spec.args.is_empty() {
        quote! {
            pyo3::class::PyMethodDefType::Method({
                #wrapper

                pyo3::class::PyMethodDef::cfunction(
                    concat!(stringify!(#python_name), "\0"),
                    __wrap,
                    #doc
                )
            })
        }
    } else {
        quote! {
            pyo3::class::PyMethodDefType::Method({
                #wrapper

                pyo3::class::PyMethodDef::cfunction_with_keywords(
                    concat!(stringify!(#python_name), "\0"),
                    __wrap,
                    0,
                    #doc
                )
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

            pyo3::class::PyMethodDef::new_func(concat!(stringify!(#python_name), "\0"), __wrap, #doc)
        })
    }
}

pub fn impl_py_method_def_class(spec: &FnSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    let doc = &spec.doc;
    quote! {
        pyo3::class::PyMethodDefType::Class({
            #wrapper

            pyo3::class::PyMethodDef::cfunction_with_keywords(
                concat!(stringify!(#python_name), "\0"),
                __wrap,
                pyo3::ffi::METH_CLASS,
                #doc
            )
        })
    }
}

pub fn impl_py_method_def_static(spec: &FnSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    let doc = &spec.doc;
    quote! {
        pyo3::class::PyMethodDefType::Static({
            #wrapper

            pyo3::class::PyMethodDef::cfunction_with_keywords(
                concat!(stringify!(#python_name), "\0"),
                __wrap,
                pyo3::ffi::METH_STATIC,
                #doc
            )
        })
    }
}

pub fn impl_py_method_class_attribute(spec: &FnSpec<'_>, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    quote! {
        pyo3::class::PyMethodDefType::ClassAttribute({
            #wrapper

            pyo3::class::PyClassAttributeDef::new(concat!(stringify!(#python_name), "\0"), __wrap)
        })
    }
}

pub fn impl_py_const_class_attribute(spec: &ConstSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    quote! {
        pyo3::class::PyMethodDefType::ClassAttribute({
            #wrapper

            pyo3::class::PyClassAttributeDef::new(concat!(stringify!(#python_name), "\0"), __wrap)
        })
    }
}

pub fn impl_py_method_def_call(spec: &FnSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name;
    let doc = &spec.doc;
    quote! {
        pyo3::class::PyMethodDefType::Call({
            #wrapper

            pyo3::class::PyMethodDef::call_func(
                concat!(stringify!(#python_name), "\0"),
                __wrap,
                pyo3::ffi::METH_STATIC,
                #doc
            )
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

            pyo3::class::PySetterDef::new(concat!(stringify!(#python_name), "\0"), __wrap, #doc)
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

            pyo3::class::PyGetterDef::new(concat!(stringify!(#python_name), "\0"), __wrap, #doc)
        })
    }
}

/// Split an argument of pyo3::Python from the front of the arg list, if present
fn split_off_python_arg<'a>(args: &'a [FnArg<'a>]) -> (Option<&FnArg>, &[FnArg]) {
    if args
        .get(0)
        .map(|py| utils::is_python(&py.ty))
        .unwrap_or(false)
    {
        (Some(&args[0]), &args[1..])
    } else {
        (None, args)
    }
}
