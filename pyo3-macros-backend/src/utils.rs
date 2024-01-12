use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{punctuated::Punctuated, Token};

use crate::attributes::{CrateAttribute, RenamingRule};

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
///
/// If this doc is for a callable, the provided `text_signature` can be passed to prepend
/// this to the documentation suitable for Python to extract this into the `__text_signature__`
/// attribute.
pub fn get_doc(attrs: &[syn::Attribute], mut text_signature: Option<String>) -> PythonDoc {
    // insert special divider between `__text_signature__` and doc
    // (assume text_signature is itself well-formed)
    if let Some(text_signature) = &mut text_signature {
        text_signature.push_str("\n--\n\n");
    }

    let mut parts = Punctuated::<TokenStream, Token![,]>::new();
    let mut first = true;
    let mut current_part = text_signature.unwrap_or_default();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Ok(nv) = attr.meta.require_name_value() {
                if !first {
                    current_part.push('\n');
                } else {
                    first = false;
                }
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &nv.value
                {
                    // Strip single left space from literal strings, if needed.
                    // e.g. `/// Hello world` expands to #[doc = " Hello world"]
                    let doc_line = lit_str.value();
                    current_part.push_str(doc_line.strip_prefix(' ').unwrap_or(&doc_line));
                } else {
                    // This is probably a macro doc from Rust 1.54, e.g. #[doc = include_str!(...)]
                    // Reset the string buffer, write that part, and then push this macro part too.
                    parts.push(current_part.to_token_stream());
                    current_part.clear();
                    parts.push(nv.value.to_token_stream());
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
        syn::token::Not(Span::call_site()).to_tokens(&mut tokens);
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

pub fn unwrap_ty_group(mut ty: &syn::Type) -> &syn::Type {
    while let syn::Type::Group(g) = ty {
        ty = &*g.elem;
    }
    ty
}

/// Extract the path to the pyo3 crate, or use the default (`::pyo3`).
pub(crate) fn get_pyo3_crate(attr: &Option<CrateAttribute>) -> syn::Path {
    match attr {
        Some(attr) => attr.value.0.clone(),
        None => syn::parse_str("::pyo3").unwrap(),
    }
}

pub fn apply_renaming_rule(rule: RenamingRule, name: &str) -> String {
    use heck::*;

    match rule {
        RenamingRule::CamelCase => name.to_lower_camel_case(),
        RenamingRule::KebabCase => name.to_kebab_case(),
        RenamingRule::Lowercase => name.to_lowercase(),
        RenamingRule::PascalCase => name.to_upper_camel_case(),
        RenamingRule::ScreamingKebabCase => name.to_shouty_kebab_case(),
        RenamingRule::ScreamingSnakeCase => name.to_shouty_snake_case(),
        RenamingRule::SnakeCase => name.to_snake_case(),
        RenamingRule::Uppercase => name.to_uppercase(),
    }
}
