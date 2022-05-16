// Copyright (c) 2017-present PyO3 Project and Contributors

use std::borrow::Cow;

use crate::attributes::NameAttribute;
use crate::method::{CallingConvention, ExtractErrorMode};
use crate::utils::{
    ensure_not_async_fn, remove_lifetime, replace_self, unwrap_ty_group, PythonDoc,
};
use crate::{deprecations::Deprecations, utils};
use crate::{
    method::{FnArg, FnSpec, FnType, SelfType},
    pyfunction::PyFunctionOptions,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::Ident;
use syn::{ext::IdentExt, spanned::Spanned, Result};
use crate::stubs::map_rust_type_to_python;

pub enum GeneratedPyMethod {
    Method(TokenStream),
    Proto(TokenStream),
    SlotTraitImpl(String, TokenStream),
}

pub struct PyMethod<'a> {
    kind: PyMethodKind,
    method_name: String,
    spec: FnSpec<'a>,
}

enum PyMethodKind {
    Fn,
    Proto(PyMethodProtoKind),
}

impl PyMethodKind {
    fn from_name(name: &str) -> Self {
        match name {
            // Protocol implemented through slots
            "__str__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__STR__)),
            "__repr__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__REPR__)),
            "__hash__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__HASH__)),
            "__richcmp__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__RICHCMP__)),
            "__get__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__GET__)),
            "__iter__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__ITER__)),
            "__next__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__NEXT__)),
            "__await__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__AWAIT__)),
            "__aiter__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__AITER__)),
            "__anext__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__ANEXT__)),
            "__len__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__LEN__)),
            "__contains__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__CONTAINS__)),
            "__concat__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__CONCAT__)),
            "__repeat__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__REPEAT__)),
            "__inplace_concat__" => {
                PyMethodKind::Proto(PyMethodProtoKind::Slot(&__INPLACE_CONCAT__))
            }
            "__inplace_repeat__" => {
                PyMethodKind::Proto(PyMethodProtoKind::Slot(&__INPLACE_REPEAT__))
            }
            "__getitem__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__GETITEM__)),
            "__pos__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__POS__)),
            "__neg__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__NEG__)),
            "__abs__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__ABS__)),
            "__invert__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__INVERT__)),
            "__index__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__INDEX__)),
            "__int__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__INT__)),
            "__float__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__FLOAT__)),
            "__bool__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__BOOL__)),
            "__iadd__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__IADD__)),
            "__isub__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__ISUB__)),
            "__imul__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__IMUL__)),
            "__imatmul__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__IMATMUL__)),
            "__itruediv__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__ITRUEDIV__)),
            "__ifloordiv__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__IFLOORDIV__)),
            "__imod__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__IMOD__)),
            "__ipow__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__IPOW__)),
            "__ilshift__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__ILSHIFT__)),
            "__irshift__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__IRSHIFT__)),
            "__iand__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__IAND__)),
            "__ixor__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__IXOR__)),
            "__ior__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__IOR__)),
            "__getbuffer__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__GETBUFFER__)),
            "__releasebuffer__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__RELEASEBUFFER__)),
            "__clear__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__CLEAR__)),
            // Protocols implemented through traits
            "__getattribute__" => {
                PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__GETATTRIBUTE__))
            }
            "__getattr__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__GETATTR__)),
            "__setattr__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__SETATTR__)),
            "__delattr__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__DELATTR__)),
            "__set__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__SET__)),
            "__delete__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__DELETE__)),
            "__setitem__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__SETITEM__)),
            "__delitem__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__DELITEM__)),
            "__add__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__ADD__)),
            "__radd__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RADD__)),
            "__sub__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__SUB__)),
            "__rsub__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RSUB__)),
            "__mul__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__MUL__)),
            "__rmul__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RMUL__)),
            "__matmul__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__MATMUL__)),
            "__rmatmul__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RMATMUL__)),
            "__floordiv__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__FLOORDIV__)),
            "__rfloordiv__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RFLOORDIV__)),
            "__truediv__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__TRUEDIV__)),
            "__rtruediv__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RTRUEDIV__)),
            "__divmod__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__DIVMOD__)),
            "__rdivmod__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RDIVMOD__)),
            "__mod__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__MOD__)),
            "__rmod__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RMOD__)),
            "__lshift__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__LSHIFT__)),
            "__rlshift__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RLSHIFT__)),
            "__rshift__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RSHIFT__)),
            "__rrshift__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RRSHIFT__)),
            "__and__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__AND__)),
            "__rand__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RAND__)),
            "__xor__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__XOR__)),
            "__rxor__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RXOR__)),
            "__or__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__OR__)),
            "__ror__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__ROR__)),
            "__pow__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__POW__)),
            "__rpow__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__RPOW__)),
            // Some tricky protocols which don't fit the pattern of the rest
            "__call__" => PyMethodKind::Proto(PyMethodProtoKind::Call),
            "__traverse__" => PyMethodKind::Proto(PyMethodProtoKind::Traverse),
            // Not a proto
            _ => PyMethodKind::Fn,
        }
    }
}

