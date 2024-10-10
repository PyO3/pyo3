use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parenthesized, token, Error, Result, Token};

#[derive(Debug)]
pub struct PycallInput {
    pyo3_path: syn::Ident,
    callable_or_receiver: syn::Expr,
    method_name: Option<PyString>,
    args: ArgList,
}

impl Parse for PycallInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let pyo3_path = input.parse()?;
        let callable = parse_ident_or_parenthesized_expr(&input)?;
        let method_name = if input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            let method_name = if input.peek(syn::Ident) {
                PyString::FromIdent(input.parse()?)
            } else {
                let callable;
                parenthesized!(callable in input);
                PyString::FromValue(callable.parse()?)
            };
            Some(method_name)
        } else {
            None
        };
        let args = input.parse()?;
        Ok(Self {
            pyo3_path,
            callable_or_receiver: callable,
            method_name,
            args,
        })
    }
}

#[derive(Debug)]
enum PyString {
    FromIdent(syn::Ident),
    FromValue(syn::Expr),
}

#[derive(Debug)]
struct ArgList(Punctuated<Arg, Token![,]>);

impl Parse for ArgList {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let args;
        parenthesized!(args in input);
        let args = Punctuated::parse_terminated(&args)?;
        Ok(Self(args))
    }
}

#[derive(Debug)]
enum Arg {
    Arg(syn::Expr),
    Kwarg {
        name: PyString,
        value: syn::Expr,
    },
    UnpackArgs {
        unpack_parens: token::Paren,
        value: syn::Expr,
    },
    UnpackKwargs {
        unpack_parens: token::Paren,
        value: syn::Expr,
    },
}

impl Parse for Arg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(token::Paren) {
            if input.peek2(Token![=]) {
                let name;
                parenthesized!(name in input);
                let name = PyString::FromValue(name.parse()?);
                input.parse::<Token![=]>()?;
                let value = input.parse()?;
                return Ok(Arg::Kwarg { name, value });
            }
            let in_parens;
            let unpack_parens = parenthesized!(in_parens in input.fork());
            if in_parens.parse::<Token![*]>().is_ok() {
                if in_parens.is_empty() {
                    let stars;
                    parenthesized!(stars in input);
                    // Necessary because syn checks we parsed the full thing.
                    stars.parse::<TokenStream>()?;
                    let value = input.parse()?;
                    return Ok(Arg::UnpackArgs {
                        unpack_parens,
                        value,
                    });
                }
                if in_parens.parse::<Token![*]>().is_ok() && in_parens.is_empty() {
                    let stars;
                    parenthesized!(stars in input);
                    // Necessary because syn checks we parsed the full thing.
                    stars.parse::<TokenStream>()?;
                    let value = input.parse()?;
                    return Ok(Arg::UnpackKwargs {
                        unpack_parens,
                        value,
                    });
                }
            }
        }
        if input.peek(syn::Ident) && input.peek2(Token![=]) {
            let name = PyString::FromIdent(input.parse()?);
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            return Ok(Arg::Kwarg { name, value });
        }
        let value = input.parse()?;
        Ok(Arg::Arg(value))
    }
}

fn parse_ident_or_parenthesized_expr(input: &ParseStream<'_>) -> Result<syn::Expr> {
    if input.peek(syn::Ident) {
        Ok(syn::Expr::Path(input.parse()?))
    } else {
        let callable;
        parenthesized!(callable in input);
        callable.parse()
    }
}

