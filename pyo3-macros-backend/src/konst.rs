use std::borrow::Cow;

use crate::{
    attributes::{self, get_pyo3_options, is_attribute_ident, take_attributes, NameAttribute},
    deprecations::Deprecations,
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Result,
};

pub struct ConstSpec {
    pub rust_ident: syn::Ident,
    pub attributes: ConstAttributes,
}

impl ConstSpec {
    pub fn python_name(&self) -> Cow<'_, Ident> {
        if let Some(name) = &self.attributes.name {
            Cow::Borrowed(&name.value.0)
        } else {
            Cow::Owned(self.rust_ident.unraw())
        }
    }

    /// Null-terminated Python name
    pub fn null_terminated_python_name(&self) -> TokenStream {
        let name = format!("{}\0", self.python_name());
        quote!({#name})
    }
}

pub struct ConstAttributes {
    pub is_class_attr: bool,
    pub name: Option<NameAttribute>,
    pub deprecations: Deprecations,
}

pub enum PyO3ConstAttribute {
    Name(NameAttribute),
}

impl Parse for PyO3ConstAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::name) {
            input.parse().map(PyO3ConstAttribute::Name)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ConstAttributes {
    pub fn from_attrs(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut attributes = ConstAttributes {
            is_class_attr: false,
            name: None,
            deprecations: Deprecations::new(),
        };

        take_attributes(attrs, |attr| {
            if is_attribute_ident(attr, "classattr") {
                ensure_spanned!(
                    attr.tokens.is_empty(),
                    attr.span() => "`#[classattr]` does not take any arguments"
                );
                attributes.is_class_attr = true;
                Ok(true)
            } else if let Some(pyo3_attributes) = get_pyo3_options(attr)? {
                for pyo3_attr in pyo3_attributes {
                    match pyo3_attr {
                        PyO3ConstAttribute::Name(name) => attributes.set_name(name)?,
                    }
                }
                Ok(true)
            } else {
                Ok(false)
            }
        })?;

        Ok(attributes)
    }

    fn set_name(&mut self, name: NameAttribute) -> Result<()> {
        ensure_spanned!(
            self.name.is_none(),
            name.span() => "`name` may only be specified once"
        );
        self.name = Some(name);
        Ok(())
    }
}
