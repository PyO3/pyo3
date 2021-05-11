// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::utils;
use crate::{attributes::FromPyWithAttribute, konst::ConstSpec};
use crate::{
    method::{FnArg, FnSpec, FnType, SelfType},
    pyfunction::PyFunctionOptions,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{ext::IdentExt, spanned::Spanned, Result};

#[derive(Clone, Copy)]
pub enum PropertyType<'a> {
    Descriptor(&'a syn::Ident),
    Function(&'a FnSpec<'a>),
}

pub enum GeneratedPyMethod {
    Method(TokenStream),
    New(TokenStream),
    Call(TokenStream),
}

pub fn gen_py_method(
    cls: &syn::Type,
    sig: &mut syn::Signature,
    meth_attrs: &mut Vec<syn::Attribute>,
    options: PyFunctionOptions,
) -> Result<GeneratedPyMethod> {
    check_generic(sig)?;
    let spec = FnSpec::parse(sig, &mut *meth_attrs, options)?;

    Ok(match &spec.tp {
        FnType::Fn(self_ty) => {
            GeneratedPyMethod::Method(impl_py_method_def(cls, &spec, self_ty, None)?)
        }
        FnType::FnNew => GeneratedPyMethod::New(impl_py_method_def_new(cls, &spec)?),
        FnType::FnCall(self_ty) => {
            GeneratedPyMethod::Call(impl_py_method_def_call(cls, &spec, self_ty)?)
        }
        FnType::FnClass => GeneratedPyMethod::Method(impl_py_method_def_class(cls, &spec)?),
        FnType::FnStatic => GeneratedPyMethod::Method(impl_py_method_def_static(cls, &spec)?),
        FnType::ClassAttribute => {
            GeneratedPyMethod::Method(impl_py_method_class_attribute(cls, &spec))
        }
        FnType::Getter(self_ty) => GeneratedPyMethod::Method(impl_py_getter_def(
            cls,
            PropertyType::Function(&spec),
            self_ty,
            &spec.doc,
        )?),
        FnType::Setter(self_ty) => GeneratedPyMethod::Method(impl_py_setter_def(
            cls,
            PropertyType::Function(&spec),
            self_ty,
            &spec.doc,
        )?),
    })
}

pub(crate) fn check_generic(sig: &syn::Signature) -> syn::Result<()> {
    let err_msg = |typ| format!("Python functions cannot have generic {} parameters", typ);
    for param in &sig.generics.params {
        match param {
            syn::GenericParam::Lifetime(_) => {}
            syn::GenericParam::Type(_) => bail_spanned!(param.span() => err_msg("type")),
            syn::GenericParam::Const(_) => bail_spanned!(param.span() => err_msg("const")),
        }
    }
    Ok(())
}

pub fn gen_py_const(cls: &syn::Type, spec: &ConstSpec) -> TokenStream {
    let member = &spec.rust_ident;
    let wrapper = quote! {{
        fn __wrap(py: pyo3::Python<'_>) -> pyo3::PyObject {
            pyo3::IntoPy::into_py(#cls::#member, py)
        }
        __wrap
    }};
    impl_py_const_class_attribute(&spec, &wrapper)
}

/// Generate function wrapper for PyCFunctionWithKeywords
pub fn impl_wrap_cfunction_with_keywords(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    self_ty: &SelfType,
) -> Result<TokenStream> {
    let body = impl_call(cls, &spec);
    let slf = self_ty.receiver(cls);
    let py = syn::Ident::new("_py", Span::call_site());
    let body = impl_arg_params(&spec, Some(cls), body, &py)?;
    Ok(quote! {{
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            pyo3::callback::handle_panic(|#py| {
                #slf
                let _args = #py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = #py.from_borrowed_ptr_or_opt(_kwargs);

                #body
            })
        }
        __wrap
    }})
}

/// Generate function wrapper PyCFunction
pub fn impl_wrap_noargs(cls: &syn::Type, spec: &FnSpec<'_>, self_ty: &SelfType) -> TokenStream {
    let body = impl_call(cls, &spec);
    let slf = self_ty.receiver(cls);
    assert!(spec.args.is_empty());
    quote! {{
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
        ) -> *mut pyo3::ffi::PyObject
        {
            pyo3::callback::handle_panic(|_py| {
                #slf
                #body
            })
        }
        __wrap
    }}
}

