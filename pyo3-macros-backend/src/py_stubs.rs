//! Parsing and serialization code for custom type stubs

use crate::json::JsonValue;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use std::collections::HashMap;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Brace;
use syn::{braced, Token};

mod kw {
    syn::custom_keyword!(import);
    syn::custom_keyword!(from);
}

/// Custom provided stubs in #[pymodule]
pub struct PyStubs {
    bracket_token: Brace,
    statements: Vec<PyStatement>,
}

impl PyStubs {
    /// Returns a JSON object following the https://docs.python.org/fr/3/library/ast.html syntax tree
    pub fn as_json(&self) -> JsonValue {
        JsonValue::Array(self.statements.iter().map(|i| i.as_json()).collect())
    }
}

impl Parse for PyStubs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        Ok(Self {
            bracket_token: braced!(content in input),
            statements: {
                let mut statements = Vec::new();
                while !content.is_empty() {
                    statements.push(content.parse()?);
                }
                statements
            },
        })
    }
}

impl ToTokens for PyStubs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.bracket_token.surround(tokens, |tokens| {
            for import in &self.statements {
                import.to_tokens(tokens)
            }
        })
    }
}

/// A Python statement
enum PyStatement {
    ImportFrom(PyImportFrom),
    Import(PyImport),
}

impl PyStatement {
    pub fn as_json(&self) -> JsonValue {
        match self {
            Self::ImportFrom(s) => s.as_json(),
            Self::Import(s) => s.as_json(),
        }
    }
}

impl Parse for PyStatement {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::from) {
            input.parse().map(Self::ImportFrom)
        } else if lookahead.peek(kw::import) {
            input.parse().map(Self::Import)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for PyStatement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::ImportFrom(s) => s.to_tokens(tokens),
            Self::Import(s) => s.to_tokens(tokens),
        }
    }
}

/// `from {module} import {names}`
struct PyImportFrom {
    pub from_token: kw::from,
    pub module: Ident,
    pub import_token: kw::import,
    pub names: Punctuated<PyAlias, Token![,]>,
}

impl PyImportFrom {
    pub fn as_json(&self) -> JsonValue {
        JsonValue::Object(HashMap::from([
            ("type", JsonValue::String("importfrom".into())),
            (
                "module",
                JsonValue::String(self.module.unraw().to_string().into()),
            ),
            (
                "names",
                JsonValue::Array(self.names.iter().map(|i| i.as_json()).collect()),
            ),
            ("level", JsonValue::Number(0)),
        ]))
    }
}

impl Parse for PyImportFrom {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            from_token: input.parse()?,
            module: input.parse()?,
            import_token: input.parse()?,
            names: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

impl ToTokens for PyImportFrom {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.from_token.to_tokens(tokens);
        self.module.to_tokens(tokens);
        self.import_token.to_tokens(tokens);
        self.names.to_tokens(tokens);
    }
}

/// `import {names}`
struct PyImport {
    pub import_token: kw::import,
    pub names: Punctuated<PyAlias, Token![,]>,
}

impl PyImport {
    pub fn as_json(&self) -> JsonValue {
        JsonValue::Object(HashMap::from([
            ("type", JsonValue::String("import".into())),
            (
                "names",
                JsonValue::Array(self.names.iter().map(|i| i.as_json()).collect()),
            ),
        ]))
    }
}

impl Parse for PyImport {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            import_token: input.parse()?,
            names: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

impl ToTokens for PyImport {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.import_token.to_tokens(tokens);
        self.names.to_tokens(tokens);
    }
}

/// `{name} [as {as_name}]`
struct PyAlias {
    pub name: Ident,
    pub as_name: Option<PyAliasAsName>,
}

impl PyAlias {
    pub fn as_json(&self) -> JsonValue {
        let mut args = HashMap::from([
            ("type", JsonValue::String("alias".into())),
            (
                "name",
                JsonValue::String(self.name.unraw().to_string().into()),
            ),
        ]);
        if let Some(as_name) = &self.as_name {
            args.insert(
                "asname",
                JsonValue::String(as_name.name.unraw().to_string().into()),
            );
        }
        JsonValue::Object(args)
    }
}

impl Parse for PyAlias {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            as_name: if input.lookahead1().peek(Token![as]) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}

impl ToTokens for PyAlias {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name.to_tokens(tokens);
        self.as_name.to_tokens(tokens);
    }
}

/// `as {name}`
struct PyAliasAsName {
    pub as_token: Token![as],
    pub name: Ident,
}

impl Parse for PyAliasAsName {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            as_token: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl ToTokens for PyAliasAsName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_token.to_tokens(tokens);
        self.name.to_tokens(tokens);
    }
}
