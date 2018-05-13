// Copyright (c) 2017-present PyO3 Project and Contributors
use syn;

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    VarArgsSeparator,
    VarArgs(String),
    KeywordArgs(String),
    Arg(String, Option<String>),
    Kwarg(String, String),
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
            &syn::NestedMeta::Meta(syn::Meta::Word(ref ident)) => {
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
                arguments.push(Argument::Arg(ident.as_ref().to_owned(), None))
            }
            &syn::NestedMeta::Meta(syn::Meta::NameValue(ref nv)) => {
                let name = nv.ident.as_ref().to_owned();
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
                            arguments.push(Argument::VarArgs(name));
                        } else if litstr.value() == "**" {  // #[args(kwargs="**")]
                            if has_kwargs {
                                println!("arguments already define ** (kw args): {:?}",
                                         args_str);
                                return Vec::new()
                            }
                            has_kwargs = true;
                            arguments.push(Argument::KeywordArgs(name));
                        } else {
                            if has_varargs {
                                arguments.push(Argument::Kwarg(name, litstr.value().clone()))
                            } else {
                                if has_kwargs {
                                    println!("syntax error, keyword arguments is defined: {:?}",
                                             args_str);
                                    return Vec::new()
                                }
                                has_kw = true;
                                arguments.push(Argument::Arg(name, Some(litstr.value().clone())))
                            }
                        }
                    }
                    syn::Lit::Int(ref litint) => {
                        if has_varargs {
                            arguments.push(Argument::Kwarg(name, format!("{}", litint.value())));
                        } else {
                            if has_kwargs {
                                println!("syntax error, keyword arguments is defined: {:?}",
                                         args_str);
                                return Vec::new()
                            }
                            has_kw = true;
                            arguments.push(Argument::Arg(name, Some(format!("{}", litint.value()))));
                        }
                    }
                    syn::Lit::Bool(ref litb) => {
                        if has_varargs {
                            arguments.push(Argument::Kwarg(name, format!("{}", litb.value)));
                        } else {
                            if has_kwargs {
                                println!("syntax error, keyword arguments is defined: {:?}",
                                         args_str);
                                return Vec::new()
                            }
                            has_kw = true;
                            arguments.push(Argument::Arg(name, Some(format!("{}", litb.value))));
                        }
                    }
                    _ => {
                        println!("Only string literal is supported, got: {:?}", nv.lit);
                        return Vec::new()
                    }
                }
            }
            &syn::NestedMeta::Literal(ref lit) => {
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
    use quote::ToTokens;
    use syn::buffer::TokenBuffer;
    use proc_macro::TokenStream;

    fn items(s: &'static str) -> Vec<syn::NestedMeta> {

        let stream = s.parse::<TokenStream>().unwrap();
        let buffer = TokenBuffer::new(stream);
        let i = syn::Attribute::parse_outer(buffer.begin()).unwrap().0;

        match i.interpret_meta() {
            Some(syn::Meta::List(syn::MetaList { nested, .. })) => {
                nested.iter().map(Clone::clone).collect()
            }
            _ => unreachable!()
        }
    }

    #[test]
    fn test_errs() {
        assert!(parse_arguments(&items("#[args(test=\"1\", test2)]")).is_empty());
        assert!(parse_arguments(&items("#[args(test=1, \"*\", args=\"*\")]")).is_empty());
        assert!(parse_arguments(&items("#[args(test=1, kwargs=\"**\", args=\"*\")]")).is_empty());
        assert!(parse_arguments(&items("#[args(test=1, kwargs=\"**\", args)]")).is_empty());
    }

    #[test]
    fn test_simple_args() {
        let args = parse_arguments(&items("#[args(test1, test2, test3=\"None\")]"));
        assert!(args == vec![Argument::Arg("test1".to_owned(), None),
                             Argument::Arg("test2".to_owned(), None),
                             Argument::Arg("test3".to_owned(), Some("None".to_owned()))]);
    }

    #[test]
    fn test_varargs() {
        let args = parse_arguments(
            &items("#[args(test1, test2=\"None\", \"*\", test3=\"None\")]"));
        assert!(args == vec![Argument::Arg("test1".to_owned(), None),
                             Argument::Arg("test2".to_owned(), Some("None".to_owned())),
                             Argument::VarArgsSeparator,
                             Argument::Kwarg("test3".to_owned(), "None".to_owned())]);
    }

    #[test]
    fn test_all() {
        let args = parse_arguments(
            &items("#[args(test1, test2=\"None\", args=\"*\", test3=\"None\", kwargs=\"**\")]"));
        assert!(args == vec![Argument::Arg("test1".to_owned(), None),
                             Argument::Arg("test2".to_owned(), Some("None".to_owned())),
                             Argument::VarArgs("args".to_owned()),
                             Argument::Kwarg("test3".to_owned(), "None".to_owned()),
                             Argument::KeywordArgs("kwargs".to_owned())]);
    }
}
