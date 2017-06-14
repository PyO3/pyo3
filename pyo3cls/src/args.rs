use nom::*;

#[derive(Debug, PartialEq)]
pub enum Argument {
    VarArgsSeparator,
    VarArgs(String),
    KeywordArgs(String),
    Arg(String, Option<String>),
    Kwarg(String, String),
}

pub fn parse_arguments(value: &[u8]) -> Vec<Argument> {
    let mut arguments = Vec::new();
    let mut has_kw = false;
    let mut has_varargs = false;
    let mut has_kwargs = false;
    let args_str = String::from_utf8_lossy(value);

    let mut curr = value;
    while curr.len() > 0 {
        match get_arg(curr) {
            IResult::Done(i, t) => {
                match argument(t) {
                    IResult::Done(rest, arg) => {
                        if rest.len() != 0 {
                            println!("can not parse arguments {:?} : {:?}",
                                     args_str, String::from_utf8_lossy(t),
                            );
                        } else {
                            match arg {
                                Argument::VarArgsSeparator => {
                                    if has_kwargs {
                                        println!("syntax error, keyword arguments is defined: {:?}",
                                                 args_str);
                                        return Vec::new()
                                    }
                                    if has_varargs {
                                        println!("arguments already define * (var args): {:?}",
                                                 args_str);
                                        return Vec::new()
                                    } else {
                                        arguments.push(Argument::VarArgsSeparator);
                                        has_varargs = true;
                                        has_kw = true;
                                    }
                                }
                                Argument::VarArgs(s) => {
                                    if has_kwargs {
                                        println!("syntax error, keyword arguments is defined: {:?}",
                                                 args_str);
                                        return Vec::new()
                                    }
                                    if has_varargs {
                                        println!("*(var args) is defined: {:?}", args_str);
                                        return Vec::new()
                                    } else {
                                        arguments.push(Argument::VarArgs(s));
                                        has_varargs = true;
                                        has_kw = true;
                                    }
                                }
                                Argument::KeywordArgs(s) => {
                                    if has_kwargs {
                                        println!("arguments already define ** (kw args): {:?}",
                                                 args_str);
                                        return Vec::new()
                                    } else {
                                        arguments.push(Argument::KeywordArgs(s));
                                        has_kwargs = true;
                                    }
                                }
                                Argument::Arg(s, opt) => {
                                    if has_kwargs {
                                        println!("syntax error, keyword arguments is defined: {:?}",
                                                 args_str);
                                        return Vec::new()
                                    }
                                    if let Some(opt) = opt {
                                        has_kw = true;
                                        if has_varargs {
                                            arguments.push(Argument::Kwarg(s, opt))
                                        } else {
                                            arguments.push(Argument::Arg(s, Some(opt)))
                                        }
                                    } else {
                                        if has_kw {
                                            println!("syntax error, argument is not allowed after keyword argument: {:?}",
                                                     args_str);
                                            return Vec::new()
                                        }
                                        arguments.push(Argument::Arg(s, None));
                                    }
                                }
                                Argument::Kwarg(_, _) => unreachable!()
                            }
                        }
                    },
                    IResult::Error(_) | IResult::Incomplete(_) => {
                        println!("can not parse arguments {:?} : {:?}",
                                 String::from_utf8_lossy(value), String::from_utf8_lossy(t),
                        );
                    }
                };
                if i.len() > 0 {
                    curr = &i[1..i.len()];
                } else {
                    break
                }
            }
            IResult::Error(_) | IResult::Incomplete(_) => {
                println!("can not parse arguments {:?}", String::from_utf8_lossy(value));
            }
        }
    }

    arguments
}

named!(get_arg, ws!(take_while!(is_not_arg_end)));

fn is_not_arg_end(c: u8) -> bool {
    c as char != ','
}

named!(argument<&[u8], Argument>, alt_complete!(
    parse_arg | parse_optional_arg | parse_kwargs | parse_var_arg | parse_var_arg_sep));

named!(parse_var_arg_sep<&[u8], Argument>,
       do_parse!(
           char!('*') >> eof!() >> ( Argument::VarArgsSeparator )
       )
);
named!(parse_var_arg<&[u8], Argument>,
       do_parse!(
           tag!("*") >>
               take_while!(is_space) >> verify!(peek!(take!(1)), alphabetic) >>
               name: take_while!(is_alphanumeric) >> take_while!(is_space) >> eof!() >> (
                   Argument::VarArgs(String::from_utf8_lossy(name).to_string())
               )
       )
);
named!(parse_kwargs<&[u8], Argument>,
       do_parse!(
           tag!("**") >>
               take_while!(is_space) >> verify!(peek!(take!(1)), alphabetic) >>
               name: take_while!(is_alphanumeric) >> take_while!(is_space) >> eof!() >> (
                   Argument::KeywordArgs(String::from_utf8_lossy(name).to_string())
               )
       )
);

named!(parse_arg<&[u8], Argument>,
       do_parse!(
           take_while!(is_space) >> verify!(peek!(take!(1)), alphabetic) >>
               name: take_while!(is_alphanumeric) >> take_while!(is_space) >> eof!() >> (
                   Argument::Arg(String::from_utf8_lossy(name).to_string(), None)
               )
       )
);

named!(parse_optional_arg<&[u8], Argument>,
       do_parse!(
           take_while!(is_space) >> verify!(peek!(take!(1)), alphabetic) >>
               name: take_while!(is_alphanumeric) >>
               take_while!(is_space) >> tag!("=") >>
               value: take_while!(any) >> (
                   Argument::Arg(
                       String::from_utf8_lossy(name).to_string(),
                       Some(String::from_utf8_lossy(value).to_string())
                   )
               )
       )
);

fn any(_: u8) -> bool { true }

fn alphabetic(b: &[u8]) -> bool {
    is_alphabetic(b[0])
}


#[cfg(test)]
mod test {
    use args::{Argument, parse_arguments};

    #[test]
    fn test_errs() {
        assert!(parse_arguments("123t2113est".as_ref()).is_empty());
        assert!(parse_arguments("test=1, test2".as_ref()).is_empty());
        assert!(parse_arguments("test=1, *, *args".as_ref()).is_empty());
        assert!(parse_arguments("test=1, **kwargs, *args".as_ref()).is_empty());
        assert!(parse_arguments("test=1, **kwargs, args".as_ref()).is_empty());
    }

    #[test]
    fn test_simple_args() {
        let args = parse_arguments("test1, test2, test3=None".as_ref());
        assert!(args == vec![Argument::Arg("test1".to_owned(), None),
                             Argument::Arg("test2".to_owned(), None),
                             Argument::Arg("test3".to_owned(), Some("None".to_owned()))]);
    }

    #[test]
    fn test_varargs() {
        let args = parse_arguments("test1, test2=None, *, test3=None".as_ref());
        assert!(args == vec![Argument::Arg("test1".to_owned(), None),
                             Argument::Arg("test2".to_owned(), Some("None".to_owned())),
                             Argument::VarArgsSeparator,
                             Argument::Kwarg("test3".to_owned(), "None".to_owned())]);
    }

    #[test]
    fn test_all() {
        let args = parse_arguments(
            "test1, test2=None, *args, test3=None, **kwargs".as_ref());
        assert!(args == vec![Argument::Arg("test1".to_owned(), None),
                             Argument::Arg("test2".to_owned(), Some("None".to_owned())),
                             Argument::VarArgs("args".to_owned()),
                             Argument::Kwarg("test3".to_owned(), "None".to_owned()),
                             Argument::KeywordArgs("kwargs".to_owned())]);
    }

}