enum PyMethodProtoKind {
    Slot(&'static SlotDef),
    Call,
    Traverse,
    SlotFragment(&'static SlotFragmentDef),
}

impl<'a> PyMethod<'a> {
    fn parse(
        sig: &'a mut syn::Signature,
        meth_attrs: &mut Vec<syn::Attribute>,
        options: PyFunctionOptions,
    ) -> Result<Self> {
        let spec = FnSpec::parse(sig, meth_attrs, options)?;

        let method_name = spec.python_name.to_string();
        let kind = PyMethodKind::from_name(&method_name);

        Ok(Self {
            kind,
            method_name,
            spec,
        })
    }
}

pub fn is_proto_method(name: &str) -> bool {
    match PyMethodKind::from_name(name) {
        PyMethodKind::Fn => false,
        PyMethodKind::Proto(_) => true,
    }
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
    let method = PyMethod::parse(sig, &mut *meth_attrs, options)?;
    let spec = &method.spec;

    generate_stub_method(cls, spec);

    Ok(match (method.kind, &spec.tp) {
        // Class attributes go before protos so that class attributes can be used to set proto
        // method to None.
        (_, FnType::ClassAttribute) => {
            GeneratedPyMethod::Method(impl_py_class_attribute(cls, spec))
        }
        (PyMethodKind::Proto(proto_kind), _) => {
            ensure_no_forbidden_protocol_attributes(spec, &method.method_name)?;
            match proto_kind {
                PyMethodProtoKind::Slot(slot_def) => {
                    let slot = slot_def.generate_type_slot(cls, spec, &method.method_name)?;
                    GeneratedPyMethod::Proto(slot)
                }
                PyMethodProtoKind::Call => {
                    GeneratedPyMethod::Proto(impl_call_slot(cls, method.spec)?)
                }
                PyMethodProtoKind::Traverse => {
                    GeneratedPyMethod::Proto(impl_traverse_slot(cls, method.spec))
                }
                PyMethodProtoKind::SlotFragment(slot_fragment_def) => {
                    let proto = slot_fragment_def.generate_pyproto_fragment(cls, spec)?;
                    GeneratedPyMethod::SlotTraitImpl(method.method_name, proto)
                }
            }
        }
        // ordinary functions (with some specialties)
        (_, FnType::Fn(_)) => GeneratedPyMethod::Method(impl_py_method_def(cls, spec, None)?),
        (_, FnType::FnClass) => GeneratedPyMethod::Method(impl_py_method_def(
            cls,
            spec,
            Some(quote!(_pyo3::ffi::METH_CLASS)),
        )?),
        (_, FnType::FnStatic) => GeneratedPyMethod::Method(impl_py_method_def(
            cls,
            spec,
            Some(quote!(_pyo3::ffi::METH_STATIC)),
        )?),
        // special prototypes
        (_, FnType::FnNew) => GeneratedPyMethod::Proto(impl_py_method_def_new(cls, spec)?),

        (_, FnType::Getter(self_type)) => GeneratedPyMethod::Method(impl_py_getter_def(
            cls,
            PropertyType::Function { self_type, spec },
        )?),
        (_, FnType::Setter(self_type)) => GeneratedPyMethod::Method(impl_py_setter_def(
            cls,
            PropertyType::Function { self_type, spec },
        )?),
        (_, FnType::FnModule) => {
            unreachable!("methods cannot be FnModule")
        }
    })
}

#[cfg_attr(not(feature = "generate-stubs"), allow(unused_variables))]
fn generate_stub_method(cls: &syn::Type, method: &FnSpec) {
    #[cfg(feature = "generate-stubs")] {
        println!("# ID: {}", cls.to_token_stream());

        match &method.tp {
            FnType::FnStatic => println!("\t@staticmethod"),
            FnType::FnClass => println!("\t@classmethod"),
            FnType::Getter(_) => println!("\t@property"),
            FnType::Setter(_) => println!("\t@{}.setter", method.python_name),
            _ => {},
        }

        print!("\tdef {}(", method.python_name);

        let mut comma_required = false;
        match &method.tp {
            FnType::Fn(_) | FnType::Getter(_) | FnType::Setter(_) => {
                comma_required = true;
                print!("self")
            },
            FnType::FnClass | FnType::FnNew => {
                comma_required = true;
                print!("cls")
            },
            _ => {}
        }

        for arg in &method.args {
            if comma_required {
                print!(", ");
            }

            print!("{}: {}", arg.name, map_rust_type_to_python(&arg.ty));

            if arg.optional.is_some() {
                print!(" = ...");
            }

            comma_required = true;
        }
        println!(") -> {}: ...", map_rust_type_to_python(&method.output));

    }
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
        bail_spanned!(pass_module.span() => "`pass_module` cannot be used on Python methods");
    }
    Ok(())
}

fn ensure_no_forbidden_protocol_attributes(
    spec: &FnSpec<'_>,
    method_name: &str,
) -> syn::Result<()> {
    if let Some(text_signature) = &spec.text_signature {
        bail_spanned!(text_signature.kw.span() => format!("`text_signature` cannot be used with `{}`", method_name));
    }
    Ok(())
}

