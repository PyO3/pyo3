use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{ext::IdentExt, spanned::Spanned, Ident, Result};

use crate::{
    attributes::{TextSignatureAttribute, TextSignatureAttributeValue},
    deprecations::{Deprecation, Deprecations},
    params::impl_arg_params,
    pyfunction::{
        FunctionSignature, PyFunctionArgPyO3Attributes, PyFunctionOptions, SignatureAttribute,
    },
    quotes,
    utils::{self, is_abi3, PythonDoc},
};

#[derive(Clone, Debug)]
pub struct FnArg<'a> {
    pub name: &'a syn::Ident,
    pub ty: &'a syn::Type,
    pub optional: Option<&'a syn::Type>,
    pub default: Option<syn::Expr>,
    pub py: bool,
    pub attrs: PyFunctionArgPyO3Attributes,
    pub is_varargs: bool,
    pub is_kwargs: bool,
    pub is_cancel_handle: bool,
}

impl<'a> FnArg<'a> {
    /// Transforms a rust fn arg parsed with syn into a method::FnArg
    pub fn parse(arg: &'a mut syn::FnArg) -> Result<Self> {
        match arg {
            syn::FnArg::Receiver(recv) => {
                bail_spanned!(recv.span() => "unexpected receiver")
            } // checked in parse_fn_type
            syn::FnArg::Typed(cap) => {
                if let syn::Type::ImplTrait(_) = &*cap.ty {
                    bail_spanned!(cap.ty.span() => IMPL_TRAIT_ERR);
                }

                let arg_attrs = PyFunctionArgPyO3Attributes::from_attrs(&mut cap.attrs)?;
                let ident = match &*cap.pat {
                    syn::Pat::Ident(syn::PatIdent { ident, .. }) => ident,
                    other => return Err(handle_argument_error(other)),
                };

                let is_cancel_handle = arg_attrs.cancel_handle.is_some();

                Ok(FnArg {
                    name: ident,
                    ty: &cap.ty,
                    optional: utils::option_type_argument(&cap.ty),
                    default: None,
                    py: utils::is_python(&cap.ty),
                    attrs: arg_attrs,
                    is_varargs: false,
                    is_kwargs: false,
                    is_cancel_handle,
                })
            }
        }
    }
}

fn handle_argument_error(pat: &syn::Pat) -> syn::Error {
    let span = pat.span();
    let msg = match pat {
        syn::Pat::Wild(_) => "wildcard argument names are not supported",
        syn::Pat::Struct(_)
        | syn::Pat::Tuple(_)
        | syn::Pat::TupleStruct(_)
        | syn::Pat::Slice(_) => "destructuring in arguments is not supported",
        _ => "unsupported argument",
    };
    syn::Error::new(span, msg)
}

#[derive(Clone, Debug)]
pub enum FnType {
    Getter(SelfType),
    Setter(SelfType),
    Fn(SelfType),
    FnNew,
    FnNewClass(Span),
    FnClass(Span),
    FnStatic,
    FnModule(Span),
    ClassAttribute,
}

impl FnType {
    pub fn skip_first_rust_argument_in_python_signature(&self) -> bool {
        match self {
            FnType::Getter(_)
            | FnType::Setter(_)
            | FnType::Fn(_)
            | FnType::FnClass(_)
            | FnType::FnNewClass(_)
            | FnType::FnModule(_) => true,
            FnType::FnNew | FnType::FnStatic | FnType::ClassAttribute => false,
        }
    }

