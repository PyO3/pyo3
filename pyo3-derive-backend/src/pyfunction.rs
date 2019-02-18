// Copyright (c) 2017-present PyO3 Project and Contributors

use syn::parse::ParseBuffer;
use syn::punctuated::Punctuated;
use syn::{Ident, NestedMeta};

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    VarArgsSeparator,
    VarArgs(syn::Ident),
    KeywordArgs(syn::Ident),
    Arg(syn::Ident, Option<String>),
    Kwarg(syn::Ident, String),
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
            NestedMeta::Meta(syn::Meta::Word(ref ident)) => self.add_work(item, ident)?,
            NestedMeta::Meta(syn::Meta::NameValue(ref nv)) => {
                self.add_name_value(item, nv)?;
            }
            NestedMeta::Literal(ref lit) => {
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

    fn add_work(&mut self, item: &NestedMeta, ident: &Ident) -> syn::Result<()> {
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
        self.arguments.push(Argument::Arg(ident.clone(), None));
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
                    self.arguments.push(Argument::VarArgs(nv.ident.clone()));
                } else if litstr.value() == "**" {
                    // kwargs="**"
                    if self.has_kwargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "self.arguments already define ** (kw args)",
                        ));
                    }
                    self.has_kwargs = true;
                    self.arguments.push(Argument::KeywordArgs(nv.ident.clone()));
                } else if self.has_varargs {
                    self.arguments
                        .push(Argument::Kwarg(nv.ident.clone(), litstr.value().clone()))
                } else {
                    if self.has_kwargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "syntax error, keyword self.arguments is defined",
                        ));
                    }
                    self.has_kw = true;
                    self.arguments.push(Argument::Arg(
                        nv.ident.clone(),
                        Some(litstr.value().clone()),
                    ))
                }
            }
            syn::Lit::Int(ref litint) => {
                if self.has_varargs {
                    self.arguments.push(Argument::Kwarg(
                        nv.ident.clone(),
                        format!("{}", litint.value()),
                    ));
                } else {
                    if self.has_kwargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "syntax error, keyword self.arguments is defined",
                        ));
                    }
                    self.has_kw = true;
                    self.arguments.push(Argument::Arg(
                        nv.ident.clone(),
                        Some(format!("{}", litint.value())),
                    ));
                }
            }
            syn::Lit::Bool(ref litb) => {
                if self.has_varargs {
                    self.arguments
                        .push(Argument::Kwarg(nv.ident.clone(), format!("{}", litb.value)));
                } else {
                    if self.has_kwargs {
                        return Err(syn::Error::new_spanned(
                            item,
                            "syntax error, keyword self.arguments is defined",
                        ));
                    }
                    self.has_kw = true;
                    self.arguments.push(Argument::Arg(
                        nv.ident.clone(),
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
