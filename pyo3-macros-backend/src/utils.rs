use crate::attributes::{CrateAttribute, RenamingRule};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use std::ffi::CString;
use syn::spanned::Spanned;
use syn::{punctuated::Punctuated, Token};

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
    };
    ($($condition:expr, $span:expr => $msg:expr;)*) => {
        if let Some(e) = [$(
            (!($condition)).then(|| err_spanned!($span => $msg)),
        )*]
            .into_iter()
            .flatten()
            .reduce(|mut acc, e| {
                acc.combine(e);
                acc
            }) {
                return Err(e);
            }
    };
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

// TODO: Replace usage of this by [`syn::LitCStr`] when on MSRV 1.77
#[derive(Clone)]
pub struct LitCStr {
    lit: CString,
    span: Span,
    pyo3_path: PyO3CratePath,
}

impl LitCStr {
    pub fn new(lit: CString, span: Span, ctx: &Ctx) -> Self {
        Self {
            lit,
            span,
            pyo3_path: ctx.pyo3_path.clone(),
        }
    }

    pub fn empty(ctx: &Ctx) -> Self {
        Self {
            lit: CString::new("").unwrap(),
            span: Span::call_site(),
            pyo3_path: ctx.pyo3_path.clone(),
        }
    }
}

impl quote::ToTokens for LitCStr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if cfg!(c_str_lit) {
            syn::LitCStr::new(&self.lit, self.span).to_tokens(tokens);
        } else {
            let pyo3_path = &self.pyo3_path;
            let lit = self.lit.to_str().unwrap();
            tokens.extend(quote::quote_spanned!(self.span => #pyo3_path::ffi::c_str!(#lit)));
        }
    }
}

/// A syntax tree which evaluates to a nul-terminated docstring for Python.
///
/// Typically the tokens will just be that string, but if the original docs included macro
/// expressions then the tokens will be a concat!("...", "\n", "\0") expression of the strings and
/// macro parts. contents such as parse the string contents.
#[derive(Clone)]
pub struct PythonDoc(PythonDocKind);

impl PythonDoc {
    /// Returns an empty docstring.
    pub fn empty(ctx: &Ctx) -> Self {
        PythonDoc(PythonDocKind::LitCStr(LitCStr::empty(ctx)))
    }
}

#[derive(Clone)]
enum PythonDocKind {
    LitCStr(LitCStr),
    // There is currently no way to `concat!` c-string literals, we fallback to the `c_str!` macro in
    // this case.
    Tokens(TokenStream),
}

enum DocParseMode {
    /// Currently generating docs for both Python and Rust.
    Both,
    /// Currently generating docs for Python only.
    PythonOnly,
    /// Currently generating docs for Rust only.
    RustOnly,
}

