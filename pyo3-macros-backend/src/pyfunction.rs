use crate::attributes::KeywordAttribute;
use crate::combine_errors::CombineErrors;
#[cfg(feature = "experimental-inspect")]
use crate::introspection::{function_introspection_code, introspection_id_const};
use crate::utils::{Ctx, LitCStr};
use crate::{
    attributes::{
        self, get_pyo3_options, take_attributes, take_pyo3_options, CrateAttribute,
        FromPyWithAttribute, NameAttribute, TextSignatureAttribute,
    },
    method::{self, CallingConvention, FnArg},
    pymethod::check_generic,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::cmp::PartialEq;
use std::ffi::CString;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{ext::IdentExt, spanned::Spanned, LitStr, Path, Result, Token};

mod signature;

pub use self::signature::{ConstructorAttribute, FunctionSignature, SignatureAttribute};

#[derive(Clone, Debug)]
pub struct PyFunctionArgPyO3Attributes {
    pub from_py_with: Option<FromPyWithAttribute>,
    pub cancel_handle: Option<attributes::kw::cancel_handle>,
}

enum PyFunctionArgPyO3Attribute {
    FromPyWith(FromPyWithAttribute),
    CancelHandle(attributes::kw::cancel_handle),
}

impl Parse for PyFunctionArgPyO3Attribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::cancel_handle) {
            input.parse().map(PyFunctionArgPyO3Attribute::CancelHandle)
        } else if lookahead.peek(attributes::kw::from_py_with) {
            input.parse().map(PyFunctionArgPyO3Attribute::FromPyWith)
        } else {
            Err(lookahead.error())
        }
    }
}

