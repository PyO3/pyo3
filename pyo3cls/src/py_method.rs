// Copyright (c) 2017-present PyO3 Project and Contributors

use syn;
use quote::{Tokens, ToTokens};
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

#[derive(Debug)]
enum FnSpec {
    Args(syn::Ident),
    Kwargs(syn::Ident),
    Default(syn::Ident, Tokens),
}


pub fn gen_py_method<'a>(cls: &Box<syn::Ty>, name: &syn::Ident,
                         sig: &mut syn::MethodSig, _block: &mut syn::Block,
                         meth_attrs: &mut Vec<syn::Attribute>) -> Tokens
{
    check_generic(name, sig);

    let (fn_type, fn_spec) = parse_attributes(meth_attrs);

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
            impl_py_method_def(name, &impl_wrap(cls, name, arguments, fn_spec)),
        FnType::Getter(getter) =>
            impl_py_getter_def(name, getter, &impl_wrap_getter(cls, name, arguments, fn_spec)),
        FnType::Setter(setter) =>
            impl_py_setter_def(name, setter, &impl_wrap_setter(cls, name, arguments, fn_spec)),
    }
}

fn parse_attributes(attrs: &mut Vec<syn::Attribute>) -> (FnType, Vec<FnSpec>) {
    let mut new_attrs = Vec::new();
    let mut spec = Vec::new();
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
                    "args" => {
                        spec.extend(parse_args(meta))
                    }
                    "defaults" => {
                        // parse: #[defaults(param2=12, param3=12)]
                        for item in meta.iter() {
                            if let Some(el) = parse_args_default(item) {
                                spec.push(el)
                            }
                        }
                    }
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
        Some(tp) => (tp, spec),
        None => (FnType::Fn, spec),
    }
}

/// parse: #[args(args="args", kw="kwargs")]
fn parse_args(items: &Vec<syn::NestedMetaItem>) -> Vec<FnSpec> {
    let mut spec = Vec::new();

    for item in items.iter() {
        match item {
            &syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref ident, ref name)) => {
                match *name {
                    syn::Lit::Str(ref name, _) => match ident.as_ref() {
                        "args" =>
                            spec.push(FnSpec::Args(syn::Ident::from(name.clone()))),
                        "kw" =>
                            spec.push(FnSpec::Kwargs(syn::Ident::from(name.clone()))),
                        _ => (),
                    },
                    _ => (),
                }
            },
            _ => (),
        }
    }

    spec
}

