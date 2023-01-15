use std::cmp::max;

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
    deprecations::{Deprecation, Deprecations},
    method::{FnArg, FnType},
    pyfunction::Argument,
};

use super::DeprecatedArgs;

pub struct Signature {
    paren_token: syn::token::Paren,
    pub items: Punctuated<SignatureItem, Token![,]>,
}

impl Parse for Signature {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        Ok(Signature {
            paren_token: syn::parenthesized!(content in input),
            items: content.parse_terminated(SignatureItem::parse)?,
        })
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

        let mut args_iter = arguments.iter_mut().filter(|arg| !arg.py); // Python<'_> arguments don't show on the Python side.

        let mut next_argument_checked = |name: &syn::Ident| match args_iter.next() {
            Some(fn_arg) => {
                ensure_spanned!(
                    name == fn_arg.name,
                    name.span() => format!(
                        "expected argument from function definition `{}` but got argument `{}`",
                        fn_arg.name.unraw(),
                        name.unraw(),
                    )
                );
                Ok(fn_arg)
            }
            None => bail_spanned!(
                name.span() => "signature entry does not have a corresponding function argument"
            ),
        };

        for item in &attribute.value.items {
            match item {
                SignatureItem::Argument(arg) => {
                    let fn_arg = next_argument_checked(&arg.ident)?;
                    parse_state.add_argument(
                        &mut python_signature,
                        arg.ident.unraw().to_string(),
                        arg.eq_and_default.is_none(),
                        arg.span(),
                    )?;
                    if let Some((_, default)) = &arg.eq_and_default {
                        fn_arg.default = Some(default.clone());
                    }
                }
                SignatureItem::VarargsSep(sep) => {
                    parse_state.finish_pos_args(&python_signature, sep.span())?
                }
                SignatureItem::Varargs(varargs) => {
                    let fn_arg = next_argument_checked(&varargs.ident)?;
                    fn_arg.is_varargs = true;
                    parse_state.add_varargs(&mut python_signature, &varargs)?;
                }
                SignatureItem::Kwargs(kwargs) => {
                    let fn_arg = next_argument_checked(&kwargs.ident)?;
                    fn_arg.is_kwargs = true;
                    parse_state.add_kwargs(&mut python_signature, &kwargs)?;
                }
                SignatureItem::PosargsSep(sep) => {
                    parse_state.finish_pos_only_args(&mut python_signature, sep.span())?
                }
            };
        }