/// Also used by pyfunction.
pub fn impl_py_method_def(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
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
        _pyo3::class::PyMethodDefType::#methoddef_type(#methoddef #add_flags)
    })
}

fn impl_py_method_def_new(cls: &syn::Type, spec: &FnSpec<'_>) -> Result<TokenStream> {
    let wrapper_ident = syn::Ident::new("__pymethod__new__", Span::call_site());
    let wrapper = spec.get_wrapper_function(&wrapper_ident, Some(cls))?;
    Ok(quote! {{
        impl #cls {
            #[doc(hidden)]
            #wrapper
        }
        _pyo3::ffi::PyType_Slot {
            slot: _pyo3::ffi::Py_tp_new,
            pfunc: #cls::#wrapper_ident as _pyo3::ffi::newfunc as _
        }
    }})
}

fn impl_call_slot(cls: &syn::Type, mut spec: FnSpec<'_>) -> Result<TokenStream> {
    // HACK: __call__ proto slot must always use varargs calling convention, so change the spec.
    // Probably indicates there's a refactoring opportunity somewhere.
    spec.convention = CallingConvention::Varargs;

    let wrapper_ident = syn::Ident::new("__wrap", Span::call_site());
    let wrapper = spec.get_wrapper_function(&wrapper_ident, Some(cls))?;
    Ok(quote! {{
        #wrapper
        _pyo3::ffi::PyType_Slot {
            slot: _pyo3::ffi::Py_tp_call,
            pfunc: __wrap as _pyo3::ffi::ternaryfunc as _
        }
    }})
}