    pub fn self_arg(
        &self,
        cls: Option<&syn::Type>,
        error_mode: ExtractErrorMode,
        holders: &mut Vec<TokenStream>,
    ) -> TokenStream {
        match self {
            FnType::Getter(st) | FnType::Setter(st) | FnType::Fn(st) => {
                let mut receiver = st.receiver(
                    cls.expect("no class given for Fn with a \"self\" receiver"),
                    error_mode,
                    holders,
                );
                syn::Token![,](Span::call_site()).to_tokens(&mut receiver);
                receiver
            }
            FnType::FnNew | FnType::FnStatic | FnType::ClassAttribute => {
                quote!()
            }
            FnType::FnClass(span) | FnType::FnNewClass(span) => {
                let py = syn::Ident::new("py", Span::call_site());
                let slf: Ident = syn::Ident::new("_slf", Span::call_site());
                quote_spanned! { *span =>
                    #[allow(clippy::useless_conversion)]
                    ::std::convert::Into::into(
                        _pyo3::impl_::pymethods::BoundRef::ref_from_ptr(#py, &#slf.cast())
                            .downcast_unchecked::<_pyo3::types::PyType>()
                    ),
                }
            }
            FnType::FnModule(span) => {
                let py = syn::Ident::new("py", Span::call_site());
                let slf: Ident = syn::Ident::new("_slf", Span::call_site());
                quote_spanned! { *span =>
                    #[allow(clippy::useless_conversion)]
                    ::std::convert::Into::into(
                        _pyo3::impl_::pymethods::BoundRef::ref_from_ptr(#py, &#slf.cast())
                            .downcast_unchecked::<_pyo3::types::PyModule>()
                    ),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum SelfType {
    Receiver { mutable: bool, span: Span },
    TryFromPyCell(Span),
}

#[derive(Clone, Copy)]
pub enum ExtractErrorMode {
    NotImplemented,
    Raise,
}

impl ExtractErrorMode {
    pub fn handle_error(self, extract: TokenStream) -> TokenStream {
        match self {
            ExtractErrorMode::Raise => quote! { #extract? },
            ExtractErrorMode::NotImplemented => quote! {
                match #extract {
                    ::std::result::Result::Ok(value) => value,
                    ::std::result::Result::Err(_) => { return _pyo3::callback::convert(py, py.NotImplemented()); },
                }
            },
        }
    }
}

impl SelfType {
    pub fn receiver(
        &self,
        cls: &syn::Type,
        error_mode: ExtractErrorMode,
        holders: &mut Vec<TokenStream>,
    ) -> TokenStream {
        // Due to use of quote_spanned in this function, need to bind these idents to the
        // main macro callsite.
        let py = syn::Ident::new("py", Span::call_site());
        let slf = syn::Ident::new("_slf", Span::call_site());
        match self {
            SelfType::Receiver { span, mutable } => {
                let method = if *mutable {
                    syn::Ident::new("extract_pyclass_ref_mut", *span)
                } else {
                    syn::Ident::new("extract_pyclass_ref", *span)
                };
                let holder = syn::Ident::new(&format!("holder_{}", holders.len()), *span);
                holders.push(quote_spanned! { *span =>
                    #[allow(clippy::let_unit_value)]
                    let mut #holder = _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT;
                });
                error_mode.handle_error(quote_spanned! { *span =>
                    _pyo3::impl_::extract_argument::#method::<#cls>(
                        #py.from_borrowed_ptr::<_pyo3::PyAny>(#slf),
                        &mut #holder,
                    )
                })
            }
            SelfType::TryFromPyCell(span) => {
                error_mode.handle_error(
                    quote_spanned! { *span =>
                        #py.from_borrowed_ptr::<_pyo3::PyAny>(#slf).downcast::<_pyo3::PyCell<#cls>>()
                            .map_err(::std::convert::Into::<_pyo3::PyErr>::into)
                            .and_then(
                                #[allow(clippy::useless_conversion)]  // In case slf is PyCell<Self>
                                #[allow(unknown_lints, clippy::unnecessary_fallible_conversions)]  // In case slf is Py<Self> (unknown_lints can be removed when MSRV is 1.75+)
                                |cell| ::std::convert::TryFrom::try_from(cell).map_err(::std::convert::Into::into)
                            )

                    }
                )
            }
        }
    }
}

/// Determines which CPython calling convention a given FnSpec uses.
#[derive(Clone, Debug)]
pub enum CallingConvention {
    Noargs,   // METH_NOARGS
    Varargs,  // METH_VARARGS | METH_KEYWORDS
    Fastcall, // METH_FASTCALL | METH_KEYWORDS (not compatible with `abi3` feature)
    TpNew,    // special convention for tp_new
}

impl CallingConvention {
    /// Determine default calling convention from an argument signature.
    ///
    /// Different other slots (tp_call, tp_new) can have other requirements
    /// and are set manually (see `parse_fn_type` below).
    pub fn from_signature(signature: &FunctionSignature<'_>) -> Self {
        if signature.python_signature.has_no_args() {
            Self::Noargs
        } else if signature.python_signature.kwargs.is_some() {
            // for functions that accept **kwargs, always prefer varargs
            Self::Varargs
        } else if !is_abi3() {
            // FIXME: available in the stable ABI since 3.10
            Self::Fastcall
        } else {
            Self::Varargs
        }
    }
}

pub struct FnSpec<'a> {
    pub tp: FnType,
    // Rust function name
    pub name: &'a syn::Ident,
    // Wrapped python name. This should not have any leading r#.
    // r# can be removed by syn::ext::IdentExt::unraw()
    pub python_name: syn::Ident,
    pub signature: FunctionSignature<'a>,
    pub output: syn::Type,
    pub convention: CallingConvention,
    pub text_signature: Option<TextSignatureAttribute>,
    pub asyncness: Option<syn::Token![async]>,
    pub unsafety: Option<syn::Token![unsafe]>,
    pub deprecations: Deprecations,
}

pub fn get_return_info(output: &syn::ReturnType) -> syn::Type {
    match output {
        syn::ReturnType::Default => syn::Type::Infer(syn::parse_quote! {_}),
        syn::ReturnType::Type(_, ty) => *ty.clone(),
    }
}

pub fn parse_method_receiver(arg: &syn::FnArg) -> Result<SelfType> {
    match arg {
        syn::FnArg::Receiver(
            recv @ syn::Receiver {
                reference: None, ..
            },
        ) => {
            bail_spanned!(recv.span() => RECEIVER_BY_VALUE_ERR);
        }
        syn::FnArg::Receiver(recv @ syn::Receiver { mutability, .. }) => Ok(SelfType::Receiver {
            mutable: mutability.is_some(),
            span: recv.span(),
        }),
        syn::FnArg::Typed(syn::PatType { ty, .. }) => {
            if let syn::Type::ImplTrait(_) = &**ty {
                bail_spanned!(ty.span() => IMPL_TRAIT_ERR);
            }
            Ok(SelfType::TryFromPyCell(ty.span()))
        }
    }
}

impl<'a> FnSpec<'a> {
    /// Parser function signature and function attributes
    pub fn parse(
        // Signature is mutable to remove the `Python` argument.
        sig: &'a mut syn::Signature,
        meth_attrs: &mut Vec<syn::Attribute>,
        options: PyFunctionOptions,
    ) -> Result<FnSpec<'a>> {
        let PyFunctionOptions {
            text_signature,
            name,
            signature,
            ..
        } = options;

        let mut python_name = name.map(|name| name.value.0);
        let mut deprecations = Deprecations::new();

        let fn_type = Self::parse_fn_type(sig, meth_attrs, &mut python_name, &mut deprecations)?;
        ensure_signatures_on_valid_method(&fn_type, signature.as_ref(), text_signature.as_ref())?;

        let name = &sig.ident;
        let ty = get_return_info(&sig.output);
        let python_name = python_name.as_ref().unwrap_or(name).unraw();

        let arguments: Vec<_> = sig
            .inputs
            .iter_mut()
            .skip(if fn_type.skip_first_rust_argument_in_python_signature() {
                1
            } else {
                0
            })
            .map(FnArg::parse)
            .collect::<Result<_>>()?;

        let signature = if let Some(signature) = signature {
            FunctionSignature::from_arguments_and_attribute(arguments, signature)?
        } else {
            FunctionSignature::from_arguments(arguments)?
        };

        let convention = if matches!(fn_type, FnType::FnNew | FnType::FnNewClass(_)) {
            CallingConvention::TpNew
        } else {
            CallingConvention::from_signature(&signature)
        };

        Ok(FnSpec {
            tp: fn_type,
            name,
            convention,
            python_name,
            signature,
            output: ty,
            text_signature,
            asyncness: sig.asyncness,
            unsafety: sig.unsafety,
            deprecations,
        })
    }

    pub fn null_terminated_python_name(&self) -> syn::LitStr {
        syn::LitStr::new(&format!("{}\0", self.python_name), self.python_name.span())
    }

    fn parse_fn_type(
        sig: &syn::Signature,
        meth_attrs: &mut Vec<syn::Attribute>,
        python_name: &mut Option<syn::Ident>,
        deprecations: &mut Deprecations,
    ) -> Result<FnType> {
        let mut method_attributes = parse_method_attributes(meth_attrs, deprecations)?;

        let name = &sig.ident;
        let parse_receiver = |msg: &'static str| {
            let first_arg = sig
                .inputs
                .first()
                .ok_or_else(|| err_spanned!(sig.span() => msg))?;
            parse_method_receiver(first_arg)
        };

        // strip get_ or set_
        let strip_fn_name = |prefix: &'static str| {
            name.unraw()
                .to_string()
                .strip_prefix(prefix)
                .map(|stripped| syn::Ident::new(stripped, name.span()))
        };

        let mut set_name_to_new = || {
            if let Some(name) = &python_name {
                bail_spanned!(name.span() => "`name` not allowed with `#[new]`");
            }
            *python_name = Some(syn::Ident::new("__new__", Span::call_site()));
            Ok(())
        };

        let fn_type = match method_attributes.as_mut_slice() {
            [] => FnType::Fn(parse_receiver(
                "static method needs #[staticmethod] attribute",
            )?),
            [MethodTypeAttribute::StaticMethod(_)] => FnType::FnStatic,
            [MethodTypeAttribute::ClassAttribute(_)] => FnType::ClassAttribute,
            [MethodTypeAttribute::New(_)] => {
                set_name_to_new()?;
                FnType::FnNew
            }
            [MethodTypeAttribute::New(_), MethodTypeAttribute::ClassMethod(span)]
            | [MethodTypeAttribute::ClassMethod(span), MethodTypeAttribute::New(_)] => {
                set_name_to_new()?;
                FnType::FnNewClass(*span)
            }
            [MethodTypeAttribute::ClassMethod(_)] => {
                // Add a helpful hint if the classmethod doesn't look like a classmethod
                let span = match sig.inputs.first() {
                    // Don't actually bother checking the type of the first argument, the compiler
                    // will error on incorrect type.
                    Some(syn::FnArg::Typed(first_arg)) => first_arg.ty.span(),
                    Some(syn::FnArg::Receiver(_)) | None => bail_spanned!(
                        sig.paren_token.span.join() => "Expected `&Bound<PyType>` or `Py<PyType>` as the first argument to `#[classmethod]`"
                    ),
                };
                FnType::FnClass(span)
            }
            [MethodTypeAttribute::Getter(_, name)] => {
                if let Some(name) = name.take() {
                    ensure_spanned!(
                        python_name.replace(name).is_none(),
                        python_name.span() => "`name` may only be specified once"
                    );
                } else if python_name.is_none() {
                    // Strip off "get_" prefix if needed
                    *python_name = strip_fn_name("get_");
                }

                FnType::Getter(parse_receiver("expected receiver for `#[getter]`")?)
            }
            [MethodTypeAttribute::Setter(_, name)] => {
                if let Some(name) = name.take() {
                    ensure_spanned!(
                        python_name.replace(name).is_none(),
                        python_name.span() => "`name` may only be specified once"
                    );
                } else if python_name.is_none() {
                    // Strip off "set_" prefix if needed
                    *python_name = strip_fn_name("set_");
                }

                FnType::Setter(parse_receiver("expected receiver for `#[setter]`")?)
            }
            [first, rest @ .., last] => {
                // Join as many of the spans together as possible
                let span = rest
                    .iter()
                    .fold(first.span(), |s, next| s.join(next.span()).unwrap_or(s));
                let span = span.join(last.span()).unwrap_or(span);
                // List all the attributes in the error message
                let mut msg = format!("`{}` may not be combined with", first);
                let mut is_first = true;
                for attr in &*rest {
                    msg.push_str(&format!(" `{}`", attr));
                    if is_first {
                        is_first = false;
                    } else {
                        msg.push(',');
                    }
                }
                if !rest.is_empty() {
                    msg.push_str(" and");
                }
                msg.push_str(&format!(" `{}`", last));
                bail_spanned!(span => msg)
            }
        };
        Ok(fn_type)
    }

    /// Return a C wrapper function for this signature.
    pub fn get_wrapper_function(
        &self,
        ident: &proc_macro2::Ident,
        cls: Option<&syn::Type>,
    ) -> Result<TokenStream> {
        let mut cancel_handle_iter = self
            .signature
            .arguments
            .iter()
            .filter(|arg| arg.is_cancel_handle);
        let cancel_handle = cancel_handle_iter.next();
        if let Some(arg) = cancel_handle {
            ensure_spanned!(self.asyncness.is_some(), arg.name.span() => "`cancel_handle` attribute can only be used with `async fn`");
            if let Some(arg2) = cancel_handle_iter.next() {
                bail_spanned!(arg2.name.span() => "`cancel_handle` may only be specified once");
            }
        }

        let rust_call = |args: Vec<TokenStream>, holders: &mut Vec<TokenStream>| {
            let self_arg = self.tp.self_arg(cls, ExtractErrorMode::Raise, holders);

            let call = if self.asyncness.is_some() {
                let throw_callback = if cancel_handle.is_some() {
                    quote! { Some(__throw_callback) }
                } else {
                    quote! { None }
                };
                let python_name = &self.python_name;
                let qualname_prefix = match cls {
                    Some(cls) => quote!(Some(<#cls as _pyo3::PyTypeInfo>::NAME)),
                    None => quote!(None),
                };
                let future = match self.tp {
                    FnType::Fn(SelfType::Receiver { mutable: false, .. }) => {
                        holders.pop().unwrap(); // does not actually use holder created by `self_arg`

                        quote! {{
                            let __guard = _pyo3::impl_::coroutine::RefGuard::<#cls>::new(py.from_borrowed_ptr::<_pyo3::types::PyAny>(_slf))?;
                            async move { function(&__guard, #(#args),*).await }
                        }}
                    }
                    FnType::Fn(SelfType::Receiver { mutable: true, .. }) => {
                        holders.pop().unwrap(); // does not actually use holder created by `self_arg`

                        quote! {{
                            let mut __guard = _pyo3::impl_::coroutine::RefMutGuard::<#cls>::new(py.from_borrowed_ptr::<_pyo3::types::PyAny>(_slf))?;
                            async move { function(&mut __guard, #(#args),*).await }
                        }}
                    }
                    _ => quote! { function(#self_arg #(#args),*) },
                };
                let mut call = quote! {{
                    let future = #future;
                    _pyo3::impl_::coroutine::new_coroutine(
                        _pyo3::intern!(py, stringify!(#python_name)),
                        #qualname_prefix,
                        #throw_callback,
                        async move { _pyo3::impl_::wrap::OkWrap::wrap(future.await) },
                    )
                }};
                if cancel_handle.is_some() {
                    call = quote! {{
                        let __cancel_handle = _pyo3::coroutine::CancelHandle::new();
                        let __throw_callback = __cancel_handle.throw_callback();
                        #call
                    }};
                }
                call
            } else {
                quote! { function(#self_arg #(#args),*) }
            };
            quotes::map_result_into_ptr(quotes::ok_wrap(call))
        };

        let func_name = &self.name;
        let rust_name = if let Some(cls) = cls {
            quote!(#cls::#func_name)
        } else {
            quote!(#func_name)
        };

        Ok(match self.convention {
            CallingConvention::Noargs => {
                let mut holders = Vec::new();
                let args = self
                    .signature
                    .arguments
                    .iter()
                    .map(|arg| {
                        if arg.py {
                            quote!(py)
                        } else if arg.is_cancel_handle {
                            quote!(__cancel_handle)
                        } else {
                            unreachable!()
                        }
                    })
                    .collect();
                let call = rust_call(args, &mut holders);

                quote! {
                    unsafe fn #ident<'py>(
                        py: _pyo3::Python<'py>,
                        _slf: *mut _pyo3::ffi::PyObject,
                    ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #( #holders )*
                        #call
                    }
                }
            }
            CallingConvention::Fastcall => {
                let mut holders = Vec::new();
                let (arg_convert, args) = impl_arg_params(self, cls, true, &mut holders)?;
                let call = rust_call(args, &mut holders);
                quote! {
                    unsafe fn #ident<'py>(
                        py: _pyo3::Python<'py>,
                        _slf: *mut _pyo3::ffi::PyObject,
                        _args: *const *mut _pyo3::ffi::PyObject,
                        _nargs: _pyo3::ffi::Py_ssize_t,
                        _kwnames: *mut _pyo3::ffi::PyObject
                    ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #arg_convert
                        #( #holders )*
                        #call
                    }
                }
            }
            CallingConvention::Varargs => {
                let mut holders = Vec::new();
                let (arg_convert, args) = impl_arg_params(self, cls, false, &mut holders)?;
                let call = rust_call(args, &mut holders);
                quote! {
                    unsafe fn #ident<'py>(
                        py: _pyo3::Python<'py>,
                        _slf: *mut _pyo3::ffi::PyObject,
                        _args: *mut _pyo3::ffi::PyObject,
                        _kwargs: *mut _pyo3::ffi::PyObject
                    ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #arg_convert
                        #( #holders )*
                        #call
                    }
                }
            }
            CallingConvention::TpNew => {
                let mut holders = Vec::new();
                let (arg_convert, args) = impl_arg_params(self, cls, false, &mut holders)?;
                let self_arg = self.tp.self_arg(cls, ExtractErrorMode::Raise, &mut holders);
                let call = quote! { #rust_name(#self_arg #(#args),*) };
                quote! {
                    unsafe fn #ident(
                        py: _pyo3::Python<'_>,
                        _slf: *mut _pyo3::ffi::PyTypeObject,
                        _args: *mut _pyo3::ffi::PyObject,
                        _kwargs: *mut _pyo3::ffi::PyObject
                    ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                        use _pyo3::callback::IntoPyCallbackOutput;
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #arg_convert
                        #( #holders )*
                        let result = #call;
                        let initializer: _pyo3::PyClassInitializer::<#cls> = result.convert(py)?;
                        let cell = initializer.create_cell_from_subtype(py, _slf)?;
                        ::std::result::Result::Ok(cell as *mut _pyo3::ffi::PyObject)
                    }
                }
            }
        })
    }

    /// Return a `PyMethodDef` constructor for this function, matching the selected
    /// calling convention.
    pub fn get_methoddef(&self, wrapper: impl ToTokens, doc: &PythonDoc) -> TokenStream {
        let python_name = self.null_terminated_python_name();
        match self.convention {
            CallingConvention::Noargs => quote! {
                _pyo3::impl_::pymethods::PyMethodDef::noargs(
                    #python_name,
                    _pyo3::impl_::pymethods::PyCFunction({
                        unsafe extern "C" fn trampoline(
                            _slf: *mut _pyo3::ffi::PyObject,
                            _args: *mut _pyo3::ffi::PyObject,
                        ) -> *mut _pyo3::ffi::PyObject
                        {
                            _pyo3::impl_::trampoline::noargs(
                                _slf,
                                _args,
                                #wrapper
                            )
                        }
                        trampoline
                    }),
                    #doc,
                )
            },
            CallingConvention::Fastcall => quote! {
                _pyo3::impl_::pymethods::PyMethodDef::fastcall_cfunction_with_keywords(
                    #python_name,
                    _pyo3::impl_::pymethods::PyCFunctionFastWithKeywords({
                        unsafe extern "C" fn trampoline(
                            _slf: *mut _pyo3::ffi::PyObject,
                            _args: *const *mut _pyo3::ffi::PyObject,
                            _nargs: _pyo3::ffi::Py_ssize_t,
                            _kwnames: *mut _pyo3::ffi::PyObject
                        ) -> *mut _pyo3::ffi::PyObject
                        {
                            _pyo3::impl_::trampoline::fastcall_with_keywords(
                                _slf,
                                _args,
                                _nargs,
                                _kwnames,
                                #wrapper
                            )
                        }
                        trampoline
                    }),
                    #doc,
                )
            },
            CallingConvention::Varargs => quote! {
                _pyo3::impl_::pymethods::PyMethodDef::cfunction_with_keywords(
                    #python_name,
                    _pyo3::impl_::pymethods::PyCFunctionWithKeywords({
                        unsafe extern "C" fn trampoline(
                            _slf: *mut _pyo3::ffi::PyObject,
                            _args: *mut _pyo3::ffi::PyObject,
                            _kwargs: *mut _pyo3::ffi::PyObject,
                        ) -> *mut _pyo3::ffi::PyObject
                        {
                            _pyo3::impl_::trampoline::cfunction_with_keywords(
                                _slf,
                                _args,
                                _kwargs,
                                #wrapper
                            )
                        }
                        trampoline
                    }),
                    #doc,
                )
            },
            CallingConvention::TpNew => unreachable!("tp_new cannot get a methoddef"),
        }
    }

    /// Forwards to [utils::get_doc] with the text signature of this spec.
    pub fn get_doc(&self, attrs: &[syn::Attribute]) -> PythonDoc {
        let text_signature = self
            .text_signature_call_signature()
            .map(|sig| format!("{}{}", self.python_name, sig));
        utils::get_doc(attrs, text_signature)
    }

    /// Creates the parenthesised arguments list for `__text_signature__` snippet based on this spec's signature
    /// and/or attributes. Prepend the callable name to make a complete `__text_signature__`.
    pub fn text_signature_call_signature(&self) -> Option<String> {
        let self_argument = match &self.tp {
            // Getters / Setters / ClassAttribute are not callables on the Python side
            FnType::Getter(_) | FnType::Setter(_) | FnType::ClassAttribute => return None,
            FnType::Fn(_) => Some("self"),
            FnType::FnModule(_) => Some("module"),
            FnType::FnClass(_) | FnType::FnNewClass(_) => Some("cls"),
            FnType::FnStatic | FnType::FnNew => None,
        };

        match self.text_signature.as_ref().map(|attr| &attr.value) {
            Some(TextSignatureAttributeValue::Str(s)) => Some(s.value()),
            None => Some(self.signature.text_signature(self_argument)),
            Some(TextSignatureAttributeValue::Disabled(_)) => None,
        }
    }
}

enum MethodTypeAttribute {
    New(Span),
    ClassMethod(Span),
    StaticMethod(Span),
    Getter(Span, Option<Ident>),
    Setter(Span, Option<Ident>),
    ClassAttribute(Span),
}

impl MethodTypeAttribute {
    fn span(&self) -> Span {
        match self {
            MethodTypeAttribute::New(span)
            | MethodTypeAttribute::ClassMethod(span)
            | MethodTypeAttribute::StaticMethod(span)
            | MethodTypeAttribute::Getter(span, _)
            | MethodTypeAttribute::Setter(span, _)
            | MethodTypeAttribute::ClassAttribute(span) => *span,
        }
    }

    /// Attempts to parse a method type attribute.
    ///
    /// If the attribute does not match one of the attribute names, returns `Ok(None)`.
    ///
    /// Otherwise will either return a parse error or the attribute.
    fn parse_if_matching_attribute(
        attr: &syn::Attribute,
        deprecations: &mut Deprecations,
    ) -> Result<Option<Self>> {
        fn ensure_no_arguments(meta: &syn::Meta, ident: &str) -> syn::Result<()> {
            match meta {
                syn::Meta::Path(_) => Ok(()),
                syn::Meta::List(l) => bail_spanned!(
                    l.span() => format!(
                        "`#[{ident}]` does not take any arguments\n= help: did you mean `#[{ident}] #[pyo3({meta})]`?",
                        ident = ident,
                        meta = l.tokens,
                    )
                ),
                syn::Meta::NameValue(nv) => {
                    bail_spanned!(nv.eq_token.span() => format!(
                        "`#[{}]` does not take any arguments\n= note: this was previously accepted and ignored",
                        ident
                    ))
                }
            }
        }

        fn extract_name(meta: &syn::Meta, ident: &str) -> Result<Option<Ident>> {
            match meta {
                syn::Meta::Path(_) => Ok(None),
                syn::Meta::NameValue(nv) => bail_spanned!(
                    nv.eq_token.span() => format!("expected `#[{}(name)]` to set the name", ident)
                ),
                syn::Meta::List(l) => {
                    if let Ok(name) = l.parse_args::<syn::Ident>() {
                        Ok(Some(name))
                    } else if let Ok(name) = l.parse_args::<syn::LitStr>() {
                        name.parse().map(Some)
                    } else {
                        bail_spanned!(l.tokens.span() => "expected ident or string literal for property name");
                    }
                }
            }
        }

        let meta = &attr.meta;
        let path = meta.path();

        if path.is_ident("new") {
            ensure_no_arguments(meta, "new")?;
            Ok(Some(MethodTypeAttribute::New(path.span())))
        } else if path.is_ident("__new__") {
            let span = path.span();
            deprecations.push(Deprecation::PyMethodsNewDeprecatedForm, span);
            ensure_no_arguments(meta, "__new__")?;
            Ok(Some(MethodTypeAttribute::New(span)))
        } else if path.is_ident("classmethod") {
            ensure_no_arguments(meta, "classmethod")?;
            Ok(Some(MethodTypeAttribute::ClassMethod(path.span())))
        } else if path.is_ident("staticmethod") {
            ensure_no_arguments(meta, "staticmethod")?;
            Ok(Some(MethodTypeAttribute::StaticMethod(path.span())))
        } else if path.is_ident("classattr") {
            ensure_no_arguments(meta, "classattr")?;
            Ok(Some(MethodTypeAttribute::ClassAttribute(path.span())))
        } else if path.is_ident("getter") {
            let name = extract_name(meta, "getter")?;
            Ok(Some(MethodTypeAttribute::Getter(path.span(), name)))
        } else if path.is_ident("setter") {
            let name = extract_name(meta, "setter")?;
            Ok(Some(MethodTypeAttribute::Setter(path.span(), name)))
        } else {
            Ok(None)
        }
    }
}

impl Display for MethodTypeAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodTypeAttribute::New(_) => "#[new]".fmt(f),
            MethodTypeAttribute::ClassMethod(_) => "#[classmethod]".fmt(f),
            MethodTypeAttribute::StaticMethod(_) => "#[staticmethod]".fmt(f),
            MethodTypeAttribute::Getter(_, _) => "#[getter]".fmt(f),
            MethodTypeAttribute::Setter(_, _) => "#[setter]".fmt(f),
            MethodTypeAttribute::ClassAttribute(_) => "#[classattr]".fmt(f),
        }
    }
}

fn parse_method_attributes(
    attrs: &mut Vec<syn::Attribute>,
    deprecations: &mut Deprecations,
) -> Result<Vec<MethodTypeAttribute>> {
    let mut new_attrs = Vec::new();
    let mut found_attrs = Vec::new();

    for attr in attrs.drain(..) {
        match MethodTypeAttribute::parse_if_matching_attribute(&attr, deprecations)? {
            Some(attr) => found_attrs.push(attr),
            None => new_attrs.push(attr),
        }
    }

    *attrs = new_attrs;

    Ok(found_attrs)
}

const IMPL_TRAIT_ERR: &str = "Python functions cannot have `impl Trait` arguments";
const RECEIVER_BY_VALUE_ERR: &str =
    "Python objects are shared, so 'self' cannot be moved out of the Python interpreter.
Try `&self`, `&mut self, `slf: PyRef<'_, Self>` or `slf: PyRefMut<'_, Self>`.";

fn ensure_signatures_on_valid_method(
    fn_type: &FnType,
    signature: Option<&SignatureAttribute>,
    text_signature: Option<&TextSignatureAttribute>,
) -> syn::Result<()> {
    if let Some(signature) = signature {
        match fn_type {
            FnType::Getter(_) => {
                bail_spanned!(signature.kw.span() => "`signature` not allowed with `getter`")
            }
            FnType::Setter(_) => {
                bail_spanned!(signature.kw.span() => "`signature` not allowed with `setter`")
            }
            FnType::ClassAttribute => {
                bail_spanned!(signature.kw.span() => "`signature` not allowed with `classattr`")
            }
            _ => {}
        }
    }
    if let Some(text_signature) = text_signature {
        match fn_type {
            FnType::Getter(_) => {
                bail_spanned!(text_signature.kw.span() => "`text_signature` not allowed with `getter`")
            }
            FnType::Setter(_) => {
                bail_spanned!(text_signature.kw.span() => "`text_signature` not allowed with `setter`")
            }
            FnType::ClassAttribute => {
                bail_spanned!(text_signature.kw.span() => "`text_signature` not allowed with `classattr`")
            }
            _ => {}
        }
    }
    Ok(())
}
