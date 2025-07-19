use std::borrow::Cow;
use std::ffi::CString;

use crate::attributes::{self, get_pyo3_options, take_attributes, NameAttribute};
use crate::utils::{Ctx, LitCStr};
use proc_macro2::{Ident, Span};
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
    pub fn null_terminated_python_name(&self, ctx: &Ctx) -> LitCStr {
        let name = self.python_name().to_string();
        LitCStr::new(CString::new(name).unwrap(), Span::call_site(), ctx)
    }
}

pub struct ConstAttributes {
    pub is_class_attr: bool,
    pub name: Option<NameAttribute>,
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
        };

        take_attributes(attrs, |attr| {
            if attr.path().is_ident("classattr") {
                ensure_spanned!(
                    matches!(attr.meta, syn::Meta::Path(..)),
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
