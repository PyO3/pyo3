// Copyright (c) 2017-present PyO3 Project and Contributors

use std::borrow::Cow;

use crate::attributes::NameAttribute;
use crate::utils::ensure_not_async_fn;
use crate::{deprecations::Deprecations, utils};
use crate::{
    method::{FnArg, FnSpec, FnType, SelfType},
    pyfunction::PyFunctionOptions,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{ext::IdentExt, spanned::Spanned, Result};

pub enum GeneratedPyMethod {
    Method(TokenStream),
    New(TokenStream),
    Call(TokenStream),
}

pub fn gen_py_method(
    cls: &syn::Type,
    sig: &mut syn::Signature,
    meth_attrs: &mut Vec<syn::Attribute>,
    options: PyFunctionOptions,
) -> Result<GeneratedPyMethod> {
    check_generic(sig)?;
    ensure_not_async_fn(sig)?;
    ensure_function_options_valid(&options)?;
    let spec = FnSpec::parse(sig, &mut *meth_attrs, options)?;

    Ok(match &spec.tp {
        // ordinary functions (with some specialties)
        FnType::Fn(_) => GeneratedPyMethod::Method(impl_py_method_def(cls, &spec, None)?),
        FnType::FnClass => GeneratedPyMethod::Method(impl_py_method_def(
            cls,
            &spec,
            Some(quote!(pyo3::ffi::METH_CLASS)),
        )?),
        FnType::FnStatic => GeneratedPyMethod::Method(impl_py_method_def(
            cls,
            &spec,
            Some(quote!(pyo3::ffi::METH_STATIC)),
        )?),
        // special prototypes
        FnType::FnNew => GeneratedPyMethod::New(impl_py_method_def_new(cls, &spec)?),
        FnType::FnCall(_) => GeneratedPyMethod::Call(impl_py_method_def_call(cls, &spec)?),
        FnType::ClassAttribute => GeneratedPyMethod::Method(impl_py_class_attribute(cls, &spec)),
        FnType::Getter(self_type) => GeneratedPyMethod::Method(impl_py_getter_def(
            cls,
            PropertyType::Function {
                self_type,
                spec: &spec,
            },
        )?),
        FnType::Setter(self_type) => GeneratedPyMethod::Method(impl_py_setter_def(
            cls,
            PropertyType::Function {
                self_type,
                spec: &spec,
            },
        )?),
        FnType::FnModule => {
            unreachable!("methods cannot be FnModule")
        }
    })
}

pub fn check_generic(sig: &syn::Signature) -> syn::Result<()> {
    let err_msg = |typ| format!("Python functions cannot have generic {} parameters", typ);
    for param in &sig.generics.params {
        match param {
            syn::GenericParam::Lifetime(_) => {}
            syn::GenericParam::Type(_) => bail_spanned!(param.span() => err_msg("type")),
            syn::GenericParam::Const(_) => bail_spanned!(param.span() => err_msg("const")),
        }
    }
    Ok(())
}

fn ensure_function_options_valid(options: &PyFunctionOptions) -> syn::Result<()> {
    if let Some(pass_module) = &options.pass_module {
        bail_spanned!(pass_module.span() => "`pass_module` cannot be used on Python methods")
    }
    Ok(())
}

/// Also used by pyfunction.
pub fn impl_py_method_def(
    cls: &syn::Type,
    spec: &FnSpec,
    flags: Option<TokenStream>,
) -> Result<TokenStream> {
    let wrapper_ident = syn::Ident::new("__wrap", Span::call_site());
    let wrapper_def = spec.get_wrapper_function(&wrapper_ident, Some(cls))?;
    let add_flags = flags.map(|flags| quote!(.flags(#flags)));
    let methoddef_type = match spec.tp {
        FnType::FnStatic => quote!(Static),
        FnType::FnClass => quote!(Class),
        _ => quote!(Method),
    };
    let methoddef = spec.get_methoddef(quote! {{ #wrapper_def #wrapper_ident }});
    Ok(quote! {
        pyo3::class::PyMethodDefType::#methoddef_type(#methoddef #add_flags)
    })
}

fn impl_py_method_def_new(cls: &syn::Type, spec: &FnSpec) -> Result<TokenStream> {
    let wrapper_ident = syn::Ident::new("__wrap", Span::call_site());
    let wrapper = spec.get_wrapper_function(&wrapper_ident, Some(cls))?;
    Ok(quote! {
        impl pyo3::class::impl_::PyClassNewImpl<#cls> for pyo3::class::impl_::PyClassImplCollector<#cls> {
            fn new_impl(self) -> Option<pyo3::ffi::newfunc> {
                Some({
                    #wrapper
                    #wrapper_ident
                })
            }
        }
    })
}

fn impl_py_method_def_call(cls: &syn::Type, spec: &FnSpec) -> Result<TokenStream> {
    let wrapper_ident = syn::Ident::new("__wrap", Span::call_site());
    let wrapper = spec.get_wrapper_function(&wrapper_ident, Some(cls))?;
    Ok(quote! {
        impl pyo3::class::impl_::PyClassCallImpl<#cls> for pyo3::class::impl_::PyClassImplCollector<#cls> {
            fn call_impl(self) -> Option<pyo3::ffi::PyCFunctionWithKeywords> {
                Some({
                    #wrapper
                    #wrapper_ident
                })
            }
        }
    })
}

fn impl_py_class_attribute(cls: &syn::Type, spec: &FnSpec) -> TokenStream {
    let name = &spec.name;
    let deprecations = &spec.deprecations;
    let python_name = spec.null_terminated_python_name();
    quote! {
        pyo3::class::PyMethodDefType::ClassAttribute({
            pyo3::class::PyClassAttributeDef::new(
                #python_name,
                pyo3::class::methods::PyClassAttributeFactory({
                    fn __wrap(py: pyo3::Python<'_>) -> pyo3::PyObject {
                        #deprecations
                        pyo3::IntoPy::into_py(#cls::#name(), py)
                    }
                    __wrap
                })
            )
        })
    }
}

fn impl_call_setter(cls: &syn::Type, spec: &FnSpec) -> syn::Result<TokenStream> {
    let (py_arg, args) = split_off_python_arg(&spec.args);

    if args.is_empty() {
        bail_spanned!(spec.name.span() => "setter function expected to have one argument");
    } else if args.len() > 1 {
        bail_spanned!(
            args[1].ty.span() =>
            "setter function can have at most two arguments ([pyo3::Python,] and value)"
        );
    }

    let name = &spec.name;
    let fncall = if py_arg.is_some() {
        quote!(#cls::#name(_slf, _py, _val))
    } else {
        quote!(#cls::#name(_slf, _val))
    };

    Ok(fncall)
}

// Used here for PropertyType::Function, used in pyclass for descriptors.
pub fn impl_py_setter_def(cls: &syn::Type, property_type: PropertyType) -> Result<TokenStream> {
    let python_name = property_type.null_terminated_python_name()?;
    let deprecations = property_type.deprecations();
    let doc = property_type.doc();
    let setter_impl = match property_type {
        PropertyType::Descriptor {
            field: syn::Field {
                ident: Some(ident), ..
            },
            ..
        } => {
            // named struct field
            quote!({ _slf.#ident = _val; })
        }
        PropertyType::Descriptor { field_index, .. } => {
            // tuple struct field
            let index = syn::Index::from(field_index);
            quote!({ _slf.#index = _val; })
        }
        PropertyType::Function { spec, .. } => impl_call_setter(cls, spec)?,
    };

    let slf = match property_type {
        PropertyType::Descriptor { .. } => SelfType::Receiver { mutable: true }.receiver(cls),
        PropertyType::Function { self_type, .. } => self_type.receiver(cls),
    };
    Ok(quote! {
        pyo3::class::PyMethodDefType::Setter({
            #deprecations
            pyo3::class::PySetterDef::new(
                #python_name,
                pyo3::class::methods::PySetter({
                    unsafe extern "C" fn __wrap(
                        _slf: *mut pyo3::ffi::PyObject,
                        _value: *mut pyo3::ffi::PyObject,
                        _: *mut std::os::raw::c_void
                    ) -> std::os::raw::c_int {
                        pyo3::callback::handle_panic(|_py| {
                            #slf
                            let _value = _py.from_borrowed_ptr::<pyo3::types::PyAny>(_value);
                            let _val = pyo3::FromPyObject::extract(_value)?;

                            pyo3::callback::convert(_py, #setter_impl)
                        })
                    }
                    __wrap
                }),
                #doc
            )
        })
    })
}

fn impl_call_getter(cls: &syn::Type, spec: &FnSpec) -> syn::Result<TokenStream> {
    let (py_arg, args) = split_off_python_arg(&spec.args);
    ensure_spanned!(
        args.is_empty(),
        args[0].ty.span() => "getter function can only have one argument (of type pyo3::Python)"
    );

    let name = &spec.name;
    let fncall = if py_arg.is_some() {
        quote!(#cls::#name(_slf, _py))
    } else {
        quote!(#cls::#name(_slf))
    };

    Ok(fncall)
}

// Used here for PropertyType::Function, used in pyclass for descriptors.
pub fn impl_py_getter_def(cls: &syn::Type, property_type: PropertyType) -> Result<TokenStream> {
    let python_name = property_type.null_terminated_python_name()?;
    let deprecations = property_type.deprecations();
    let doc = property_type.doc();
    let getter_impl = match property_type {
        PropertyType::Descriptor {
            field: syn::Field {
                ident: Some(ident), ..
            },
            ..
        } => {
            // named struct field
            quote!(_slf.#ident.clone())
        }
        PropertyType::Descriptor { field_index, .. } => {
            // tuple struct field
            let index = syn::Index::from(field_index);
            quote!(_slf.#index.clone())
        }
        PropertyType::Function { spec, .. } => impl_call_getter(cls, spec)?,
    };

    let slf = match property_type {
        PropertyType::Descriptor { .. } => SelfType::Receiver { mutable: false }.receiver(cls),
        PropertyType::Function { self_type, .. } => self_type.receiver(cls),
    };
    Ok(quote! {
        pyo3::class::PyMethodDefType::Getter({
            #deprecations
            pyo3::class::PyGetterDef::new(
                #python_name,
                pyo3::class::methods::PyGetter({
                    unsafe extern "C" fn __wrap(
                        _slf: *mut pyo3::ffi::PyObject,
                        _: *mut std::os::raw::c_void
                    ) -> *mut pyo3::ffi::PyObject {
                        pyo3::callback::handle_panic(|_py| {
                            #slf
                            pyo3::callback::convert(_py, #getter_impl)
                        })
                    }
                    __wrap
                }),
                #doc
            )
        })
    })
}

/// Split an argument of pyo3::Python from the front of the arg list, if present
fn split_off_python_arg<'a>(args: &'a [FnArg<'a>]) -> (Option<&FnArg>, &[FnArg]) {
    if args
        .get(0)
        .map(|py| utils::is_python(py.ty))
        .unwrap_or(false)
    {
        (Some(&args[0]), &args[1..])
    } else {
        (None, args)
    }
}

pub enum PropertyType<'a> {
    Descriptor {
        field_index: usize,
        field: &'a syn::Field,
        python_name: Option<&'a NameAttribute>,
    },
    Function {
        self_type: &'a SelfType,
        spec: &'a FnSpec<'a>,
    },
}

impl PropertyType<'_> {
    fn null_terminated_python_name(&self) -> Result<syn::LitStr> {
        match self {
            PropertyType::Descriptor {
                field, python_name, ..
            } => {
                let name = match (python_name, &field.ident) {
                    (Some(name), _) => name.0.to_string(),
                    (None, Some(field_name)) => format!("{}\0", field_name.unraw()),
                    (None, None) => {
                        bail_spanned!(field.span() => "`get` and `set` with tuple struct fields require `name`")
                    }
                };
                Ok(syn::LitStr::new(&name, field.span()))
            }
            PropertyType::Function { spec, .. } => Ok(spec.null_terminated_python_name()),
        }
    }

    fn deprecations(&self) -> Option<&Deprecations> {
        match self {
            PropertyType::Descriptor { .. } => None,
            PropertyType::Function { spec, .. } => Some(&spec.deprecations),
        }
    }

    fn doc(&self) -> Cow<syn::LitStr> {
        match self {
            PropertyType::Descriptor { field, .. } => {
                let doc = utils::get_doc(&field.attrs, None)
                    .unwrap_or_else(|_| syn::LitStr::new("", Span::call_site()));
                Cow::Owned(doc)
            }
            PropertyType::Function { spec, .. } => Cow::Borrowed(&spec.doc),
        }
    }
}