/// Generate class method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_new(cls: &syn::Type, spec: &FnSpec<'_>) -> Result<TokenStream> {
    let name = &spec.name;
    let names: Vec<syn::Ident> = get_arg_names(&spec);
    let cb = quote! { #cls::#name(#(#names),*) };
    let py = syn::Ident::new("_py", Span::call_site());
    let body = impl_arg_params(spec, Some(cls), cb, &py)?;

    Ok(quote! {{
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            subtype: *mut pyo3::ffi::PyTypeObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            use pyo3::callback::IntoPyCallbackOutput;

            pyo3::callback::handle_panic(|#py| {
                let _args = #py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = #py.from_borrowed_ptr_or_opt(_kwargs);

                let initializer: pyo3::PyClassInitializer::<#cls> = #body.convert(#py)?;
                let cell = initializer.create_cell_from_subtype(#py, subtype)?;
                Ok(cell as *mut pyo3::ffi::PyObject)
            })
        }
        __wrap
    }})
}

/// Generate class method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_class(cls: &syn::Type, spec: &FnSpec<'_>) -> Result<TokenStream> {
    let name = &spec.name;
    let names: Vec<syn::Ident> = get_arg_names(&spec);
    let cb = quote! { pyo3::callback::convert(_py, #cls::#name(&_cls, #(#names),*)) };
    let py = syn::Ident::new("_py", Span::call_site());
    let body = impl_arg_params(spec, Some(cls), cb, &py)?;

    Ok(quote! {{
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _cls: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            pyo3::callback::handle_panic(|#py| {
                let _cls = pyo3::types::PyType::from_type_ptr(#py, _cls as *mut pyo3::ffi::PyTypeObject);
                let _args = #py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = #py.from_borrowed_ptr_or_opt(_kwargs);

                #body
            })
        }
        __wrap
    }})
}

/// Generate static method wrapper (PyCFunction, PyCFunctionWithKeywords)
pub fn impl_wrap_static(cls: &syn::Type, spec: &FnSpec<'_>) -> Result<TokenStream> {
    let name = &spec.name;
    let names: Vec<syn::Ident> = get_arg_names(&spec);
    let cb = quote! { pyo3::callback::convert(_py, #cls::#name(#(#names),*)) };
    let py = syn::Ident::new("_py", Span::call_site());
    let body = impl_arg_params(spec, Some(cls), cb, &py)?;

    Ok(quote! {{
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _args: *mut pyo3::ffi::PyObject,
            _kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            pyo3::callback::handle_panic(|#py| {
                let _args = #py.from_borrowed_ptr::<pyo3::types::PyTuple>(_args);
                let _kwargs: Option<&pyo3::types::PyDict> = #py.from_borrowed_ptr_or_opt(_kwargs);

                #body
            })
        }
        __wrap
    }})
}

