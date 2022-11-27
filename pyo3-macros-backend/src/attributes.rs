use proc_macro2::TokenStream;
use quote::ToTokens;
use std::iter::FromIterator;
use syn::parse::Parser;
use syn::punctuated::{IntoPairs, Pair};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, Expr, ExprPath, Ident, Lit, LitStr, Meta, MetaList, NestedMeta, Path, Result, Token,
};

pub mod kw {
    syn::custom_keyword!(args);
    syn::custom_keyword!(annotation);
    syn::custom_keyword!(attribute);
    syn::custom_keyword!(dict);
    syn::custom_keyword!(extends);
    syn::custom_keyword!(freelist);
    syn::custom_keyword!(from_py_with);
    syn::custom_keyword!(frozen);
    syn::custom_keyword!(gc);
    syn::custom_keyword!(get);
    syn::custom_keyword!(get_all);
    syn::custom_keyword!(item);
    syn::custom_keyword!(mapping);
    syn::custom_keyword!(module);
    syn::custom_keyword!(name);
    syn::custom_keyword!(pass_module);
    syn::custom_keyword!(sequence);
    syn::custom_keyword!(set);
    syn::custom_keyword!(set_all);
    syn::custom_keyword!(signature);
    syn::custom_keyword!(subclass);
    syn::custom_keyword!(text_signature);
    syn::custom_keyword!(transparent);
    syn::custom_keyword!(unsendable);
    syn::custom_keyword!(weakref);
}

#[derive(Clone, Debug)]
pub struct KeywordAttribute<K, V> {
    pub kw: K,
    pub value: V,
}

/// A helper type which parses the inner type via a literal string
/// e.g. `LitStrValue<Path>` -> parses "some::path" in quotes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LitStrValue<T>(pub T);

impl<T: Parse> Parse for LitStrValue<T> {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lit_str: LitStr = input.parse()?;
        lit_str.parse().map(LitStrValue)
    }
}

impl<T: ToTokens> ToTokens for LitStrValue<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

/// A helper type which parses a name via a literal string
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameLitStr(pub Ident);

impl Parse for NameLitStr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let string_literal: LitStr = input.parse()?;
        if let Ok(ident) = string_literal.parse() {
            Ok(NameLitStr(ident))
        } else {
            bail_spanned!(string_literal.span() => "expected a single identifier in double quotes")
        }
    }
}

impl ToTokens for NameLitStr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

/// Text signatue can be either a literal string or opt-in/out
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextSignatureAttributeValue {
    Str(LitStr),
    // `None` ident to disable automatic text signature generation
    Disabled(Ident),
}

impl Parse for TextSignatureAttributeValue {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if let Ok(lit_str) = input.parse::<LitStr>() {
            return Ok(TextSignatureAttributeValue::Str(lit_str));
        }

        let err_span = match input.parse::<Ident>() {
            Ok(ident) if ident == "None" => {
                return Ok(TextSignatureAttributeValue::Disabled(ident));
            }
            Ok(other_ident) => other_ident.span(),
            Err(e) => e.span(),
        };

        Err(err_spanned!(err_span => "expected a string literal or `None`"))
    }
}

impl ToTokens for TextSignatureAttributeValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TextSignatureAttributeValue::Str(s) => s.to_tokens(tokens),
            TextSignatureAttributeValue::Disabled(b) => b.to_tokens(tokens),
        }
    }
}

pub type ExtendsAttribute = KeywordAttribute<kw::extends, Path>;
pub type FreelistAttribute = KeywordAttribute<kw::freelist, Box<Expr>>;
pub type ModuleAttribute = KeywordAttribute<kw::module, LitStr>;
pub type NameAttribute = KeywordAttribute<kw::name, NameLitStr>;
pub type TextSignatureAttribute = KeywordAttribute<kw::text_signature, TextSignatureAttributeValue>;

impl<K: Parse + std::fmt::Debug, V: Parse> Parse for KeywordAttribute<K, V> {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let kw: K = input.parse()?;
        let _: Token![=] = input.parse()?;
        let value = input.parse()?;
        Ok(KeywordAttribute { kw, value })
    }
}

impl<K: ToTokens, V: ToTokens> ToTokens for KeywordAttribute<K, V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.kw.to_tokens(tokens);
        Token![=](self.kw.span()).to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

pub type FromPyWithAttribute = KeywordAttribute<kw::from_py_with, LitStrValue<ExprPath>>;

/// For specifying the path to the pyo3 crate.
pub type CrateAttribute = KeywordAttribute<Token![crate], LitStrValue<Path>>;

