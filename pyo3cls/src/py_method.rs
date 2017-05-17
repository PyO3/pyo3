// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::Tokens;
use utils::for_err_msg;


#[derive(Debug)]
struct Arg<'a> {
    pub name: &'a syn::Ident,
    pub mode: &'a syn::BindingMode,
    pub ty: &'a syn::Ty,
    pub optional: Option<&'a syn::Ty>,
}

#[derive(PartialEq, Debug)]
enum FnType {
    Getter(Option<String>),
    Setter(Option<String>),
    Fn,
}


pub fn gen_py_method<'a>(cls: &Box<syn::Ty>, name: &syn::Ident,
                         sig: &mut syn::MethodSig, _block: &mut syn::Block,
                         meth_attrs: &mut Vec<syn::Attribute>) -> Tokens
{
    check_generic(name, sig);

    let fn_type = parse_attributes(meth_attrs);

    //let mut has_self = false;
    let mut py = false;
    let mut arguments: Vec<Arg> = Vec::new();

    for input in sig.decl.inputs.iter() {
        match input {
            &syn::FnArg::SelfRef(_, _) => {
                //has_self = true;
            },
            &syn::FnArg::SelfValue(_) => {
                //has_self = true;
            }
            &syn::FnArg::Captured(ref pat, ref ty) => {
                let (mode, ident) = match pat {
                    &syn::Pat::Ident(ref mode, ref ident, _) =>
                        (mode, ident),
                    _ =>
                        panic!("unsupported argument: {:?}", pat),
                };
                // TODO add check for first py: Python arg
                if py {
                    let opt = check_arg_ty_and_optional(name, ty);
                    arguments.push(Arg{name: ident, mode: mode, ty: ty, optional: opt});
                } else {
                    py = true;
                }
            }
            &syn::FnArg::Ignored(_) =>
                panic!("ignored argument: {:?}", name),
        }
    }

    match fn_type {
        FnType::Fn =>
            impl_py_method_def(name, &impl_wrap(cls, name, arguments)),
        FnType::Getter(getter) =>
            impl_py_getter_def(name, getter, &impl_wrap_getter(cls, name, arguments)),
        FnType::Setter(setter) =>
            impl_py_setter_def(name, setter, &impl_wrap_setter(cls, name, arguments)),
    }
}

fn parse_attributes(attrs: &mut Vec<syn::Attribute>) -> FnType {
    let mut new_attrs = Vec::new();
    let mut res: Option<FnType> = None;

    for attr in attrs.iter() {
        match attr.value {
            syn::MetaItem::Word(ref name) => {
                match name.as_ref() {
                    "setter" | "getter" => {
                        if attr.style == syn::AttrStyle::Inner {
                            panic!("Inner style attribute is not
                                    supported for setter and getter");
                        }
                        if res != None {
                            panic!("setter/getter attribute can not be used mutiple times");
                        }
                        if name.as_ref() == "setter" {
                            res = Some(FnType::Setter(None))
                        } else {
                            res = Some(FnType::Getter(None))
                        }
                    },
                    _ => {
                        new_attrs.push(attr.clone())
                    }
                }
            },
            syn::MetaItem::List(ref name, ref meta) => {
                match name.as_ref() {
                    "setter" | "getter" => {
                        if attr.style == syn::AttrStyle::Inner {
                            panic!("Inner style attribute is not
                                    supported for setter and getter");
                        }
                        if res != None {
                            panic!("setter/getter attribute can not be used mutiple times");
                        }
                        if meta.len() != 1 {
                            panic!("setter/getter requires one value");
                        }
                        match *meta.first().unwrap() {
                            syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref w)) => {
                                if name.as_ref() == "setter" {
                                    res = Some(FnType::Setter(Some(w.to_string())))
                                } else {
                                    res = Some(FnType::Getter(Some(w.to_string())))
                                }
                            },
                            syn::NestedMetaItem::Literal(ref lit) => {
                                match *lit {
                                    syn::Lit::Str(ref s, syn::StrStyle::Cooked) => {
                                        if name.as_ref() == "setter" {
                                            res = Some(FnType::Setter(Some(s.clone())))
                                        } else {
                                            res = Some(FnType::Getter(Some(s.clone())))
                                        }
                                    },
                                    _ => {
                                        panic!("setter/getter attribute requires str value");
                                    },
                                }
                            }
                            _ => {
                                println!("cannot parse {:?} attribute: {:?}", name, meta);
                            },
                        }
                    },
                    _ => {
                        new_attrs.push(attr.clone())
                    }
                }
            },
            syn::MetaItem::NameValue(_, _) => {
                new_attrs.push(attr.clone())
            },
        }
    }
    attrs.clear();
    attrs.extend(new_attrs);

    match res {
        Some(tp) => tp,
        None => FnType::Fn,
    }
}

