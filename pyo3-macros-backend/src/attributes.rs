use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::Parser;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, Expr, ExprPath, Ident, Index, LitBool, LitStr, Member, Path, Result, Token,
};

use crate::combine_errors::CombineErrors;

pub mod kw {
    syn::custom_keyword!(annotation);
    syn::custom_keyword!(attribute);
    syn::custom_keyword!(cancel_handle);
    syn::custom_keyword!(constructor);
    syn::custom_keyword!(dict);
    syn::custom_keyword!(eq);
    syn::custom_keyword!(eq_int);
    syn::custom_keyword!(extends);
    syn::custom_keyword!(freelist);
    syn::custom_keyword!(from_py_with);
    syn::custom_keyword!(frozen);
    syn::custom_keyword!(get);
    syn::custom_keyword!(get_all);
    syn::custom_keyword!(hash);
    syn::custom_keyword!(into_py_with);
    syn::custom_keyword!(item);
    syn::custom_keyword!(immutable_type);
    syn::custom_keyword!(from_item_all);
    syn::custom_keyword!(mapping);
    syn::custom_keyword!(module);
    syn::custom_keyword!(name);
    syn::custom_keyword!(ord);
    syn::custom_keyword!(pass_module);
    syn::custom_keyword!(rename_all);
    syn::custom_keyword!(sequence);
    syn::custom_keyword!(set);
    syn::custom_keyword!(set_all);
    syn::custom_keyword!(signature);
    syn::custom_keyword!(str);
    syn::custom_keyword!(subclass);
    syn::custom_keyword!(submodule);
    syn::custom_keyword!(text_signature);
    syn::custom_keyword!(transparent);
    syn::custom_keyword!(unsendable);
    syn::custom_keyword!(weakref);
    syn::custom_keyword!(generic);
    syn::custom_keyword!(gil_used);
    syn::custom_keyword!(warn);
    syn::custom_keyword!(message);
    syn::custom_keyword!(category);
}

fn take_int(read: &mut &str, tracker: &mut usize) -> String {
    let mut int = String::new();
    for (i, ch) in read.char_indices() {
        match ch {
            '0'..='9' => {
                *tracker += 1;
                int.push(ch)
            }
            _ => {
                *read = &read[i..];
                break;
            }
        }
    }
    int
}

fn take_ident(read: &mut &str, tracker: &mut usize) -> Ident {
    let mut ident = String::new();
    if read.starts_with("r#") {
        ident.push_str("r#");
        *tracker += 2;
        *read = &read[2..];
    }
    for (i, ch) in read.char_indices() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                *tracker += 1;
                ident.push(ch)
            }
            _ => {
                *read = &read[i..];
                break;
            }
        }
    }
    Ident::parse_any.parse_str(&ident).unwrap()
}

// shorthand parsing logic inspiration taken from https://github.com/dtolnay/thiserror/blob/master/impl/src/fmt.rs
fn parse_shorthand_format(fmt: LitStr) -> Result<(LitStr, Vec<Member>)> {
    let span = fmt.span();
    let token = fmt.token();
    let value = fmt.value();
    let mut read = value.as_str();
    let mut out = String::new();
    let mut members = Vec::new();
    let mut tracker = 1;
    while let Some(brace) = read.find('{') {
        tracker += brace;
        out += &read[..brace + 1];
        read = &read[brace + 1..];
        if read.starts_with('{') {
            out.push('{');
            read = &read[1..];
            tracker += 2;
            continue;
        }
        let next = match read.chars().next() {
            Some(next) => next,
            None => break,
        };
        tracker += 1;
        let member = match next {
            '0'..='9' => {
                let start = tracker;
                let index = take_int(&mut read, &mut tracker).parse::<u32>().unwrap();
                let end = tracker;
                let subspan = token.subspan(start..end).unwrap_or(span);
                let idx = Index {
                    index,
                    span: subspan,
                };
                Member::Unnamed(idx)
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = tracker;
                let mut ident = take_ident(&mut read, &mut tracker);
                let end = tracker;
                let subspan = token.subspan(start..end).unwrap_or(span);
                ident.set_span(subspan);
                Member::Named(ident)
            }
            '}' | ':' => {
                let start = tracker;
                tracker += 1;
                let end = tracker;
                let subspan = token.subspan(start..end).unwrap_or(span);
                // we found a closing bracket or formatting ':' without finding a member, we assume the user wants the instance formatted here
                bail_spanned!(subspan.span() => "No member found, you must provide a named or positionally specified member.")
            }
            _ => continue,
        };
        members.push(member);
    }
    out += read;
    Ok((LitStr::new(&out, span), members))
}

