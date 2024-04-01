use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Token,
};

use crate::{
    attributes::{kw, KeywordAttribute},
    method::{FnArg, FnArgKind},
};

pub struct Signature {
    paren_token: syn::token::Paren,
    pub items: Punctuated<SignatureItem, Token![,]>,
}

impl Parse for Signature {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        let paren_token = syn::parenthesized!(content in input);

        let items = content.parse_terminated(SignatureItem::parse, Token![,])?;

        Ok(Signature { paren_token, items })
    }
}

impl ToTokens for Signature {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.paren_token
            .surround(tokens, |tokens| self.items.to_tokens(tokens))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SignatureItemArgument {
    pub ident: syn::Ident,
    pub eq_and_default: Option<(Token![=], syn::Expr)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SignatureItemPosargsSep {
    pub slash: Token![/],
}

#[derive(Debug, PartialEq, Eq)]
pub struct SignatureItemVarargsSep {
    pub asterisk: Token![*],
}

#[derive(Debug, PartialEq, Eq)]
pub struct SignatureItemVarargs {
    pub sep: SignatureItemVarargsSep,
    pub ident: syn::Ident,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SignatureItemKwargs {
    pub asterisks: (Token![*], Token![*]),
    pub ident: syn::Ident,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SignatureItem {
    Argument(Box<SignatureItemArgument>),
    PosargsSep(SignatureItemPosargsSep),
    VarargsSep(SignatureItemVarargsSep),
    Varargs(SignatureItemVarargs),
    Kwargs(SignatureItemKwargs),
}

impl Parse for SignatureItem {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![*]) {
            if input.peek2(Token![*]) {
                input.parse().map(SignatureItem::Kwargs)
            } else {
                let sep = input.parse()?;
                if input.is_empty() || input.peek(Token![,]) {
                    Ok(SignatureItem::VarargsSep(sep))
                } else {
                    Ok(SignatureItem::Varargs(SignatureItemVarargs {
                        sep,
                        ident: input.parse()?,
                    }))
                }
            }
        } else if lookahead.peek(Token![/]) {
            input.parse().map(SignatureItem::PosargsSep)
        } else {
            input.parse().map(SignatureItem::Argument)
        }
    }
}

impl ToTokens for SignatureItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SignatureItem::Argument(arg) => arg.to_tokens(tokens),
            SignatureItem::Varargs(varargs) => varargs.to_tokens(tokens),
            SignatureItem::VarargsSep(sep) => sep.to_tokens(tokens),
            SignatureItem::Kwargs(kwargs) => kwargs.to_tokens(tokens),
            SignatureItem::PosargsSep(sep) => sep.to_tokens(tokens),
        }
    }
}

impl Parse for SignatureItemArgument {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            eq_and_default: if input.peek(Token![=]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            },
        })
    }
}

impl ToTokens for SignatureItemArgument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        if let Some((eq, default)) = &self.eq_and_default {
            eq.to_tokens(tokens);
            default.to_tokens(tokens);
        }
    }
}

impl Parse for SignatureItemVarargsSep {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            asterisk: input.parse()?,
        })
    }
}

impl ToTokens for SignatureItemVarargsSep {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.asterisk.to_tokens(tokens);
    }
}

impl Parse for SignatureItemVarargs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            sep: input.parse()?,
            ident: input.parse()?,
        })
    }
}

impl ToTokens for SignatureItemVarargs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.sep.to_tokens(tokens);
        self.ident.to_tokens(tokens);
    }
}

impl Parse for SignatureItemKwargs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            asterisks: (input.parse()?, input.parse()?),
            ident: input.parse()?,
        })
    }
}

impl ToTokens for SignatureItemKwargs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.asterisks.0.to_tokens(tokens);
        self.asterisks.1.to_tokens(tokens);
        self.ident.to_tokens(tokens);
    }
}

impl Parse for SignatureItemPosargsSep {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            slash: input.parse()?,
        })
    }
}

impl ToTokens for SignatureItemPosargsSep {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.slash.to_tokens(tokens);
    }
}

pub type SignatureAttribute = KeywordAttribute<kw::signature, Signature>;

#[derive(Default)]
pub struct PythonSignature {
    pub positional_parameters: Vec<String>,
    pub positional_only_parameters: usize,
    pub required_positional_parameters: usize,
    pub varargs: Option<String>,
    // Tuples of keyword name and whether it is required
    pub keyword_only_parameters: Vec<(String, bool)>,
    pub kwargs: Option<String>,
}

