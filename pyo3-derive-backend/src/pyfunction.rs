// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::module::add_fn_to_module;
use proc_macro2::TokenStream;
use syn::ext::IdentExt;
use syn::parse::ParseBuffer;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{NestedMeta, Path};

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    VarArgsSeparator,
    VarArgs(syn::Path),
    KeywordArgs(syn::Path),
    Arg(syn::Path, Option<String>),
    Kwarg(syn::Path, String),
}

/// The attributes of the pyfunction macro
#[derive(Default)]
pub struct PyFunctionAttr {
    pub arguments: Vec<Argument>,
    has_kw: bool,
    has_varargs: bool,
    has_kwargs: bool,
}

impl syn::parse::Parse for PyFunctionAttr {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let attr = Punctuated::<NestedMeta, syn::Token![,]>::parse_terminated(input)?;
        Self::from_meta(&attr)
    }
}

impl PyFunctionAttr {
    pub fn from_meta<'a>(iter: impl IntoIterator<Item = &'a NestedMeta>) -> syn::Result<Self> {
        let mut slf = PyFunctionAttr::default();

        for item in iter {
            slf.add_item(item)?
        }
        Ok(slf)
    }

    pub fn add_item(&mut self, item: &NestedMeta) -> syn::Result<()> {
        match item {
            NestedMeta::Meta(syn::Meta::Path(ref ident)) => self.add_work(item, ident)?,
            NestedMeta::Meta(syn::Meta::NameValue(ref nv)) => {
                self.add_name_value(item, nv)?;
            }
            NestedMeta::Lit(ref lit) => {
                self.add_literal(item, lit)?;
            }
            _ => {
                return Err(syn::Error::new_spanned(item, "Unknown argument"));
            }
        }

        Ok(())
    }

    fn add_literal(&mut self, item: &NestedMeta, lit: &syn::Lit) -> syn::Result<()> {
        match lit {
            syn::Lit::Str(ref lits) => {
                // "*"
                if lits.value() == "*" {
                    if self.has_kwargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "syntax error, keyword self.arguments is defined",
                        ));
                    }
                    if self.has_varargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "self.arguments already define * (var args)",
                        ));
                    }
                    self.has_varargs = true;
                    self.arguments.push(Argument::VarArgsSeparator);
                } else {
                    return Err(syn::Error::new_spanned(lits, "Unknown string literal"));
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    item,
                    format!("Only string literal is supported, got: {:?}", lit),
                ));
            }
        };
        Ok(())
    }

    fn add_work(&mut self, item: &NestedMeta, path: &Path) -> syn::Result<()> {
        // self.arguments in form somename
        if self.has_kwargs {
            return Err(syn::Error::new_spanned(
                item,
                "syntax error, keyword self.arguments is defined",
            ));
        }
        if self.has_kw {
            return Err(syn::Error::new_spanned(
                item,
                "syntax error, argument is not allowed after keyword argument",
            ));
        }
        self.arguments.push(Argument::Arg(path.clone(), None));
        Ok(())
    }

    fn add_name_value(&mut self, item: &NestedMeta, nv: &syn::MetaNameValue) -> syn::Result<()> {
        match nv.lit {
            syn::Lit::Str(ref litstr) => {
                if litstr.value() == "*" {
                    // args="*"
                    if self.has_kwargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "* - syntax error, keyword self.arguments is defined",
                        ));
                    }
                    if self.has_varargs {
                        return Err(syn::Error::new_spanned(item, "*(var args) is defined"));
                    }
                    self.has_varargs = true;
                    self.arguments.push(Argument::VarArgs(nv.path.clone()));
                } else if litstr.value() == "**" {
                    // kwargs="**"
                    if self.has_kwargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "self.arguments already define ** (kw args)",
                        ));
                    }
                    self.has_kwargs = true;
                    self.arguments.push(Argument::KeywordArgs(nv.path.clone()));
                } else if self.has_varargs {
                    self.arguments
                        .push(Argument::Kwarg(nv.path.clone(), litstr.value()))
                } else {
                    if self.has_kwargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "syntax error, keyword self.arguments is defined",
                        ));
                    }
                    self.has_kw = true;
                    self.arguments
                        .push(Argument::Arg(nv.path.clone(), Some(litstr.value())))
                }
            }
            syn::Lit::Int(ref litint) => {
                if self.has_varargs {
                    self.arguments
                        .push(Argument::Kwarg(nv.path.clone(), format!("{}", litint)));
                } else {
                    if self.has_kwargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "syntax error, keyword self.arguments is defined",
                        ));
                    }
                    self.has_kw = true;
                    self.arguments
                        .push(Argument::Arg(nv.path.clone(), Some(format!("{}", litint))));
                }
            }
            syn::Lit::Bool(ref litb) => {
                if self.has_varargs {
                    self.arguments
                        .push(Argument::Kwarg(nv.path.clone(), format!("{}", litb.value)));
                } else {
                    if self.has_kwargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "syntax error, keyword self.arguments is defined",
                        ));
                    }
                    self.has_kw = true;
                    self.arguments.push(Argument::Arg(
                        nv.path.clone(),
                        Some(format!("{}", litb.value)),
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    nv.lit.clone(),
                    "Only string literal is supported",
                ));
            }
        };
        Ok(())
    }
}

pub fn parse_name_attribute(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Option<syn::Ident>> {
    let mut name_attrs = Vec::new();

    // Using retain will extract all name attributes from the attribute list
    attrs.retain(|attr| match attr.parse_meta() {
        Ok(syn::Meta::NameValue(ref nv)) if nv.path.is_ident("name") => {
            name_attrs.push((nv.lit.clone(), attr.span()));
            false
        }
        _ => true,
    });

    match &*name_attrs {
        [] => Ok(None),
        [(syn::Lit::Str(s), span)] => {
            let mut ident: syn::Ident = s.parse()?;
            // This span is the whole attribute span, which is nicer for reporting errors.
            ident.set_span(*span);
            Ok(Some(ident))
        }
        [(_, span)] => Err(syn::Error::new(
            *span,
            "Expected string literal for #[name] argument",
        )),
        // TODO: The below pattern is unstable, so instead we match the wildcard.
        // slice_patterns due to be stable soon: https://github.com/rust-lang/rust/issues/62254
        // [(_, span), _, ..] => {
        _ => Err(syn::Error::new(
            name_attrs[0].1,
            "#[name] can not be specified multiple times",
        )),
    }
}

pub fn build_py_function(ast: &mut syn::ItemFn, args: PyFunctionAttr) -> syn::Result<TokenStream> {
    let python_name =
        parse_name_attribute(&mut ast.attrs)?.unwrap_or_else(|| ast.sig.ident.unraw());
    Ok(add_fn_to_module(ast, python_name, args.arguments))
}

#[cfg(test)]
mod test {
    use super::{Argument, PyFunctionAttr};
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::parse_quote;

    fn items(input: TokenStream) -> syn::Result<Vec<Argument>> {
        let py_fn_attr: PyFunctionAttr = syn::parse2(input)?;
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
                Argument::Kwarg(parse_quote! {test3}, "None".to_owned()),
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
                Argument::Kwarg(parse_quote! {test3}, "None".to_owned()),
                Argument::KeywordArgs(parse_quote! {kwargs}),
            ]
        );
    }
}