fn check_generic(name: &syn::Ident, sig: &syn::MethodSig) {
    if !sig.generics.ty_params.is_empty() {
        panic!("python method can not be generic: {:?}", name);
    }
}

fn check_arg_ty_and_optional<'a>(name: &'a syn::Ident, ty: &'a syn::Ty) -> Option<&'a syn::Ty> {
    match ty {
        &syn::Ty::Path(ref qs, ref path) => {
            if let &Some(ref qs) = qs {
                panic!("explicit Self type in a 'qualified path' is not supported: {:?} - {:?}",
                       name, qs);
            }

            if let Some(segment) = path.segments.last() {
                match segment.ident.as_ref() {
                    "Option" => {
                        match segment.parameters {
                            syn::PathParameters::AngleBracketed(ref params) => {
                                if params.types.len() != 1 {
                                    panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                           for_err_msg(name),
                                           for_err_msg(ty),
                                           for_err_msg(path));
                                }
                                Some(&params.types[0])
                            },
                            _ => {
                                panic!("argument type is not supported by python method: {:?} ({:?}) {:?}",
                                       for_err_msg(name),
                                       for_err_msg(ty),
                                       for_err_msg(path));
                            }
                        }
                    },
                    _ => None,
                }
            } else {
                None
            }
        },
        _ => {
            None
            //panic!("argument type is not supported by python method: {:?} ({:?})",
            //for_err_msg(name),
            //for_err_msg(ty));
        },
    }
}