impl PythonSignature {
    pub fn has_no_args(&self) -> bool {
        self.positional_parameters.is_empty()
            && self.keyword_only_parameters.is_empty()
            && self.varargs.is_none()
            && self.kwargs.is_none()
    }
}

pub struct FunctionSignature<'a> {
    pub arguments: Vec<FnArg<'a>>,
    pub python_signature: PythonSignature,
    pub attribute: Option<SignatureAttribute>,
}

pub enum ParseState {
    /// Accepting positional parameters, which might be positional only
    Positional,
    /// Accepting positional parameters after '/'
    PositionalAfterPosargs,
    /// Accepting keyword-only parameters after '*' or '*args'
    Keywords,
    /// After `**kwargs` nothing is allowed
    Done,
}

impl ParseState {
    fn add_argument(
        &mut self,
        signature: &mut PythonSignature,
        name: String,
        required: bool,
        span: Span,
    ) -> syn::Result<()> {
        match self {
            ParseState::Positional | ParseState::PositionalAfterPosargs => {
                signature.positional_parameters.push(name);
                if required {
                    signature.required_positional_parameters += 1;
                    ensure_spanned!(
                        signature.required_positional_parameters == signature.positional_parameters.len(),
                        span => "cannot have required positional parameter after an optional parameter"
                    );
                }
                Ok(())
            }
            ParseState::Keywords => {
                signature.keyword_only_parameters.push((name, required));
                Ok(())
            }
            ParseState::Done => {
                bail_spanned!(span => format!("no more arguments are allowed after `**{}`", signature.kwargs.as_deref().unwrap_or("")))
            }
        }
    }

    fn add_varargs(
        &mut self,
        signature: &mut PythonSignature,
        varargs: &SignatureItemVarargs,
    ) -> syn::Result<()> {
        match self {
            ParseState::Positional | ParseState::PositionalAfterPosargs => {
                signature.varargs = Some(varargs.ident.to_string());
                *self = ParseState::Keywords;
                Ok(())
            }
            ParseState::Keywords => {
                bail_spanned!(varargs.span() => format!("`*{}` not allowed after `*{}`", varargs.ident, signature.varargs.as_deref().unwrap_or("")))
            }
            ParseState::Done => {
                bail_spanned!(varargs.span() => format!("`*{}` not allowed after `**{}`", varargs.ident, signature.kwargs.as_deref().unwrap_or("")))
            }
        }
    }

    fn add_kwargs(
        &mut self,
        signature: &mut PythonSignature,
        kwargs: &SignatureItemKwargs,
    ) -> syn::Result<()> {
        match self {
            ParseState::Positional | ParseState::PositionalAfterPosargs | ParseState::Keywords => {
                signature.kwargs = Some(kwargs.ident.to_string());
                *self = ParseState::Done;
                Ok(())
            }
            ParseState::Done => {
                bail_spanned!(kwargs.span() => format!("`**{}` not allowed after `**{}`", kwargs.ident, signature.kwargs.as_deref().unwrap_or("")))
            }
        }
    }

    fn finish_pos_only_args(
        &mut self,
        signature: &mut PythonSignature,
        span: Span,
    ) -> syn::Result<()> {
        match self {
            ParseState::Positional => {
                signature.positional_only_parameters = signature.positional_parameters.len();
                *self = ParseState::PositionalAfterPosargs;
                Ok(())
            }
            ParseState::PositionalAfterPosargs => {
                bail_spanned!(span => "`/` not allowed after `/`")
            }
            ParseState::Keywords => {
                bail_spanned!(span => format!("`/` not allowed after `*{}`", signature.varargs.as_deref().unwrap_or("")))
            }
            ParseState::Done => {
                bail_spanned!(span => format!("`/` not allowed after `**{}`", signature.kwargs.as_deref().unwrap_or("")))
            }
        }
    }

    fn finish_pos_args(&mut self, signature: &PythonSignature, span: Span) -> syn::Result<()> {
        match self {
            ParseState::Positional | ParseState::PositionalAfterPosargs => {
                *self = ParseState::Keywords;
                Ok(())
            }
            ParseState::Keywords => {
                bail_spanned!(span => format!("`*` not allowed after `*{}`", signature.varargs.as_deref().unwrap_or("")))
            }
            ParseState::Done => {
                bail_spanned!(span => format!("`*` not allowed after `**{}`", signature.kwargs.as_deref().unwrap_or("")))
            }
        }
    }
}

