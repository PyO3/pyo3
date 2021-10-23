// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::{
    attributes::{
        self, get_deprecated_name_attribute, get_deprecated_text_signature_attribute,
        get_pyo3_options, take_attributes, FromPyWithAttribute, NameAttribute,
        TextSignatureAttribute,
    },
    deprecations::Deprecations,
    method::{self, CallingConvention, FnArg},
    pymethod::check_generic,
    utils::{self, ensure_not_async_fn},
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{ext::IdentExt, spanned::Spanned, Ident, NestedMeta, Path, Result};
use syn::{
    parse::{Parse, ParseBuffer, ParseStream},
    token::Comma,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    PosOnlyArgsSeparator,
    VarArgsSeparator,
    VarArgs(syn::Path),
    KeywordArgs(syn::Path),
    PosOnlyArg(syn::Path, Option<String>),
    Arg(syn::Path, Option<String>),
    Kwarg(syn::Path, Option<String>),
}

/// The attributes of the pyfunction macro
#[derive(Default)]
pub struct PyFunctionSignature {
    pub arguments: Vec<Argument>,
    has_kw: bool,
    has_posonly_args: bool,
    has_varargs: bool,
    has_kwargs: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PyFunctionArgPyO3Attributes {
    pub from_py_with: Option<FromPyWithAttribute>,
}

enum PyFunctionArgPyO3Attribute {
    FromPyWith(FromPyWithAttribute),
}

impl Parse for PyFunctionArgPyO3Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::from_py_with) {
            input.parse().map(PyFunctionArgPyO3Attribute::FromPyWith)
        } else {
            Err(lookahead.error())
        }
    }
}