/// Collects all #[doc = "..."] attributes into a TokenStream evaluating to a null-terminated string.
///
/// If this doc is for a callable, the provided `text_signature` can be passed to prepend
/// this to the documentation suitable for Python to extract this into the `__text_signature__`
/// attribute.
pub fn get_doc(
    attrs: &mut Vec<syn::Attribute>,
    mut text_signature: Option<String>,
    ctx: &Ctx,
) -> syn::Result<PythonDoc> {
    let Ctx { pyo3_path, .. } = ctx;
    // insert special divider between `__text_signature__` and doc
    // (assume text_signature is itself well-formed)
    if let Some(text_signature) = &mut text_signature {
        text_signature.push_str("\n--\n\n");
    }

    let mut parts = Punctuated::<TokenStream, Token![,]>::new();
    let mut first = true;
    let mut current_part = text_signature.unwrap_or_default();
    let mut current_part_span = None; // Track span for error reporting

    let mut mode = DocParseMode::Both;

    let mut to_retain = vec![]; // Collect indices of attributes to retain

    for (i, attr) in attrs.iter().enumerate() {
        if attr.path().is_ident("doc") {
            if let Ok(nv) = attr.meta.require_name_value() {
                let include_in_python;
                let retain_in_rust;

                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &nv.value
                {
                    // Update span for error reporting
                    if current_part_span.is_none() {
                        current_part_span = Some(lit_str.span());
                    }

                    // Strip single left space from literal strings, if needed.
                    let doc_line = lit_str.value();
                    let stripped_line = doc_line.strip_prefix(' ').unwrap_or(&doc_line);
                    let trimmed = stripped_line.trim();

                    // Check if this is a mode switch instruction
                    if let Some(content) = trimmed
                        .strip_prefix("<!--")
                        .and_then(|s| s.strip_suffix("-->"))
                    {
                        let content_trimmed = content.trim();
                        if content_trimmed.starts_with("pyo3_doc_mode:") {
                            let value = content_trimmed
                                .strip_prefix("pyo3_doc_mode:")
                                .unwrap_or("")
                                .trim();
                            mode = match value {
                                "python" => DocParseMode::PythonOnly,
                                "rust" => DocParseMode::RustOnly,
                                "both" => DocParseMode::Both,
                                _ => return Err(syn::Error::new(
                                    lit_str.span(),
                                    format!("Invalid doc_mode: '{}'. Expected 'python', 'rust', or 'both'.", value)
                                )),
                            };
                            // Do not retain mode switch lines in Rust, and skip in Python
                            continue;
                        } else if is_likely_pyo3_doc_mode_typo(content_trimmed) {
                            // Handle potential typos in pyo3_doc_mode prefix
                            return Err(syn::Error::new(
                                lit_str.span(),
                                format!(
                                    "Suspicious comment '{}' - did you mean 'pyo3_doc_mode'? Valid format: <!-- pyo3_doc_mode: python/rust/both -->",
                                    content_trimmed
                                )
                            ));
                        }
                        // If it's an HTML comment but not pyo3_doc_mode related,
                        // it will be included based on current mode (no special handling)
                    }

                    // Not a mode switch, decide based on current mode
                    include_in_python =
                        matches!(mode, DocParseMode::Both | DocParseMode::PythonOnly);
                    retain_in_rust = matches!(mode, DocParseMode::Both | DocParseMode::RustOnly);

                    // Include in Python doc if needed
                    if include_in_python {
                        if !first {
                            current_part.push('\n');
                        } else {
                            first = false;
                        }
                        current_part.push_str(stripped_line);
                    }
                } else {
                    // This is probably a macro doc, e.g. #[doc = include_str!(...)]
                    // Decide based on current mode
                    include_in_python =
                        matches!(mode, DocParseMode::Both | DocParseMode::PythonOnly);
                    retain_in_rust = matches!(mode, DocParseMode::Both | DocParseMode::RustOnly);

                    // Include in Python doc if needed
                    if include_in_python {
                        // Reset the string buffer, write that part, and then push this macro part too.
                        if !current_part.is_empty() {
                            parts.push(current_part.to_token_stream());
                            current_part.clear();
                        }
                        parts.push(nv.value.to_token_stream());
                    }
                }

                // Collect to retain if needed
                if retain_in_rust {
                    to_retain.push(i);
                }
            }
        } else {
            // Non-doc attributes are always retained
            to_retain.push(i);
        }
    }

    // Retain only the selected attributes
    *attrs = to_retain.into_iter().map(|i| attrs[i].clone()).collect();

    // Check if mode ended in Both; if not, error to enforce "pairing"
    if !matches!(mode, DocParseMode::Both) {
        return Err(syn::Error::new(
            Span::call_site(),
            "doc_mode did not end in 'both' mode; consider adding <!-- pyo3_doc_mode: both --> at the end"
        ));
    }

    if !parts.is_empty() {
        // Doc contained macro pieces - return as `concat!` expression
        if !current_part.is_empty() {
            parts.push(
                quote_spanned!(current_part_span.unwrap_or(Span::call_site()) => #current_part),
            );
        }

        let mut tokens = TokenStream::new();

        syn::Ident::new("concat", Span::call_site()).to_tokens(&mut tokens);
        syn::token::Not(Span::call_site()).to_tokens(&mut tokens);
        syn::token::Paren(Span::call_site()).surround(&mut tokens, |tokens| {
            parts.to_tokens(tokens);
        });

        Ok(PythonDoc(PythonDocKind::Tokens(
            quote!(#pyo3_path::ffi::c_str!(#tokens)),
        )))
    } else {
        // Just a string doc - return directly with nul terminator
        let docs = CString::new(current_part).unwrap();
        Ok(PythonDoc(PythonDocKind::LitCStr(LitCStr::new(
            docs,
            current_part_span.unwrap_or(Span::call_site()),
            ctx,
        ))))
    }
}

/// Helper function to detect likely typos in pyo3_doc_mode prefix
fn is_likely_pyo3_doc_mode_typo(content: &str) -> bool {
    // Simple fuzzy matching for common typos
    let potential_typos = [
        "pyo3_doc_mde",
        "pyo3_docc_mode",
        "pyo3_doc_mod",
        "py03_doc_mode",
        "pyo3doc_mode",
        "pyo3_docmode",
        "pyo_doc_mode",
        "pyo3_doc_node",
    ];

    potential_typos.iter().any(|&typo| {
        content.starts_with(typo)
            || (content.len() >= typo.len() - 2
                && simple_edit_distance(content.split(':').next().unwrap_or(""), typo) <= 2)
    })
}

/// Simple edit distance calculation for typo detection
fn simple_edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    // Initialize first row and column
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = (matrix[i - 1][j] + 1) // deletion
                .min(matrix[i][j - 1] + 1) // insertion
                .min(matrix[i - 1][j - 1] + cost); // substitution
        }
    }

    matrix[a_len][b_len]
}