pub fn build_pycall_output(input: PycallInput) -> Result<TokenStream> {
    check_args_order(&input.args)?;
    check_duplicate_kwargs(&input.args)?;
    let pyo3_path = &input.pyo3_path;
    let result = store_values_in_variables(input.args, |args_with_variables| {
        let args = build_args(pyo3_path, args_with_variables);
        let kwargs = build_kwargs(pyo3_path, args_with_variables);
        let callable_or_receiver = &input.callable_or_receiver;
        let tokens = match &input.method_name {
            Some(method_name) => {
                let method_name = match method_name {
                    PyString::FromIdent(method_name) => {
                        let method_name = method_name.to_string();
                        let method_name = method_name.trim_start_matches("r#");
                        quote! { #pyo3_path::intern!(unsafe { #pyo3_path::Python::assume_gil_acquired() }, #method_name) }
                    }
                    PyString::FromValue(method_name) => quote! { #method_name },
                };
                quote! {
                    #pyo3_path::pycall::call_method(&(#callable_or_receiver), #method_name, #args, #kwargs)
                }
            }
            None => quote! {
                #pyo3_path::pycall::call(&(#callable_or_receiver), #args, #kwargs)
            },
        };
        wrap_call(pyo3_path, tokens)
    });
    Ok(result)
}

fn check_duplicate_kwargs(args: &ArgList) -> Result<()> {
    let kwargs = args.0.iter().filter_map(|arg| match arg {
        Arg::Kwarg {
            name: PyString::FromIdent(name),
            value: _,
        } => Some(name),
        _ => None,
    });
    let mut prev_kwargs = HashSet::new();
    let mut errors = Vec::new();
    for kwarg in kwargs {
        let kwarg_string = kwarg.to_string().trim_start_matches("r#").to_owned();
        if prev_kwargs.contains(&kwarg_string) {
            errors.push(syn::Error::new_spanned(kwarg, "duplicate kwarg"));
        } else {
            prev_kwargs.insert(kwarg_string);
        }
    }
    let errors = errors.into_iter().reduce(|mut errors, error| {
        errors.combine(error);
        errors
    });
    match errors {
        None => Ok(()),
        Some(errors) => Err(errors),
    }
}

fn check_args_order(args: &ArgList) -> Result<()> {
    let mut started_kwargs = false;
    for arg in &args.0 {
        match arg {
            Arg::Kwarg { .. } | Arg::UnpackKwargs { .. } => started_kwargs = true,
            Arg::Arg(arg) => {
                if started_kwargs {
                    return Err(Error::new_spanned(arg, "normal argument after kwargs"));
                }
            }
            Arg::UnpackArgs { unpack_parens, .. } => {
                if started_kwargs {
                    return Err(Error::new(
                        unpack_parens.span.span(),
                        "normal arguments unpack after kwargs",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn wrap_call(pyo3_path: &syn::Ident, call: TokenStream) -> TokenStream {
    quote! {
        {
            #[allow(unused_imports)]
            use #pyo3_path::pycall::select_traits::*;
            #call
        }
    }
}

const MAX_TUPLE_SIZE: usize = 13;

#[derive(Debug)]
enum ArgWithVariable {
    Arg(syn::Expr),
    Kwarg { name: PyString, value: syn::Expr },
    UnpackArgs(syn::Ident),
    UnpackKwargs(syn::Ident),
}

fn store_values_in_variables(
    args: ArgList,
    callback: impl FnOnce(&[ArgWithVariable]) -> TokenStream,
) -> TokenStream {
    let idents_exprs = args
        .0
        .iter()
        .filter_map(|arg| match arg {
            Arg::UnpackArgs {
                unpack_parens: _,
                value,
            }
            | Arg::UnpackKwargs {
                unpack_parens: _,
                value,
            } => Some(value.clone()),
            Arg::Arg(..) | Arg::Kwarg { .. } => None,
        })
        .collect::<Vec<_>>();
    let args_with_variables = args
        .0
        .into_iter()
        .enumerate()
        .map(|(index, arg)| match arg {
            Arg::Arg(expr) => ArgWithVariable::Arg(expr),
            Arg::Kwarg { name, value } => ArgWithVariable::Kwarg { name, value },
            Arg::UnpackKwargs {
                unpack_parens,
                value: _,
            } => ArgWithVariable::UnpackKwargs(format_ident!(
                "__py_arg_{index}",
                span = unpack_parens.span.span()
            )),
            Arg::UnpackArgs {
                unpack_parens,
                value: _,
            } => ArgWithVariable::UnpackArgs(format_ident!(
                "__py_arg_{index}",
                span = unpack_parens.span.span()
            )),
        })
        .collect::<Vec<_>>();
    let idents = args_with_variables.iter().filter_map(|arg| match arg {
        ArgWithVariable::UnpackArgs(ident) | ArgWithVariable::UnpackKwargs(ident) => Some(ident),
        ArgWithVariable::Arg(..) | ArgWithVariable::Kwarg { .. } => None,
    });
    let inside_match = callback(&args_with_variables);
    quote! {
        // `match` so that temporaries will live well.
        match ( #( #idents_exprs, )* ) {
            ( #( #idents, )* ) => {
                #inside_match
            }
        }
    }
}

fn build_args(pyo3_path: &syn::Ident, args: &[ArgWithVariable]) -> TokenStream {
    fn write_normal_args(
        pyo3_path: &syn::Ident,
        tokens: TokenStream,
        consecutive_normal_args: &mut Vec<&syn::Expr>,
    ) -> TokenStream {
        if consecutive_normal_args.is_empty() {
            return tokens;
        }

        let new_args = quote! {
            #pyo3_path::pycall::non_unpacked_args( ( #( #consecutive_normal_args, )* ) )
        };
        let result = quote! {
            #pyo3_path::pycall::concat_args( #tokens, #new_args )
        };
        consecutive_normal_args.clear();
        result
    }

    let mut consecutive_normal_args = Vec::new();
    let mut tokens = quote!(#pyo3_path::pycall::EmptyArgsStorage);
    for arg in args {
        match arg {
            ArgWithVariable::Arg(arg) => {
                consecutive_normal_args.push(arg);
                if consecutive_normal_args.len() == MAX_TUPLE_SIZE {
                    tokens = write_normal_args(pyo3_path, tokens, &mut consecutive_normal_args);
                }
            }
            ArgWithVariable::UnpackArgs(variable) => {
                tokens = write_normal_args(pyo3_path, tokens, &mut consecutive_normal_args);

                let selector = quote! {
                    (&&&&&&&&&&&#pyo3_path::pycall::ArgsStorageSelector::new(loop {
                        break None;
                        // The block is needed because the compiler doesn't respect the `#[allow]` otherwise.
                        #[allow(unreachable_code)]
                        {
                            break Some(#variable);
                        }
                    }))
                };
                let select = quote_spanned! { variable.span() =>
                    #selector.__py_unpack_args_select(#variable)
                };
                tokens = quote_spanned! { variable.span() =>
                    #pyo3_path::pycall::concat_args( #tokens, #select )
                };
            }
            ArgWithVariable::Kwarg { .. } | ArgWithVariable::UnpackKwargs(..) => break,
        }
    }
    if !consecutive_normal_args.is_empty() {
        tokens = write_normal_args(pyo3_path, tokens, &mut consecutive_normal_args);
    }
    tokens
}

fn build_known_kwargs(pyo3_path: &syn::Ident, args: &[ArgWithVariable]) -> Option<TokenStream> {
    let mut known_kwargs = args.iter().filter_map(|it| match it {
        ArgWithVariable::Kwarg {
            name: PyString::FromIdent(name),
            value,
        } => Some((name, value)),
        _ => None,
    });
    let names = known_kwargs
        .clone()
        .map(|(name, _)| name.to_string().trim_start_matches("r#").to_owned());
    let mut values = match known_kwargs.next() {
        Some((_, first_kwarg)) => quote! { #pyo3_path::pycall::first_known_kwarg( #first_kwarg ) },
        None => return None,
    };
    for (_, kwarg) in known_kwargs {
        values = quote! { #pyo3_path::pycall::add_known_kwarg( #kwarg, #values ) };
    }
    Some(quote! {
        #pyo3_path::pycall::known_kwargs_with_names(
            #pyo3_path::known_kwargs!( #(#names)* ),
            #values,
        )
    })
}

fn build_unknown_non_unpacked_kwargs(
    pyo3_path: &syn::Ident,
    args: &[ArgWithVariable],
) -> TokenStream {
    fn write_kwargs(
        pyo3_path: &syn::Ident,
        mut tokens: TokenStream,
        consecutive_kwargs: &mut Vec<TokenStream>,
    ) -> TokenStream {
        tokens = quote! {
            #pyo3_path::pycall::concat_kwargs(
                #tokens,
                #pyo3_path::pycall::non_unpacked_kwargs( ( #( #consecutive_kwargs, )* ) ),
            )
        };
        consecutive_kwargs.clear();
        tokens
    }

    let kwargs = args.iter().filter_map(|it| match it {
        ArgWithVariable::Kwarg {
            name: PyString::FromValue(name),
            value,
        } => Some((name, value)),
        _ => None,
    });
    let mut tokens = quote! { #pyo3_path::pycall::EmptyKwargsStorage };
    let mut consecutive_kwargs = Vec::new();
    for (name, value) in kwargs {
        consecutive_kwargs.push(quote! { (#name, #value) });

        if consecutive_kwargs.len() == MAX_TUPLE_SIZE {
            tokens = write_kwargs(pyo3_path, tokens, &mut consecutive_kwargs);
        }
    }
    if !consecutive_kwargs.is_empty() {
        tokens = write_kwargs(pyo3_path, tokens, &mut consecutive_kwargs);
    }
    tokens
}

fn build_kwargs(pyo3_path: &syn::Ident, args: &[ArgWithVariable]) -> TokenStream {
    let known = build_known_kwargs(pyo3_path, args);
    let unknown_non_unpacked = build_unknown_non_unpacked_kwargs(pyo3_path, args);
    let mut tokens = match known {
        Some(known) => {
            quote! { #pyo3_path::pycall::concat_kwargs( #known, #unknown_non_unpacked ) }
        }
        None => unknown_non_unpacked,
    };

    let unpacked_kwargs = args.iter().filter_map(|it| match it {
        ArgWithVariable::UnpackKwargs(variable) => Some(variable),
        _ => None,
    });
    for variable in unpacked_kwargs {
        let selector = quote! {
            (&&&&&&&&&&&#pyo3_path::pycall::KwargsStorageSelector::new(loop {
                break None;
                // The block is needed because the compiler doesn't respect the `#[allow]` otherwise.
                #[allow(unreachable_code)]
                {
                    break Some(#variable);
                }
            }))
        };
        let select = quote_spanned! { variable.span() =>
            #selector.__py_unpack_kwargs_select(#variable)
        };
        tokens = quote_spanned! { variable.span() =>
            #pyo3_path::pycall::concat_kwargs( #tokens, #select )
        };
    }
    tokens
}