impl PyFunctionArgPyO3Attributes {
    /// Parses #[pyo3(from_python_with = "func")]
    pub fn from_attrs(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut attributes = PyFunctionArgPyO3Attributes { from_py_with: None };
        take_attributes(attrs, |attr| {
            if let Some(pyo3_attrs) = get_pyo3_options(attr)? {
                for attr in pyo3_attrs {
                    match attr {
                        PyFunctionArgPyO3Attribute::FromPyWith(from_py_with) => {
                            ensure_spanned!(
                                attributes.from_py_with.is_none(),
                                from_py_with.0.span() => "`from_py_with` may only be specified once per argument"
                            );
                            attributes.from_py_with = Some(from_py_with);
                        }
                    }
                }
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        Ok(attributes)
    }
}

impl syn::parse::Parse for PyFunctionSignature {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let attr = Punctuated::<NestedMeta, syn::Token![,]>::parse_terminated(input)?;
        Self::from_meta(&attr)
    }
}

impl PyFunctionSignature {
    pub fn from_meta<'a>(iter: impl IntoIterator<Item = &'a NestedMeta>) -> syn::Result<Self> {
        let mut slf = PyFunctionSignature::default();

        for item in iter {
            slf.add_item(item)?
        }
        Ok(slf)
    }

    pub fn add_item(&mut self, item: &NestedMeta) -> syn::Result<()> {
        match item {
            NestedMeta::Meta(syn::Meta::Path(ident)) => self.add_work(item, ident)?,
            NestedMeta::Meta(syn::Meta::NameValue(nv)) => {
                self.add_name_value(item, nv)?;
            }
            NestedMeta::Lit(lit) => {
                self.add_literal(item, lit)?;
            }
            NestedMeta::Meta(syn::Meta::List(list)) => bail_spanned!(
                list.span() => "list is not supported as argument"
            ),
        }
        Ok(())
    }

    fn add_literal(&mut self, item: &NestedMeta, lit: &syn::Lit) -> syn::Result<()> {
        match lit {
            syn::Lit::Str(lits) if lits.value() == "*" => {
                // "*"
                self.vararg_is_ok(item)?;
                self.has_varargs = true;
                self.arguments.push(Argument::VarArgsSeparator);
                Ok(())
            }
            syn::Lit::Str(lits) if lits.value() == "/" => {
                // "/"
                self.posonly_arg_is_ok(item)?;
                self.has_posonly_args = true;
                // any arguments _before_ this become positional-only
                self.arguments.iter_mut().for_each(|a| {
                    if let Argument::Arg(path, name) = a {
                        *a = Argument::PosOnlyArg(path.clone(), name.clone());
                    } else {
                        unreachable!();
                    }
                });
                self.arguments.push(Argument::PosOnlyArgsSeparator);
                Ok(())
            }
            _ => bail_spanned!(item.span() => "expected \"/\" or \"*\""),
        }
    }

    fn add_work(&mut self, item: &NestedMeta, path: &Path) -> syn::Result<()> {
        ensure_spanned!(
            !(self.has_kw || self.has_kwargs),
            item.span() => "positional argument or varargs(*) not allowed after keyword arguments"
        );
        if self.has_varargs {
            self.arguments.push(Argument::Kwarg(path.clone(), None));
        } else {
            self.arguments.push(Argument::Arg(path.clone(), None));
        }
        Ok(())
    }

    fn posonly_arg_is_ok(&self, item: &NestedMeta) -> syn::Result<()> {
        ensure_spanned!(
            !(self.has_posonly_args || self.has_kwargs || self.has_varargs),
            item.span() => "/ is not allowed after /, varargs(*), or kwargs(**)"
        );
        Ok(())
    }

    fn vararg_is_ok(&self, item: &NestedMeta) -> syn::Result<()> {
        ensure_spanned!(
            !(self.has_kwargs || self.has_varargs),
            item.span() => "* is not allowed after varargs(*) or kwargs(**)"
        );
        Ok(())
    }

    fn kw_arg_is_ok(&self, item: &NestedMeta) -> syn::Result<()> {
        ensure_spanned!(
            !self.has_kwargs,
            item.span() => "keyword argument or kwargs(**) is not allowed after kwargs(**)"
        );
        Ok(())
    }

    fn add_nv_common(
        &mut self,
        item: &NestedMeta,
        name: &syn::Path,
        value: String,
    ) -> syn::Result<()> {
        self.kw_arg_is_ok(item)?;
        if self.has_varargs {
            // kw only
            self.arguments
                .push(Argument::Kwarg(name.clone(), Some(value)));
        } else {
            self.has_kw = true;
            self.arguments
                .push(Argument::Arg(name.clone(), Some(value)));
        }
        Ok(())
    }

    fn add_name_value(&mut self, item: &NestedMeta, nv: &syn::MetaNameValue) -> syn::Result<()> {
        match &nv.lit {
            syn::Lit::Str(litstr) => {
                if litstr.value() == "*" {
                    // args="*"
                    self.vararg_is_ok(item)?;
                    self.has_varargs = true;
                    self.arguments.push(Argument::VarArgs(nv.path.clone()));
                } else if litstr.value() == "**" {
                    // kwargs="**"
                    self.kw_arg_is_ok(item)?;
                    self.has_kwargs = true;
                    self.arguments.push(Argument::KeywordArgs(nv.path.clone()));
                } else {
                    self.add_nv_common(item, &nv.path, litstr.value())?;
                }
            }
            syn::Lit::Int(litint) => {
                self.add_nv_common(item, &nv.path, format!("{}", litint))?;
            }
            syn::Lit::Bool(litb) => {
                self.add_nv_common(item, &nv.path, format!("{}", litb.value))?;
            }
            _ => bail_spanned!(nv.lit.span() => "expected a string literal"),
        };
        Ok(())
    }
}

#[derive(Default)]
pub struct PyFunctionOptions {
    pub pass_module: Option<attributes::kw::pass_module>,
    pub name: Option<NameAttribute>,
    pub signature: Option<PyFunctionSignature>,
    pub text_signature: Option<TextSignatureAttribute>,
    pub deprecations: Deprecations,
}

impl Parse for PyFunctionOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut options = PyFunctionOptions {
            pass_module: None,
            name: None,
            signature: None,
            text_signature: None,
            deprecations: Deprecations::new(),
        };

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(attributes::kw::name)
                || lookahead.peek(attributes::kw::pass_module)
                || lookahead.peek(attributes::kw::signature)
                || lookahead.peek(attributes::kw::text_signature)
            {
                options.add_attributes(std::iter::once(input.parse()?))?;
                if !input.is_empty() {
                    let _: Comma = input.parse()?;
                }
            } else {
                // If not recognised attribute, this is "legacy" pyfunction syntax #[pyfunction(a, b)]
                //
                // TODO deprecate in favour of #[pyfunction(signature = (a, b), name = "foo")]
                options.signature = Some(input.parse()?);
                break;
            }
        }

        Ok(options)
    }
}

pub enum PyFunctionOption {
    Name(NameAttribute),
    PassModule(attributes::kw::pass_module),
    Signature(PyFunctionSignature),
    TextSignature(TextSignatureAttribute),
}