/// Generate a wrapper for initialization of a class attribute from a method
/// annotated with `#[classattr]`.
/// To be called in `pyo3::pyclass::initialize_type_object`.
pub fn impl_wrap_class_attribute(cls: &syn::Type, spec: &FnSpec<'_>) -> TokenStream {
    let name = &spec.name;
    let cb = quote! { #cls::#name() };

    quote! {{
        fn __wrap(py: pyo3::Python<'_>) -> pyo3::PyObject {
            pyo3::IntoPy::into_py(#cb, py)
        }
        __wrap
    }}
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
    let getter_impl = match &property_type {
        PropertyType::Descriptor(ident) => {
            quote!(_slf.#ident.clone())
        }
        PropertyType::Function(spec) => impl_call_getter(cls, spec)?,
    };

    let slf = self_ty.receiver(cls);
    Ok(quote! {{
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject, _: *mut std::os::raw::c_void) -> *mut pyo3::ffi::PyObject
        {
            pyo3::callback::handle_panic(|_py| {
                #slf
                pyo3::callback::convert(_py, #getter_impl)
            })
        }
        __wrap
    }})
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
    let setter_impl = match &property_type {
        PropertyType::Descriptor(ident) => {
            quote!({ _slf.#ident = _val; })
        }
        PropertyType::Function(spec) => impl_call_setter(cls, spec)?,
    };

    let slf = self_ty.receiver(cls);
    Ok(quote! {{
        #[allow(unused_mut)]
        unsafe extern "C" fn __wrap(
            _slf: *mut pyo3::ffi::PyObject,
            _value: *mut pyo3::ffi::PyObject, _: *mut std::os::raw::c_void) -> std::os::raw::c_int
        {
            pyo3::callback::handle_panic(|_py| {
                #slf
                let _value = _py.from_borrowed_ptr::<pyo3::types::PyAny>(_value);
                let _val = pyo3::FromPyObject::extract(_value)?;

                pyo3::callback::convert(_py, #setter_impl)
            })
        }
        __wrap
    }})
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
    quote! { pyo3::callback::convert(_py, #cls::#fname(_slf, #(#names),*)) }
}

pub fn impl_arg_params(
    spec: &FnSpec<'_>,
    self_: Option<&syn::Type>,
    body: TokenStream,
    py: &syn::Ident,
) -> Result<TokenStream> {
    if spec.args.is_empty() {
        return Ok(body);
    }

    let mut positional_parameter_names = Vec::new();
    let mut required_positional_parameters = 0usize;
    let mut keyword_only_parameters = Vec::new();

    for arg in spec.args.iter() {
        if arg.py || spec.is_args(&arg.name) || spec.is_kwargs(&arg.name) {
            continue;
        }
        let name = arg.name.unraw().to_string();
        let kwonly = spec.is_kw_only(&arg.name);
        let required = !(arg.optional.is_some() || spec.default_value(&arg.name).is_some());

        if kwonly {
            keyword_only_parameters.push(quote! {
                pyo3::derive_utils::KeywordOnlyParameterDescription {
                    name: #name,
                    required: #required,
                }
            });
        } else {
            if required {
                required_positional_parameters += 1;
            }
            positional_parameter_names.push(name);
        }
    }

    let num_params = positional_parameter_names.len() + keyword_only_parameters.len();
    let args_array = syn::Ident::new("output", Span::call_site());

    let mut param_conversion = Vec::new();
    let mut option_pos = 0;
    for (idx, arg) in spec.args.iter().enumerate() {
        param_conversion.push(impl_arg_param(
            &arg,
            &spec,
            idx,
            self_,
            &mut option_pos,
            py,
            &args_array,
        )?);
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

    let cls_name = if let Some(cls) = self_ {
        quote! { Some(<#cls as pyo3::type_object::PyTypeInfo>::NAME) }
    } else {
        quote! { None }
    };
    let python_name = &spec.python_name;

    // create array of arguments, and then parse
    Ok(quote! {
        {
            const DESCRIPTION: pyo3::derive_utils::FunctionDescription = pyo3::derive_utils::FunctionDescription {
                cls_name: #cls_name,
                func_name: stringify!(#python_name),
                positional_parameter_names: &[#(#positional_parameter_names),*],
                // TODO: https://github.com/PyO3/pyo3/issues/1439 - support specifying these
                positional_only_parameters: 0,
                required_positional_parameters: #required_positional_parameters,
                keyword_only_parameters: &[#(#keyword_only_parameters),*],
                accept_varargs: #accept_args,
                accept_varkeywords: #accept_kwargs,
            };

            let mut #args_array = [None; #num_params];
            let (_args, _kwargs) = DESCRIPTION.extract_arguments(_args, _kwargs, &mut #args_array)?;

            #(#param_conversion)*

            #body
        }
    })
}

/// Re option_pos: The option slice doesn't contain the py: Python argument, so the argument
/// index and the index in option diverge when using py: Python
fn impl_arg_param(
    arg: &FnArg<'_>,
    spec: &FnSpec<'_>,
    idx: usize,
    self_: Option<&syn::Type>,
    option_pos: &mut usize,
    py: &syn::Ident,
    args_array: &syn::Ident,
) -> Result<TokenStream> {
    // Use this macro inside this function, to ensure that all code generated here is associated
    // with the function argument
    macro_rules! quote_arg_span {
        ($($tokens:tt)*) => { quote_spanned!(arg.ty.span() => $($tokens)*) }
    }

    let arg_name = syn::Ident::new(&format!("arg{}", idx), Span::call_site());

    if arg.py {
        return Ok(quote_arg_span! { let #arg_name = #py; });
    }

    let ty = arg.ty;
    let name = arg.name;
    let transform_error = quote_arg_span! {
        |e| pyo3::derive_utils::argument_extraction_error(#py, stringify!(#name), e)
    };

    if spec.is_args(&name) {
        ensure_spanned!(
            arg.optional.is_none(),
            arg.name.span() => "args cannot be optional"
        );
        return Ok(quote_arg_span! {
            let #arg_name = _args.unwrap().extract().map_err(#transform_error)?;
        });
    } else if spec.is_kwargs(&name) {
        ensure_spanned!(
            arg.optional.is_some(),
            arg.name.span() => "kwargs must be Option<_>"
        );
        return Ok(quote_arg_span! {
            let #arg_name = _kwargs.map(|kwargs| kwargs.extract())
                .transpose()
                .map_err(#transform_error)?;
        });
    }

    let arg_value = quote_arg_span!(#args_array[#option_pos]);
    *option_pos += 1;

    let default = match (spec.default_value(name), arg.optional.is_some()) {
        (Some(default), true) if default.to_string() != "None" => {
            quote_arg_span! { Some(#default) }
        }
        (Some(default), _) => quote_arg_span! { #default },
        (None, true) => quote_arg_span! { None },
        (None, false) => quote_arg_span! { panic!("Failed to extract required method argument") },
    };

    let extract = if let Some(FromPyWithAttribute(expr_path)) = &arg.attrs.from_py_with {
        quote_arg_span! { #expr_path(_obj).map_err(#transform_error)?}
    } else {
        quote_arg_span! { _obj.extract().map_err(#transform_error)?}
    };

    return if let syn::Type::Reference(tref) = arg.optional.as_ref().unwrap_or(&ty) {
        let (tref, mut_) = preprocess_tref(tref, self_);
        let (target_ty, borrow_tmp) = if arg.optional.is_some() {
            // Get Option<&T> from Option<PyRef<T>>
            (
                quote_arg_span! { Option<<#tref as pyo3::derive_utils::ExtractExt>::Target> },
                if mut_.is_some() {
                    quote_arg_span! { _tmp.as_deref_mut() }
                } else {
                    quote_arg_span! { _tmp.as_deref() }
                },
            )
        } else {
            // Get &T from PyRef<T>
            (
                quote_arg_span! { <#tref as pyo3::derive_utils::ExtractExt>::Target },
                quote_arg_span! { &#mut_ *_tmp },
            )
        };

        Ok(quote_arg_span! {
            let #mut_ _tmp: #target_ty = match #arg_value {
                Some(_obj) => #extract,
                None => #default,
            };
            let #arg_name = #borrow_tmp;
        })
    } else {
        Ok(quote_arg_span! {
            let #arg_name = match #arg_value {
                Some(_obj) => #extract,
                None => #default,
            };
        })
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

pub fn impl_py_method_def(
    cls: &syn::Type,
    spec: &FnSpec,
    self_ty: &SelfType,
    flags: Option<TokenStream>,
) -> Result<TokenStream> {
    let add_flags = flags.map(|flags| quote!(.flags(#flags)));
    let python_name = spec.python_name_with_deprecation();
    let doc = &spec.doc;
    if spec.args.is_empty() {
        let wrapper = impl_wrap_noargs(cls, spec, self_ty);
        Ok(quote! {
            pyo3::class::PyMethodDefType::Method({
                pyo3::class::PyMethodDef::noargs(
                    #python_name,
                    pyo3::class::methods::PyCFunction(#wrapper),
                    #doc
                )
                #add_flags

            })
        })
    } else {
        let wrapper = impl_wrap_cfunction_with_keywords(cls, &spec, self_ty)?;
        Ok(quote! {
            pyo3::class::PyMethodDefType::Method({
                pyo3::class::PyMethodDef::cfunction_with_keywords(
                    #python_name,
                    pyo3::class::methods::PyCFunctionWithKeywords(#wrapper),
                    #doc
                )
                #add_flags
            })
        })
    }
}

pub fn impl_py_method_def_new(cls: &syn::Type, spec: &FnSpec) -> Result<TokenStream> {
    let wrapper = impl_wrap_new(cls, &spec)?;
    Ok(quote! {
        impl pyo3::class::impl_::PyClassNewImpl<#cls> for pyo3::class::impl_::PyClassImplCollector<#cls> {
            fn new_impl(self) -> Option<pyo3::ffi::newfunc> {
                Some(#wrapper)
            }
        }
    })
}

pub fn impl_py_method_def_class(cls: &syn::Type, spec: &FnSpec) -> Result<TokenStream> {
    let wrapper = impl_wrap_class(cls, &spec)?;
    let python_name = spec.python_name_with_deprecation();
    let doc = &spec.doc;
    Ok(quote! {
        pyo3::class::PyMethodDefType::Class({
            pyo3::class::PyMethodDef::cfunction_with_keywords(
                #python_name,
                pyo3::class::methods::PyCFunctionWithKeywords(#wrapper),
                #doc
            ).flags(pyo3::ffi::METH_CLASS)
        })
    })
}

pub fn impl_py_method_def_static(cls: &syn::Type, spec: &FnSpec) -> Result<TokenStream> {
    let wrapper = impl_wrap_static(cls, &spec)?;
    let python_name = spec.python_name_with_deprecation();
    let doc = &spec.doc;
    Ok(quote! {
        pyo3::class::PyMethodDefType::Static({
            pyo3::class::PyMethodDef::cfunction_with_keywords(
                #python_name,
                pyo3::class::methods::PyCFunctionWithKeywords(#wrapper),
                #doc
            ).flags(pyo3::ffi::METH_STATIC)
        })
    })
}

pub fn impl_py_method_class_attribute(cls: &syn::Type, spec: &FnSpec) -> TokenStream {
    let wrapper = impl_wrap_class_attribute(cls, &spec);
    let python_name = spec.python_name_with_deprecation();
    quote! {
        pyo3::class::PyMethodDefType::ClassAttribute({
            pyo3::class::PyClassAttributeDef::new(
                #python_name,
                pyo3::class::methods::PyClassAttributeFactory(#wrapper)
            )
        })
    }
}

pub fn impl_py_const_class_attribute(spec: &ConstSpec, wrapper: &TokenStream) -> TokenStream {
    let python_name = &spec.python_name_with_deprecation();
    quote! {
        {
            pyo3::class::PyMethodDefType::ClassAttribute({
                pyo3::class::PyClassAttributeDef::new(
                    #python_name,
                    pyo3::class::methods::PyClassAttributeFactory(#wrapper)
                )
            })
        }
    }
}

pub fn impl_py_method_def_call(
    cls: &syn::Type,
    spec: &FnSpec,
    self_ty: &SelfType,
) -> Result<TokenStream> {
    let wrapper = impl_wrap_cfunction_with_keywords(cls, &spec, self_ty)?;
    Ok(quote! {
        impl pyo3::class::impl_::PyClassCallImpl<#cls> for pyo3::class::impl_::PyClassImplCollector<#cls> {
            fn call_impl(self) -> Option<pyo3::ffi::PyCFunctionWithKeywords> {
                Some(#wrapper)
            }
        }
    })
}

pub(crate) fn impl_py_setter_def(
    cls: &syn::Type,
    property_type: PropertyType,
    self_ty: &SelfType,
    doc: &syn::LitStr,
) -> Result<TokenStream> {
    let python_name = match property_type {
        PropertyType::Descriptor(ident) => {
            let formatted_name = format!("{}\0", ident.unraw());
            quote!(#formatted_name)
        }
        PropertyType::Function(spec) => spec.python_name_with_deprecation(),
    };
    let wrapper = impl_wrap_setter(cls, property_type, self_ty)?;
    Ok(quote! {
        pyo3::class::PyMethodDefType::Setter({
            pyo3::class::PySetterDef::new(
                #python_name,
                pyo3::class::methods::PySetter(#wrapper),
                #doc
            )
        })
    })
}

pub(crate) fn impl_py_getter_def(
    cls: &syn::Type,
    property_type: PropertyType,
    self_ty: &SelfType,
    doc: &syn::LitStr,
) -> Result<TokenStream> {
    let python_name = match property_type {
        PropertyType::Descriptor(ident) => {
            let formatted_name = format!("{}\0", ident.unraw());
            quote!(#formatted_name)
        }
        PropertyType::Function(spec) => spec.python_name_with_deprecation(),
    };
    let wrapper = impl_wrap_getter(cls, property_type, self_ty)?;
    Ok(quote! {
        pyo3::class::PyMethodDefType::Getter({
            pyo3::class::PyGetterDef::new(
                #python_name,
                pyo3::class::methods::PyGetter(#wrapper),
                #doc
            )
        })
    })
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
