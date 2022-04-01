// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::attributes::TextSignatureAttribute;
use crate::deprecations::Deprecation;
use crate::params::{accept_args_kwargs, impl_arg_params};
use crate::pyfunction::PyFunctionOptions;
use crate::pyfunction::{PyFunctionArgPyO3Attributes, PyFunctionSignature};
use crate::utils::{self, get_pyo3_crate, PythonDoc};
use crate::{deprecations::Deprecations, pyfunction::Argument};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use quote::{quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::Result;

#[derive(Clone, Debug)]
pub struct FnArg<'a> {
    pub name: &'a syn::Ident,
    pub by_ref: &'a Option<syn::token::Ref>,
    pub mutability: &'a Option<syn::token::Mut>,
    pub ty: &'a syn::Type,
    pub optional: Option<&'a syn::Type>,
    pub py: bool,
    pub attrs: PyFunctionArgPyO3Attributes,
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
                let (ident, by_ref, mutability) = match *cap.pat {
                    syn::Pat::Ident(syn::PatIdent {
                        ref ident,
                        ref by_ref,
                        ref mutability,
                        ..
                    }) => (ident, by_ref, mutability),
                    _ => bail_spanned!(cap.pat.span() => "unsupported argument"),
                };

                Ok(FnArg {
                    name: ident,
                    by_ref,
                    mutability,
                    ty: &cap.ty,
                    optional: utils::option_type_argument(&cap.ty),
                    py: utils::is_python(&cap.ty),
                    attrs: arg_attrs,
                })
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug, Copy, Eq)]
pub enum MethodTypeAttribute {
    /// `#[new]`
    New,
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
            FnType::FnClass => {
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
    Receiver { mutable: bool },
    TryFromPyCell(Span),
}

#[derive(Clone, Copy)]
pub enum ExtractErrorMode {
    NotImplemented,
    Raise,
}

impl SelfType {
    pub fn receiver(&self, cls: &syn::Type, error_mode: ExtractErrorMode) -> TokenStream {
        let cell = match error_mode {
            ExtractErrorMode::Raise => {
                quote! { _py.from_borrowed_ptr::<_pyo3::PyAny>(_slf).downcast::<_pyo3::PyCell<#cls>>()? }
            }
            ExtractErrorMode::NotImplemented => {
                quote! {
                    match _py.from_borrowed_ptr::<_pyo3::PyAny>(_slf).downcast::<_pyo3::PyCell<#cls>>() {
                        ::std::result::Result::Ok(cell) => cell,
                        ::std::result::Result::Err(_) => return _pyo3::callback::convert(_py, _py.NotImplemented()),
                    }
                }
            }
        };
        match self {
            SelfType::Receiver { mutable: false } => {
                quote! {
                    let _cell = #cell;
                    let _ref = _cell.try_borrow()?;
                    let _slf: &#cls = &*_ref;
                }
            }
            SelfType::Receiver { mutable: true } => {
                quote! {
                    let _cell = #cell;
                    let mut _ref = _cell.try_borrow_mut()?;
                    let _slf: &mut #cls = &mut *_ref;
                }
            }
            SelfType::TryFromPyCell(span) => {
                quote_spanned! { *span =>
                    let _cell = #cell;
                    #[allow(clippy::useless_conversion)]  // In case _slf is PyCell<Self>
                    let _slf = ::std::convert::TryFrom::try_from(_cell)?;
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
    pub fn from_args(args: &[FnArg<'_>], attrs: &[Argument]) -> Self {
        let (_, accept_kwargs) = accept_args_kwargs(attrs);
        if args.is_empty() {
            Self::Noargs
        } else if accept_kwargs {
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
    pub attrs: Vec<Argument>,
    pub args: Vec<FnArg<'a>>,
    pub output: syn::Type,
    pub doc: PythonDoc,
    pub deprecations: Deprecations,
    pub convention: CallingConvention,
    pub text_signature: Option<TextSignatureAttribute>,
    pub krate: syn::Path,
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
        syn::FnArg::Receiver(syn::Receiver { mutability, .. }) => Ok(SelfType::Receiver {
            mutable: mutability.is_some(),
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
            krate,
            name,
            mut deprecations,
            ..
        } = options;

        let MethodAttributes {
            ty: fn_type_attr,
            args: fn_attrs,
            mut python_name,
        } = parse_method_attributes(meth_attrs, name.map(|name| name.value.0), &mut deprecations)?;

        let (fn_type, skip_first_arg, fixed_convention) =
            Self::parse_fn_type(sig, fn_type_attr, &mut python_name)?;
        Self::ensure_text_signature_on_valid_method(&fn_type, text_signature.as_ref())?;

        let name = &sig.ident;
        let ty = get_return_info(&sig.output);
        let python_name = python_name.as_ref().unwrap_or(name).unraw();
        let krate = get_pyo3_crate(&krate);

        let doc = utils::get_doc(
            meth_attrs,
            text_signature.as_ref().map(|attr| (&python_name, attr)),
        );

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

        let convention =
            fixed_convention.unwrap_or_else(|| CallingConvention::from_args(&arguments, &fn_attrs));

        Ok(FnSpec {
            tp: fn_type,
            name,
            convention,
            python_name,
            attrs: fn_attrs,
            args: arguments,
            output: ty,
            doc,
            deprecations,
            text_signature,
            krate,
            unsafety: sig.unsafety,
        })
    }

    pub fn null_terminated_python_name(&self) -> syn::LitStr {
        syn::LitStr::new(&format!("{}\0", self.python_name), self.python_name.span())
    }

    fn ensure_text_signature_on_valid_method(
        fn_type: &FnType,
        text_signature: Option<&TextSignatureAttribute>,
    ) -> syn::Result<()> {
        if let Some(text_signature) = text_signature {
            match &fn_type {
                FnType::FnNew => bail_spanned!(
                    text_signature.kw.span() =>
                    "text_signature not allowed on __new__; if you want to add a signature on \
                     __new__, put it on the struct definition instead"
                ),
                FnType::Getter(_) | FnType::Setter(_) | FnType::ClassAttribute => bail_spanned!(
                    text_signature.kw.span() => "text_signature not allowed with this method type"
                ),
                _ => {}
            }
        }
        Ok(())
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
            Some(MethodTypeAttribute::ClassAttribute) => {
                ensure_spanned!(
                    sig.inputs.is_empty(),
                    sig.inputs.span() => "class attribute methods cannot take arguments"
                );
                (FnType::ClassAttribute, false, None)
            }
            Some(MethodTypeAttribute::New) => {
                if let Some(name) = &python_name {
                    bail_spanned!(name.span() => "`name` not allowed with `#[new]`");
                }
                *python_name = Some(syn::Ident::new("__new__", Span::call_site()));
                (FnType::FnNew, false, Some(CallingConvention::TpNew))
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

    pub fn default_value(&self, name: &syn::Ident) -> Option<TokenStream> {
        for s in &self.attrs {
            match s {
                Argument::Arg(path, opt) | Argument::Kwarg(path, opt) => {
                    if path.is_ident(name) {
                        if let Some(val) = opt {
                            let i: syn::Expr = syn::parse_str(val).unwrap();
                            return Some(i.into_token_stream());
                        }
                    }
                }
                _ => (),
            }
        }
        None
    }

    pub fn is_pos_only(&self, name: &syn::Ident) -> bool {
        for s in &self.attrs {
            if let Argument::PosOnlyArg(path, _) = s {
                if path.is_ident(name) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_kw_only(&self, name: &syn::Ident) -> bool {
        for s in &self.attrs {
            if let Argument::Kwarg(path, _) = s {
                if path.is_ident(name) {
                    return true;
                }
            }
        }
        false
    }

    /// Return a C wrapper function for this signature.
    pub fn get_wrapper_function(
        &self,
        ident: &proc_macro2::Ident,
        cls: Option<&syn::Type>,
    ) -> Result<TokenStream> {
        let deprecations = &self.deprecations;
        let self_conversion = self.tp.self_conversion(cls, ExtractErrorMode::Raise);
        let self_arg = self.tp.self_arg();
        let arg_names = (0..self.args.len())
            .map(|pos| syn::Ident::new(&format!("arg{}", pos), Span::call_site()))
            .collect::<Vec<_>>();
        let py = syn::Ident::new("_py", Span::call_site());
        let func_name = &self.name;
        let rust_name = if let Some(cls) = cls {
            quote!(#cls::#func_name)
        } else {
            quote!(#func_name)
        };
        let rust_call =
            quote! { _pyo3::callback::convert(#py, #rust_name(#self_arg #(#arg_names),*)) };
        let krate = &self.krate;
        Ok(match self.convention {
            CallingConvention::Noargs => {
                quote! {
                    unsafe extern "C" fn #ident (
                        _slf: *mut #krate::ffi::PyObject,
                        _args: *mut #krate::ffi::PyObject,
                    ) -> *mut #krate::ffi::PyObject
                    {
                        use #krate as _pyo3;
                        #deprecations
                        let gil = _pyo3::GILPool::new();
                        let #py = gil.python();
                        _pyo3::callback::panic_result_into_callback_output(#py, ::std::panic::catch_unwind(move || -> _pyo3::PyResult<_> {
                            #self_conversion
                            #rust_call
                        }))
                    }
                }
            }
            CallingConvention::Fastcall => {
                let arg_convert = impl_arg_params(self, cls, &py, true)?;
                quote! {
                    unsafe extern "C" fn #ident (
                        _slf: *mut #krate::ffi::PyObject,
                        _args: *const *mut #krate::ffi::PyObject,
                        _nargs: #krate::ffi::Py_ssize_t,
                        _kwnames: *mut #krate::ffi::PyObject) -> *mut #krate::ffi::PyObject
                    {
                        use #krate as _pyo3;
                        #deprecations
                        let gil = _pyo3::GILPool::new();
                        let #py = gil.python();
                        _pyo3::callback::panic_result_into_callback_output(#py, ::std::panic::catch_unwind(move || -> _pyo3::PyResult<_> {
                            #self_conversion
                            #arg_convert
                            #rust_call
                        }))
                    }
                }
            }
            CallingConvention::Varargs => {
                let arg_convert = impl_arg_params(self, cls, &py, false)?;
                quote! {
                    unsafe extern "C" fn #ident (
                        _slf: *mut #krate::ffi::PyObject,
                        _args: *mut #krate::ffi::PyObject,
                        _kwargs: *mut #krate::ffi::PyObject) -> *mut #krate::ffi::PyObject
                    {
                        use #krate as _pyo3;
                        #deprecations
                        let gil = _pyo3::GILPool::new();
                        let #py = gil.python();
                        _pyo3::callback::panic_result_into_callback_output(#py, ::std::panic::catch_unwind(move || -> _pyo3::PyResult<_> {
                            #self_conversion
                            #arg_convert
                            #rust_call
                        }))
                    }
                }
            }
            CallingConvention::TpNew => {
                let rust_call = quote! { #rust_name(#(#arg_names),*) };
                let arg_convert = impl_arg_params(self, cls, &py, false)?;
                quote! {
                    unsafe extern "C" fn #ident (
                        subtype: *mut #krate::ffi::PyTypeObject,
                        _args: *mut #krate::ffi::PyObject,
                        _kwargs: *mut #krate::ffi::PyObject) -> *mut #krate::ffi::PyObject
                    {
                        use #krate as _pyo3;
                        #deprecations
                        use _pyo3::callback::IntoPyCallbackOutput;
                        let gil = _pyo3::GILPool::new();
                        let #py = gil.python();
                        _pyo3::callback::panic_result_into_callback_output(#py, ::std::panic::catch_unwind(move || -> _pyo3::PyResult<_> {
                            #arg_convert
                            let result = #rust_call;
                            let initializer: _pyo3::PyClassInitializer::<#cls> = result.convert(#py)?;
                            let cell = initializer.create_cell_from_subtype(#py, subtype)?;
                            ::std::result::Result::Ok(cell as *mut _pyo3::ffi::PyObject)
                        }))
                    }
                }
            }
        })
    }

    /// Return a `PyMethodDef` constructor for this function, matching the selected
    /// calling convention.
    pub fn get_methoddef(&self, wrapper: impl ToTokens) -> TokenStream {
        let python_name = self.null_terminated_python_name();
        let doc = &self.doc;
        match self.convention {
            CallingConvention::Noargs => quote! {
                _pyo3::impl_::pymethods::PyMethodDef::noargs(
                    #python_name,
                    _pyo3::impl_::pymethods::PyCFunction(#wrapper),
                    #doc,
                )
            },
            CallingConvention::Fastcall => quote! {
                _pyo3::impl_::pymethods::PyMethodDef::fastcall_cfunction_with_keywords(
                    #python_name,
                    _pyo3::impl_::pymethods::PyCFunctionFastWithKeywords(#wrapper),
                    #doc,
                )
            },
            CallingConvention::Varargs => quote! {
                _pyo3::impl_::pymethods::PyMethodDef::cfunction_with_keywords(
                    #python_name,
                    _pyo3::impl_::pymethods::PyCFunctionWithKeywords(#wrapper),
                    #doc,
                )
            },
            CallingConvention::TpNew => unreachable!("tp_new cannot get a methoddef"),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct MethodAttributes {
    ty: Option<MethodTypeAttribute>,
    args: Vec<Argument>,
    python_name: Option<syn::Ident>,
}

fn parse_method_attributes(
    attrs: &mut Vec<syn::Attribute>,
    mut python_name: Option<syn::Ident>,
    deprecations: &mut Deprecations,
) -> Result<MethodAttributes> {
    let mut new_attrs = Vec::new();
    let mut args = Vec::new();
    let mut ty: Option<MethodTypeAttribute> = None;

    macro_rules! set_ty {
        ($new_ty:expr, $ident:expr) => {
            ensure_spanned!(
               ty.replace($new_ty).is_none(),
               $ident.span() => "cannot specify a second method type"
            );
        };
    }

    for attr in attrs.drain(..) {
        match attr.parse_meta() {
            Ok(syn::Meta::Path(name)) => {
                if name.is_ident("new") || name.is_ident("__new__") {
                    set_ty!(MethodTypeAttribute::New, name);
                } else if name.is_ident("init") || name.is_ident("__init__") {
                    bail_spanned!(name.span() => "#[init] is disabled since PyO3 0.9.0");
                } else if name.is_ident("call") || name.is_ident("__call__") {
                    deprecations.push(Deprecation::CallAttribute, name.span());
                    ensure_spanned!(
                        python_name.is_none(),
                        python_name.span() => "`name` may not be used with `#[call]`"
                    );
                    python_name = Some(syn::Ident::new("__call__", Span::call_site()));
                } else if name.is_ident("classmethod") {
                    set_ty!(MethodTypeAttribute::ClassMethod, name);
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
            Ok(syn::Meta::List(syn::MetaList {
                path, mut nested, ..
            })) => {
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
                    ensure_spanned!(
                        nested.len() == 1,
                        attr.span() => "setter/getter requires one value"
                    );

                    if path.is_ident("setter") {
                        set_ty!(MethodTypeAttribute::Setter, path);
                    } else {
                        set_ty!(MethodTypeAttribute::Getter, path);
                    };

                    ensure_spanned!(
                        python_name.is_none(),
                        python_name.span() => "`name` may only be specified once"
                    );

                    python_name = match nested.pop().unwrap().into_value() {
                        syn::NestedMeta::Meta(syn::Meta::Path(w)) if w.segments.len() == 1 => {
                            Some(w.segments[0].ident.clone())
                        }
                        syn::NestedMeta::Lit(lit) => match lit {
                            syn::Lit::Str(s) => Some(s.parse()?),
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    lit,
                                    "setter/getter attribute requires str value",
                                ))
                            }
                        },
                        _ => {
                            return Err(syn::Error::new_spanned(
                                nested.first().unwrap(),
                                "expected ident or string literal for property name",
                            ))
                        }
                    };
                } else if path.is_ident("args") {
                    let attrs = PyFunctionSignature::from_meta(&nested)?;
                    args.extend(attrs.arguments)
                } else {
                    new_attrs.push(attr)
                }
            }
            Ok(syn::Meta::NameValue(_)) | Err(_) => new_attrs.push(attr),
        }
    }

    *attrs = new_attrs;

    Ok(MethodAttributes {
        ty,
        args,
        python_name,
    })
}

const IMPL_TRAIT_ERR: &str = "Python functions cannot have `impl Trait` arguments";
const RECEIVER_BY_VALUE_ERR: &str =
    "Python objects are shared, so 'self' cannot be moved out of the Python interpreter.
Try `&self`, `&mut self, `slf: PyRef<'_, Self>` or `slf: PyRefMut<'_, Self>`.";