impl Parse for PyFunctionOption {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::name) {
            input.parse().map(PyFunctionOption::Name)
        } else if lookahead.peek(attributes::kw::pass_module) {
            input.parse().map(PyFunctionOption::PassModule)
        } else if lookahead.peek(attributes::kw::signature) {
            input.parse().map(PyFunctionOption::Signature)
        } else if lookahead.peek(attributes::kw::text_signature) {
            input.parse().map(PyFunctionOption::TextSignature)
        } else {
            Err(lookahead.error())
        }
    }
}

impl PyFunctionOptions {
    pub fn from_attrs(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut options = PyFunctionOptions::default();
        options.take_pyo3_options(attrs)?;
        Ok(options)
    }

    pub fn take_pyo3_options(&mut self, attrs: &mut Vec<syn::Attribute>) -> syn::Result<()> {
        take_attributes(attrs, |attr| {
            if let Some(pyo3_attributes) = get_pyo3_options(attr)? {
                self.add_attributes(pyo3_attributes)?;
                Ok(true)
            } else if let Some(name) = get_deprecated_name_attribute(attr, &mut self.deprecations)?
            {
                self.set_name(name)?;
                Ok(true)
            } else if let Some(text_signature) =
                get_deprecated_text_signature_attribute(attr, &mut self.deprecations)?
            {
                self.add_attributes(std::iter::once(PyFunctionOption::TextSignature(
                    text_signature,
                )))?;
                Ok(true)
            } else {
                Ok(false)
            }
        })?;

        Ok(())
    }

    pub fn add_attributes(
        &mut self,
        attrs: impl IntoIterator<Item = PyFunctionOption>,
    ) -> Result<()> {
        for attr in attrs {
            match attr {
                PyFunctionOption::Name(name) => self.set_name(name)?,
                PyFunctionOption::PassModule(kw) => {
                    ensure_spanned!(
                        self.pass_module.is_none(),
                        kw.span() => "`pass_module` may only be specified once"
                    );
                    self.pass_module = Some(kw);
                }
                PyFunctionOption::Signature(signature) => {
                    ensure_spanned!(
                        self.signature.is_none(),
                        // FIXME: improve the span of this error message
                        Span::call_site() => "`signature` may only be specified once"
                    );
                    self.signature = Some(signature);
                }
                PyFunctionOption::TextSignature(text_signature) => {
                    ensure_spanned!(
                        self.text_signature.is_none(),
                        text_signature.kw.span() => "`text_signature` may only be specified once"
                    );
                    self.text_signature = Some(text_signature);
                }
            }
        }
        Ok(())
    }

    pub fn set_name(&mut self, name: NameAttribute) -> Result<()> {
        ensure_spanned!(
            self.name.is_none(),
            name.0.span() => "`name` may only be specified once"
        );
        self.name = Some(name);
        Ok(())
    }
}

pub fn build_py_function(
    ast: &mut syn::ItemFn,
    mut options: PyFunctionOptions,
) -> syn::Result<TokenStream> {
    options.take_pyo3_options(&mut ast.attrs)?;
    Ok(impl_wrap_pyfunction(ast, options)?.1)
}

/// Coordinates the naming of a the add-function-to-python-module function
fn function_wrapper_ident(name: &Ident) -> Ident {
    // Make sure this ident matches the one of wrap_pyfunction
    format_ident!("__pyo3_get_function_{}", name)
}