        if let Some(arg) = args_iter.next() {
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

    /// The difference to `from_arguments_and_signature` is that deprecated args allowed entries to be:
    ///  - missing
    ///  - out of order
    pub fn from_arguments_and_deprecated_args(
        mut arguments: Vec<FnArg<'a>>,
        deprecated_args: DeprecatedArgs,
    ) -> syn::Result<Self> {
        let mut varargs = None;
        let mut kwargs = None;
        let mut keyword_only_parameters = Vec::new();

        fn first_n_argument_names(arguments: &[FnArg<'_>], count: usize) -> Vec<String> {
            arguments
                .iter()
                .filter_map(|fn_arg| {
                    if fn_arg.py {
                        None
                    } else {
                        Some(fn_arg.name.unraw().to_string())
                    }
                })
                .take(count)
                .collect()
        }

        // Record highest counts observed based off argument positions
        let mut positional_only_arguments_count = None;
        let mut positional_arguments_count = None;
        let mut required_positional_parameters = 0;

        let args_iter = arguments.iter_mut().filter(|arg| !arg.py); // Python<'_> arguments don't show on the Python side.

        for (i, fn_arg) in args_iter.enumerate() {
            if let Some(argument) = deprecated_args
                .arguments
                .iter()
                .find(|argument| match argument {
                    Argument::PosOnlyArg(path, _)
                    | Argument::Arg(path, _)
                    | Argument::Kwarg(path, _)
                    | Argument::VarArgs(path)
                    | Argument::KeywordArgs(path) => path.get_ident() == Some(fn_arg.name),
                    _ => false,
                })
            {
                match argument {
                    Argument::PosOnlyArg(_, default) | Argument::Arg(_, default) => {
                        if let Some(default) = default {
                            fn_arg.default = Some(syn::parse_str(default)?);
                        } else if fn_arg.optional.is_none() {
                            // Option<_> arguments always have an implicit None default with the old
                            // `#[args]`
                            required_positional_parameters = i + 1;
                        }
                        if matches!(argument, Argument::PosOnlyArg(_, _)) {
                            positional_only_arguments_count = Some(i + 1);
                        }
                        positional_arguments_count = Some(i + 1);
                    }
                    Argument::Kwarg(_, default) => {
                        fn_arg.default = default.as_deref().map(syn::parse_str).transpose()?;
                        keyword_only_parameters.push((fn_arg.name.to_string(), default.is_none()));
                    }
                    Argument::PosOnlyArgsSeparator => {}
                    Argument::VarArgsSeparator => {}
                    Argument::VarArgs(path) => {
                        fn_arg.is_varargs = true;
                        if let Some(ident) = path.get_ident() {
                            varargs = Some(ident.to_string());
                        } else {
                            bail_spanned!(path.span() => "expected ident for *args");
                        };
                    }
                    Argument::KeywordArgs(path) => {
                        fn_arg.is_kwargs = true;
                        if let Some(ident) = path.get_ident() {
                            kwargs = Some(ident.to_string());
                        } else {
                            bail_spanned!(path.span() => "expected ident for **kwargs");
                        };
                    }
                }
            } else {
                // Assume this is a required positional parameter
                required_positional_parameters = i + 1;
                positional_arguments_count = Some(i + 1);
            }
        }

        // fix up state based on observations above
        let positional_only_parameters = positional_only_arguments_count.unwrap_or(0);
        let positional_parameters = first_n_argument_names(
            &arguments,
            max(
                positional_arguments_count.unwrap_or(0),
                positional_only_arguments_count.unwrap_or(0),
            ),
        );

        Ok(FunctionSignature {
            arguments,
            python_signature: PythonSignature {
                positional_parameters,
                positional_only_parameters,
                required_positional_parameters,
                varargs,
                keyword_only_parameters,
                kwargs,
            },
            attribute: None,
        })
    }

    /// Without `#[pyo3(signature)]` or `#[args]` - just take the Rust function arguments as positional.
    pub fn from_arguments(mut arguments: Vec<FnArg<'a>>, deprecations: &mut Deprecations) -> Self {
        let mut python_signature = PythonSignature::default();
        for arg in &arguments {
            // Python<'_> arguments don't show in Python signature
            if arg.py {
                continue;
            }

            if arg.optional.is_none() {
                // This argument is required
                if python_signature.required_positional_parameters
                    != python_signature.positional_parameters.len()
                {
                    // A previous argument was not required
                    deprecations.push(Deprecation::RequiredArgumentAfterOption, arg.name.span());
                }

                python_signature.required_positional_parameters =
                    python_signature.positional_parameters.len() + 1;
            }

            python_signature
                .positional_parameters
                .push(arg.name.unraw().to_string());
        }

        // Fixup any `Option<_>` arguments that were made implicitly made required by the deprecated
        // branch above
        for arg in arguments
            .iter_mut()
            .take(python_signature.required_positional_parameters)
        {
            arg.optional = None;
        }

        Self {
            arguments,
            python_signature,
            attribute: None,
        }
    }

    pub fn text_signature(&self, fn_type: &FnType) -> String {
        // automatic text signature generation
        let self_argument = match fn_type {
            FnType::FnNew | FnType::Getter(_) | FnType::Setter(_) | FnType::ClassAttribute => {
                unreachable!()
            }
            FnType::Fn(_) => Some("self"),
            FnType::FnModule => Some("module"),
            FnType::FnClass => Some("cls"),
            FnType::FnStatic => None,
        };

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
                // has a default, just use ... for now
                output.push_str("=...");
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
                // has a default, just use ... for now
                output.push_str("=...")
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