fn impl_traverse_slot(cls: &syn::Type, spec: FnSpec<'_>) -> TokenStream {
    let ident = spec.name;
    quote! {{
        pub unsafe extern "C" fn __wrap_(
            slf: *mut _pyo3::ffi::PyObject,
            visit: _pyo3::ffi::visitproc,
            arg: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int
        {
            let pool = _pyo3::GILPool::new();
            let py = pool.python();
            _pyo3::callback::abort_on_traverse_panic(::std::panic::catch_unwind(move || {
                let slf = py.from_borrowed_ptr::<_pyo3::PyCell<#cls>>(slf);

                let visit = _pyo3::class::gc::PyVisit::from_raw(visit, arg, py);
                let borrow = slf.try_borrow();
                if let ::std::result::Result::Ok(borrow) = borrow {
                    _pyo3::impl_::pymethods::unwrap_traverse_result(borrow.#ident(visit))
                } else {
                    0
                }
            }))
        }
        _pyo3::ffi::PyType_Slot {
            slot: _pyo3::ffi::Py_tp_traverse,
            pfunc: __wrap_ as _pyo3::ffi::traverseproc as _
        }
    }}
}

fn impl_py_class_attribute(cls: &syn::Type, spec: &FnSpec<'_>) -> TokenStream {
    let name = &spec.name;
    let deprecations = &spec.deprecations;
    let python_name = spec.null_terminated_python_name();
    quote! {
        _pyo3::class::PyMethodDefType::ClassAttribute({
            _pyo3::class::PyClassAttributeDef::new(
                #python_name,
                _pyo3::impl_::pymethods::PyClassAttributeFactory({
                    fn __wrap(py: _pyo3::Python<'_>) -> _pyo3::PyObject {
                        #deprecations
                        _pyo3::IntoPy::into_py(#cls::#name(), py)
                    }
                    __wrap
                })
            )
        })
    }
}

fn impl_call_setter(cls: &syn::Type, spec: &FnSpec<'_>) -> syn::Result<TokenStream> {
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
pub fn impl_py_setter_def(cls: &syn::Type, property_type: PropertyType<'_>) -> Result<TokenStream> {
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
        PropertyType::Descriptor { .. } => {
            SelfType::Receiver { mutable: true }.receiver(cls, ExtractErrorMode::Raise)
        }
        PropertyType::Function { self_type, .. } => {
            self_type.receiver(cls, ExtractErrorMode::Raise)
        }
    };
    Ok(quote! {
        _pyo3::class::PyMethodDefType::Setter({
            #deprecations
            _pyo3::class::PySetterDef::new(
                #python_name,
                _pyo3::impl_::pymethods::PySetter({
                    unsafe extern "C" fn __wrap(
                        _slf: *mut _pyo3::ffi::PyObject,
                        _value: *mut _pyo3::ffi::PyObject,
                        _: *mut ::std::os::raw::c_void
                    ) -> ::std::os::raw::c_int {
                        let gil = _pyo3::GILPool::new();
                        let _py = gil.python();
                        _pyo3::callback::panic_result_into_callback_output(_py, ::std::panic::catch_unwind(move || -> _pyo3::PyResult<_> {
                            #slf
                            let _value = _py
                                .from_borrowed_ptr_or_opt(_value)
                                .ok_or_else(|| {
                                    _pyo3::exceptions::PyAttributeError::new_err("can't delete attribute")
                                })?;
                            let _val = _pyo3::FromPyObject::extract(_value)?;

                            _pyo3::callback::convert(_py, #setter_impl)
                        }))
                    }
                    __wrap
                }),
                #doc
            )
        })
    })
}

fn impl_call_getter(cls: &syn::Type, spec: &FnSpec<'_>) -> syn::Result<TokenStream> {
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
pub fn impl_py_getter_def(cls: &syn::Type, property_type: PropertyType<'_>) -> Result<TokenStream> {
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
            //quote!(_slf.#ident.clone())
            quote!(::std::clone::Clone::clone(&(_slf.#ident)))
        }
        PropertyType::Descriptor { field_index, .. } => {
            // tuple struct field
            let index = syn::Index::from(field_index);
            quote!(::std::clone::Clone::clone(&(_slf.#index)))
        }
        PropertyType::Function { spec, .. } => impl_call_getter(cls, spec)?,
    };

    let slf = match property_type {
        PropertyType::Descriptor { .. } => {
            SelfType::Receiver { mutable: false }.receiver(cls, ExtractErrorMode::Raise)
        }
        PropertyType::Function { self_type, .. } => {
            self_type.receiver(cls, ExtractErrorMode::Raise)
        }
    };

    let conversion = match property_type {
        PropertyType::Descriptor { .. } => {
            quote! {
                let item: _pyo3::Py<_pyo3::PyAny> = _pyo3::IntoPy::into_py(item, _py);
                ::std::result::Result::Ok(_pyo3::conversion::IntoPyPointer::into_ptr(item))
            }
        }
        // Forward to `IntoPyCallbackOutput`, to handle `#[getter]`s returning results.
        PropertyType::Function { .. } => {
            quote! {
                _pyo3::callback::convert(_py, item)
            }
        }
    };

    Ok(quote! {
        _pyo3::class::PyMethodDefType::Getter({
            #deprecations
            _pyo3::class::PyGetterDef::new(
                #python_name,
                _pyo3::impl_::pymethods::PyGetter({
                    unsafe extern "C" fn __wrap(
                        _slf: *mut _pyo3::ffi::PyObject,
                        _: *mut ::std::os::raw::c_void
                    ) -> *mut _pyo3::ffi::PyObject {
                        let gil = _pyo3::GILPool::new();
                        let _py = gil.python();
                        _pyo3::callback::panic_result_into_callback_output(_py, ::std::panic::catch_unwind(move || -> _pyo3::PyResult<_> {
                            #slf
                            let item = #getter_impl;
                            #conversion
                        }))
                    }
                    __wrap
                }),
                #doc
            )
        })
    })
}

/// Split an argument of pyo3::Python from the front of the arg list, if present
fn split_off_python_arg<'a>(args: &'a [FnArg<'a>]) -> (Option<&FnArg<'_>>, &[FnArg<'_>]) {
    match args {
        [py, args @ ..] if utils::is_python(py.ty) => (Some(py), args),
        args => (None, args),
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
                    (Some(name), _) => name.value.0.to_string(),
                    (None, Some(field_name)) => format!("{}\0", field_name.unraw()),
                    (None, None) => {
                        bail_spanned!(field.span() => "`get` and `set` with tuple struct fields require `name`");
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

    fn doc(&self) -> Cow<'_, PythonDoc> {
        match self {
            PropertyType::Descriptor { field, .. } => {
                Cow::Owned(utils::get_doc(&field.attrs, None))
            }
            PropertyType::Function { spec, .. } => Cow::Borrowed(&spec.doc),
        }
    }
}

const __STR__: SlotDef = SlotDef::new("Py_tp_str", "reprfunc");
const __REPR__: SlotDef = SlotDef::new("Py_tp_repr", "reprfunc");
const __HASH__: SlotDef = SlotDef::new("Py_tp_hash", "hashfunc")
    .ret_ty(Ty::PyHashT)
    .return_conversion(TokenGenerator(
        || quote! { _pyo3::callback::HashCallbackOutput },
    ));
const __RICHCMP__: SlotDef = SlotDef::new("Py_tp_richcompare", "richcmpfunc")
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .arguments(&[Ty::Object, Ty::CompareOp]);
const __GET__: SlotDef = SlotDef::new("Py_tp_descr_get", "descrgetfunc")
    .arguments(&[Ty::MaybeNullObject, Ty::MaybeNullObject]);
const __ITER__: SlotDef = SlotDef::new("Py_tp_iter", "getiterfunc");
const __NEXT__: SlotDef = SlotDef::new("Py_tp_iternext", "iternextfunc").return_conversion(
    TokenGenerator(|| quote! { _pyo3::class::iter::IterNextOutput::<_, _> }),
);
const __AWAIT__: SlotDef = SlotDef::new("Py_am_await", "unaryfunc");
const __AITER__: SlotDef = SlotDef::new("Py_am_aiter", "unaryfunc");
const __ANEXT__: SlotDef = SlotDef::new("Py_am_anext", "unaryfunc").return_conversion(
    TokenGenerator(|| quote! { _pyo3::class::pyasync::IterANextOutput::<_, _> }),
);
const __LEN__: SlotDef = SlotDef::new("Py_mp_length", "lenfunc").ret_ty(Ty::PySsizeT);
const __CONTAINS__: SlotDef = SlotDef::new("Py_sq_contains", "objobjproc")
    .arguments(&[Ty::Object])
    .ret_ty(Ty::Int);
const __CONCAT__: SlotDef = SlotDef::new("Py_sq_concat", "binaryfunc").arguments(&[Ty::Object]);
const __REPEAT__: SlotDef = SlotDef::new("Py_sq_repeat", "ssizeargfunc").arguments(&[Ty::PySsizeT]);
const __INPLACE_CONCAT__: SlotDef =
    SlotDef::new("Py_sq_concat", "binaryfunc").arguments(&[Ty::Object]);
const __INPLACE_REPEAT__: SlotDef =
    SlotDef::new("Py_sq_repeat", "ssizeargfunc").arguments(&[Ty::PySsizeT]);
const __GETITEM__: SlotDef = SlotDef::new("Py_mp_subscript", "binaryfunc").arguments(&[Ty::Object]);

const __POS__: SlotDef = SlotDef::new("Py_nb_positive", "unaryfunc");
const __NEG__: SlotDef = SlotDef::new("Py_nb_negative", "unaryfunc");
const __ABS__: SlotDef = SlotDef::new("Py_nb_absolute", "unaryfunc");
const __INVERT__: SlotDef = SlotDef::new("Py_nb_invert", "unaryfunc");
const __INDEX__: SlotDef = SlotDef::new("Py_nb_index", "unaryfunc");
const __INT__: SlotDef = SlotDef::new("Py_nb_int", "unaryfunc");
const __FLOAT__: SlotDef = SlotDef::new("Py_nb_float", "unaryfunc");
const __BOOL__: SlotDef = SlotDef::new("Py_nb_bool", "inquiry").ret_ty(Ty::Int);

const __IADD__: SlotDef = SlotDef::new("Py_nb_inplace_add", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __ISUB__: SlotDef = SlotDef::new("Py_nb_inplace_subtract", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __IMUL__: SlotDef = SlotDef::new("Py_nb_inplace_multiply", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __IMATMUL__: SlotDef = SlotDef::new("Py_nb_inplace_matrix_multiply", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __ITRUEDIV__: SlotDef = SlotDef::new("Py_nb_inplace_true_divide", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __IFLOORDIV__: SlotDef = SlotDef::new("Py_nb_inplace_floor_divide", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __IMOD__: SlotDef = SlotDef::new("Py_nb_inplace_remainder", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __IPOW__: SlotDef = SlotDef::new("Py_nb_inplace_power", "ipowfunc")
    .arguments(&[Ty::Object, Ty::IPowModulo])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __ILSHIFT__: SlotDef = SlotDef::new("Py_nb_inplace_lshift", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __IRSHIFT__: SlotDef = SlotDef::new("Py_nb_inplace_rshift", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __IAND__: SlotDef = SlotDef::new("Py_nb_inplace_and", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __IXOR__: SlotDef = SlotDef::new("Py_nb_inplace_xor", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __IOR__: SlotDef = SlotDef::new("Py_nb_inplace_or", "binaryfunc")
    .arguments(&[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();
const __GETBUFFER__: SlotDef = SlotDef::new("Py_bf_getbuffer", "getbufferproc")
    .arguments(&[Ty::PyBuffer, Ty::Int])
    .ret_ty(Ty::Int)
    .require_unsafe();
const __RELEASEBUFFER__: SlotDef = SlotDef::new("Py_bf_releasebuffer", "releasebufferproc")
    .arguments(&[Ty::PyBuffer])
    .ret_ty(Ty::Void)
    .require_unsafe();
const __CLEAR__: SlotDef = SlotDef::new("Py_tp_clear", "inquiry")
    .arguments(&[])
    .ret_ty(Ty::Int);

#[derive(Clone, Copy)]
enum Ty {
    Object,
    MaybeNullObject,
    NonNullObject,
    IPowModulo,
    CompareOp,
    Int,
    PyHashT,
    PySsizeT,
    Void,
    PyBuffer,
}

impl Ty {
    fn ffi_type(self) -> TokenStream {
        match self {
            Ty::Object | Ty::MaybeNullObject => quote! { *mut _pyo3::ffi::PyObject },
            Ty::NonNullObject => quote! { ::std::ptr::NonNull<_pyo3::ffi::PyObject> },
            Ty::IPowModulo => quote! { _pyo3::impl_::pymethods::IPowModulo },
            Ty::Int | Ty::CompareOp => quote! { ::std::os::raw::c_int },
            Ty::PyHashT => quote! { _pyo3::ffi::Py_hash_t },
            Ty::PySsizeT => quote! { _pyo3::ffi::Py_ssize_t },
            Ty::Void => quote! { () },
            Ty::PyBuffer => quote! { *mut _pyo3::ffi::Py_buffer },
        }
    }

    fn extract(
        self,
        cls: &syn::Type,
        py: &syn::Ident,
        ident: &syn::Ident,
        arg: &FnArg<'_>,
        extract_error_mode: ExtractErrorMode,
    ) -> TokenStream {
        match self {
            Ty::Object => {
                let extract = handle_error(
                    extract_error_mode,
                    py,
                    quote! {
                        #py.from_borrowed_ptr::<_pyo3::PyAny>(#ident).extract()
                    },
                );
                extract_object(cls, arg.ty, ident, extract)
            }
            Ty::MaybeNullObject => {
                let extract = handle_error(
                    extract_error_mode,
                    py,
                    quote! {
                        #py.from_borrowed_ptr::<_pyo3::PyAny>(
                            if #ident.is_null() {
                                _pyo3::ffi::Py_None()
                            } else {
                                #ident
                            }
                        ).extract()
                    },
                );
                extract_object(cls, arg.ty, ident, extract)
            }
            Ty::NonNullObject => {
                let extract = handle_error(
                    extract_error_mode,
                    py,
                    quote! {
                        #py.from_borrowed_ptr::<_pyo3::PyAny>(#ident.as_ptr()).extract()
                    },
                );
                extract_object(cls, arg.ty, ident, extract)
            }
            Ty::IPowModulo => {
                let extract = handle_error(
                    extract_error_mode,
                    py,
                    quote! {
                        #ident.extract(#py)
                    },
                );
                extract_object(cls, arg.ty, ident, extract)
            }
            Ty::CompareOp => {
                let extract = handle_error(
                    extract_error_mode,
                    py,
                    quote! {
                        _pyo3::class::basic::CompareOp::from_raw(#ident)
                            .ok_or_else(|| _pyo3::exceptions::PyValueError::new_err("invalid comparison operator"))
                    },
                );
                quote! {
                    let #ident = #extract;
                }
            }
            Ty::PySsizeT => {
                let ty = arg.ty;
                let extract = handle_error(
                    extract_error_mode,
                    py,
                    quote! {
                            ::std::convert::TryInto::<#ty>::try_into(#ident).map_err(|e| _pyo3::exceptions::PyValueError::new_err(e.to_string()))
                    },
                );
                quote! {
                    let #ident = #extract;
                }
            }
            // Just pass other types through unmodified
            Ty::PyBuffer | Ty::Int | Ty::PyHashT | Ty::Void => quote! {},
        }
    }
}

fn handle_error(
    extract_error_mode: ExtractErrorMode,
    py: &syn::Ident,
    extract: TokenStream,
) -> TokenStream {
    match extract_error_mode {
        ExtractErrorMode::Raise => quote! { #extract? },
        ExtractErrorMode::NotImplemented => quote! {
            match #extract {
                ::std::result::Result::Ok(value) => value,
                ::std::result::Result::Err(_) => { return _pyo3::callback::convert(#py, #py.NotImplemented()); },
            }
        },
    }
}

fn extract_object(
    cls: &syn::Type,
    target: &syn::Type,
    ident: &syn::Ident,
    extract: TokenStream,
) -> TokenStream {
    if let syn::Type::Reference(tref) = unwrap_ty_group(target) {
        let mut tref = remove_lifetime(tref);
        replace_self(&mut tref.elem, cls);
        let mut_ = tref.mutability;
        quote! {
            let #mut_ #ident: <#tref as _pyo3::derive_utils::ExtractExt<'_>>::Target = #extract;
            let #ident = &#mut_ *#ident;
        }
    } else {
        quote! {
            let #ident = #extract;
        }
    }
}

enum ReturnMode {
    ReturnSelf,
    Conversion(TokenGenerator),
}

impl ReturnMode {
    fn return_call_output(&self, py: &syn::Ident, call: TokenStream) -> TokenStream {
        match self {
            ReturnMode::Conversion(conversion) => quote! {
                let _result: _pyo3::PyResult<#conversion> = #call;
                _pyo3::callback::convert(#py, _result)
            },
            ReturnMode::ReturnSelf => quote! {
                let _result: _pyo3::PyResult<()> = #call;
                _result?;
                _pyo3::ffi::Py_XINCREF(_raw_slf);
                ::std::result::Result::Ok(_raw_slf)
            },
        }
    }
}

struct SlotDef {
    slot: StaticIdent,
    func_ty: StaticIdent,
    arguments: &'static [Ty],
    ret_ty: Ty,
    extract_error_mode: ExtractErrorMode,
    return_mode: Option<ReturnMode>,
    require_unsafe: bool,
}

const NO_ARGUMENTS: &[Ty] = &[];

impl SlotDef {
    const fn new(slot: &'static str, func_ty: &'static str) -> Self {
        SlotDef {
            slot: StaticIdent(slot),
            func_ty: StaticIdent(func_ty),
            arguments: NO_ARGUMENTS,
            ret_ty: Ty::Object,
            extract_error_mode: ExtractErrorMode::Raise,
            return_mode: None,
            require_unsafe: false,
        }
    }

    const fn arguments(mut self, arguments: &'static [Ty]) -> Self {
        self.arguments = arguments;
        self
    }

    const fn ret_ty(mut self, ret_ty: Ty) -> Self {
        self.ret_ty = ret_ty;
        self
    }

    const fn return_conversion(mut self, return_conversion: TokenGenerator) -> Self {
        self.return_mode = Some(ReturnMode::Conversion(return_conversion));
        self
    }

    const fn extract_error_mode(mut self, extract_error_mode: ExtractErrorMode) -> Self {
        self.extract_error_mode = extract_error_mode;
        self
    }

    const fn return_self(mut self) -> Self {
        self.return_mode = Some(ReturnMode::ReturnSelf);
        self
    }

    const fn require_unsafe(mut self) -> Self {
        self.require_unsafe = true;
        self
    }

    fn generate_type_slot(
        &self,
        cls: &syn::Type,
        spec: &FnSpec<'_>,
        method_name: &str,
    ) -> Result<TokenStream> {
        let SlotDef {
            slot,
            func_ty,
            arguments,
            extract_error_mode,
            ret_ty,
            return_mode,
            require_unsafe,
        } = self;
        if *require_unsafe {
            ensure_spanned!(
                spec.unsafety.is_some(),
                spec.name.span() => format!("`{}` must be `unsafe fn`", method_name)
            );
        }
        let py = syn::Ident::new("_py", Span::call_site());
        let method_arguments = generate_method_arguments(arguments);
        let ret_ty = ret_ty.ffi_type();
        let body = generate_method_body(
            cls,
            spec,
            &py,
            arguments,
            *extract_error_mode,
            return_mode.as_ref(),
        )?;
        Ok(quote!({
            unsafe extern "C" fn __wrap(_raw_slf: *mut _pyo3::ffi::PyObject, #(#method_arguments),*) -> #ret_ty {
                let _slf = _raw_slf;
                let gil = _pyo3::GILPool::new();
                let #py = gil.python();
                _pyo3::callback::panic_result_into_callback_output(#py, ::std::panic::catch_unwind(move || -> _pyo3::PyResult<_> {
                    #body
                }))
            }
            _pyo3::ffi::PyType_Slot {
                slot: _pyo3::ffi::#slot,
                pfunc: __wrap as _pyo3::ffi::#func_ty as _
            }
        }))
    }
}

fn generate_method_arguments(arguments: &[Ty]) -> impl Iterator<Item = TokenStream> + '_ {
    arguments.iter().enumerate().map(|(i, arg)| {
        let ident = syn::Ident::new(&format!("arg{}", i), Span::call_site());
        let ffi_type = arg.ffi_type();
        quote! {
            #ident: #ffi_type
        }
    })
}

fn generate_method_body(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    py: &syn::Ident,
    arguments: &[Ty],
    extract_error_mode: ExtractErrorMode,
    return_mode: Option<&ReturnMode>,
) -> Result<TokenStream> {
    let self_conversion = spec.tp.self_conversion(Some(cls), extract_error_mode);
    let rust_name = spec.name;
    let (arg_idents, arg_count, conversions) =
        extract_proto_arguments(cls, py, &spec.args, arguments, extract_error_mode)?;
    if arg_count != arguments.len() {
        bail_spanned!(spec.name.span() => format!("Expected {} arguments, got {}", arguments.len(), arg_count));
    }
    let call = quote! { _pyo3::callback::convert(#py, #cls::#rust_name(_slf, #(#arg_idents),*)) };
    let body = if let Some(return_mode) = return_mode {
        return_mode.return_call_output(py, call)
    } else {
        call
    };
    Ok(quote! {
        #self_conversion
        #conversions
        #body
    })
}

struct SlotFragmentDef {
    fragment: &'static str,
    arguments: &'static [Ty],
    extract_error_mode: ExtractErrorMode,
    ret_ty: Ty,
}

impl SlotFragmentDef {
    const fn new(fragment: &'static str, arguments: &'static [Ty]) -> Self {
        SlotFragmentDef {
            fragment,
            arguments,
            extract_error_mode: ExtractErrorMode::Raise,
            ret_ty: Ty::Void,
        }
    }

    const fn extract_error_mode(mut self, extract_error_mode: ExtractErrorMode) -> Self {
        self.extract_error_mode = extract_error_mode;
        self
    }

    const fn ret_ty(mut self, ret_ty: Ty) -> Self {
        self.ret_ty = ret_ty;
        self
    }

    fn generate_pyproto_fragment(&self, cls: &syn::Type, spec: &FnSpec<'_>) -> Result<TokenStream> {
        let SlotFragmentDef {
            fragment,
            arguments,
            extract_error_mode,
            ret_ty,
        } = self;
        let fragment_trait = format_ident!("PyClass{}SlotFragment", fragment);
        let method = syn::Ident::new(fragment, Span::call_site());
        let py = syn::Ident::new("_py", Span::call_site());
        let method_arguments = generate_method_arguments(arguments);
        let body = generate_method_body(cls, spec, &py, arguments, *extract_error_mode, None)?;
        let ret_ty = ret_ty.ffi_type();
        Ok(quote! {
            impl _pyo3::impl_::pyclass::#fragment_trait<#cls> for _pyo3::impl_::pyclass::PyClassImplCollector<#cls> {

                #[inline]
                unsafe fn #method(
                    self,
                    #py: _pyo3::Python,
                    _raw_slf: *mut _pyo3::ffi::PyObject,
                    #(#method_arguments),*
                ) -> _pyo3::PyResult<#ret_ty> {
                    let _slf = _raw_slf;
                    #body
                }
            }
        })
    }
}

const __GETATTRIBUTE__: SlotFragmentDef =
    SlotFragmentDef::new("__getattribute__", &[Ty::Object]).ret_ty(Ty::Object);
const __GETATTR__: SlotFragmentDef =
    SlotFragmentDef::new("__getattr__", &[Ty::Object]).ret_ty(Ty::Object);
const __SETATTR__: SlotFragmentDef =
    SlotFragmentDef::new("__setattr__", &[Ty::Object, Ty::NonNullObject]);
const __DELATTR__: SlotFragmentDef = SlotFragmentDef::new("__delattr__", &[Ty::Object]);
const __SET__: SlotFragmentDef = SlotFragmentDef::new("__set__", &[Ty::Object, Ty::NonNullObject]);
const __DELETE__: SlotFragmentDef = SlotFragmentDef::new("__delete__", &[Ty::Object]);
const __SETITEM__: SlotFragmentDef =
    SlotFragmentDef::new("__setitem__", &[Ty::Object, Ty::NonNullObject]);
const __DELITEM__: SlotFragmentDef = SlotFragmentDef::new("__delitem__", &[Ty::Object]);

macro_rules! binary_num_slot_fragment_def {
    ($ident:ident, $name:literal) => {
        const $ident: SlotFragmentDef = SlotFragmentDef::new($name, &[Ty::Object])
            .extract_error_mode(ExtractErrorMode::NotImplemented)
            .ret_ty(Ty::Object);
    };
}

binary_num_slot_fragment_def!(__ADD__, "__add__");
binary_num_slot_fragment_def!(__RADD__, "__radd__");
binary_num_slot_fragment_def!(__SUB__, "__sub__");
binary_num_slot_fragment_def!(__RSUB__, "__rsub__");
binary_num_slot_fragment_def!(__MUL__, "__mul__");
binary_num_slot_fragment_def!(__RMUL__, "__rmul__");
binary_num_slot_fragment_def!(__MATMUL__, "__matmul__");
binary_num_slot_fragment_def!(__RMATMUL__, "__rmatmul__");
binary_num_slot_fragment_def!(__FLOORDIV__, "__floordiv__");
binary_num_slot_fragment_def!(__RFLOORDIV__, "__rfloordiv__");
binary_num_slot_fragment_def!(__TRUEDIV__, "__truediv__");
binary_num_slot_fragment_def!(__RTRUEDIV__, "__rtruediv__");
binary_num_slot_fragment_def!(__DIVMOD__, "__divmod__");
binary_num_slot_fragment_def!(__RDIVMOD__, "__rdivmod__");
binary_num_slot_fragment_def!(__MOD__, "__mod__");
binary_num_slot_fragment_def!(__RMOD__, "__rmod__");
binary_num_slot_fragment_def!(__LSHIFT__, "__lshift__");
binary_num_slot_fragment_def!(__RLSHIFT__, "__rlshift__");
binary_num_slot_fragment_def!(__RSHIFT__, "__rshift__");
binary_num_slot_fragment_def!(__RRSHIFT__, "__rrshift__");
binary_num_slot_fragment_def!(__AND__, "__and__");
binary_num_slot_fragment_def!(__RAND__, "__rand__");
binary_num_slot_fragment_def!(__XOR__, "__xor__");
binary_num_slot_fragment_def!(__RXOR__, "__rxor__");
binary_num_slot_fragment_def!(__OR__, "__or__");
binary_num_slot_fragment_def!(__ROR__, "__ror__");

const __POW__: SlotFragmentDef = SlotFragmentDef::new("__pow__", &[Ty::Object, Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .ret_ty(Ty::Object);
const __RPOW__: SlotFragmentDef = SlotFragmentDef::new("__rpow__", &[Ty::Object, Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .ret_ty(Ty::Object);

fn extract_proto_arguments(
    cls: &syn::Type,
    py: &syn::Ident,
    method_args: &[FnArg<'_>],
    proto_args: &[Ty],
    extract_error_mode: ExtractErrorMode,
) -> Result<(Vec<Ident>, usize, TokenStream)> {
    let mut arg_idents = Vec::with_capacity(method_args.len());
    let mut non_python_args = 0;

    let mut args_conversions = Vec::with_capacity(proto_args.len());

    for arg in method_args {
        if arg.py {
            arg_idents.push(py.clone());
        } else {
            let ident = syn::Ident::new(&format!("arg{}", non_python_args), Span::call_site());
            let conversions = proto_args.get(non_python_args)
                .ok_or_else(|| err_spanned!(arg.ty.span() => format!("Expected at most {} non-python arguments", proto_args.len())))?
                .extract(cls, py, &ident, arg, extract_error_mode);
            non_python_args += 1;
            args_conversions.push(conversions);
            arg_idents.push(ident);
        }
    }
    let conversions = quote!(#(#args_conversions)*);
    Ok((arg_idents, non_python_args, conversions))
}

struct StaticIdent(&'static str);

impl ToTokens for StaticIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        syn::Ident::new(self.0, Span::call_site()).to_tokens(tokens)
    }
}

struct TokenGenerator(fn() -> TokenStream);

impl ToTokens for TokenGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0().to_tokens(tokens)
    }
}
