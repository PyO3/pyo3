use crate::attributes::{TextSignatureAttribute, TextSignatureAttributeValue};
use crate::params::impl_arg_params;
use crate::pyfunction::{FunctionSignature, PyFunctionArgPyO3Attributes};
use crate::pyfunction::{PyFunctionOptions, SignatureAttribute};
use crate::utils::{self, PythonDoc};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use quote::{quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{Result, Token};

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

                Ok(FnArg {
                    name: ident,
                    ty: &cap.ty,
                    optional: utils::option_type_argument(&cap.ty),
                    default: None,
                    py: utils::is_python(&cap.ty),
                    attrs: arg_attrs,
                    is_varargs: false,
                    is_kwargs: false,
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

#[derive(Clone, PartialEq, Debug, Copy, Eq)]
pub enum MethodTypeAttribute {
    /// `#[new]`
    New,
    /// `#[new]` && `#[classmethod]`
    NewClassMethod,
    /// `#[classmethod]`
    ClassMethod,
    /// `#[classattr]`
    ClassAttribute,
    /// `#[staticmethod]`
    StaticMethod,
    /// `#[getter]`
    Getter,
    /// `#[setter]`
    Setter,
}

#[derive(Clone, Debug)]
pub enum FnType {
    Getter(SelfType),
    Setter(SelfType),
    Fn(SelfType),
    FnNew,
    FnNewClass,
    FnClass,
    FnStatic,
    FnModule,
    ClassAttribute,
}

impl FnType {
    pub fn self_conversion(
        &self,
        cls: Option<&syn::Type>,
        error_mode: ExtractErrorMode,
    ) -> TokenStream {
        match self {
            FnType::Getter(st) | FnType::Setter(st) | FnType::Fn(st) => st.receiver(
                cls.expect("no class given for Fn with a \"self\" receiver"),
                error_mode,
            ),
            FnType::FnNew | FnType::FnStatic | FnType::ClassAttribute => {
                quote!()
            }
            FnType::FnClass | FnType::FnNewClass => {
                quote! {
                    let _slf = _pyo3::types::PyType::from_type_ptr(_py, _slf as *mut _pyo3::ffi::PyTypeObject);
                }
            }
            FnType::FnModule => {
                quote! {
                    let _slf = _py.from_borrowed_ptr::<_pyo3::types::PyModule>(_slf);
                }
            }
        }
    }

    pub fn self_arg(&self) -> TokenStream {
        match self {
            FnType::FnNew | FnType::FnStatic | FnType::ClassAttribute => quote!(),
            _ => quote!(_slf,),
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
    pub fn handle_error(self, py: &syn::Ident, extract: TokenStream) -> TokenStream {
        match self {
            ExtractErrorMode::Raise => quote! { #extract? },
            ExtractErrorMode::NotImplemented => quote! {
                match #extract {
                    ::std::result::Result::Ok(value) => value,
                    ::std::result::Result::Err(_) => { return _pyo3::callback::convert(#py, #py.NotImplemented()); },
                }
            },
        }
    }
}

impl SelfType {
    pub fn receiver(&self, cls: &syn::Type, error_mode: ExtractErrorMode) -> TokenStream {
        let py = syn::Ident::new("_py", Span::call_site());
        let _slf = syn::Ident::new("_slf", Span::call_site());
        match self {
            SelfType::Receiver { span, mutable } => {
                let (method, mutability) = if *mutable {
                    (
                        quote_spanned! { *span => extract_pyclass_ref_mut },
                        Some(Token![mut](*span)),
                    )
                } else {
                    (quote_spanned! { *span => extract_pyclass_ref }, None)
                };
                let extract = error_mode.handle_error(
                    &py,
                    quote_spanned! { *span =>
                        _pyo3::impl_::extract_argument::#method(
                            #py.from_borrowed_ptr::<_pyo3::PyAny>(#_slf),
                            &mut holder,
                        )
                    },
                );
                quote_spanned! { *span =>
                    let mut holder = _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT;
                    let #_slf: &#mutability #cls = #extract;
                }
            }
            SelfType::TryFromPyCell(span) => {
                let cell = error_mode.handle_error(
                    &py,
                    quote!{
                        _py.from_borrowed_ptr::<_pyo3::PyAny>(_slf).downcast::<_pyo3::PyCell<#cls>>()
                    }
                );
                quote_spanned! { *span =>
                    let _cell = #cell;
                    #[allow(clippy::useless_conversion)]  // In case _slf is PyCell<Self>
                    let #_slf = ::std::convert::TryFrom::try_from(_cell)?;
                }
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
        } else if cfg!(not(feature = "abi3")) {
            // Not available in the Stable ABI as of Python 3.10
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
    pub unsafety: Option<syn::Token![unsafe]>,
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

        let MethodAttributes {
            ty: fn_type_attr,
            mut python_name,
        } = parse_method_attributes(meth_attrs, name.map(|name| name.value.0))?;

        let (fn_type, skip_first_arg, fixed_convention) =
            Self::parse_fn_type(sig, fn_type_attr, &mut python_name)?;
        ensure_signatures_on_valid_method(&fn_type, signature.as_ref(), text_signature.as_ref())?;

        let name = &sig.ident;
        let ty = get_return_info(&sig.output);
        let python_name = python_name.as_ref().unwrap_or(name).unraw();

        let arguments: Vec<_> = if skip_first_arg {
            sig.inputs
                .iter_mut()
                .skip(1)
                .map(FnArg::parse)
                .collect::<Result<_>>()?
        } else {
            sig.inputs
                .iter_mut()
                .map(FnArg::parse)
                .collect::<Result<_>>()?
        };

        let signature = if let Some(signature) = signature {
            FunctionSignature::from_arguments_and_attribute(arguments, signature)?
        } else {
            FunctionSignature::from_arguments(arguments)?
        };

        let convention =
            fixed_convention.unwrap_or_else(|| CallingConvention::from_signature(&signature));

        Ok(FnSpec {
            tp: fn_type,
            name,
            convention,
            python_name,
            signature,
            output: ty,
            text_signature,
            unsafety: sig.unsafety,
        })
    }

    pub fn null_terminated_python_name(&self) -> syn::LitStr {
        syn::LitStr::new(&format!("{}\0", self.python_name), self.python_name.span())
    }

    fn parse_fn_type(
        sig: &syn::Signature,
        fn_type_attr: Option<MethodTypeAttribute>,
        python_name: &mut Option<syn::Ident>,
    ) -> Result<(FnType, bool, Option<CallingConvention>)> {
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

        let (fn_type, skip_first_arg, fixed_convention) = match fn_type_attr {
            Some(MethodTypeAttribute::StaticMethod) => (FnType::FnStatic, false, None),
            Some(MethodTypeAttribute::ClassAttribute) => (FnType::ClassAttribute, false, None),
            Some(MethodTypeAttribute::New) | Some(MethodTypeAttribute::NewClassMethod) => {
                if let Some(name) = &python_name {
                    bail_spanned!(name.span() => "`name` not allowed with `#[new]`");
                }
                *python_name = Some(syn::Ident::new("__new__", Span::call_site()));
                if matches!(fn_type_attr, Some(MethodTypeAttribute::New)) {
                    (FnType::FnNew, false, Some(CallingConvention::TpNew))
                } else {
                    (FnType::FnNewClass, true, Some(CallingConvention::TpNew))
                }
            }
            Some(MethodTypeAttribute::ClassMethod) => (FnType::FnClass, true, None),
            Some(MethodTypeAttribute::Getter) => {
                // Strip off "get_" prefix if needed
                if python_name.is_none() {
                    *python_name = strip_fn_name("get_");
                }

                (
                    FnType::Getter(parse_receiver("expected receiver for #[getter]")?),
                    true,
                    None,
                )
            }
            Some(MethodTypeAttribute::Setter) => {
                // Strip off "set_" prefix if needed
                if python_name.is_none() {
                    *python_name = strip_fn_name("set_");
                }

                (
                    FnType::Setter(parse_receiver("expected receiver for #[setter]")?),
                    true,
                    None,
                )
            }
            None => (
                FnType::Fn(parse_receiver(
                    "static method needs #[staticmethod] attribute",
                )?),
                true,
                None,
            ),
        };
        Ok((fn_type, skip_first_arg, fixed_convention))
    }

    /// Return a C wrapper function for this signature.
    pub fn get_wrapper_function(
        &self,
        ident: &proc_macro2::Ident,
        cls: Option<&syn::Type>,
    ) -> Result<TokenStream> {
        let self_conversion = self.tp.self_conversion(cls, ExtractErrorMode::Raise);
        let self_arg = self.tp.self_arg();
        let py = syn::Ident::new("_py", Span::call_site());
        let func_name = &self.name;

        let rust_call = |args: Vec<TokenStream>| {
            quote! {
                _pyo3::impl_::pymethods::OkWrap::wrap(function(#self_arg #(#args),*), #py)
                    .map(|obj| _pyo3::conversion::IntoPyPointer::into_ptr(obj))
                    .map_err(::core::convert::Into::into)
            }
        };

        let rust_name = if let Some(cls) = cls {
            quote!(#cls::#func_name)
        } else {
            quote!(#func_name)
        };

        Ok(match self.convention {
            CallingConvention::Noargs => {
                let call = if !self.signature.arguments.is_empty() {
                    // Only `py` arg can be here
                    rust_call(vec![quote!(#py)])
                } else {
                    rust_call(vec![])
                };
                quote! {
                    unsafe fn #ident<'py>(
                        #py: _pyo3::Python<'py>,
                        _slf: *mut _pyo3::ffi::PyObject,
                    ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #self_conversion
                        #call
                    }
                }
            }
            CallingConvention::Fastcall => {
                let (arg_convert, args) = impl_arg_params(self, cls, &py, true)?;
                let call = rust_call(args);
                quote! {
                    unsafe fn #ident<'py>(
                        #py: _pyo3::Python<'py>,
                        _slf: *mut _pyo3::ffi::PyObject,
                        _args: *const *mut _pyo3::ffi::PyObject,
                        _nargs: _pyo3::ffi::Py_ssize_t,
                        _kwnames: *mut _pyo3::ffi::PyObject
                    ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #self_conversion
                        #arg_convert
                        #call
                    }
                }
            }
            CallingConvention::Varargs => {
                let (arg_convert, args) = impl_arg_params(self, cls, &py, false)?;
                let call = rust_call(args);
                quote! {
                    unsafe fn #ident<'py>(
                        #py: _pyo3::Python<'py>,
                        _slf: *mut _pyo3::ffi::PyObject,
                        _args: *mut _pyo3::ffi::PyObject,
                        _kwargs: *mut _pyo3::ffi::PyObject
                    ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #self_conversion
                        #arg_convert
                        #call
                    }
                }
            }
            CallingConvention::TpNew => {
                let (arg_convert, args) = impl_arg_params(self, cls, &py, false)?;
                let call = match &self.tp {
                    FnType::FnNew => quote! { #rust_name(#(#args),*) },
                    FnType::FnNewClass => {
                        quote! { #rust_name(_pyo3::types::PyType::from_type_ptr(#py, subtype), #(#args),*) }
                    }
                    x => panic!("Only `FnNew` or `FnNewClass` may use the `TpNew` calling convention. Got: {:?}", x),
                };
                quote! {
                    unsafe fn #ident(
                        #py: _pyo3::Python<'_>,
                        subtype: *mut _pyo3::ffi::PyTypeObject,
                        _args: *mut _pyo3::ffi::PyObject,
                        _kwargs: *mut _pyo3::ffi::PyObject
                    ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                        use _pyo3::callback::IntoPyCallbackOutput;
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #arg_convert
                        let result = #call;
                        let initializer: _pyo3::PyClassInitializer::<#cls> = result.convert(#py)?;
                        let cell = initializer.create_cell_from_subtype(#py, subtype)?;
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
            FnType::FnModule => Some("module"),
            FnType::FnClass | FnType::FnNewClass => Some("cls"),
            FnType::FnStatic | FnType::FnNew => None,
        };

        match self.text_signature.as_ref().map(|attr| &attr.value) {
            Some(TextSignatureAttributeValue::Str(s)) => Some(s.value()),
            None => Some(self.signature.text_signature(self_argument)),
            Some(TextSignatureAttributeValue::Disabled(_)) => None,
        }
    }
}

#[derive(Debug)]
struct MethodAttributes {
    ty: Option<MethodTypeAttribute>,
    python_name: Option<syn::Ident>,
}

fn parse_method_attributes(
    attrs: &mut Vec<syn::Attribute>,
    mut python_name: Option<syn::Ident>,
) -> Result<MethodAttributes> {
    let mut new_attrs = Vec::new();
    let mut ty: Option<MethodTypeAttribute> = None;

    macro_rules! set_compound_ty {
        ($new_ty:expr, $ident:expr) => {
            ty = match (ty, $new_ty) {
                (None, new_ty) => Some(new_ty),
                (Some(MethodTypeAttribute::ClassMethod), MethodTypeAttribute::New) => Some(MethodTypeAttribute::NewClassMethod),
                (Some(MethodTypeAttribute::New), MethodTypeAttribute::ClassMethod) => Some(MethodTypeAttribute::NewClassMethod),
                (Some(_), _) => bail_spanned!($ident.span() => "can only combine `new` and `classmethod`"),
            };
        };
    }

    macro_rules! set_ty {
        ($new_ty:expr, $ident:expr) => {
            ensure_spanned!(
               ty.replace($new_ty).is_none(),
               $ident.span() => "cannot combine these method types"
            );
        };
    }

    for attr in attrs.drain(..) {
        match attr.meta {
            syn::Meta::Path(ref name) => {
                if name.is_ident("new") || name.is_ident("__new__") {
                    set_compound_ty!(MethodTypeAttribute::New, name);
                } else if name.is_ident("init") || name.is_ident("__init__") {
                    bail_spanned!(name.span() => "#[init] is disabled since PyO3 0.9.0");
                } else if name.is_ident("call") || name.is_ident("__call__") {
                    bail_spanned!(name.span() => "use `fn __call__` instead of `#[call]` attribute since PyO3 0.15.0");
                } else if name.is_ident("classmethod") {
                    set_compound_ty!(MethodTypeAttribute::ClassMethod, name);
                } else if name.is_ident("staticmethod") {
                    set_ty!(MethodTypeAttribute::StaticMethod, name);
                } else if name.is_ident("classattr") {
                    set_ty!(MethodTypeAttribute::ClassAttribute, name);
                } else if name.is_ident("setter") || name.is_ident("getter") {
                    if let syn::AttrStyle::Inner(_) = attr.style {
                        bail_spanned!(
                            attr.span() => "inner attribute is not supported for setter and getter"
                        );
                    }
                    if name.is_ident("setter") {
                        set_ty!(MethodTypeAttribute::Setter, name);
                    } else {
                        set_ty!(MethodTypeAttribute::Getter, name);
                    }
                } else {
                    new_attrs.push(attr)
                }
            }
            syn::Meta::List(ref ml @ syn::MetaList { ref path, .. }) => {
                if path.is_ident("new") {
                    set_ty!(MethodTypeAttribute::New, path);
                } else if path.is_ident("init") {
                    bail_spanned!(path.span() => "#[init] is disabled since PyO3 0.9.0");
                } else if path.is_ident("call") {
                    ensure_spanned!(
                        python_name.is_none(),
                        python_name.span() => "`name` may not be used with `#[call]`"
                    );
                    python_name = Some(syn::Ident::new("__call__", Span::call_site()));
                } else if path.is_ident("setter") || path.is_ident("getter") {
                    if let syn::AttrStyle::Inner(_) = attr.style {
                        bail_spanned!(
                            attr.span() => "inner attribute is not supported for setter and getter"
                        );
                    }

                    if path.is_ident("setter") {
                        set_ty!(MethodTypeAttribute::Setter, path);
                    } else {
                        set_ty!(MethodTypeAttribute::Getter, path);
                    };

                    ensure_spanned!(
                        python_name.is_none(),
                        python_name.span() => "`name` may only be specified once"
                    );

                    if let Ok(ident) = ml.parse_args::<syn::Ident>() {
                        python_name = Some(ident);
                    } else if let Ok(syn::Lit::Str(s)) = ml.parse_args::<syn::Lit>() {
                        python_name = Some(s.parse()?);
                    } else {
                        return Err(syn::Error::new_spanned(
                            ml,
                            "expected ident or string literal for property name",
                        ));
                    }
                } else {
                    new_attrs.push(attr)
                }
            }
            syn::Meta::NameValue(_) => new_attrs.push(attr),
        }
    }

    *attrs = new_attrs;

    Ok(MethodAttributes { ty, python_name })
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
