use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Attribute, ExprPath, Ident, LitStr, Path, Result, Token,
};

pub mod kw {
    syn::custom_keyword!(annotation);
    syn::custom_keyword!(attribute);
    syn::custom_keyword!(from_py_with);
    syn::custom_keyword!(get);
    syn::custom_keyword!(item);
    syn::custom_keyword!(pass_module);
    syn::custom_keyword!(name);
    syn::custom_keyword!(set);
    syn::custom_keyword!(signature);
    syn::custom_keyword!(text_signature);
    syn::custom_keyword!(transparent);
}

#[derive(Clone, Debug, PartialEq)]
pub struct FromPyWithAttribute(pub ExprPath);

impl Parse for FromPyWithAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let _: kw::from_py_with = input.parse()?;
        let _: Token![=] = input.parse()?;
        let string_literal: LitStr = input.parse()?;
        string_literal.parse().map(FromPyWithAttribute)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NameAttribute(pub Ident);

impl Parse for NameAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let _: kw::name = input.parse()?;
        let _: Token![=] = input.parse()?;
        let string_literal: LitStr = input.parse()?;
        string_literal.parse().map(NameAttribute)
    }
}

/// For specifying the path to the pyo3 crate.
#[derive(Clone, Debug, PartialEq)]
pub struct CrateAttribute(pub Path);

impl Parse for CrateAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let _: Token![crate] = input.parse()?;
        let _: Token![=] = input.parse()?;
        let string_literal: LitStr = input.parse()?;
        string_literal.parse().map(CrateAttribute)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextSignatureAttribute {
    pub kw: kw::text_signature,
    pub eq_token: Token![=],
    pub lit: LitStr,
}

impl Parse for TextSignatureAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(TextSignatureAttribute {
            kw: input.parse()?,
            eq_token: input.parse()?,
            lit: input.parse()?,
        })
    }
}

pub fn get_pyo3_options<T: Parse>(attr: &syn::Attribute) -> Result<Option<Punctuated<T, Comma>>> {
    if is_attribute_ident(attr, "pyo3") {
        attr.parse_args_with(Punctuated::parse_terminated).map(Some)
    } else {
        Ok(None)
    }
}

pub fn is_attribute_ident(attr: &syn::Attribute, name: &str) -> bool {
    if let Some(path_segment) = attr.path.segments.last() {
        attr.path.segments.len() == 1 && path_segment.ident == name
    } else {
        false
    }
}

/// Takes attributes from an attribute vector.
///
/// For each attribute in `attrs`, `extractor` is called. If `extractor` returns `Ok(true)`, then
/// the attribute will be removed from the vector.
///
/// This is similar to `Vec::retain` except the closure is fallible and the condition is reversed.
/// (In `retain`, returning `true` keeps the element, here it removes it.)
pub fn take_attributes(
    attrs: &mut Vec<Attribute>,
    mut extractor: impl FnMut(&Attribute) -> Result<bool>,
) -> Result<()> {
    *attrs = attrs
        .drain(..)
        .filter_map(|attr| {
            extractor(&attr)
                .map(move |attribute_handled| if attribute_handled { None } else { Some(attr) })
                .transpose()
        })
        .collect::<Result<_>>()?;
    Ok(())
}

pub fn take_pyo3_options<T: Parse>(attrs: &mut Vec<syn::Attribute>) -> Result<Vec<T>> {
    let mut out = Vec::new();
    take_attributes(attrs, |attr| {
        if let Some(options) = get_pyo3_options(attr)? {
            out.extend(options.into_iter());
            Ok(true)
        } else {
            Ok(false)
        }
    })?;
    Ok(out)
}
