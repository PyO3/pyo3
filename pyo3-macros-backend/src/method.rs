use std::borrow::Cow;
use std::ffi::CString;
use std::fmt::Display;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::LitCStr;
use syn::{ext::IdentExt, spanned::Spanned, Ident, Result};

#[cfg(feature = "experimental-inspect")]
use crate::py_expr::PyExpr;
use crate::pyfunction::{PyFunctionWarning, WarningFactory};
use crate::pyversions::is_abi3_before;
use crate::utils::Ctx;
use crate::{
    attributes::{FromPyWithAttribute, TextSignatureAttribute, TextSignatureAttributeValue},
    params::{impl_arg_params, Holders},
    pyfunction::{
        FunctionSignature, PyFunctionArgPyO3Attributes, PyFunctionOptions, SignatureAttribute,
    },
    quotes,
    utils::{self, PythonDoc},
};

#[derive(Clone, Debug)]
pub struct RegularArg<'a> {
    pub name: Cow<'a, syn::Ident>,
    pub ty: &'a syn::Type,
    pub from_py_with: Option<FromPyWithAttribute>,
    pub default_value: Option<syn::Expr>,
    pub option_wrapped_type: Option<&'a syn::Type>,
    #[cfg(feature = "experimental-inspect")]
    pub annotation: Option<PyExpr>,
}

/// Pythons *args argument
#[derive(Clone, Debug)]
pub struct VarargsArg<'a> {
    pub name: Cow<'a, syn::Ident>,
    pub ty: &'a syn::Type,
    #[cfg(feature = "experimental-inspect")]
    pub annotation: Option<PyExpr>,
}

/// Pythons **kwarg argument
#[derive(Clone, Debug)]
pub struct KwargsArg<'a> {
    pub name: Cow<'a, syn::Ident>,
    pub ty: &'a syn::Type,
    #[cfg(feature = "experimental-inspect")]
    pub annotation: Option<PyExpr>,
}

#[derive(Clone, Debug)]
pub struct CancelHandleArg<'a> {
    pub name: &'a syn::Ident,
    pub ty: &'a syn::Type,
}

#[derive(Clone, Debug)]
pub struct PyArg<'a> {
    pub name: &'a syn::Ident,
    pub ty: &'a syn::Type,
}