/// Generates python wrapper over a function that allows adding it to a python module as a python
/// function
pub fn impl_wrap_pyfunction(
    func: &mut syn::ItemFn,
    options: PyFunctionOptions,
) -> syn::Result<(Ident, TokenStream)> {
    check_generic(&func.sig)?;
    ensure_not_async_fn(&func.sig)?;

    let python_name = options
        .name
        .map_or_else(|| func.sig.ident.unraw(), |name| name.0);

    let signature = options.signature.unwrap_or_default();

    let mut arguments = func
        .sig
        .inputs
        .iter_mut()
        .map(FnArg::parse)
        .collect::<syn::Result<Vec<_>>>()?;

    if options.pass_module.is_some() {
        const PASS_MODULE_ERR: &str = "expected &PyModule as first argument with `pass_module`";
        ensure_spanned!(
            !arguments.is_empty(),
            func.span() => PASS_MODULE_ERR
        );
        let arg = arguments.remove(0);
        ensure_spanned!(
            type_is_pymodule(arg.ty),
            arg.ty.span() => PASS_MODULE_ERR
        );
    }

    let ty = method::get_return_info(&func.sig.output);

    let doc = utils::get_doc(
        &func.attrs,
        options
            .text_signature
            .as_ref()
            .map(|attr| (&python_name, attr)),
    );

    let function_wrapper_ident = function_wrapper_ident(&func.sig.ident);

    let spec = method::FnSpec {
        tp: if options.pass_module.is_some() {
            method::FnType::FnModule
        } else {
            method::FnType::FnStatic
        },
        name: &func.sig.ident,
        convention: CallingConvention::from_args(&arguments, &signature.arguments),
        python_name,
        attrs: signature.arguments,
        args: arguments,
        output: ty,
        doc,
        deprecations: options.deprecations,
        text_signature: options.text_signature,
    };

    let wrapper_ident = format_ident!("__pyo3_raw_{}", spec.name);
    let wrapper = spec.get_wrapper_function(&wrapper_ident, None)?;
    let methoddef = spec.get_methoddef(wrapper_ident);

    let wrapped_pyfunction = quote! {
        #wrapper
        pub(crate) fn #function_wrapper_ident<'a>(
            args: impl ::std::convert::Into<::pyo3::derive_utils::PyFunctionArguments<'a>>
        ) -> ::pyo3::PyResult<&'a ::pyo3::types::PyCFunction> {
            ::pyo3::types::PyCFunction::internal_new(#methoddef, args.into())
        }
    };
    Ok((function_wrapper_ident, wrapped_pyfunction))
}

fn type_is_pymodule(ty: &syn::Type) -> bool {
    if let syn::Type::Reference(tyref) = ty {
        if let syn::Type::Path(typath) = tyref.elem.as_ref() {
            if typath
                .path
                .segments
                .last()
                .map(|seg| seg.ident == "PyModule")
                .unwrap_or(false)
            {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{Argument, PyFunctionSignature};
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::parse_quote;

    fn items(input: TokenStream) -> syn::Result<Vec<Argument>> {
        let py_fn_attr: PyFunctionSignature = syn::parse2(input)?;
        Ok(py_fn_attr.arguments)
    }

    #[test]
    fn test_errs() {
        assert!(items(quote! {test="1", test2}).is_err());
        assert!(items(quote! {test, "*", args="*"}).is_err());
        assert!(items(quote! {test, kwargs="**", args="*"}).is_err());
        assert!(items(quote! {test, kwargs="**", args}).is_err());
    }

    #[test]
    fn test_simple_args() {
        let args = items(quote! {test1, test2, test3="None"}).unwrap();
        assert!(
            args == vec![
                Argument::Arg(parse_quote! {test1}, None),
                Argument::Arg(parse_quote! {test2}, None),
                Argument::Arg(parse_quote! {test3}, Some("None".to_owned())),
            ]
        );
    }

    #[test]
    fn test_varargs() {
        let args = items(quote! {test1, test2="None", "*", test3="None"}).unwrap();
        assert!(
            args == vec![
                Argument::Arg(parse_quote! {test1}, None),
                Argument::Arg(parse_quote! {test2}, Some("None".to_owned())),
                Argument::VarArgsSeparator,
                Argument::Kwarg(parse_quote! {test3}, Some("None".to_owned())),
            ]
        );

        let args = items(quote! {"*", test1, test2}).unwrap();
        assert!(
            args == vec![
                Argument::VarArgsSeparator,
                Argument::Kwarg(parse_quote! {test1}, None),
                Argument::Kwarg(parse_quote! {test2}, None),
            ]
        );

        let args = items(quote! {"*", test1, test2="None"}).unwrap();
        assert!(
            args == vec![
                Argument::VarArgsSeparator,
                Argument::Kwarg(parse_quote! {test1}, None),
                Argument::Kwarg(parse_quote! {test2}, Some("None".to_owned())),
            ]
        );

        let args = items(quote! {"*", test1="None", test2}).unwrap();
        assert!(
            args == vec![
                Argument::VarArgsSeparator,
                Argument::Kwarg(parse_quote! {test1}, Some("None".to_owned())),
                Argument::Kwarg(parse_quote! {test2}, None),
            ]
        );
    }

    #[test]
    fn test_all() {
        let args =
            items(quote! {test1, test2="None", args="*", test3="None", kwargs="**"}).unwrap();
        assert!(
            args == vec![
                Argument::Arg(parse_quote! {test1}, None),
                Argument::Arg(parse_quote! {test2}, Some("None".to_owned())),
                Argument::VarArgs(parse_quote! {args}),
                Argument::Kwarg(parse_quote! {test3}, Some("None".to_owned())),
                Argument::KeywordArgs(parse_quote! {kwargs}),
            ]
        );
    }
}
