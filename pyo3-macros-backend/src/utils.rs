// Copyright (c) 2017-present PyO3 Project and Contributors

use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::Type;
use syn::{spanned::Spanned, Ident};

use crate::attributes::{CrateAttribute, TextSignatureAttribute};

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

/// If `ty` is Option<T>, return `Some(T)`, else None.
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

/// A syntax tree which evaluates to a null-terminated docstring for Python.
///
/// It's built as a `concat!` evaluation, so it's hard to do anything with this
/// contents such as parse the string contents.
#[derive(Clone)]
pub struct PythonDoc(TokenStream);

/// Collects all #[doc = "..."] attributes into a TokenStream evaluating to a null-terminated string
/// e.g. concat!("...", "\n", "\0")
pub fn get_doc(
    attrs: &[syn::Attribute],
    text_signature: Option<(Cow<'_, Ident>, &TextSignatureAttribute)>,
) -> PythonDoc {
    let mut tokens = TokenStream::new();
    let comma = syn::token::Comma(Span::call_site());
    let newline = syn::LitStr::new("\n", Span::call_site());

    syn::Ident::new("concat", Span::call_site()).to_tokens(&mut tokens);
    syn::token::Bang(Span::call_site()).to_tokens(&mut tokens);
    syn::token::Bracket(Span::call_site()).surround(&mut tokens, |tokens| {
        if let Some((python_name, text_signature)) = text_signature {
            // create special doc string lines to set `__text_signature__`
            let signature_lines =
                format!("{}{}\n--\n\n", python_name, text_signature.value.value());
            signature_lines.to_tokens(tokens);
            comma.to_tokens(tokens);
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
                        newline.to_tokens(tokens);
                        comma.to_tokens(tokens);
                    } else {
                        first = false;
                    }
                    if let Ok(syn::Lit::Str(lit_str)) = syn::parse2(token_stream.clone()) {
                        // Strip single left space from literal strings, if needed.
                        // e.g. `/// Hello world` expands to #[doc = " Hello world"]
                        let doc_line = lit_str.value();
                        doc_line
                            .strip_prefix(' ')
                            .map(|stripped| syn::LitStr::new(stripped, lit_str.span()))
                            .unwrap_or(lit_str)
                            .to_tokens(tokens);
                    } else {
                        // This is probably a macro doc from Rust 1.54, e.g. #[doc = include_str!(...)]
                        token_stream.to_tokens(tokens)
                    }
                    comma.to_tokens(tokens);
                }
            }
        }

        syn::LitStr::new("\0", Span::call_site()).to_tokens(tokens);
    });

    PythonDoc(tokens)
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

/// Remove lifetime from reference
pub(crate) fn remove_lifetime(tref: &syn::TypeReference) -> syn::TypeReference {
    let mut tref = tref.to_owned();
    tref.lifetime = None;
    tref
}

/// Extract the path to the pyo3 crate, or use the default (`::pyo3`).
pub(crate) fn get_pyo3_crate(attr: &Option<CrateAttribute>) -> syn::Path {
    attr.as_ref()
        .map(|p| p.value.0.clone())
        .unwrap_or_else(|| syn::parse_str("::pyo3").unwrap())
}

/// Generates a unique identifier based on a type and (optionally) a field.
///
/// For the same input values, the result should be the same output, and for different input values,
/// the output should be different. No other guarantees are made (do not try to parse it).
pub(crate) fn generate_unique_ident(class: &Type, field: Option<&Ident>) -> Ident {
    let span = if let Some(field) = field {
        field.span()
    } else {
        class.span()
    };

    let mut result = "".to_string();

    // Attempt to generate something unique for each type
    // Types that cannot be annotated with #[pyclass] are ignored
    match class {
        Type::Array(_) => unreachable!("Cannot generate a unique name for an array: {:?}", class),
        Type::BareFn(_) => unreachable!("Cannot generate a unique name for a function: {:?}", class),
        Type::Group(_) => unreachable!("Cannot generate a unique name for a group: {:?}", class),
        Type::ImplTrait(_) => unreachable!("Cannot generate a unique name for an impl trait: {:?}", class),
        Type::Infer(_) => unreachable!("Cannot generate a unique name for an inferred type: {:?}", class),
        Type::Macro(_) => unreachable!("Cannot generate a unique name for a macro: {:?}", class),
        Type::Never(_) => {
            result += "_never";
        },
        Type::Paren(_) => unreachable!("Cannot generate a unique name for a type in parenthesis: {:?}", class),
        Type::Path(path) => {
            result += "_path";
            for segment in &path.path.segments {
                result += "_";
                result += &*segment.ident.to_string();
            }
        }
        Type::Ptr(_) => unreachable!("Cannot generate a unique name for a pointer: {:?}", class),
        Type::Reference(_) => unreachable!("Cannot generate a unique name for a reference: {:?}", class),
        Type::Slice(_) => unreachable!("Cannot generate a unique name for a slice: {:?}", class),
        Type::TraitObject(_) => unreachable!("Cannot generate a unique name for a trait object: {:?}", class),
        Type::Tuple(_) => unreachable!("Cannot generate a unique name for a tuple: {:?}", class),
        _ => unreachable!("Cannot generate a unique name for an unknown type: {:?}", class),
    }

    if let Some(field) = field {
        result += "_";
        result += &*field.to_string()
    }

    Ident::new(&*result, span)
}