/// Generate functiona wrapper (PyCFunction, PyCFunctionWithKeywords)
fn impl_wrap(cls: &Box<syn::Ty>, name: &syn::Ident, args: Vec<Arg>) -> Tokens {
    let cb = impl_call(cls, name, &args);
    let body = impl_arg_params(args, cb);

    quote! {
        unsafe extern "C" fn wrap
            (slf: *mut pyo3::ffi::PyObject,
             args: *mut pyo3::ffi::PyObject,
             kwargs: *mut pyo3::ffi::PyObject) -> *mut pyo3::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(
                stringify!(#cls), ".", stringify!(#name), "()");
            pyo3::callback::handle_callback(
                LOCATION, pyo3::callback::PyObjectCallbackConverter, |py|
                {
                    let args: pyo3::PyTuple =
                        pyo3::PyObject::from_borrowed_ptr(py, args).unchecked_cast_into();
                    let kwargs: Option<pyo3::PyDict> = pyo3::argparse::get_kwargs(py, kwargs);

                    let ret = {
                        #body
                    };
                    pyo3::PyDrop::release_ref(args, py);
                    pyo3::PyDrop::release_ref(kwargs, py);
                    ret
                })
        }
    }
}


/// Generate functiona wrapper (PyCFunction, PyCFunctionWithKeywords)
fn impl_wrap_getter(cls: &Box<syn::Ty>, name: &syn::Ident, _args: Vec<Arg>) -> Tokens {
    quote! {
        unsafe extern "C" fn wrap (slf: *mut pyo3::ffi::PyObject,
                                   _: *mut pyo3::c_void)
                                   -> *mut pyo3::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(
                stringify!(#cls), ".getter_", stringify!(#name), "()");
            pyo3::callback::handle_callback(
                LOCATION, pyo3::callback::PyObjectCallbackConverter, |py|
                {
                    let slf = pyo3::PyObject::from_borrowed_ptr(
                        py, slf).unchecked_cast_into::<#cls>();
                    let ret = slf.#name(py);
                    pyo3::PyDrop::release_ref(slf, py);
                    ret
                })
        }
    }
}

/// Generate functiona wrapper (PyCFunction, PyCFunctionWithKeywords)
fn impl_wrap_setter(cls: &Box<syn::Ty>, name: &syn::Ident, _args: Vec<Arg>) -> Tokens {
    quote! {
        unsafe extern "C" fn wrap(slf: *mut pyo3::ffi::PyObject,
                                  value: *mut pyo3::ffi::PyObject,
                                  _: *mut pyo3::c_void) -> pyo3::c_int
        {
            const LOCATION: &'static str = concat!(
                stringify!(#cls), ".setter", stringify!(#name), "()");
            pyo3::callback::handle_callback(
                LOCATION, pyo3::callback::UnitCallbackConverter, |py|
                {
                    let slf = pyo3::PyObject::from_borrowed_ptr(py, slf)
                        .unchecked_cast_into::<#cls>();
                    let value = pyo3::PyObject::from_borrowed_ptr(py, value);
                    let ret = slf.#name(py, &value);
                    pyo3::PyDrop::release_ref(slf, py);
                    pyo3::PyDrop::release_ref(value, py);
                    ret.map(|o| ())
                })
        }
    }
}


fn impl_call(cls: &Box<syn::Ty>, fname: &syn::Ident, args: &Vec<Arg>) -> Tokens {
    let names: Vec<&syn::Ident> = args.iter().map(|item| item.name).collect();
    quote! {
        {
            let slf = pyo3::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<#cls>();
            let ret = slf.#fname(py, #(#names),*);
            pyo3::PyDrop::release_ref(slf, py);
            ret
        }
    }
}

fn impl_arg_params(mut args: Vec<Arg>, body: Tokens) -> Tokens {
    let mut params = Vec::new();

    for arg in args.iter() {
        let name = arg.name.as_ref();
        let opt = if let Some(_) = arg.optional {
            syn::Ident::from("true")
        } else {
            syn::Ident::from("false")
        };
        params.push(
            quote! {
                pyo3::argparse::ParamDescription{name: #name, is_optional: #opt,}
            }
        );
    }
    let placeholders: Vec<syn::Ident> = params.iter().map(
        |_| syn::Ident::from("None")).collect();

    // generate extrat args
    args.reverse();
    let mut body = body;
    for arg in args.iter() {
        body = impl_arg_param(&arg, &body);
    }

    // create array of arguments, and then parse
    quote! {
        const PARAMS: &'static [pyo3::argparse::ParamDescription<'static>] = &[
            #(#params),*
        ];

        let mut output = [#(#placeholders),*];
        match pyo3::argparse::parse_args(
            py, Some(LOCATION), PARAMS, &args, kwargs.as_ref(), &mut output) {
            Ok(_) => {
                let mut _iter = output.iter();

                #body
            },
            Err(err) => Err(err)
        }
    }
}

fn impl_arg_param(arg: &Arg, body: &Tokens) -> Tokens {
    let ty = arg.ty;
    let name = arg.name;

    // First unwrap() asserts the iterated sequence is long enough (which should be guaranteed);
    // second unwrap() asserts the parameter was not missing (which fn
    // parse_args already checked for).

    if let Some(ref opt_ty) = arg.optional {
        quote! {
            match match _iter.next().unwrap().as_ref() {
                Some(obj) => {
                    match <#opt_ty as pyo3::FromPyObject>::extract(py, obj) {
                        Ok(obj) => Ok(Some(obj)),
                        Err(e) => Err(e),
                    }
                },
                None => Ok(None)
            } {
                Ok(#name) => #body,
                Err(e) => Err(e)
            }
        }
    } else {
        quote! {
            match <#ty as pyo3::FromPyObject>::extract(
                py, _iter.next().unwrap().as_ref().unwrap())
            {
                Ok(#name) => {
                    #body
                }
                Err(e) => Err(e)
            }
        }
    }
}

fn impl_py_method_def(name: &syn::Ident, wrapper: &Tokens) -> Tokens {
    quote! {
        pyo3::class::PyMethodDefType::Method({
            #wrapper

            pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: pyo3::class::PyMethodType::PyCFunctionWithKeywords(wrap),
                ml_flags: pyo3::ffi::METH_VARARGS | pyo3::ffi::METH_KEYWORDS,
                ml_doc: "",
            }
        })
    }
}

fn impl_py_setter_def(name: &syn::Ident, setter: Option<String>, wrapper: &Tokens) -> Tokens {
    let n = if let Some(ref name) = setter {
        name.to_string()
    } else {
        let n = String::from(name.as_ref());
        if n.starts_with("set_") {
            n[4..].to_string()
        } else {
            n
        }
    };

    quote! {
        pyo3::class::PyMethodDefType::Setter({
            #wrapper

            pyo3::class::PySetterDef {
                name: #n,
                meth: wrap,
                doc: "",
            }
        })
    }
}

fn impl_py_getter_def(name: &syn::Ident, getter: Option<String>, wrapper: &Tokens) -> Tokens {
    let n = if let Some(ref name) = getter {
        name.to_string()
    } else {
        let n = String::from(name.as_ref());
        if n.starts_with("get_") {
            n[4..].to_string()
        } else {
            n
        }
    };

    quote! {
        pyo3::class::PyMethodDefType::Getter({
            #wrapper

            pyo3::class::PyGetterDef {
                name: #n,
                meth: wrap,
                doc: "",
            }
        })
    }
}