impl quote::ToTokens for PythonDoc {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match &self.0 {
            PythonDocKind::LitCStr(lit) => lit.to_tokens(tokens),
            PythonDocKind::Tokens(toks) => toks.to_tokens(tokens),
        }
    }
}

pub fn unwrap_ty_group(mut ty: &syn::Type) -> &syn::Type {
    while let syn::Type::Group(g) = ty {
        ty = &*g.elem;
    }
    ty
}

pub struct Ctx {
    /// Where we can find the pyo3 crate
    pub pyo3_path: PyO3CratePath,

    /// If we are in a pymethod or pyfunction,
    /// this will be the span of the return type
    pub output_span: Span,
}

impl Ctx {
    pub(crate) fn new(attr: &Option<CrateAttribute>, signature: Option<&syn::Signature>) -> Self {
        let pyo3_path = match attr {
            Some(attr) => PyO3CratePath::Given(attr.value.0.clone()),
            None => PyO3CratePath::Default,
        };

        let output_span = if let Some(syn::Signature {
            output: syn::ReturnType::Type(_, output_type),
            ..
        }) = &signature
        {
            output_type.span()
        } else {
            Span::call_site()
        };

        Self {
            pyo3_path,
            output_span,
        }
    }
}

#[derive(Clone)]
pub enum PyO3CratePath {
    Given(syn::Path),
    Default,
}

impl PyO3CratePath {
    pub fn to_tokens_spanned(&self, span: Span) -> TokenStream {
        match self {
            Self::Given(path) => quote::quote_spanned! { span => #path },
            Self::Default => quote::quote_spanned! {  span => ::pyo3 },
        }
    }
}

impl quote::ToTokens for PyO3CratePath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Given(path) => path.to_tokens(tokens),
            Self::Default => quote::quote! { ::pyo3 }.to_tokens(tokens),
        }
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

pub(crate) enum IdentOrStr<'a> {
    Str(&'a str),
    Ident(syn::Ident),
}

pub(crate) fn has_attribute(attrs: &[syn::Attribute], ident: &str) -> bool {
    has_attribute_with_namespace(attrs, None, &[ident])
}

pub(crate) fn has_attribute_with_namespace(
    attrs: &[syn::Attribute],
    crate_path: Option<&PyO3CratePath>,
    idents: &[&str],
) -> bool {
    let mut segments = vec![];
    if let Some(c) = crate_path {
        match c {
            PyO3CratePath::Given(paths) => {
                for p in &paths.segments {
                    segments.push(IdentOrStr::Ident(p.ident.clone()));
                }
            }
            PyO3CratePath::Default => segments.push(IdentOrStr::Str("pyo3")),
        }
    };
    for i in idents {
        segments.push(IdentOrStr::Str(i));
    }

    attrs.iter().any(|attr| {
        segments
            .iter()
            .eq(attr.path().segments.iter().map(|v| &v.ident))
    })
}

pub fn expr_to_python(expr: &syn::Expr) -> String {
    match expr {
        // literal values
        syn::Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
            syn::Lit::Str(s) => s.token().to_string(),
            syn::Lit::Char(c) => c.token().to_string(),
            syn::Lit::Int(i) => i.base10_digits().to_string(),
            syn::Lit::Float(f) => f.base10_digits().to_string(),
            syn::Lit::Bool(b) => {
                if b.value() {
                    "True".to_string()
                } else {
                    "False".to_string()
                }
            }
            _ => "...".to_string(),
        },
        // None
        syn::Expr::Path(syn::ExprPath { qself, path, .. })
            if qself.is_none() && path.is_ident("None") =>
        {
            "None".to_string()
        }
        // others, unsupported yet so defaults to `...`
        _ => "...".to_string(),
    }
}