impl<'a> FunctionSignature<'a> {
    pub fn from_arguments_and_attribute(
        mut arguments: Vec<FnArg<'a>>,
        attribute: SignatureAttribute,
    ) -> syn::Result<Self> {
        let mut parse_state = ParseState::Positional;
        let mut python_signature = PythonSignature::default();

        let mut args_iter = arguments.iter_mut();

        let mut next_non_py_argument_checked = |name: &syn::Ident| {
            for fn_arg in args_iter.by_ref() {
                match fn_arg.kind {
                    crate::method::FnArgKind::Py => {
                        // If the user incorrectly tried to include py: Python in the
                        // signature, give a useful error as a hint.
                        ensure_spanned!(
                            name != fn_arg.name,
                            name.span() => "arguments of type `Python` must not be part of the signature"
                        );
                        // Otherwise try next argument.
                        continue;
                    }
                    crate::method::FnArgKind::CancelHandle => {
                        // If the user incorrectly tried to include cancel: CoroutineCancel in the
                        // signature, give a useful error as a hint.
                        ensure_spanned!(
                            name != fn_arg.name,
                            name.span() => "`cancel_handle` argument must not be part of the signature"
                        );
                        // Otherwise try next argument.
                        continue;
                    }
                    _ => {
                        ensure_spanned!(
                            name == fn_arg.name,
                            name.span() => format!(
                                "expected argument from function definition `{}` but got argument `{}`",
                                fn_arg.name.unraw(),
                                name.unraw(),
                            )
                        );
                        return Ok(fn_arg);
                    }
                }
            }
            bail_spanned!(
                name.span() => "signature entry does not have a corresponding function argument"
            )
        };

        for item in &attribute.value.items {
            match item {
                SignatureItem::Argument(arg) => {
                    let fn_arg = next_non_py_argument_checked(&arg.ident)?;
                    parse_state.add_argument(
                        &mut python_signature,
                        arg.ident.unraw().to_string(),
                        arg.eq_and_default.is_none(),
                        arg.span(),
                    )?;
                    if let Some((_, default_expr)) = &arg.eq_and_default {
                        if let FnArgKind::Regular { default, .. } = &mut fn_arg.kind {
                            *default = Some(default_expr.clone());
                        } else {
                            // FIXME: In what case can this happen? What should the error message be?
                            bail_spanned!(
                                default_expr.span() => "todo"
                            )
                        }
                    }
                }
                SignatureItem::VarargsSep(sep) => {
                    parse_state.finish_pos_args(&python_signature, sep.span())?
                }
                SignatureItem::Varargs(varargs) => {
                    let fn_arg = next_non_py_argument_checked(&varargs.ident)?;
                    ensure_spanned!(
                        matches!(fn_arg.kind, FnArgKind::Regular { ty_opt: None, .. }),
                        fn_arg.name.span() => "args cannot be optional"
                    );
                    fn_arg.kind = FnArgKind::VarArgs;
                    parse_state.add_varargs(&mut python_signature, varargs)?;
                }
                SignatureItem::Kwargs(kwargs) => {
                    let fn_arg = next_non_py_argument_checked(&kwargs.ident)?;
                    ensure_spanned!(
                        matches!(fn_arg.kind, FnArgKind::Regular { ty_opt: Some(_), .. }),
                        fn_arg.name.span() => "kwargs must be Option<_>"
                    );
                    fn_arg.kind = FnArgKind::KwArgs;
                    parse_state.add_kwargs(&mut python_signature, kwargs)?;
                }
                SignatureItem::PosargsSep(sep) => {
                    parse_state.finish_pos_only_args(&mut python_signature, sep.span())?
                }
            };
        }

        // Ensure no non-py arguments remain
        if let Some(arg) =
            args_iter.find(|arg| !matches!(arg.kind, FnArgKind::Py | FnArgKind::CancelHandle))
        {
            bail_spanned!(
                attribute.kw.span() => format!("missing signature entry for argument `{}`", arg.name)
            );
        }

        Ok(FunctionSignature {
            arguments,
            python_signature,
            attribute: Some(attribute),
        })
    }

