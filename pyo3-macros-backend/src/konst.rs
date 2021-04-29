use crate::attributes::{
    self, get_deprecated_name_attribute, get_pyo3_attributes, is_attribute_ident, take_attributes,
    NameAttribute,
};
use crate::utils;
use proc_macro2::TokenStream;
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
    /// Null-terminated Python name
    pub fn python_name_with_deprecation(&self) -> TokenStream {
        if let Some(name) = &self.attributes.name {
            let deprecation =
                utils::name_deprecation_token(name.0.span(), self.attributes.name_is_deprecated);
            let name = format!("{}\0", name.0);
            quote!({#deprecation #name})
        } else {
            let name = format!("{}\0", self.rust_ident.unraw().to_string());
            quote!(#name)
        }
    }
}

pub struct ConstAttributes {
    pub is_class_attr: bool,
    pub name: Option<NameAttribute>,
    pub name_is_deprecated: bool,
}

pub enum PyO3ConstAttribute {
    Name(NameAttribute),
}

impl Parse for PyO3ConstAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
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
            name_is_deprecated: false,
        };

        take_attributes(attrs, |attr| {
            if is_attribute_ident(attr, "classattr") {
                ensure_spanned!(
                    attr.tokens.is_empty(),
                    attr.span() => "`#[classattr]` does not take any arguments"
                );
                attributes.is_class_attr = true;
                Ok(true)
            } else if let Some(pyo3_attributes) = get_pyo3_attributes(attr)? {
                for pyo3_attr in pyo3_attributes {
                    match pyo3_attr {
                        PyO3ConstAttribute::Name(name) => attributes.set_name(name)?,
                    }
                }
                Ok(true)
            } else if let Some(name) = get_deprecated_name_attribute(attr)? {
                attributes.set_name(name)?;
                attributes.name_is_deprecated = true;
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
            name.0.span() => "`name` may only be specified once"
        );
        self.name = Some(name);
        Ok(())
    }
}