#[allow(clippy::large_enum_variant)] // See #5039
#[derive(Clone, Debug)]
pub enum FnArg<'a> {
    Regular(RegularArg<'a>),
    VarArgs(VarargsArg<'a>),
    KwArgs(KwargsArg<'a>),
    Py(PyArg<'a>),
    CancelHandle(CancelHandleArg<'a>),
}

impl<'a> FnArg<'a> {
    pub fn name(&self) -> &syn::Ident {
        match self {
            FnArg::Regular(RegularArg { name, .. }) => name,
            FnArg::VarArgs(VarargsArg { name, .. }) => name,
            FnArg::KwArgs(KwargsArg { name, .. }) => name,
            FnArg::Py(PyArg { name, .. }) => name,
            FnArg::CancelHandle(CancelHandleArg { name, .. }) => name,
        }
    }

    pub fn ty(&self) -> &'a syn::Type {
        match self {
            FnArg::Regular(RegularArg { ty, .. }) => ty,
            FnArg::VarArgs(VarargsArg { ty, .. }) => ty,
            FnArg::KwArgs(KwargsArg { ty, .. }) => ty,
            FnArg::Py(PyArg { ty, .. }) => ty,
            FnArg::CancelHandle(CancelHandleArg { ty, .. }) => ty,
        }
    }

    #[expect(
        clippy::wrong_self_convention,
        reason = "called `from_` but not a constructor"
    )]
    pub fn from_py_with(&self) -> Option<&FromPyWithAttribute> {
        if let FnArg::Regular(RegularArg { from_py_with, .. }) = self {
            from_py_with.as_ref()
        } else {
            None
        }
    }

    pub fn to_varargs_mut(&mut self) -> Result<&mut Self> {
        if let Self::Regular(RegularArg {
            name,
            ty,
            option_wrapped_type: None,
            #[cfg(feature = "experimental-inspect")]
            annotation,
            ..
        }) = self
        {
            *self = Self::VarArgs(VarargsArg {
                name: name.clone(),
                ty,
                #[cfg(feature = "experimental-inspect")]
                annotation: annotation.clone(),
            });
            Ok(self)
        } else {
            bail_spanned!(self.name().span() => "args cannot be optional")
        }
    }

    pub fn to_kwargs_mut(&mut self) -> Result<&mut Self> {
        if let Self::Regular(RegularArg {
            name,
            ty,
            option_wrapped_type: Some(..),
            #[cfg(feature = "experimental-inspect")]
            annotation,
            ..
        }) = self
        {
            *self = Self::KwArgs(KwargsArg {
                name: name.clone(),
                ty,
                #[cfg(feature = "experimental-inspect")]
                annotation: annotation.clone(),
            });
            Ok(self)
        } else {
            bail_spanned!(self.name().span() => "kwargs must be Option<_>")
        }
    }

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

                let PyFunctionArgPyO3Attributes {
                    from_py_with,
                    cancel_handle,
                } = PyFunctionArgPyO3Attributes::from_attrs(&mut cap.attrs)?;
                let ident = match &*cap.pat {
                    syn::Pat::Ident(syn::PatIdent { ident, .. }) => ident,
                    other => return Err(handle_argument_error(other)),
                };

                if utils::is_python(&cap.ty) {
                    return Ok(Self::Py(PyArg {
                        name: ident,
                        ty: &cap.ty,
                    }));
                }

                if cancel_handle.is_some() {
                    // `PyFunctionArgPyO3Attributes::from_attrs` validates that
                    // only compatible attributes are specified, either
                    // `cancel_handle` or `from_py_with`, duplicates and any
                    // combination of the two are already rejected.
                    return Ok(Self::CancelHandle(CancelHandleArg {
                        name: ident,
                        ty: &cap.ty,
                    }));
                }

                Ok(Self::Regular(RegularArg {
                    name: Cow::Borrowed(ident),
                    ty: &cap.ty,
                    from_py_with,
                    default_value: None,
                    option_wrapped_type: utils::option_type_argument(&cap.ty),
                    #[cfg(feature = "experimental-inspect")]
                    annotation: None,
                }))
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

/// Represents what kind of a function a pyfunction or pymethod is
#[derive(Clone, Debug)]
pub enum FnType {
    /// Represents a pymethod annotated with `#[getter]`
    Getter(SelfType),
    /// Represents a pymethod annotated with `#[setter]`
    Setter(SelfType),
    /// Represents a pymethod annotated with `#[deleter]`
    Deleter(SelfType),
    /// Represents a regular pymethod
    Fn(SelfType),
    /// Represents a pymethod annotated with `#[classmethod]`, like a `@classmethod`
    FnClass(Span),
    /// Represents a pyfunction or a pymethod annotated with `#[staticmethod]`, like a `@staticmethod`
    FnStatic,
    /// Represents a pyfunction annotated with `#[pyo3(pass_module)]
    FnModule(Span),
    /// Represents a pymethod or associated constant annotated with `#[classattr]`
    ClassAttribute,
}

impl FnType {
    pub fn skip_first_rust_argument_in_python_signature(&self) -> bool {
        match self {
            FnType::Getter(_)
            | FnType::Setter(_)
            | FnType::Deleter(_)
            | FnType::Fn(_)
            | FnType::FnClass(_)
            | FnType::FnModule(_) => true,
            FnType::FnStatic | FnType::ClassAttribute => false,
        }
    }

    pub fn signature_attribute_allowed(&self) -> bool {
        match self {
            FnType::Fn(_) | FnType::FnStatic | FnType::FnClass(_) | FnType::FnModule(_) => true,
            // Getter, Setter and Deleter and ClassAttribute all have fixed signatures (either take 0 or 1
            // arguments) so cannot have a `signature = (...)` attribute.
            FnType::Getter(_) | FnType::Setter(_) | FnType::Deleter(_) | FnType::ClassAttribute => {
                false
            }
        }
    }

    pub fn self_arg(
        &self,
        cls: Option<&syn::Type>,
        error_mode: ExtractErrorMode,
        holders: &mut Holders,
        ctx: &Ctx,
    ) -> Option<TokenStream> {
        let Ctx { pyo3_path, .. } = ctx;
        match self {
            FnType::Getter(st) | FnType::Setter(st) | FnType::Deleter(st) | FnType::Fn(st) => {
                Some(st.receiver(
                    cls.expect("no class given for Fn with a \"self\" receiver"),
                    error_mode,
                    holders,
                    ctx,
                ))
            }
            FnType::FnClass(span) => {
                let py = syn::Ident::new("py", Span::call_site());
                let slf: Ident = syn::Ident::new("_slf", Span::call_site());
                let pyo3_path = pyo3_path.to_tokens_spanned(*span);
                let ret = quote_spanned! { *span =>
                    #[allow(clippy::useless_conversion, reason = "#[classmethod] accepts anything which implements `From<BoundRef<PyType>>`")]
                    ::std::convert::Into::into(
                        #pyo3_path::impl_::pymethods::BoundRef::ref_from_ptr(#py, &*(&#slf as *const _ as *const *mut _))
                            .cast_unchecked::<#pyo3_path::types::PyType>()
                    )
                };
                Some(quote! { unsafe { #ret } })
            }
            FnType::FnModule(span) => {
                let py = syn::Ident::new("py", Span::call_site());
                let slf: Ident = syn::Ident::new("_slf", Span::call_site());
                let pyo3_path = pyo3_path.to_tokens_spanned(*span);
                let ret = quote_spanned! { *span =>
                    #[allow(clippy::useless_conversion, reason = "`pass_module` accepts anything which implements `From<BoundRef<PyModule>>`")]
                    ::std::convert::Into::into(
                        #pyo3_path::impl_::pymethods::BoundRef::ref_from_ptr(#py, &*(&#slf as *const _ as *const *mut _))
                            .cast_unchecked::<#pyo3_path::types::PyModule>()
                    )
                };
                Some(quote! { unsafe { #ret } })
            }
            FnType::FnStatic | FnType::ClassAttribute => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SelfType {
    Receiver { mutable: bool, span: Span },
    TryFromBoundRef(Span),
}

#[derive(Clone, Copy)]
pub enum ExtractErrorMode {
    NotImplemented,
    Raise,
}

impl ExtractErrorMode {
    pub fn handle_error(self, extract: TokenStream, ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        match self {
            ExtractErrorMode::Raise => quote! { #extract? },
            ExtractErrorMode::NotImplemented => quote! {
                match #extract {
                    ::std::result::Result::Ok(value) => value,
                    ::std::result::Result::Err(_) => { return #pyo3_path::impl_::callback::convert(py, py.NotImplemented()); },
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
        holders: &mut Holders,
        ctx: &Ctx,
    ) -> TokenStream {
        // Due to use of quote_spanned in this function, need to bind these idents to the
        // main macro callsite.
        let py = syn::Ident::new("py", Span::call_site());
        let slf = syn::Ident::new("_slf", Span::call_site());
        let Ctx { pyo3_path, .. } = ctx;
        match self {
            SelfType::Receiver { span, mutable } => {
                let arg = quote! { unsafe { #pyo3_path::impl_::extract_argument::cast_function_argument(#py, #slf) } };
                let method = if *mutable {
                    syn::Ident::new("extract_pyclass_ref_mut", *span)
                } else {
                    syn::Ident::new("extract_pyclass_ref", *span)
                };
                let holder = holders.push_holder(*span);
                let pyo3_path = pyo3_path.to_tokens_spanned(*span);
                error_mode.handle_error(
                    quote_spanned! { *span =>
                        #pyo3_path::impl_::extract_argument::#method::<#cls>(
                            #arg,
                            &mut #holder,
                        )
                    },
                    ctx,
                )
            }
            SelfType::TryFromBoundRef(span) => {
                let bound_ref = quote! { unsafe { #pyo3_path::impl_::pymethods::BoundRef::ref_from_ptr(#py, &#slf) } };
                let pyo3_path = pyo3_path.to_tokens_spanned(*span);
                error_mode.handle_error(
                    quote_spanned! { *span =>
                        #bound_ref.cast::<#cls>()
                            .map_err(::std::convert::Into::<#pyo3_path::PyErr>::into)
                            .and_then(
                                #[allow(clippy::unnecessary_fallible_conversions, reason = "anything implementing `TryFrom<BoundRef>` is permitted")]
                                |bound| ::std::convert::TryFrom::try_from(bound).map_err(::std::convert::Into::into)
                            )

                    },
                    ctx
                )
            }
        }
    }
}

/// Determines which CPython calling convention a given FnSpec uses.
#[derive(Clone, Debug, Copy)]
pub enum CallingConvention {
    Noargs,   // METH_NOARGS
    Varargs,  // METH_VARARGS | METH_KEYWORDS
    Fastcall, // METH_FASTCALL | METH_KEYWORDS (not compatible with `abi3` feature before 3.10)
}

impl CallingConvention {
    /// Determine default calling convention from an argument signature.
    ///
    /// Different other slots (tp_call, tp_new) can have other requirements
    /// and are set manually (see `parse_fn_type` below).
    pub fn from_signature(signature: &FunctionSignature<'_>) -> Self {
        if signature.python_signature.has_no_args() {
            Self::Noargs
        } else if signature.python_signature.kwargs.is_none() && !is_abi3_before(3, 10) {
            // For functions that accept **kwargs, always prefer varargs for now based on
            // historical performance testing.
            //
            // FASTCALL not compatible with `abi3` before 3.10
            Self::Fastcall
        } else {
            Self::Varargs
        }
    }
}

#[derive(Clone)]
pub struct FnSpec<'a> {
    pub tp: FnType,
    // Rust function name
    pub name: &'a syn::Ident,
    // Wrapped python name. This should not have any leading r#.
    // r# can be removed by syn::ext::IdentExt::unraw()
    pub python_name: syn::Ident,
    pub signature: FunctionSignature<'a>,
    pub text_signature: Option<TextSignatureAttribute>,
    pub asyncness: Option<syn::Token![async]>,
    pub unsafety: Option<syn::Token![unsafe]>,
    pub warnings: Vec<PyFunctionWarning>,
    pub output: syn::ReturnType,
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
            Ok(SelfType::TryFromBoundRef(ty.span()))
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
            warnings,
            ..
        } = options;

        let mut python_name = name.map(|name| name.value.0);

        let fn_type = Self::parse_fn_type(sig, meth_attrs, &mut python_name)?;
        ensure_signatures_on_valid_method(&fn_type, signature.as_ref(), text_signature.as_ref())?;

        let name = &sig.ident;
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
            FunctionSignature::from_arguments(arguments)
        };

        Ok(FnSpec {
            tp: fn_type,
            name,
            python_name,
            signature,
            text_signature,
            asyncness: sig.asyncness,
            unsafety: sig.unsafety,
            warnings,
            output: sig.output.clone(),
        })
    }

    pub fn null_terminated_python_name(&self) -> LitCStr {
        let name = self.python_name.to_string();
        let name = CString::new(name).unwrap();
        LitCStr::new(&name, self.python_name.span())
    }

    fn parse_fn_type(
        sig: &syn::Signature,
        meth_attrs: &mut Vec<syn::Attribute>,
        python_name: &mut Option<syn::Ident>,
    ) -> Result<FnType> {
        let mut method_attributes = parse_method_attributes(meth_attrs)?;

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
                FnType::FnStatic
            }
            [MethodTypeAttribute::New(_), MethodTypeAttribute::ClassMethod(span)]
            | [MethodTypeAttribute::ClassMethod(span), MethodTypeAttribute::New(_)] => {
                set_name_to_new()?;
                FnType::FnClass(*span)
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
            [MethodTypeAttribute::Deleter(_, name)] => {
                if let Some(name) = name.take() {
                    ensure_spanned!(
                        python_name.replace(name).is_none(),
                        python_name.span() => "`name` may only be specified once"
                    );
                } else if python_name.is_none() {
                    // Strip off "delete_" prefix if needed
                    *python_name = strip_fn_name("delete_");
                }

                FnType::Deleter(parse_receiver("expected receiver for `#[deleter]`")?)
            }
            [first, rest @ .., last] => {
                // Join as many of the spans together as possible
                let span = rest
                    .iter()
                    .fold(first.span(), |s, next| s.join(next.span()).unwrap_or(s));
                let span = span.join(last.span()).unwrap_or(span);
                // List all the attributes in the error message
                let mut msg = format!("`{first}` may not be combined with");
                let mut is_first = true;
                for attr in &*rest {
                    msg.push_str(&format!(" `{attr}`"));
                    if is_first {
                        is_first = false;
                    } else {
                        msg.push(',');
                    }
                }
                if !rest.is_empty() {
                    msg.push_str(" and");
                }
                msg.push_str(&format!(" `{last}`"));
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
        convention: CallingConvention,
        ctx: &Ctx,
    ) -> Result<TokenStream> {
        let Ctx {
            pyo3_path,
            output_span,
        } = ctx;
        let mut cancel_handle_iter = self
            .signature
            .arguments
            .iter()
            .filter(|arg| matches!(arg, FnArg::CancelHandle(..)));
        let cancel_handle = cancel_handle_iter.next();
        if let Some(FnArg::CancelHandle(CancelHandleArg { name, .. })) = cancel_handle {
            ensure_spanned!(self.asyncness.is_some(), name.span() => "`cancel_handle` attribute can only be used with `async fn`");
            if let Some(FnArg::CancelHandle(CancelHandleArg { name, .. })) =
                cancel_handle_iter.next()
            {
                bail_spanned!(name.span() => "`cancel_handle` may only be specified once");
            }
        }

        let rust_call = |args: Vec<TokenStream>, mut holders: Holders| {
            let self_arg = self
                .tp
                .self_arg(cls, ExtractErrorMode::Raise, &mut holders, ctx);
            let init_holders = holders.init_holders(ctx);

            // We must assign the output_span to the return value of the call,
            // but *not* of the call itself otherwise the spans get really weird
            let ret_ident = Ident::new("ret", *output_span);

            if self.asyncness.is_some() {
                // For async functions, we need to build up a coroutine object to return from the initial function call.
                //
                // Extraction of the call signature (positional & keyword arguments) happens as part of the initial function
                // call. The Python objects are then moved into the Rust future that will be executed when the coroutine is
                // awaited.
                //
                // The argument extraction from Python objects to Rust values then happens inside the future, this allows
                // things like extraction to `&MyClass` which needs a holder (for the class guard) to work properly inside
                // async code.
                //
                // It *might* be possible in the future to do the extraction before the coroutine is created, but that would require
                // changing argument extraction code to first create holders and then read the values from them later.
                let (throw_callback, init_throw_callback) = if cancel_handle.is_some() {
                    (
                        quote! { Some(__throw_callback) },
                        Some(
                            quote! { let __cancel_handle = #pyo3_path::coroutine::CancelHandle::new();
                            let __throw_callback = __cancel_handle.throw_callback(); },
                        ),
                    )
                } else {
                    (quote! { None }, None)
                };
                let python_name = &self.python_name;
                let qualname_prefix = match cls {
                    Some(cls) => quote!(Some(<#cls as #pyo3_path::PyClass>::NAME)),
                    None => quote!(None),
                };
                // copy self arg into async block
                // slf_py will create the owned value to store in the future
                // slf_ptr recreates the raw pointer temporarily when building the future
                let (slf_py, slf_ptr) = if self_arg.is_some() {
                    (
                        Some(
                            quote! { let _slf = #pyo3_path::Borrowed::from_ptr(py, _slf).to_owned().unbind(); },
                        ),
                        Some(quote! { let _slf = _slf.as_ptr(); }),
                    )
                } else {
                    (None, None)
                };
                // copy extracted arguments into async block
                // output_py will create the owned arguments to store in the future
                // output_args recreates the borrowed objects temporarily when building the future
                let (output_py, output_args) = if !matches!(convention, CallingConvention::Noargs) {
                    (
                        Some(quote! {
                            let output = output.map(|o| o.map(Py::from));
                        }),
                        Some(quote! {
                            let output = output.each_ref().map(|o| o.as_ref().map(|obj| obj.bind_borrowed(assume_attached.py())));
                        }),
                    )
                } else {
                    (None, None)
                };
                let args = self_arg.into_iter().chain(args);
                let ok_wrap = quotes::ok_wrap(ret_ident.to_token_stream(), ctx);
                quote! {
                    {
                        let coroutine = {
                            #slf_py
                            #output_py
                            #init_throw_callback
                            #pyo3_path::impl_::coroutine::new_coroutine(
                                #pyo3_path::intern!(py, stringify!(#python_name)),
                                #qualname_prefix,
                                #throw_callback,
                                async move {
                                    // SAFETY: attached when future is polled (see `Coroutine::poll`)
                                    let assume_attached = unsafe { #pyo3_path::impl_::coroutine::AssumeAttachedInCoroutine::new() };
                                    #init_holders
                                    let future = {
                                        let py = assume_attached.py();
                                        #slf_ptr
                                        #output_args
                                        function(#(#args),*)
                                    };
                                    let #ret_ident = future.await;
                                    let #ret_ident = #ok_wrap;
                                    #pyo3_path::impl_::wrap::converter(&#ret_ident).map_into_pyobject(assume_attached.py(), #ret_ident)
                                },
                            )
                        };
                        #pyo3_path::Py::new(py, coroutine).map(#pyo3_path::Py::into_ptr)
                    }
                }
            } else {
                let args = self_arg.into_iter().chain(args);
                let return_conversion = quotes::map_result_into_ptr(
                    quotes::ok_wrap(ret_ident.to_token_stream(), ctx),
                    ctx,
                );
                quote! {
                    {
                        #init_holders
                        let #ret_ident = function(#(#args),*);
                        #return_conversion
                    }
                }
            }
        };

        let func_name = &self.name;
        let rust_name = if let Some(cls) = cls {
            quote!(#cls::#func_name)
        } else {
            quote!(#func_name)
        };

        let warnings = self.warnings.build_py_warning(ctx);
        let mut holders = Holders::new();

        Ok(match convention {
            CallingConvention::Noargs => {
                let args = self
                    .signature
                    .arguments
                    .iter()
                    .map(|arg| match arg {
                        FnArg::Py(..) => quote!(py),
                        FnArg::CancelHandle(..) => quote!(__cancel_handle),
                        _ => unreachable!("`CallingConvention::Noargs` should not contain any arguments (reaching Python) except for `self`, which is handled below."),
                    })
                    .collect();
                let call = rust_call(args, holders);
                quote! {
                    unsafe fn #ident<'py>(
                        py: #pyo3_path::Python<'py>,
                        _slf: *mut #pyo3_path::ffi::PyObject,
                    ) -> #pyo3_path::PyResult<*mut #pyo3_path::ffi::PyObject> {
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #warnings
                        let result = #call;
                        result
                    }
                }
            }
            CallingConvention::Fastcall => {
                let (arg_convert, args) = impl_arg_params(self, cls, true, &mut holders, ctx);
                let call = rust_call(args, holders);

                quote! {
                    unsafe fn #ident<'py>(
                        py: #pyo3_path::Python<'py>,
                        _slf: *mut #pyo3_path::ffi::PyObject,
                        _args: *const *mut #pyo3_path::ffi::PyObject,
                        _nargs: #pyo3_path::ffi::Py_ssize_t,
                        _kwnames: *mut #pyo3_path::ffi::PyObject
                    ) -> #pyo3_path::PyResult<*mut #pyo3_path::ffi::PyObject> {
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #arg_convert
                        #warnings
                        let result = #call;
                        result
                    }
                }
            }
            CallingConvention::Varargs => {
                let (arg_convert, args) = impl_arg_params(self, cls, false, &mut holders, ctx);
                let call = rust_call(args, holders);

                quote! {
                    unsafe fn #ident<'py>(
                        py: #pyo3_path::Python<'py>,
                        _slf: *mut #pyo3_path::ffi::PyObject,
                        _args: *mut #pyo3_path::ffi::PyObject,
                        _kwargs: *mut #pyo3_path::ffi::PyObject
                    ) -> #pyo3_path::PyResult<*mut #pyo3_path::ffi::PyObject> {
                        let function = #rust_name; // Shadow the function name to avoid #3017
                        #arg_convert
                        #warnings
                        let result = #call;
                        result
                    }
                }
            }
        })
    }

    /// Return a `PyMethodDef` constructor for this function, matching the selected
    /// calling convention.
    pub fn get_methoddef(
        &self,
        wrapper: impl ToTokens,
        doc: &PythonDoc,
        convention: CallingConvention,
        ctx: &Ctx,
    ) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        let python_name = self.null_terminated_python_name();
        let flags = match self.tp {
            FnType::FnClass(_) => quote! { .flags(#pyo3_path::ffi::METH_CLASS) },
            FnType::FnStatic => quote! { .flags(#pyo3_path::ffi::METH_STATIC) },
            _ => quote! {},
        };
        let trampoline = match convention {
            CallingConvention::Noargs => Ident::new("noargs", Span::call_site()),
            CallingConvention::Fastcall => {
                Ident::new("fastcall_cfunction_with_keywords", Span::call_site())
            }
            CallingConvention::Varargs => Ident::new("cfunction_with_keywords", Span::call_site()),
        };
        quote! {
            #pyo3_path::impl_::pymethods::PyMethodDef::#trampoline(
                #python_name,
                #pyo3_path::impl_::trampoline::get_trampoline_function!(#trampoline, #wrapper),
                #doc,
            ) #flags
        }
    }

    /// Forwards to [utils::get_doc] with the text signature of this spec.
    pub fn get_doc(&self, attrs: &[syn::Attribute], ctx: &Ctx) -> syn::Result<PythonDoc> {
        let text_signature = self
            .text_signature_call_signature()
            .map(|sig| format!("{}{}", self.python_name, sig));
        utils::get_doc(attrs, text_signature, ctx)
    }

    /// Creates the parenthesised arguments list for `__text_signature__` snippet based on this spec's signature
    /// and/or attributes. Prepend the callable name to make a complete `__text_signature__`.
    pub fn text_signature_call_signature(&self) -> Option<String> {
        let self_argument = match &self.tp {
            // Getters / Setters / deleter / ClassAttribute are not callables on the Python side
            FnType::Getter(_) | FnType::Setter(_) | FnType::Deleter(_) | FnType::ClassAttribute => {
                return None
            }
            FnType::Fn(_) => Some("self"),
            FnType::FnModule(_) => Some("module"),
            FnType::FnClass(_) => Some("cls"),
            FnType::FnStatic => None,
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
    Deleter(Span, Option<Ident>),
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
            | MethodTypeAttribute::Deleter(span, _)
            | MethodTypeAttribute::ClassAttribute(span) => *span,
        }
    }

    /// Attempts to parse a method type attribute.
    ///
    /// If the attribute does not match one of the attribute names, returns `Ok(None)`.
    ///
    /// Otherwise will either return a parse error or the attribute.
    fn parse_if_matching_attribute(attr: &syn::Attribute) -> Result<Option<Self>> {
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
        } else if path.is_ident("deleter") {
            let name = extract_name(meta, "deleter")?;
            Ok(Some(MethodTypeAttribute::Deleter(path.span(), name)))
        } else {
            Ok(None)
        }
    }
}

impl Display for MethodTypeAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MethodTypeAttribute::New(_) => "#[new]",
            MethodTypeAttribute::ClassMethod(_) => "#[classmethod]",
            MethodTypeAttribute::StaticMethod(_) => "#[staticmethod]",
            MethodTypeAttribute::Getter(_, _) => "#[getter]",
            MethodTypeAttribute::Setter(_, _) => "#[setter]",
            MethodTypeAttribute::Deleter(_, _) => "#[deleter]",
            MethodTypeAttribute::ClassAttribute(_) => "#[classattr]",
        })
    }
}

fn parse_method_attributes(attrs: &mut Vec<syn::Attribute>) -> Result<Vec<MethodTypeAttribute>> {
    let mut new_attrs = Vec::new();
    let mut found_attrs = Vec::new();

    for attr in attrs.drain(..) {
        match MethodTypeAttribute::parse_if_matching_attribute(&attr)? {
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
Try `&self`, `&mut self, `slf: PyClassGuard<'_, Self>` or `slf: PyClassGuardMut<'_, Self>`.";

fn ensure_signatures_on_valid_method(
    fn_type: &FnType,
    signature: Option<&SignatureAttribute>,
    text_signature: Option<&TextSignatureAttribute>,
) -> syn::Result<()> {
    if let Some(signature) = signature {
        match fn_type {
            FnType::Getter(_) => {
                debug_assert!(!fn_type.signature_attribute_allowed());
                bail_spanned!(signature.kw.span() => "`signature` not allowed with `getter`")
            }
            FnType::Setter(_) => {
                debug_assert!(!fn_type.signature_attribute_allowed());
                bail_spanned!(signature.kw.span() => "`signature` not allowed with `setter`")
            }
            FnType::Deleter(_) => {
                debug_assert!(!fn_type.signature_attribute_allowed());
                bail_spanned!(signature.kw.span() => "`signature` not allowed with `deleter`")
            }
            FnType::ClassAttribute => {
                debug_assert!(!fn_type.signature_attribute_allowed());
                bail_spanned!(signature.kw.span() => "`signature` not allowed with `classattr`")
            }
            _ => debug_assert!(fn_type.signature_attribute_allowed()),
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
            FnType::Deleter(_) => {
                bail_spanned!(text_signature.kw.span() => "`text_signature` not allowed with `deleter`")
            }
            FnType::ClassAttribute => {
                bail_spanned!(text_signature.kw.span() => "`text_signature` not allowed with `classattr`")
            }
            _ => {}
        }
    }
    Ok(())
}
