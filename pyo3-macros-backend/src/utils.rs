use std::{borrow::Cow, fmt::Write};

// Copyright (c) 2017-present PyO3 Project and Contributors
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{punctuated::Punctuated, spanned::Spanned, Ident, Token};

use crate::attributes::CrateAttribute;

/// Macro inspired by `anyhow::anyhow!` to create a compiler error with the given span.
macro_rules! err_spanned {
    ($span:expr => $msg:expr) => {
        syn::Error::new($span, $msg)
    };
}

/// Macro inspired by `anyhow::bail!` to return a compiler error with the given span.
macro_rules! bail_spanned {
    ($span:expr => $msg:expr) => {
        return Err(err_spanned!($span => $msg))
    };
}

/// Macro inspired by `anyhow::ensure!` to return a compiler error with the given span if the
/// specified condition is not met.
macro_rules! ensure_spanned {
    ($condition:expr, $span:expr => $msg:expr) => {
        if !($condition) {
            bail_spanned!($span => $msg);
        }
    }
}

/// Check if the given type `ty` is `pyo3::Python`.
pub fn is_python(ty: &syn::Type) -> bool {
    match unwrap_ty_group(ty) {
        syn::Type::Path(typath) => typath
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "Python")
            .unwrap_or(false),
        _ => false,
    }
}

/// If `ty` is `Option<T>`, return `Some(T)`, else `None`.
pub fn option_type_argument(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        let seg = path.segments.last().filter(|s| s.ident == "Option")?;
        if let syn::PathArguments::AngleBracketed(params) = &seg.arguments {
            if let syn::GenericArgument::Type(ty) = params.args.first()? {
                return Some(ty);
            }
        }
    }
    None
}

/// A syntax tree which evaluates to a nul-terminated docstring for Python.
///
/// Typically the tokens will just be that string, but if the original docs included macro
/// expressions then the tokens will be a concat!("...", "\n", "\0") expression of the strings and
/// macro parts.
/// contents such as parse the string contents.
#[derive(Clone)]
pub struct PythonDoc(TokenStream);

/// Collects all #[doc = "..."] attributes into a TokenStream evaluating to a null-terminated string.
pub fn get_doc(
    attrs: &[syn::Attribute],
    text_signature: Option<(Cow<'_, Ident>, String)>,
) -> PythonDoc {
    let mut parts = Punctuated::<TokenStream, Token![,]>::new();
    let mut current_part = String::new();

    if let Some((python_name, text_signature)) = text_signature {
        // create special doc string lines to set `__text_signature__`
        write!(
            &mut current_part,
            "{}{}\n--\n\n",
            python_name, text_signature
        )
        .expect("error occurred while trying to format text_signature to string")
    }

    let mut first = true;

    for attr in attrs.iter() {
        if attr.path.is_ident("doc") {
            if let Ok(DocArgs {
                _eq_token,
                token_stream,
            }) = syn::parse2(attr.tokens.clone())
            {
                if !first {
                    current_part.push('\n');
                } else {
                    first = false;
                }
                if let Ok(syn::Lit::Str(lit_str)) = syn::parse2(token_stream.clone()) {
                    // Strip single left space from literal strings, if needed.
                    // e.g. `/// Hello world` expands to #[doc = " Hello world"]
                    let doc_line = lit_str.value();
                    current_part.push_str(doc_line.strip_prefix(' ').unwrap_or(&doc_line));
                } else {
                    // This is probably a macro doc from Rust 1.54, e.g. #[doc = include_str!(...)]
                    // Reset the string buffer, write that part, and then push this macro part too.
                    parts.push(current_part.to_token_stream());
                    current_part.clear();
                    parts.push(token_stream);
                }
            }
        }
    }

    if !parts.is_empty() {
        // Doc contained macro pieces - return as `concat!` expression
        if !current_part.is_empty() {
            parts.push(current_part.to_token_stream());
        }

        let mut tokens = TokenStream::new();

        syn::Ident::new("concat", Span::call_site()).to_tokens(&mut tokens);
        syn::token::Bang(Span::call_site()).to_tokens(&mut tokens);
        syn::token::Bracket(Span::call_site()).surround(&mut tokens, |tokens| {
            parts.to_tokens(tokens);
            syn::token::Comma(Span::call_site()).to_tokens(tokens);
            syn::LitStr::new("\0", Span::call_site()).to_tokens(tokens);
        });

        PythonDoc(tokens)
    } else {
        // Just a string doc - return directly with nul terminator
        current_part.push('\0');
        PythonDoc(current_part.to_token_stream())
    }
}

impl quote::ToTokens for PythonDoc {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

struct DocArgs {
    _eq_token: syn::Token![=],
    token_stream: TokenStream,
}

impl syn::parse::Parse for DocArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let this = Self {
            _eq_token: input.parse()?,
            token_stream: input.parse()?,
        };
        ensure_spanned!(input.is_empty(), input.span() => "expected end of doc attribute");
        Ok(this)
    }
}

pub fn ensure_not_async_fn(sig: &syn::Signature) -> syn::Result<()> {
    if let Some(asyncness) = &sig.asyncness {
        bail_spanned!(
            asyncness.span() => "`async fn` is not yet supported for Python functions.\n\n\
            Additional crates such as `pyo3-asyncio` can be used to integrate async Rust and \
            Python. For more information, see https://github.com/PyO3/pyo3/issues/1632"
        );
    };
    Ok(())
}

pub fn unwrap_ty_group(mut ty: &syn::Type) -> &syn::Type {
    while let syn::Type::Group(g) = ty {
        ty = &*g.elem;
    }
    ty
}

/// Extract the path to the pyo3 crate, or use the default (`::pyo3`).
pub(crate) fn get_pyo3_crate(attr: &Option<CrateAttribute>) -> syn::Path {
    attr.as_ref()
        .map(|p| p.value.0.clone())
        .unwrap_or_else(|| syn::parse_str("::pyo3").unwrap())
}
