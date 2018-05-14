// Copyright (c) 2017-present PyO3 Project and Contributors
use syn;

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    VarArgsSeparator,
    VarArgs(syn::Ident),
    KeywordArgs(syn::Ident),
    Arg(syn::Ident, Option<String>),
    Kwarg(syn::Ident, String),
}

pub fn parse_arguments(items: &[syn::NestedMeta]) -> Vec<Argument> {
    let mut arguments = Vec::new();
    let mut has_kw = false;
    let mut has_varargs = false;
    let mut has_kwargs = false;

    let args_str = quote! {
        #(#items),*
    }.to_string();

    for item in items.iter() {
        match item {
            syn::NestedMeta::Meta(syn::Meta::Word(ref ident)) => {
                // arguments in form #[args(somename)]
                if has_kwargs {
                    println!("syntax error, keyword arguments is defined: {:?}", args_str);
                    return Vec::new();
                }
                if has_kw {
                    println!("syntax error, argument is not allowed after keyword argument: {:?}",
                             args_str);
                    return Vec::new()
                }
                arguments.push(Argument::Arg(*ident, None))
            }
            syn::NestedMeta::Meta(syn::Meta::NameValue(ref nv)) => {
                match nv.lit {
                    syn::Lit::Str(ref litstr) => {
                        if litstr.value() == "*" {  // #[args(args="*")]
                            if has_kwargs {
                                println!("* - syntax error, keyword arguments is defined: {:?}",
                                         args_str);
                                return Vec::new()
                            }
                            if has_varargs {
                                println!("*(var args) is defined: {:?}", args_str);
                                return Vec::new()
                            }
                            has_varargs = true;
                            arguments.push(Argument::VarArgs(nv.ident));
                        } else if litstr.value() == "**" {  // #[args(kwargs="**")]
                            if has_kwargs {
                                println!("arguments already define ** (kw args): {:?}",
                                         args_str);
                                return Vec::new()
                            }
                            has_kwargs = true;
                            arguments.push(Argument::KeywordArgs(nv.ident));
                        } else {
                            if has_varargs {
                                arguments.push(Argument::Kwarg(nv.ident, litstr.value().clone()))
                            } else {
                                if has_kwargs {
                                    println!("syntax error, keyword arguments is defined: {:?}",
                                             args_str);
                                    return Vec::new()
                                }
                                has_kw = true;
                                arguments.push(Argument::Arg(nv.ident, Some(litstr.value().clone())))
                            }
                        }
                    }
                    syn::Lit::Int(ref litint) => {
                        if has_varargs {
                            arguments.push(Argument::Kwarg(nv.ident, format!("{}", litint.value())));
                        } else {
                            if has_kwargs {
                                println!("syntax error, keyword arguments is defined: {:?}",
                                         args_str);
                                return Vec::new()
                            }
                            has_kw = true;
                            arguments.push(Argument::Arg(nv.ident, Some(format!("{}", litint.value()))));
                        }
                    }
                    syn::Lit::Bool(ref litb) => {
                        if has_varargs {
                            arguments.push(Argument::Kwarg(nv.ident, format!("{}", litb.value)));
                        } else {
                            if has_kwargs {
                                println!("syntax error, keyword arguments is defined: {:?}",
                                         args_str);
                                return Vec::new()
                            }
                            has_kw = true;
                            arguments.push(Argument::Arg(nv.ident, Some(format!("{}", litb.value))));
                        }
                    }
                    _ => {
                        println!("Only string literal is supported, got: {:?}", nv.lit);
                        return Vec::new()
                    }
                }
            }
            syn::NestedMeta::Literal(ref lit) => {
                match lit {
                    &syn::Lit::Str(ref lits) => {
                        // #[args("*")]
                        if lits.value() == "*" {
                            if has_kwargs {
                                println!(
                                    "syntax error, keyword arguments is defined: {:?}",
                                    args_str);
                                return Vec::new()
                            }
                            if has_varargs {
                                println!(
                                    "arguments already define * (var args): {:?}",
                                    args_str);
                                return Vec::new()
                            }
                            has_varargs = true;
                            arguments.push(Argument::VarArgsSeparator);
                        } else {
                            println!("Unknown string literal, got: {:?} args: {:?}",
                                     lits.value(), args_str);
                            return Vec::new()
                        }
                    }
                    _ => {
                        println!("Only string literal is supported, got: {:?} args: {:?}",
                                 lit, args_str);
                        return Vec::new()
                    }
                }
            }
            _ => {
                println!("Unknown argument, got: {:?} args: {:?}", item, args_str);
                return Vec::new()
            }
        }
    }

    arguments
}


#[cfg(test)]
mod test {

    use syn;
    use args::{Argument, parse_arguments};
    use quote::Tokens;

    fn items(s: Tokens) -> Vec<syn::NestedMeta> {
        let dummy: syn::ItemFn = parse_quote!{#s fn dummy() {}};
        match dummy.attrs[0].interpret_meta() {
            Some(syn::Meta::List(syn::MetaList { nested, .. })) => {
                nested.iter().map(Clone::clone).collect()
            }
            _ => unreachable!()
        }
    }

    #[test]
    fn test_errs() {
        assert!(parse_arguments(&items(quote!{#[args(test="1", test2)]})).is_empty());
        assert!(parse_arguments(&items(quote!{#[args(test=1, "*", args="*")]})).is_empty());
        assert!(parse_arguments(&items(quote!{#[args(test=1, kwargs="**", args="*")]})).is_empty());
        assert!(parse_arguments(&items(quote!{#[args(test=1, kwargs="**", args)]})).is_empty());
    }

    #[test]
    fn test_simple_args() {
        let args = parse_arguments(&items(quote!{#[args(test1, test2, test3="None")]}));
        assert!(args == vec![Argument::Arg(parse_quote!{test1}, None),
                             Argument::Arg(parse_quote!{test2}, None),
                             Argument::Arg(parse_quote!{test3}, Some("None".to_owned()))]);
    }

    #[test]
    fn test_varargs() {
        let args = parse_arguments(&items(quote!{#[args(test1, test2="None", "*", test3="None")]}));
        assert!(args == vec![Argument::Arg(parse_quote!{test1}, None),
                             Argument::Arg(parse_quote!{test2}, Some("None".to_owned())),
                             Argument::VarArgsSeparator,
                             Argument::Kwarg(parse_quote!{test3}, "None".to_owned())]);
    }

    #[test]
    fn test_all() {
        let args = parse_arguments(
            &items(quote!{#[args(test1, test2="None", args="*", test3="None", kwargs="**")]}));
        assert!(args == vec![Argument::Arg(parse_quote!{test1}, None),
                             Argument::Arg(parse_quote!{test2}, Some("None".to_owned())),
                             Argument::VarArgs(parse_quote!{args}),
                             Argument::Kwarg(parse_quote!{test3}, "None".to_owned()),
                             Argument::KeywordArgs(parse_quote!{kwargs})]);
    }
}