impl PyFunctionArgPyO3Attributes {
    /// Parses #[pyo3(from_python_with = "func")]
    pub fn from_attrs(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut attributes = PyFunctionArgPyO3Attributes {
            from_py_with: None,
            cancel_handle: None,
        };
        take_attributes(attrs, |attr| {
            if let Some(pyo3_attrs) = get_pyo3_options(attr)? {
                for attr in pyo3_attrs {
                    match attr {
                        PyFunctionArgPyO3Attribute::FromPyWith(from_py_with) => {
                            ensure_spanned!(
                                attributes.from_py_with.is_none(),
                                from_py_with.span() => "`from_py_with` may only be specified once per argument"
                            );
                            attributes.from_py_with = Some(from_py_with);
                        }
                        PyFunctionArgPyO3Attribute::CancelHandle(cancel_handle) => {
                            ensure_spanned!(
                                attributes.cancel_handle.is_none(),
                                cancel_handle.span() => "`cancel_handle` may only be specified once per argument"
                            );
                            attributes.cancel_handle = Some(cancel_handle);
                        }
                    }
                    ensure_spanned!(
                        attributes.from_py_with.is_none() || attributes.cancel_handle.is_none(),
                        attributes.cancel_handle.unwrap().span() => "`from_py_with` and `cancel_handle` cannot be specified together"
                    );
                }
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        Ok(attributes)
    }
}

type PyFunctionWarningMessageAttribute = KeywordAttribute<attributes::kw::message, LitStr>;
type PyFunctionWarningCategoryAttribute = KeywordAttribute<attributes::kw::category, Path>;

pub struct PyFunctionWarningAttribute {
    pub message: PyFunctionWarningMessageAttribute,
    pub category: Option<PyFunctionWarningCategoryAttribute>,
    pub span: Span,
}

#[derive(PartialEq, Clone)]
pub enum PyFunctionWarningCategory {
    Path(Path),
    UserWarning,
    DeprecationWarning, // TODO: unused for now, intended for pyo3(deprecated) special-case
}

#[derive(Clone)]
pub struct PyFunctionWarning {
    pub message: LitStr,
    pub category: PyFunctionWarningCategory,
    pub span: Span,
}

impl From<PyFunctionWarningAttribute> for PyFunctionWarning {
    fn from(value: PyFunctionWarningAttribute) -> Self {
        Self {
            message: value.message.value,
            category: value
                .category
                .map_or(PyFunctionWarningCategory::UserWarning, |cat| {
                    PyFunctionWarningCategory::Path(cat.value)
                }),
            span: value.span,
        }
    }
}

pub trait WarningFactory {
    fn build_py_warning(&self, ctx: &Ctx) -> TokenStream;
    fn span(&self) -> Span;
}

impl WarningFactory for PyFunctionWarning {
    fn build_py_warning(&self, ctx: &Ctx) -> TokenStream {
        let message = &self.message.value();
        let c_message = LitCStr::new(
            CString::new(message.clone()).unwrap(),
            Spanned::span(&message),
            ctx,
        );
        let pyo3_path = &ctx.pyo3_path;
        let category = match &self.category {
            PyFunctionWarningCategory::Path(path) => quote! {#path},
            PyFunctionWarningCategory::UserWarning => {
                quote! {#pyo3_path::exceptions::PyUserWarning}
            }
            PyFunctionWarningCategory::DeprecationWarning => {
                quote! {#pyo3_path::exceptions::PyDeprecationWarning}
            }
        };
        quote! {
            #pyo3_path::PyErr::warn(py, &<#category as #pyo3_path::PyTypeInfo>::type_object(py), #c_message, 1)?;
        }
    }

    fn span(&self) -> Span {
        self.span
    }
}

impl<T: WarningFactory> WarningFactory for Vec<T> {
    fn build_py_warning(&self, ctx: &Ctx) -> TokenStream {
        let warnings = self.iter().map(|warning| warning.build_py_warning(ctx));

        quote! {
            #(#warnings)*
        }
    }

    fn span(&self) -> Span {
        self.iter()
            .map(|val| val.span())
            .reduce(|acc, span| acc.join(span).unwrap_or(acc))
            .unwrap()
    }
}

impl Parse for PyFunctionWarningAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut message: Option<PyFunctionWarningMessageAttribute> = None;
        let mut category: Option<PyFunctionWarningCategoryAttribute> = None;

        let span = input.parse::<attributes::kw::warn>()?.span();

        let content;
        syn::parenthesized!(content in input);

        while !content.is_empty() {
            let lookahead = content.lookahead1();

            if lookahead.peek(attributes::kw::message) {
                message = content
                    .parse::<PyFunctionWarningMessageAttribute>()
                    .map(Some)?;
            } else if lookahead.peek(attributes::kw::category) {
                category = content
                    .parse::<PyFunctionWarningCategoryAttribute>()
                    .map(Some)?;
            } else {
                return Err(lookahead.error());
            }

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(PyFunctionWarningAttribute {
            message: message.ok_or(syn::Error::new(
                content.span(),
                "missing `message` in `warn` attribute",
            ))?,
            category,
            span,
        })
    }
}

impl ToTokens for PyFunctionWarningAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let message_tokens = self.message.to_token_stream();
        let category_tokens = self
            .category
            .as_ref()
            .map_or(quote! {}, |cat| cat.to_token_stream());

        let token_stream = quote! {
            warn(#message_tokens, #category_tokens)
        };

        tokens.extend(token_stream);
    }
}

#[derive(Default)]
pub struct PyFunctionOptions {
    pub pass_module: Option<attributes::kw::pass_module>,
    pub name: Option<NameAttribute>,
    pub signature: Option<SignatureAttribute>,
    pub text_signature: Option<TextSignatureAttribute>,
    pub krate: Option<CrateAttribute>,
    pub warnings: Vec<PyFunctionWarning>,
}

impl Parse for PyFunctionOptions {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut options = PyFunctionOptions::default();

        let attrs = Punctuated::<PyFunctionOption, syn::Token![,]>::parse_terminated(input)?;
        options.add_attributes(attrs)?;

        Ok(options)
    }
}

pub enum PyFunctionOption {
    Name(NameAttribute),
    PassModule(attributes::kw::pass_module),
    Signature(SignatureAttribute),
    TextSignature(TextSignatureAttribute),
    Crate(CrateAttribute),
    Warning(PyFunctionWarningAttribute),
}

impl Parse for PyFunctionOption {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::name) {
            input.parse().map(PyFunctionOption::Name)
        } else if lookahead.peek(attributes::kw::pass_module) {
            input.parse().map(PyFunctionOption::PassModule)
        } else if lookahead.peek(attributes::kw::signature) {
            input.parse().map(PyFunctionOption::Signature)
        } else if lookahead.peek(attributes::kw::text_signature) {
            input.parse().map(PyFunctionOption::TextSignature)
        } else if lookahead.peek(syn::Token![crate]) {
            input.parse().map(PyFunctionOption::Crate)
        } else if lookahead.peek(attributes::kw::warn) {
            input.parse().map(PyFunctionOption::Warning)
        } else {
            Err(lookahead.error())
        }
    }
}

impl PyFunctionOptions {
    pub fn from_attrs(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut options = PyFunctionOptions::default();
        options.add_attributes(take_pyo3_options(attrs)?)?;
        Ok(options)
    }