    /// Without `#[pyo3(signature)]` or `#[args]` - just take the Rust function arguments as positional.
    pub fn from_arguments(arguments: Vec<FnArg<'a>>) -> syn::Result<Self> {
        let mut python_signature = PythonSignature::default();
        for arg in &arguments {
            // Python<'_> arguments don't show in Python signature
            if matches!(arg.kind, FnArgKind::Py | FnArgKind::CancelHandle) {
                continue;
            }

            if matches!(arg.kind, FnArgKind::Regular { ty_opt: None, .. }) {
                // This argument is required, all previous arguments must also have been required
                ensure_spanned!(
                    python_signature.required_positional_parameters == python_signature.positional_parameters.len(),
                    arg.ty.span() => "required arguments after an `Option<_>` argument are ambiguous\n\
                    = help: add a `#[pyo3(signature)]` annotation on this function to unambiguously specify the default values for all optional parameters"
                );

                python_signature.required_positional_parameters =
                    python_signature.positional_parameters.len() + 1;
            }

            python_signature
                .positional_parameters
                .push(arg.name.unraw().to_string());
        }

        Ok(Self {
            arguments,
            python_signature,
            attribute: None,
        })
    }

    fn default_value_for_parameter(&self, parameter: &str) -> String {
        let mut default = "...".to_string();
        if let Some(fn_arg) = self.arguments.iter().find(|arg| arg.name == parameter) {
            if let FnArg {
                kind:
                    FnArgKind::Regular {
                        default: Some(arg_default),
                        ..
                    },
                ..
            } = fn_arg
            {
                match arg_default {
                    // literal values
                    syn::Expr::Lit(syn::ExprLit { lit, .. }) => match lit {
                        syn::Lit::Str(s) => default = s.token().to_string(),
                        syn::Lit::Char(c) => default = c.token().to_string(),
                        syn::Lit::Int(i) => default = i.base10_digits().to_string(),
                        syn::Lit::Float(f) => default = f.base10_digits().to_string(),
                        syn::Lit::Bool(b) => {
                            default = if b.value() {
                                "True".to_string()
                            } else {
                                "False".to_string()
                            }
                        }
                        _ => {}
                    },
                    // None
                    syn::Expr::Path(syn::ExprPath {
                        qself: None, path, ..
                    }) if path.is_ident("None") => {
                        default = "None".to_string();
                    }
                    // others, unsupported yet so defaults to `...`
                    _ => {}
                }
            } else if let FnArg {
                kind:
                    FnArgKind::Regular {
                        ty_opt: Some(..), ..
                    },
                ..
            } = fn_arg
            {
                // functions without a `#[pyo3(signature = (...))]` option
                // will treat trailing `Option<T>` arguments as having a default of `None`
                default = "None".to_string();
            }
        }
        default
    }

    pub fn text_signature(&self, self_argument: Option<&str>) -> String {
        let mut output = String::new();
        output.push('(');

        if let Some(arg) = self_argument {
            output.push('$');
            output.push_str(arg);
        }

        let mut maybe_push_comma = {
            let mut first = self_argument.is_none();
            move |output: &mut String| {
                if !first {
                    output.push_str(", ");
                } else {
                    first = false;
                }
            }
        };

        let py_sig = &self.python_signature;

        for (i, parameter) in py_sig.positional_parameters.iter().enumerate() {
            maybe_push_comma(&mut output);

            output.push_str(parameter);

            if i >= py_sig.required_positional_parameters {
                output.push('=');
                output.push_str(&self.default_value_for_parameter(parameter));
            }

            if py_sig.positional_only_parameters > 0 && i + 1 == py_sig.positional_only_parameters {
                output.push_str(", /")
            }
        }

        if let Some(varargs) = &py_sig.varargs {
            maybe_push_comma(&mut output);
            output.push('*');
            output.push_str(varargs);
        } else if !py_sig.keyword_only_parameters.is_empty() {
            maybe_push_comma(&mut output);
            output.push('*');
        }

        for (parameter, required) in &py_sig.keyword_only_parameters {
            maybe_push_comma(&mut output);
            output.push_str(parameter);
            if !required {
                output.push('=');
                output.push_str(&self.default_value_for_parameter(parameter));
            }
        }

        if let Some(kwargs) = &py_sig.kwargs {
            maybe_push_comma(&mut output);
            output.push_str("**");
            output.push_str(kwargs);
        }

        output.push(')');
        output
    }
}