#[derive(Clone, Debug)]
pub struct StringFormatter {
    pub fmt: LitStr,
    pub args: Vec<Member>,
}

impl Parse for crate::attributes::StringFormatter {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let (fmt, args) = parse_shorthand_format(input.parse()?)?;
        Ok(Self { fmt, args })
    }
}

impl ToTokens for crate::attributes::StringFormatter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.fmt.to_tokens(tokens);
        tokens.extend(quote! {self.args})
    }
}

#[derive(Clone, Debug)]
pub struct KeywordAttribute<K, V> {
    pub kw: K,
    pub value: V,
}

#[derive(Clone, Debug)]
pub struct OptionalKeywordAttribute<K, V> {
    pub kw: K,
    pub value: Option<V>,
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
        if let Ok(ident) = string_literal.parse_with(Ident::parse_any) {
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

/// Available renaming rules
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenamingRule {
    CamelCase,
    KebabCase,
    Lowercase,
    PascalCase,
    ScreamingKebabCase,
    ScreamingSnakeCase,
    SnakeCase,
    Uppercase,
}

/// A helper type which parses a renaming rule via a literal string
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenamingRuleLitStr {
    pub lit: LitStr,
    pub rule: RenamingRule,
}

impl Parse for RenamingRuleLitStr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let string_literal: LitStr = input.parse()?;
        let rule = match string_literal.value().as_ref() {
            "camelCase" => RenamingRule::CamelCase,
            "kebab-case" => RenamingRule::KebabCase,
            "lowercase" => RenamingRule::Lowercase,
            "PascalCase" => RenamingRule::PascalCase,
            "SCREAMING-KEBAB-CASE" => RenamingRule::ScreamingKebabCase,
            "SCREAMING_SNAKE_CASE" => RenamingRule::ScreamingSnakeCase,
            "snake_case" => RenamingRule::SnakeCase,
            "UPPERCASE" => RenamingRule::Uppercase,
            _ => {
                bail_spanned!(string_literal.span() => "expected a valid renaming rule, possible values are: \"camelCase\", \"kebab-case\", \"lowercase\", \"PascalCase\", \"SCREAMING-KEBAB-CASE\", \"SCREAMING_SNAKE_CASE\", \"snake_case\", \"UPPERCASE\"")
            }
        };
        Ok(Self {
            lit: string_literal,
            rule,
        })
    }
}

impl ToTokens for RenamingRuleLitStr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lit.to_tokens(tokens)
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
pub type RenameAllAttribute = KeywordAttribute<kw::rename_all, RenamingRuleLitStr>;
pub type StrFormatterAttribute = OptionalKeywordAttribute<kw::str, StringFormatter>;
pub type TextSignatureAttribute = KeywordAttribute<kw::text_signature, TextSignatureAttributeValue>;
pub type SubmoduleAttribute = kw::submodule;
pub type GILUsedAttribute = KeywordAttribute<kw::gil_used, LitBool>;

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

impl<K: Parse + std::fmt::Debug, V: Parse> Parse for OptionalKeywordAttribute<K, V> {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let kw: K = input.parse()?;
        let value = match input.parse::<Token![=]>() {
            Ok(_) => Some(input.parse()?),
            Err(_) => None,
        };
        Ok(OptionalKeywordAttribute { kw, value })
    }
}

impl<K: ToTokens, V: ToTokens> ToTokens for OptionalKeywordAttribute<K, V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.kw.to_tokens(tokens);
        if self.value.is_some() {
            Token![=](self.kw.span()).to_tokens(tokens);
            self.value.to_tokens(tokens);
        }
    }
}

pub type FromPyWithAttribute = KeywordAttribute<kw::from_py_with, ExprPath>;
pub type IntoPyWithAttribute = KeywordAttribute<kw::into_py_with, ExprPath>;

pub type DefaultAttribute = OptionalKeywordAttribute<Token![default], Expr>;

/// For specifying the path to the pyo3 crate.
pub type CrateAttribute = KeywordAttribute<Token![crate], LitStrValue<Path>>;

pub fn get_pyo3_options<T: Parse>(attr: &syn::Attribute) -> Result<Option<Punctuated<T, Comma>>> {
    if attr.path().is_ident("pyo3") {
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

    take_attributes(attrs, |attr| match get_pyo3_options(attr) {
        Ok(result) => {
            if let Some(options) = result {
                out.extend(options.into_iter().map(|a| Ok(a)));
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(err) => {
            out.push(Err(err));
            Ok(true)
        }
    })?;

    let out: Vec<T> = out.into_iter().try_combine_syn_errors()?;

    Ok(out)
}