    pub fn add_attributes(
        &mut self,
        attrs: impl IntoIterator<Item = PyFunctionOption>,
    ) -> Result<()> {
        macro_rules! set_option {
            ($key:ident) => {
                {
                    ensure_spanned!(
                        self.$key.is_none(),
                        $key.span() => concat!("`", stringify!($key), "` may only be specified once")
                    );
                    self.$key = Some($key);
                }
            };
        }
        for attr in attrs {
            match attr {
                PyFunctionOption::Name(name) => set_option!(name),
                PyFunctionOption::PassModule(pass_module) => set_option!(pass_module),
                PyFunctionOption::Signature(signature) => set_option!(signature),
                PyFunctionOption::TextSignature(text_signature) => set_option!(text_signature),
                PyFunctionOption::Crate(krate) => set_option!(krate),
                PyFunctionOption::Warning(warning) => {
                    self.warnings.push(warning.into());
                }
            }
        }
        Ok(())
    }
}

pub fn build_py_function(
    ast: &mut syn::ItemFn,
    mut options: PyFunctionOptions,
) -> syn::Result<TokenStream> {
    options.add_attributes(take_pyo3_options(&mut ast.attrs)?)?;
    impl_wrap_pyfunction(ast, options)
}

/// Generates python wrapper over a function that allows adding it to a python module as a python
/// function
pub fn impl_wrap_pyfunction(
    func: &mut syn::ItemFn,
    options: PyFunctionOptions,
) -> syn::Result<TokenStream> {
    check_generic(&func.sig)?;
    let PyFunctionOptions {
        pass_module,
        name,
        signature,
        text_signature,
        krate,
        warnings,
    } = options;

    let ctx = &Ctx::new(&krate, Some(&func.sig));
    let Ctx { pyo3_path, .. } = &ctx;

    let python_name = name
        .as_ref()
        .map_or_else(|| &func.sig.ident, |name| &name.value.0)
        .unraw();

    let tp = if pass_module.is_some() {
        let span = match func.sig.inputs.first() {
            Some(syn::FnArg::Typed(first_arg)) => first_arg.ty.span(),
            Some(syn::FnArg::Receiver(_)) | None => bail_spanned!(
                func.sig.paren_token.span.join() => "expected `&PyModule` or `Py<PyModule>` as first argument with `pass_module`"
            ),
        };
        method::FnType::FnModule(span)
    } else {
        method::FnType::FnStatic
    };

    let arguments = func
        .sig
        .inputs
        .iter_mut()
        .skip(if tp.skip_first_rust_argument_in_python_signature() {
            1
        } else {
            0
        })
        .map(FnArg::parse)
        .try_combine_syn_errors()?;

    let signature = if let Some(signature) = signature {
        FunctionSignature::from_arguments_and_attribute(arguments, signature)?
    } else {
        FunctionSignature::from_arguments(arguments)
    };

    let vis = &func.vis;
    let name = &func.sig.ident;

    #[cfg(feature = "experimental-inspect")]
    let introspection = function_introspection_code(
        pyo3_path,
        Some(name),
        &name.to_string(),
        &signature,
        None,
        func.sig.output.clone(),
        [] as [String; 0],
        None,
    );
    #[cfg(not(feature = "experimental-inspect"))]
    let introspection = quote! {};
    #[cfg(feature = "experimental-inspect")]
    let introspection_id = introspection_id_const();
    #[cfg(not(feature = "experimental-inspect"))]
    let introspection_id = quote! {};

    let spec = method::FnSpec {
        tp,
        name: &func.sig.ident,
        convention: CallingConvention::from_signature(&signature),
        python_name,
        signature,
        text_signature,
        asyncness: func.sig.asyncness,
        unsafety: func.sig.unsafety,
        warnings,
        #[cfg(feature = "experimental-inspect")]
        output: func.sig.output.clone(),
    };

    let wrapper_ident = format_ident!("__pyfunction_{}", spec.name);
    if spec.asyncness.is_some() {
        ensure_spanned!(
            cfg!(feature = "experimental-async"),
            spec.asyncness.span() => "async functions are only supported with the `experimental-async` feature"
        );
    }
    let wrapper = spec.get_wrapper_function(&wrapper_ident, None, ctx)?;
    let methoddef = spec.get_methoddef(wrapper_ident, &spec.get_doc(&func.attrs, ctx)?, ctx);

    let wrapped_pyfunction = quote! {
        // Create a module with the same name as the `#[pyfunction]` - this way `use <the function>`
        // will actually bring both the module and the function into scope.
        #[doc(hidden)]
        #vis mod #name {
            pub(crate) struct MakeDef;
            pub const _PYO3_DEF: #pyo3_path::impl_::pymethods::PyMethodDef = MakeDef::_PYO3_DEF;
            #introspection_id
        }

        // Generate the definition inside an anonymous function in the same scope as the original function -
        // this avoids complications around the fact that the generated module has a different scope
        // (and `super` doesn't always refer to the outer scope, e.g. if the `#[pyfunction] is
        // inside a function body)
        #[allow(unknown_lints, non_local_definitions)]
        impl #name::MakeDef {
            const _PYO3_DEF: #pyo3_path::impl_::pymethods::PyMethodDef = #methoddef;
        }

        #[allow(non_snake_case)]
        #wrapper

        #introspection
    };
    Ok(wrapped_pyfunction)
}
