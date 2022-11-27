use std::borrow::Cow;

use crate::attributes::handle_cfg_feature_pyo3;
use crate::{
    attributes::{self, NameAttribute},
    deprecations::Deprecations,
};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Meta, Result, Token,
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
    pub fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut attributes = ConstAttributes {
            is_class_attr: false,
            name: None,
            deprecations: Deprecations::new(),
        };

        let mut new_attrs = Vec::new();

        for mut attr in attrs.drain(..) {
            let parse_attr = |meta, _attr: &Attribute| parse_attribute(&mut attributes, &meta);
            if let Ok(mut meta) = attr.parse_meta() {
                if handle_cfg_feature_pyo3(&mut attr, &mut meta, parse_attr)? {
                    continue;
                }

                if parse_attribute(&mut attributes, &meta)? {
                    continue;
                }
            }
            new_attrs.push(attr)
        }

        *attrs = new_attrs;

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

/// Returns whether this attribute was parsed and should be discarded
fn parse_attribute(mut attributes: &mut ConstAttributes, meta: &Meta) -> Result<bool> {
    if let Meta::Path(path) = meta {
        if path.is_ident("classattr") {
            attributes.is_class_attr = true;
            return Ok(true);
        }
    } else if let Meta::List(meta_list) = meta {
        if meta_list.path.is_ident("classattr") {
            return Err(syn::Error::new(
                meta_list.nested.span(),
                "`#[classattr]` does not take any arguments",
            ));
        }
    }

    if let Meta::List(meta_list) = meta {
        if meta_list.path.is_ident("pyo3") {
            if let Ok(parsed) = Punctuated::<_, Token![,]>::parse_terminated
                .parse2(meta_list.nested.to_token_stream())
            {
                for pyo3_attr in parsed {
                    match pyo3_attr {
                        PyO3ConstAttribute::Name(name) => attributes.set_name(name)?,
                    }
                }
                return Ok(true);
            }
        }
    }

    Ok(false)
}

#[cfg(test)]
mod test {
    use crate::konst::ConstAttributes;
    use quote::ToTokens;
    use syn::ItemConst;

    #[test]
    fn test_const_attributes() {
        let inputs = [
            ("#[classattr]  const MAX: u16 = 65535;", 0),
            (
                r#"#[cfg_attr(feature = "pyo3", classattr)]  const MAX: u16 = 65535;"#,
                0,
            ),
            (
                r#"#[cfg_attr(feature = "pyo3", other, classattr, still_other)]  const MAX: u16 = 65535;"#,
                1,
            ),
        ];
        for (code, attrs_remaining) in inputs {
            let mut konst: ItemConst = syn::parse_str(code).unwrap();
            let actual = ConstAttributes::from_attrs(&mut konst.attrs).unwrap();
            assert!(actual.is_class_attr);
            assert!(actual.name.is_none());
            assert!(actual.deprecations.to_token_stream().is_empty());
            assert_eq!(konst.attrs.len(), attrs_remaining);
        }
    }
}