/// We can either have `#[pyo3(...)]` or `#[cfg_attr(feature = "pyo3", pyo3(...))]`,
/// with a comma separated list of options parsed into `T` inside
pub fn get_pyo3_options<T: Parse>(attr: &Attribute) -> Result<Option<Punctuated<T, Comma>>> {
    if attr.path.is_ident("pyo3") {
        attr.parse_args_with(Punctuated::parse_terminated).map(Some)
    } else {
        Ok(None)
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
    mut extractor: impl FnMut(&mut Attribute) -> Result<bool>,
) -> Result<()> {
    *attrs = attrs
        .drain(..)
        .filter_map(|mut attr| {
            extractor(&mut attr)
                .map(move |attribute_handled| if attribute_handled { None } else { Some(attr) })
                .transpose()
        })
        .collect::<Result<_>>()?;
    Ok(())
}

pub fn take_pyo3_options<T: Parse>(attrs: &mut Vec<Attribute>) -> Result<Vec<T>> {
    let mut out = Vec::new();
    let mut new_attrs = Vec::new();

    for mut attr in attrs.drain(..) {
        let parse_attr = |meta, _attributes: &Attribute| {
            if let Meta::List(meta_list) = meta {
                if meta_list.path.is_ident("pyo3") {
                    let parsed = Punctuated::<_, Token![,]>::parse_terminated
                        .parse2(meta_list.nested.to_token_stream())?;
                    out.extend(parsed.into_iter());
                    return Ok(true);
                }
            }
            Ok(false)
        };
        if let Ok(mut meta) = attr.parse_meta() {
            if handle_cfg_feature_pyo3(&mut attr, &mut meta, parse_attr)? {
                continue;
            }
        }

        if let Some(options) = get_pyo3_options(&attr)? {
            out.extend(options.into_iter());
            continue;
        } else {
            new_attrs.push(attr)
        }
    }

    *attrs = new_attrs;
    Ok(out)
}

/// Look for #[cfg_attr(feature = "pyo3", ...)]
///            ^^^^^^^^ ^^^^^^^ ^ ^^^^^^
fn is_cfg_feature_pyo3(
    list: &MetaList,
    keep: &mut Vec<Pair<NestedMeta, Comma>>,
    iter: &mut IntoPairs<NestedMeta, Comma>,
) -> bool {
    // #[cfg_attr(feature = "pyo3", ...)]
    //   ^^^^^^^^
    if list.path.is_ident("cfg_attr") {
        // #[cfg_attr(feature = "pyo3", ...)]
        //            ------- ^ ------
        if let Some(pair) = iter.next() {
            let pair_tuple = pair.into_tuple();
            if let (NestedMeta::Meta(Meta::NameValue(name_value)), _) = &pair_tuple {
                // #[cfg_attr(feature = "pyo3", ...)]
                //            ^^^^^^^
                if name_value.path.is_ident("feature") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        // #[cfg_attr(feature = "pyo3", ...)]
                        //                      ^^^^^^
                        if lit_str.value() == "pyo3" {
                            // We want to keep the none-pyo3 pairs intact
                            keep.push(Pair::new(pair_tuple.0, pair_tuple.1));
                            return true;
                        }
                    }
                }
            }
            keep.push(Pair::new(pair_tuple.0, pair_tuple.1));
        }
    }
    false
}

/// Handle #[cfg_attr(feature = "pyo3", ...)]
///
/// Returns whether the attribute was completely handled and can be discarded (because there were
/// blocks in cfg_attr tail that weren't handled)
///
/// Attributes are icky: by default, we get an `Attribute` where all the real data is hidden in a
/// `TokenStream` member. Most of the attribute parsing are therefore custom `Parse` impls. We can
/// also ask syn to parse the attribute into `Meta`, which is essentially an attribute AST, which
/// also some code uses.
///
/// With `cfg_attr` we can additionally have multiple attributes rolled into one behind a gate. So
/// we have to parse and look for `cfg_attr(feature = "pyo3",`, then segment the parts behind it.
/// For each one we have to check whether it parses and also keep those where it doesn't parse for
/// subsequent proc macros (or rustc) to parse. The least bad option for this seems to parsing into
/// `Meta`, checking for `cfg_attr(feature = "pyo3",`, then splitting and letting the caller process
/// each attribute, including calling `.to_token_stream()` and then using `Parse` if necessary
/// (as e.g. [take_pyo3_options] does).
pub fn handle_cfg_feature_pyo3(
    mut attr: &mut Attribute,
    meta: &mut Meta,
    // Return true if handled
    mut parse_attr: impl FnMut(Meta, &Attribute) -> Result<bool>,
) -> Result<bool> {
    if let Meta::List(list) = meta {
        // These are the parts of the attr `parse_attr` told us we didn't parse and we should
        // keep for subsequent proc macros
        let mut keep = Vec::new();
        // handrolled drain function, because `Punctuated` doesn't have one.
        // We keep the comma around so what we do is lossless (keeping the spans)
        let mut drain = list.nested.clone().into_pairs();
        // Look for #[cfg_attr(feature = "pyo3", ...)]
        if !is_cfg_feature_pyo3(list, &mut keep, &mut drain) {
            // No match? Put the meta back we just swapped out, we don't actually want to drain
            list.nested = Punctuated::from_iter(keep.into_iter().chain(drain));
            return Ok(false);
        }

        // #[cfg_attr(feature = "pyo3", staticmethod, pair(name = "ferris"))]
        //                              ^^^^^^^^^^^^^ ^^^^^^^^^^^^^^^^^^^^
        for nested_attr in drain {
            if let NestedMeta::Meta(meta) = nested_attr.value() {
                if !parse_attr(meta.clone(), &attr)? {
                    keep.push(nested_attr)
                }
            }
        }

        // The one that is always there is the condition in the cfg_attr (we put it in in
        // is_cfg_feature_pyo3)
        assert!(!keep.is_empty());
        // If it's exactly 1, we handled all attributes
        if keep.len() > 1 {
            list.nested = Punctuated::from_iter(keep);

            // Keep only the attributes we didn't parse.
            // I couldn't find any method to just get the `attr.tokens` part again but with
            // parentheses so here's token stream editing
            let mut tokens = TokenStream::new();
            list.paren_token.surround(&mut tokens, |inner| {
                inner.extend(list.nested.to_token_stream())
            });
            attr.tokens = tokens;

            return Ok(false);
        }

        // We handled this entire attribute, next
        return Ok(true);
    }
    Ok(false)
}
