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
    Kwarg(syn::Path, Option<String>),
}

/// The attributes of the pyfunction macro
#[derive(Default)]
pub struct PyFunctionAttr {
    pub arguments: Vec<Argument>,
    has_kw: bool,
    has_varargs: bool,
    has_kwargs: bool,
    pub pass_module: bool,
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
            NestedMeta::Meta(syn::Meta::Path(ident)) if ident.is_ident("pass_module") => {
                self.pass_module = true;
            }
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
            _ => bail_spanned!(item.span() => "expected \"*\""),
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

pub fn parse_name_attribute(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Option<syn::Ident>> {
    let mut name_attrs = Vec::new();

    // Using retain will extract all name attributes from the attribute list
    attrs.retain(|attr| match attr.parse_meta() {
        Ok(syn::Meta::NameValue(nv)) if nv.path.is_ident("name") => {
            name_attrs.push((nv.lit, attr.span()));
            false
        }
        _ => true,
    });

    match name_attrs.as_slice() {
        [] => Ok(None),
        [(syn::Lit::Str(s), span)] => {
            let mut ident: syn::Ident = s.parse()?;
            // This span is the whole attribute span, which is nicer for reporting errors.
            ident.set_span(*span);
            Ok(Some(ident))
        }
        [(_, span)] => bail_spanned!(*span => "expected string literal for #[name] argument"),
        slice => bail_spanned!(
            slice[1].1 => "#[name] can not be specified multiple times"
        ),
    }
}

pub fn build_py_function(ast: &mut syn::ItemFn, args: PyFunctionAttr) -> syn::Result<TokenStream> {
    let python_name =
        parse_name_attribute(&mut ast.attrs)?.unwrap_or_else(|| ast.sig.ident.unraw());
    add_fn_to_module(ast, python_name, args)
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