fn parse_args_default(item: &syn::NestedMetaItem) -> Option<FnSpec> {
    match *item {
        syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref name, ref lit)) => {
            let mut t = Tokens::new();
            match lit {
                &syn::Lit::Str(ref val, _) => {
                    syn::Ident::from(val.as_str()).to_tokens(&mut t);
                },
                _ => {
                    lit.to_tokens(&mut t);
                }
            }
            Some(FnSpec::Default(name.clone(), t))
        }
        _ => {
            println!("expected name value {:?}", item);
            None
        }
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
fn impl_wrap(cls: &Box<syn::Ty>,
             name: &syn::Ident,
             args: Vec<Arg>, spec: Vec<FnSpec>) -> Tokens {
    let cb = impl_call(cls, name, &args);
    let body = impl_arg_params(args, &spec, cb);

    quote! {
        unsafe extern "C" fn wrap
            (slf: *mut _pyo3::ffi::PyObject,
             args: *mut _pyo3::ffi::PyObject,
             kwargs: *mut _pyo3::ffi::PyObject) -> *mut _pyo3::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(
                stringify!(#cls), ".", stringify!(#name), "()");
            _pyo3::callback::handle_callback(
                LOCATION, _pyo3::callback::PyObjectCallbackConverter, |py|
                {
                    let args: _pyo3::PyTuple =
                        _pyo3::PyObject::from_borrowed_ptr(py, args).unchecked_cast_into();
                    let kwargs: Option<_pyo3::PyDict> = _pyo3::argparse::get_kwargs(py, kwargs);

                    let ret = {
                        #body
                    };
                    _pyo3::PyDrop::release_ref(args, py);
                    _pyo3::PyDrop::release_ref(kwargs, py);
                    ret
                })
        }
    }
}


/// Generate functiona wrapper (PyCFunction, PyCFunctionWithKeywords)
fn impl_wrap_getter(cls: &Box<syn::Ty>,
                    name: &syn::Ident, _args: Vec<Arg>, _spec: Vec<FnSpec>) -> Tokens {
    quote! {
        unsafe extern "C" fn wrap (slf: *mut _pyo3::ffi::PyObject,
                                   _: *mut _pyo3::c_void)
                                   -> *mut _pyo3::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(
                stringify!(#cls), ".getter_", stringify!(#name), "()");
            _pyo3::callback::handle_callback(
                LOCATION, _pyo3::callback::PyObjectCallbackConverter, |py|
                {
                    let slf = _pyo3::PyObject::from_borrowed_ptr(
                        py, slf).unchecked_cast_into::<#cls>();
                    let ret = slf.#name(py);
                    _pyo3::PyDrop::release_ref(slf, py);
                    ret
                })
        }
    }
}

/// Generate functiona wrapper (PyCFunction, PyCFunctionWithKeywords)
fn impl_wrap_setter(cls: &Box<syn::Ty>,
                    name: &syn::Ident, _args: Vec<Arg>, _spec: Vec<FnSpec>) -> Tokens {
    quote! {
        unsafe extern "C" fn wrap(slf: *mut _pyo3::ffi::PyObject,
                                  value: *mut _pyo3::ffi::PyObject,
                                  _: *mut _pyo3::c_void) -> _pyo3::c_int
        {
            const LOCATION: &'static str = concat!(
                stringify!(#cls), ".setter", stringify!(#name), "()");
            _pyo3::callback::handle_callback(
                LOCATION, _pyo3::callback::UnitCallbackConverter, |py|
                {
                    let slf = _pyo3::PyObject::from_borrowed_ptr(py, slf)
                        .unchecked_cast_into::<#cls>();
                    let value = _pyo3::PyObject::from_borrowed_ptr(py, value);
                    let ret = slf.#name(py, &value);
                    _pyo3::PyDrop::release_ref(slf, py);
                    _pyo3::PyDrop::release_ref(value, py);
                    ret.map(|o| ())
                })
        }
    }
}


fn impl_call(cls: &Box<syn::Ty>, fname: &syn::Ident, args: &Vec<Arg>) -> Tokens {
    let names: Vec<&syn::Ident> = args.iter().map(|item| item.name).collect();
    quote! {
        {
            let slf = _pyo3::PyObject::from_borrowed_ptr(py, slf).unchecked_cast_into::<#cls>();
            let ret = slf.#fname(py, #(#names),*);
            _pyo3::PyDrop::release_ref(slf, py);
            ret
        }
    }
}

fn impl_arg_params(mut args: Vec<Arg>, spec: &Vec<FnSpec>, body: Tokens) -> Tokens {
    let mut params = Vec::new();

    for arg in args.iter() {
        if ! (is_args(&arg.name, &spec) || is_kwargs(&arg.name, &spec)) {
            let name = arg.name.as_ref();
            let opt = if let Some(_) = arg.optional {
                syn::Ident::from("true")
            } else {
                if let Some(_) = get_default_value(&arg.name, spec) {
                    syn::Ident::from("true")
                } else {
                    syn::Ident::from("false")
                }
            };
            params.push(
                quote! {
                    _pyo3::argparse::ParamDescription{name: #name, is_optional: #opt,}
                }
            );
        }
    }
    let placeholders: Vec<syn::Ident> = params.iter().map(
        |_| syn::Ident::from("None")).collect();

    // generate extrat args
    args.reverse();
    let mut body = body;
    for arg in args.iter() {
        body = impl_arg_param(&arg, &spec, &body);
    }

    let accept_args = syn::Ident::from(
        if accept_args(spec) { "true" } else { "false" });
    let accept_kwargs = syn::Ident::from(
        if accept_kwargs(spec) { "true" } else { "false" });

    // create array of arguments, and then parse
    quote! {
        const PARAMS: &'static [_pyo3::argparse::ParamDescription<'static>] = &[
            #(#params),*
        ];

        let mut output = [#(#placeholders),*];
        match _pyo3::argparse::parse_args(
            py, Some(LOCATION), PARAMS, &args,
            kwargs.as_ref(), #accept_args, #accept_kwargs, &mut output) {
            Ok(_) => {
                let mut _iter = output.iter();

                #body
            },
            Err(err) => Err(err)
        }
    }
}

fn impl_arg_param(arg: &Arg, spec: &Vec<FnSpec>, body: &Tokens) -> Tokens {
    let ty = arg.ty;
    let name = arg.name;

    // First unwrap() asserts the iterated sequence is long enough (which should be guaranteed);
    // second unwrap() asserts the parameter was not missing (which fn
    // parse_args already checked for).

    if is_args(&name, &spec) {
        quote! {
            match <#ty as _pyo3::FromPyObject>::extract(py, args.as_object())
            {
                Ok(#name) => {
                    #body
                }
                Err(e) => Err(e)
            }
        }
    }
    else if is_kwargs(&name, &spec) {
        quote! {
            let #name = kwargs.as_ref();
            #body
        }
    }
    else {
        if let Some(ref opt_ty) = arg.optional {
            // default value
            let mut default = Tokens::new();
            if let Some(d) = get_default_value(name, spec) {
                let dt = quote!{ Some(#d) };
                dt.to_tokens(&mut default);
            } else {
                syn::Ident::from("None").to_tokens(&mut default);
            }
            
            quote! {
                match match _iter.next().unwrap().as_ref() {
                    Some(obj) => {
                        match <#opt_ty as _pyo3::FromPyObject>::extract(py, obj) {
                            Ok(obj) => Ok(Some(obj)),
                            Err(e) => Err(e),
                        }
                    },
                    None => Ok(#default)
                } {
                    Ok(#name) => #body,
                    Err(e) => Err(e)
                }
            }
        } else if let Some(default) = get_default_value(name, spec) {
            quote! {
                match match _iter.next().unwrap().as_ref() {
                    Some(obj) => {
                        match <#ty as _pyo3::FromPyObject>::extract(py, obj) {
                            Ok(obj) => Ok(obj),
                            Err(e) => Err(e),
                        }
                    },
                    None => Ok(#default)
                } {
                    Ok(#name) => #body,
                    Err(e) => Err(e)
                }
            }
        }
        else {
            quote! {
                match <#ty as _pyo3::FromPyObject>::extract(
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
}

fn is_args(name: &syn::Ident, spec: &Vec<FnSpec>) -> bool {
    for s in spec.iter() {
        match *s {
            FnSpec::Args(ref ident) =>
                return name == ident,
            _ => (),
        }
    }
    false
}

fn accept_args(spec: &Vec<FnSpec>) -> bool {
    for s in spec.iter() {
        match *s {
            FnSpec::Args(_) => return true,
            _ => (),
        }
    }
    false
}

fn is_kwargs(name: &syn::Ident, spec: &Vec<FnSpec>) -> bool {
    for s in spec.iter() {
        match *s {
            FnSpec::Kwargs(ref ident) =>
                return name == ident,
            _ => (),
        }
    }
    false
}

fn accept_kwargs(spec: &Vec<FnSpec>) -> bool {
    for s in spec.iter() {
        match *s {
            FnSpec::Kwargs(_) => return true,
            _ => (),
        }
    }
    false
}

fn get_default_value<'a>(name: &syn::Ident, spec: &'a Vec<FnSpec>) -> Option<&'a Tokens> {
    for s in spec.iter() {
        match *s {
            FnSpec::Default(ref ident, ref val) => {
                if ident == name {
                    return Some(val)
                }
            },
            _ => (),
        }
    }
    None
}

fn impl_py_method_def(name: &syn::Ident, wrapper: &Tokens) -> Tokens {
    quote! {
        _pyo3::class::PyMethodDefType::Method({
            #wrapper

            _pyo3::class::PyMethodDef {
                ml_name: stringify!(#name),
                ml_meth: _pyo3::class::PyMethodType::PyCFunctionWithKeywords(wrap),
                ml_flags: _pyo3::ffi::METH_VARARGS | _pyo3::ffi::METH_KEYWORDS,
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
        _pyo3::class::PyMethodDefType::Setter({
            #wrapper

            _pyo3::class::PySetterDef {
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
        _pyo3::class::PyMethodDefType::Getter({
            #wrapper

            _pyo3::class::PyGetterDef {
                name: #n,
                meth: wrap,
                doc: "",
            }
        })
    }
}
