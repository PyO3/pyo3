use std::borrow::Cow;
use std::ffi::CString;

use crate::attributes::{FromPyWithAttribute, NameAttribute, RenamingRule};
#[cfg(feature = "experimental-inspect")]
use crate::introspection::unique_element_id;
use crate::method::{CallingConvention, ExtractErrorMode, PyArg};
use crate::params::{impl_arg_params, impl_regular_arg_param, Holders};
use crate::pyfunction::WarningFactory;
use crate::utils::PythonDoc;
use crate::utils::{Ctx, StaticIdent};
use crate::{
    method::{FnArg, FnSpec, FnType, SelfType},
    pyfunction::PyFunctionOptions,
};
use crate::{quotes, utils};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{ext::IdentExt, spanned::Spanned, Field, Ident, Result};
use syn::{parse_quote, LitCStr};

/// Generated code for a single pymethod item.
pub struct MethodAndMethodDef {
    /// The implementation of the Python wrapper for the pymethod
    pub associated_method: TokenStream,
    /// The method def which will be used to register this pymethod
    pub method_def: TokenStream,
}

#[cfg(feature = "experimental-inspect")]
impl MethodAndMethodDef {
    pub fn add_introspection(&mut self, data: TokenStream) {
        let const_name = format_ident!("_{}", unique_element_id()); // We need an explicit name here
        self.associated_method.extend(quote! {
            const #const_name: () = {
                #data
            };
        });
    }
}

/// Generated code for a single pymethod item which is registered by a slot.
pub struct MethodAndSlotDef {
    /// The implementation of the Python wrapper for the pymethod
    pub associated_method: TokenStream,
    /// The slot def which will be used to register this pymethod
    pub slot_def: TokenStream,
}

#[cfg(feature = "experimental-inspect")]
impl MethodAndSlotDef {
    pub fn add_introspection(&mut self, data: TokenStream) {
        let const_name = format_ident!("_{}", unique_element_id()); // We need an explicit name here
        self.associated_method.extend(quote! {
            const #const_name: () = {
                #data
            };
        });
    }
}

pub enum GeneratedPyMethod {
    Method(MethodAndMethodDef),
    Proto(MethodAndSlotDef),
    SlotTraitImpl(String, TokenStream),
}

pub struct PyMethod<'a> {
    kind: PyMethodKind,
    method_name: String,
    pub spec: FnSpec<'a>,
}

enum PyMethodKind {
    Fn,
    Proto(PyMethodProtoKind),
}

impl PyMethodKind {
    fn from_name(name: &str) -> Self {
        match name {
            // Protocol implemented through slots
            "__new__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__NEW__)),
            "__init__" => PyMethodKind::Proto(PyMethodProtoKind::Slot(&__INIT__)),
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
            "__lt__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__LT__)),
            "__le__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__LE__)),
            "__eq__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__EQ__)),
            "__ne__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__NE__)),
            "__gt__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__GT__)),
            "__ge__" => PyMethodKind::Proto(PyMethodProtoKind::SlotFragment(&__GE__)),
            // Some tricky protocols which don't fit the pattern of the rest
            "__call__" => PyMethodKind::Proto(PyMethodProtoKind::Call),
            "__traverse__" => PyMethodKind::Proto(PyMethodProtoKind::Traverse),
            "__clear__" => PyMethodKind::Proto(PyMethodProtoKind::Clear),
            // Not a proto
            _ => PyMethodKind::Fn,
        }
    }
}

enum PyMethodProtoKind {
    Slot(&'static SlotDef),
    Call,
    Traverse,
    Clear,
    SlotFragment(&'static SlotFragmentDef),
}

impl<'a> PyMethod<'a> {
    pub fn parse(
        sig: &'a mut syn::Signature,
        meth_attrs: &mut Vec<syn::Attribute>,
        options: PyFunctionOptions,
    ) -> Result<Self> {
        check_generic(sig)?;
        ensure_function_options_valid(&options)?;
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
    method: PyMethod<'_>,
    meth_attrs: &[syn::Attribute],
    ctx: &Ctx,
) -> Result<GeneratedPyMethod> {
    let spec = &method.spec;

    if spec.asyncness.is_some() {
        ensure_spanned!(
            cfg!(feature = "experimental-async"),
            spec.asyncness.span() => "async functions are only supported with the `experimental-async` feature"
        );
    }

    Ok(match (method.kind, &spec.tp) {
        // Class attributes go before protos so that class attributes can be used to set proto
        // method to None.
        (_, FnType::ClassAttribute) => {
            GeneratedPyMethod::Method(impl_py_class_attribute(cls, spec, ctx)?)
        }
        (PyMethodKind::Proto(proto_kind), _) => {
            ensure_no_forbidden_protocol_attributes(&proto_kind, spec, &method.method_name)?;
            match proto_kind {
                PyMethodProtoKind::Slot(slot_def) => {
                    let slot = slot_def.generate_type_slot(cls, spec, &method.method_name, ctx)?;
                    GeneratedPyMethod::Proto(slot)
                }
                PyMethodProtoKind::Call => {
                    GeneratedPyMethod::Proto(impl_call_slot(cls, spec, ctx)?)
                }
                PyMethodProtoKind::Traverse => {
                    GeneratedPyMethod::Proto(impl_traverse_slot(cls, spec, ctx)?)
                }
                PyMethodProtoKind::Clear => {
                    GeneratedPyMethod::Proto(impl_clear_slot(cls, spec, ctx)?)
                }
                PyMethodProtoKind::SlotFragment(slot_fragment_def) => {
                    let proto = slot_fragment_def.generate_pyproto_fragment(cls, spec, ctx)?;
                    GeneratedPyMethod::SlotTraitImpl(method.method_name, proto)
                }
            }
        }
        // ordinary functions (with some specialties)
        (_, FnType::Fn(_) | FnType::FnClass(_) | FnType::FnStatic) => GeneratedPyMethod::Method(
            impl_py_method_def(cls, spec, &spec.get_doc(meth_attrs, ctx)?, ctx)?,
        ),
        (_, FnType::Getter(self_type)) => GeneratedPyMethod::Method(impl_py_getter_def(
            cls,
            PropertyType::Function {
                self_type,
                spec,
                doc: spec.get_doc(meth_attrs, ctx)?,
            },
            ctx,
        )?),
        (_, FnType::Setter(self_type)) => GeneratedPyMethod::Method(impl_py_setter_def(
            cls,
            PropertyType::Function {
                self_type,
                spec,
                doc: spec.get_doc(meth_attrs, ctx)?,
            },
            ctx,
        )?),
        (_, FnType::Deleter(self_type)) => GeneratedPyMethod::Method(impl_py_deleter_def(
            cls,
            self_type,
            spec,
            spec.get_doc(meth_attrs, ctx)?,
            ctx,
        )?),
        (_, FnType::FnModule(_)) => {
            unreachable!("methods cannot be FnModule")
        }
    })
}

pub fn check_generic(sig: &syn::Signature) -> syn::Result<()> {
    let err_msg = |typ| format!("Python functions cannot have generic {typ} parameters");
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
    proto_kind: &PyMethodProtoKind,
    spec: &FnSpec<'_>,
    method_name: &str,
) -> syn::Result<()> {
    if let Some(signature) = &spec.signature.attribute {
        // __new__, __init__ and __call__ are allowed to have a signature, but nothing else is.
        if !matches!(
            proto_kind,
            PyMethodProtoKind::Slot(SlotDef {
                calling_convention: SlotCallingConvention::TpNew | SlotCallingConvention::TpInit,
                ..
            })
        ) && !matches!(proto_kind, PyMethodProtoKind::Call)
        {
            bail_spanned!(signature.kw.span() => format!("`signature` cannot be used with magic method `{}`", method_name));
        }
    }
    if let Some(text_signature) = &spec.text_signature {
        // __new__ is also allowed a text_signature (no other proto method is)
        if !matches!(
            proto_kind,
            PyMethodProtoKind::Slot(SlotDef {
                calling_convention: SlotCallingConvention::TpNew,
                ..
            })
        ) {
            bail_spanned!(text_signature.kw.span() => format!("`text_signature` cannot be used with magic method `{}`", method_name));
        }
    }
    Ok(())
}

pub fn impl_py_method_def(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    doc: &PythonDoc,
    ctx: &Ctx,
) -> Result<MethodAndMethodDef> {
    let Ctx { pyo3_path, .. } = ctx;
    let wrapper_ident = format_ident!("__pymethod_{}__", spec.python_name);
    let calling_convention = CallingConvention::from_signature(&spec.signature);
    let associated_method =
        spec.get_wrapper_function(&wrapper_ident, Some(cls), calling_convention, ctx)?;
    let methoddef = spec.get_methoddef(
        quote! { #cls::#wrapper_ident },
        doc,
        calling_convention,
        ctx,
    );
    let method_def = quote! {
        #pyo3_path::impl_::pymethods::PyMethodDefType::Method(#methoddef)
    };
    Ok(MethodAndMethodDef {
        associated_method,
        method_def,
    })
}

fn impl_call_slot(cls: &syn::Type, spec: &FnSpec<'_>, ctx: &Ctx) -> Result<MethodAndSlotDef> {
    let Ctx { pyo3_path, .. } = ctx;
    let wrapper_ident = syn::Ident::new("__pymethod___call____", Span::call_site());
    let associated_method =
        spec.get_wrapper_function(&wrapper_ident, Some(cls), CallingConvention::Varargs, ctx)?;
    let slot_def = quote! {
        #pyo3_path::ffi::PyType_Slot {
            slot: #pyo3_path::ffi::Py_tp_call,
            pfunc: #pyo3_path::impl_::trampoline::get_trampoline_function!(ternaryfunc, #cls::#wrapper_ident) as _
        }
    };
    Ok(MethodAndSlotDef {
        associated_method,
        slot_def,
    })
}

fn impl_traverse_slot(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    ctx: &Ctx,
) -> syn::Result<MethodAndSlotDef> {
    let Ctx { pyo3_path, .. } = ctx;
    if let (Some(py_arg), _) = split_off_python_arg(&spec.signature.arguments) {
        return Err(syn::Error::new_spanned(py_arg.ty, "__traverse__ may not take `Python`. \
            Usually, an implementation of `__traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>` \
            should do nothing but calls to `visit.call`. Most importantly, safe access to the Python interpreter is \
            prohibited inside implementations of `__traverse__`, i.e. `Python::attach` will panic."));
    }

    // check that the receiver does not try to smuggle an (implicit) `Python` token into here
    if let FnType::Fn(SelfType::TryFromBoundRef(span))
    | FnType::Fn(SelfType::Receiver {
        mutable: true,
        span,
    }) = spec.tp
    {
        bail_spanned! { span =>
            "__traverse__ may not take a receiver other than `&self`. Usually, an implementation of \
            `__traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError>` \
            should do nothing but calls to `visit.call`. Most importantly, safe access to the Python interpreter is \
            prohibited inside implementations of `__traverse__`, i.e. `Python::attach` will panic."
        }
    }

    ensure_spanned!(
        spec.warnings.is_empty(),
        spec.warnings.span() => "__traverse__ cannot be used with #[pyo3(warn)]"
    );

    let rust_fn_ident = spec.name;

    let associated_method = quote! {
        pub unsafe extern "C" fn __pymethod_traverse__(
            slf: *mut #pyo3_path::ffi::PyObject,
            visit: #pyo3_path::ffi::visitproc,
            arg: *mut ::std::ffi::c_void,
        ) -> ::std::ffi::c_int {
            #pyo3_path::impl_::pymethods::_call_traverse::<#cls>(slf, #cls::#rust_fn_ident, visit, arg, #cls::__pymethod_traverse__)
        }
    };
    let slot_def = quote! {
        #pyo3_path::ffi::PyType_Slot {
            slot: #pyo3_path::ffi::Py_tp_traverse,
            pfunc: #cls::__pymethod_traverse__ as #pyo3_path::ffi::traverseproc as _
        }
    };
    Ok(MethodAndSlotDef {
        associated_method,
        slot_def,
    })
}

fn impl_clear_slot(cls: &syn::Type, spec: &FnSpec<'_>, ctx: &Ctx) -> syn::Result<MethodAndSlotDef> {
    let Ctx { pyo3_path, .. } = ctx;
    let (py_arg, args) = split_off_python_arg(&spec.signature.arguments);
    let self_type = match &spec.tp {
        FnType::Fn(self_type) => self_type,
        _ => bail_spanned!(spec.name.span() => "expected instance method for `__clear__` function"),
    };
    let mut holders = Holders::new();
    let slf = self_type.receiver(cls, ExtractErrorMode::Raise, &mut holders, ctx);

    if let [arg, ..] = args {
        bail_spanned!(arg.ty().span() => "`__clear__` function expected to have no arguments");
    }

    let name = &spec.name;
    let holders = holders.init_holders(ctx);
    let fncall = if py_arg.is_some() {
        quote!(#cls::#name(#slf, py))
    } else {
        quote!(#cls::#name(#slf))
    };

    let associated_method = quote! {
        pub unsafe extern "C" fn __pymethod___clear____(
            _slf: *mut #pyo3_path::ffi::PyObject,
        ) -> ::std::ffi::c_int {
            #pyo3_path::impl_::pymethods::_call_clear(_slf, |py, _slf| {
                #holders
                let result = #fncall;
                let result = #pyo3_path::impl_::wrap::converter(&result).wrap(result)?;
                ::std::result::Result::Ok(result)
            }, #cls::__pymethod___clear____)
        }
    };
    let slot_def = quote! {
        #pyo3_path::ffi::PyType_Slot {
            slot: #pyo3_path::ffi::Py_tp_clear,
            pfunc: #cls::__pymethod___clear____ as #pyo3_path::ffi::inquiry as _
        }
    };
    Ok(MethodAndSlotDef {
        associated_method,
        slot_def,
    })
}

pub(crate) fn impl_py_class_attribute(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    ctx: &Ctx,
) -> syn::Result<MethodAndMethodDef> {
    let Ctx { pyo3_path, .. } = ctx;
    let (py_arg, args) = split_off_python_arg(&spec.signature.arguments);
    ensure_spanned!(
        args.is_empty(),
        args[0].ty().span() => "#[classattr] can only have one argument (of type pyo3::Python)"
    );

    ensure_spanned!(
        spec.warnings.is_empty(),
        spec.warnings.span()
        => "#[classattr] cannot be used with #[pyo3(warn)]"
    );

    let name = &spec.name;
    let fncall = if py_arg.is_some() {
        quote!(function(py))
    } else {
        quote!(function())
    };

    let wrapper_ident = format_ident!("__pymethod_{}__", name);
    let python_name = spec.null_terminated_python_name();
    let body = quotes::ok_wrap(fncall, ctx);

    let associated_method = quote! {
        fn #wrapper_ident(py: #pyo3_path::Python<'_>) -> #pyo3_path::PyResult<#pyo3_path::Py<#pyo3_path::PyAny>> {
            let function = #cls::#name; // Shadow the method name to avoid #3017
            let result = #body;
            #pyo3_path::impl_::wrap::converter(&result).map_into_pyobject(py, result)
        }
    };

    let method_def = quote! {
        #pyo3_path::impl_::pymethods::PyMethodDefType::ClassAttribute({
            #pyo3_path::impl_::pymethods::PyClassAttributeDef::new(
                #python_name,
                #cls::#wrapper_ident
            )
        })
    };

    Ok(MethodAndMethodDef {
        associated_method,
        method_def,
    })
}

fn impl_call_setter(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    self_type: &SelfType,
    holders: &mut Holders,
    ctx: &Ctx,
) -> syn::Result<TokenStream> {
    let (py_arg, args) = split_off_python_arg(&spec.signature.arguments);
    let slf = self_type.receiver(cls, ExtractErrorMode::Raise, holders, ctx);

    if args.is_empty() {
        bail_spanned!(spec.name.span() => "setter function expected to have one argument");
    } else if args.len() > 1 {
        bail_spanned!(
            args[1].ty().span() =>
            "setter function can have at most two arguments ([pyo3::Python,] and value)"
        );
    }

    let name = &spec.name;
    let fncall = if py_arg.is_some() {
        quote!(#cls::#name(#slf, py, _val))
    } else {
        quote!(#cls::#name(#slf, _val))
    };

    Ok(fncall)
}

// Used here for PropertyType::Function, used in pyclass for descriptors.
pub fn impl_py_setter_def(
    cls: &syn::Type,
    property_type: PropertyType<'_>,
    ctx: &Ctx,
) -> Result<MethodAndMethodDef> {
    let Ctx { pyo3_path, .. } = ctx;
    let python_name = property_type.null_terminated_python_name()?;
    let doc = property_type.doc(ctx)?;
    let mut holders = Holders::new();
    let setter_impl = match property_type {
        PropertyType::Descriptor {
            field_index, field, ..
        } => {
            let slf = SelfType::Receiver {
                mutable: true,
                span: Span::call_site(),
            }
            .receiver(cls, ExtractErrorMode::Raise, &mut holders, ctx);
            if let Some(ident) = &field.ident {
                // named struct field
                quote!({ #slf.#ident = _val; })
            } else {
                // tuple struct field
                let index = syn::Index::from(field_index);
                quote!({ #slf.#index = _val; })
            }
        }
        PropertyType::Function {
            spec, self_type, ..
        } => impl_call_setter(cls, spec, self_type, &mut holders, ctx)?,
    };

    let wrapper_ident = match property_type {
        PropertyType::Descriptor {
            field: syn::Field {
                ident: Some(ident), ..
            },
            ..
        } => {
            format_ident!("__pymethod_set_{}__", ident)
        }
        PropertyType::Descriptor { field_index, .. } => {
            format_ident!("__pymethod_set_field_{}__", field_index)
        }
        PropertyType::Function { spec, .. } => {
            format_ident!("__pymethod_set_{}__", spec.name)
        }
    };

    let extract = match &property_type {
        PropertyType::Function { spec, .. } => {
            let (_, args) = split_off_python_arg(&spec.signature.arguments);
            let value_arg = &args[0];
            let (from_py_with, ident) =
                if let Some(from_py_with) = &value_arg.from_py_with().as_ref().map(|f| &f.value) {
                    let ident = syn::Ident::new("from_py_with", from_py_with.span());
                    (
                        quote_spanned! { from_py_with.span() =>
                            let #ident = #from_py_with;
                        },
                        ident,
                    )
                } else {
                    (quote!(), syn::Ident::new("dummy", Span::call_site()))
                };

            let arg = if let FnArg::Regular(arg) = &value_arg {
                arg
            } else {
                bail_spanned!(value_arg.name().span() => "The #[setter] value argument can't be *args, **kwargs or `cancel_handle`.");
            };

            let extract = impl_regular_arg_param(
                arg,
                ident,
                quote!(::std::option::Option::Some(_value)),
                &mut holders,
                ctx,
            );

            quote! {
                #from_py_with
                let _val = #extract;
            }
        }
        PropertyType::Descriptor { field, .. } => {
            let span = field.ty.span();
            let name = field
                .ident
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_default();

            let holder = holders.push_holder(span);
            quote! {
                #[allow(unused_imports, reason = "`Probe` trait used on negative case only")]
                use #pyo3_path::impl_::pyclass::Probe as _;
                let _val = #pyo3_path::impl_::extract_argument::extract_argument(_value, &mut #holder, #name)?;
            }
        }
    };

    let mut cfg_attrs = TokenStream::new();
    if let PropertyType::Descriptor { field, .. } = &property_type {
        for attr in field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("cfg"))
        {
            attr.to_tokens(&mut cfg_attrs);
        }
    }

    let warnings = if let PropertyType::Function { spec, .. } = &property_type {
        spec.warnings.build_py_warning(ctx)
    } else {
        quote!()
    };

    let init_holders = holders.init_holders(ctx);
    let associated_method = quote! {
        #cfg_attrs
        unsafe fn #wrapper_ident(
            py: #pyo3_path::Python<'_>,
            _slf: *mut #pyo3_path::ffi::PyObject,
            _value: *mut #pyo3_path::ffi::PyObject,
        ) -> #pyo3_path::PyResult<::std::ffi::c_int> {
            use ::std::convert::Into;
            let _value = #pyo3_path::impl_::extract_argument::cast_function_argument(py, _value);
            #init_holders
            #extract
            #warnings
            let result = #setter_impl;
            #pyo3_path::impl_::callback::convert(py, result)
        }
    };

    let method_def = quote! {
        #cfg_attrs
        #pyo3_path::impl_::pymethods::PyMethodDefType::Setter(
            #pyo3_path::impl_::pymethods::PySetterDef::new(
                #python_name,
                #cls::#wrapper_ident,
                #doc
            )
        )
    };

    Ok(MethodAndMethodDef {
        associated_method,
        method_def,
    })
}

fn impl_call_getter(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    self_type: &SelfType,
    holders: &mut Holders,
    ctx: &Ctx,
) -> syn::Result<TokenStream> {
    let (py_arg, args) = split_off_python_arg(&spec.signature.arguments);
    let slf = self_type.receiver(cls, ExtractErrorMode::Raise, holders, ctx);
    ensure_spanned!(
        args.is_empty(),
        args[0].ty().span() => "getter function can only have one argument (of type pyo3::Python)"
    );

    let name = &spec.name;
    let fncall = if py_arg.is_some() {
        quote!(#cls::#name(#slf, py))
    } else {
        quote!(#cls::#name(#slf))
    };

    Ok(fncall)
}

// Used here for PropertyType::Function, used in pyclass for descriptors.
pub fn impl_py_getter_def(
    cls: &syn::Type,
    property_type: PropertyType<'_>,
    ctx: &Ctx,
) -> Result<MethodAndMethodDef> {
    let Ctx { pyo3_path, .. } = ctx;
    let python_name = property_type.null_terminated_python_name()?;
    let doc = property_type.doc(ctx)?;

    let mut cfg_attrs = TokenStream::new();
    if let PropertyType::Descriptor { field, .. } = &property_type {
        for attr in field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("cfg"))
        {
            attr.to_tokens(&mut cfg_attrs);
        }
    }

    let mut holders = Holders::new();
    match property_type {
        PropertyType::Descriptor {
            field_index, field, ..
        } => {
            let ty = &field.ty;
            let field = if let Some(ident) = &field.ident {
                ident.to_token_stream()
            } else {
                syn::Index::from(field_index).to_token_stream()
            };

            let generator = quote_spanned! { ty.span() =>
                GENERATOR.generate(#python_name, #doc)
            };
            // This is separate from `generator` so that the unsafe below does not inherit the span and thus does not
            // trigger the `unsafe_code` lint
            let method_def = quote! {
                #cfg_attrs
                {
                    #[allow(unused_imports, reason = "`Probe` trait used on negative case only")]
                    use #pyo3_path::impl_::pyclass::Probe as _;

                    const GENERATOR: #pyo3_path::impl_::pyclass::PyClassGetterGenerator::<
                        #cls,
                        #ty,
                        { ::std::mem::offset_of!(#cls, #field) },
                        { #pyo3_path::impl_::pyclass::IsPyT::<#ty>::VALUE },
                        { #pyo3_path::impl_::pyclass::IsIntoPyObjectRef::<#ty>::VALUE },
                        { #pyo3_path::impl_::pyclass::IsIntoPyObject::<#ty>::VALUE },
                    > = unsafe { #pyo3_path::impl_::pyclass::PyClassGetterGenerator::new() };
                    #generator
                }
            };

            Ok(MethodAndMethodDef {
                associated_method: quote! {},
                method_def,
            })
        }
        // Forward to `IntoPyCallbackOutput`, to handle `#[getter]`s returning results.
        PropertyType::Function {
            spec, self_type, ..
        } => {
            let wrapper_ident = format_ident!("__pymethod_get_{}__", spec.name);
            let call = impl_call_getter(cls, spec, self_type, &mut holders, ctx)?;
            let body = quote! {
                #pyo3_path::impl_::callback::convert(py, #call)
            };

            let init_holders = holders.init_holders(ctx);
            let warnings = spec.warnings.build_py_warning(ctx);

            let associated_method = quote! {
                #cfg_attrs
                unsafe fn #wrapper_ident(
                    py: #pyo3_path::Python<'_>,
                    _slf: *mut #pyo3_path::ffi::PyObject
                ) -> #pyo3_path::PyResult<*mut #pyo3_path::ffi::PyObject> {
                    #init_holders
                    #warnings
                    let result = #body;
                    result
                }
            };

            let method_def = quote! {
                #cfg_attrs
                #pyo3_path::impl_::pymethods::PyMethodDefType::Getter(
                    #pyo3_path::impl_::pymethods::PyGetterDef::new(
                        #python_name,
                        #cls::#wrapper_ident,
                        #doc
                    )
                )
            };

            Ok(MethodAndMethodDef {
                associated_method,
                method_def,
            })
        }
    }
}

pub fn impl_py_deleter_def(
    cls: &syn::Type,
    self_type: &SelfType,
    spec: &FnSpec<'_>,
    doc: PythonDoc,
    ctx: &Ctx,
) -> Result<MethodAndMethodDef> {
    let Ctx { pyo3_path, .. } = ctx;
    let python_name = spec.null_terminated_python_name();
    let mut holders = Holders::new();
    let deleter_impl = impl_call_deleter(cls, spec, self_type, &mut holders, ctx)?;
    let wrapper_ident = format_ident!("__pymethod_delete_{}__", spec.name);
    let warnings = spec.warnings.build_py_warning(ctx);
    let init_holders = holders.init_holders(ctx);
    let associated_method = quote! {
        unsafe fn #wrapper_ident(
            py: #pyo3_path::Python<'_>,
            _slf: *mut #pyo3_path::ffi::PyObject,
        ) -> #pyo3_path::PyResult<::std::ffi::c_int> {
            #init_holders
            #warnings
            let result = #deleter_impl;
            #pyo3_path::impl_::callback::convert(py, result)
        }
    };

    let method_def = quote! {
        #pyo3_path::impl_::pymethods::PyMethodDefType::Deleter(
            #pyo3_path::impl_::pymethods::PyDeleterDef::new(
                #python_name,
                #cls::#wrapper_ident,
                #doc
            )
        )
    };

    Ok(MethodAndMethodDef {
        associated_method,
        method_def,
    })
}

fn impl_call_deleter(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    self_type: &SelfType,
    holders: &mut Holders,
    ctx: &Ctx,
) -> Result<TokenStream> {
    let (py_arg, args) = split_off_python_arg(&spec.signature.arguments);
    let slf = self_type.receiver(cls, ExtractErrorMode::Raise, holders, ctx);

    if !args.is_empty() {
        bail_spanned!(spec.name.span() =>
            "deleter function can have at most one argument ([pyo3::Python,])"
        );
    }

    let name = &spec.name;
    let fncall = if py_arg.is_some() {
        quote!(#cls::#name(#slf, py))
    } else {
        quote!(#cls::#name(#slf))
    };

    Ok(fncall)
}

/// Split an argument of pyo3::Python from the front of the arg list, if present
fn split_off_python_arg<'a, 'b>(args: &'a [FnArg<'b>]) -> (Option<&'a PyArg<'b>>, &'a [FnArg<'b>]) {
    match args {
        [FnArg::Py(py), args @ ..] => (Some(py), args),
        args => (None, args),
    }
}

pub enum PropertyType<'a> {
    Descriptor {
        field_index: usize,
        field: &'a Field,
        python_name: Option<&'a NameAttribute>,
        renaming_rule: Option<RenamingRule>,
    },
    Function {
        self_type: &'a SelfType,
        spec: &'a FnSpec<'a>,
        doc: PythonDoc,
    },
}

impl PropertyType<'_> {
    fn null_terminated_python_name(&self) -> Result<LitCStr> {
        match self {
            PropertyType::Descriptor {
                field,
                python_name,
                renaming_rule,
                ..
            } => {
                let name = field_python_name(field, *python_name, *renaming_rule)?;
                let name = CString::new(name).unwrap();
                Ok(LitCStr::new(&name, field.span()))
            }
            PropertyType::Function { spec, .. } => Ok(spec.null_terminated_python_name()),
        }
    }

    fn doc(&self, ctx: &Ctx) -> Result<Cow<'_, PythonDoc>> {
        match self {
            PropertyType::Descriptor { field, .. } => {
                utils::get_doc(&field.attrs, None, ctx).map(Cow::Owned)
            }
            PropertyType::Function { doc, .. } => Ok(Cow::Borrowed(doc)),
        }
    }
}

pub const __NEW__: SlotDef = SlotDef::new("Py_tp_new", "newfunc");
pub const __INIT__: SlotDef = SlotDef::new("Py_tp_init", "initproc");
pub const __STR__: SlotDef = SlotDef::new("Py_tp_str", "reprfunc");
pub const __REPR__: SlotDef = SlotDef::new("Py_tp_repr", "reprfunc");
pub const __HASH__: SlotDef =
    SlotDef::new("Py_tp_hash", "hashfunc").return_conversion(TokenGenerator(
        |Ctx { pyo3_path, .. }: &Ctx| quote! { #pyo3_path::impl_::callback::HashCallbackOutput },
    ));
pub const __RICHCMP__: SlotDef = SlotDef::new("Py_tp_richcompare", "richcmpfunc")
    .extract_error_mode(ExtractErrorMode::NotImplemented);
const __GET__: SlotDef = SlotDef::new("Py_tp_descr_get", "descrgetfunc");
const __ITER__: SlotDef = SlotDef::new("Py_tp_iter", "getiterfunc");
const __NEXT__: SlotDef = SlotDef::new("Py_tp_iternext", "iternextfunc")
    .return_specialized_conversion(
        TokenGenerator(|_| quote! { IterBaseKind, IterOptionKind, IterResultOptionKind }),
        TokenGenerator(|_| quote! { iter_tag }),
    );
const __AWAIT__: SlotDef = SlotDef::new("Py_am_await", "unaryfunc");
const __AITER__: SlotDef = SlotDef::new("Py_am_aiter", "unaryfunc");
const __ANEXT__: SlotDef = SlotDef::new("Py_am_anext", "unaryfunc").return_specialized_conversion(
    TokenGenerator(
        |_| quote! { AsyncIterBaseKind, AsyncIterOptionKind, AsyncIterResultOptionKind },
    ),
    TokenGenerator(|_| quote! { async_iter_tag }),
);
pub const __LEN__: SlotDef = SlotDef::new("Py_mp_length", "lenfunc");
const __CONTAINS__: SlotDef = SlotDef::new("Py_sq_contains", "objobjproc");
const __CONCAT__: SlotDef = SlotDef::new("Py_sq_concat", "binaryfunc");
const __REPEAT__: SlotDef = SlotDef::new("Py_sq_repeat", "ssizeargfunc");
const __INPLACE_CONCAT__: SlotDef = SlotDef::new("Py_sq_concat", "binaryfunc");
const __INPLACE_REPEAT__: SlotDef = SlotDef::new("Py_sq_repeat", "ssizeargfunc");
pub const __GETITEM__: SlotDef = SlotDef::new("Py_mp_subscript", "binaryfunc");

const __POS__: SlotDef = SlotDef::new("Py_nb_positive", "unaryfunc");
const __NEG__: SlotDef = SlotDef::new("Py_nb_negative", "unaryfunc");
const __ABS__: SlotDef = SlotDef::new("Py_nb_absolute", "unaryfunc");
const __INVERT__: SlotDef = SlotDef::new("Py_nb_invert", "unaryfunc");
const __INDEX__: SlotDef = SlotDef::new("Py_nb_index", "unaryfunc");
pub const __INT__: SlotDef = SlotDef::new("Py_nb_int", "unaryfunc");
const __FLOAT__: SlotDef = SlotDef::new("Py_nb_float", "unaryfunc");
const __BOOL__: SlotDef = SlotDef::new("Py_nb_bool", "inquiry");

const __IADD__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_add");
const __ISUB__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_subtract");
const __IMUL__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_multiply");
const __IMATMUL__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_matrix_multiply");
const __ITRUEDIV__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_true_divide");
const __IFLOORDIV__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_floor_divide");
const __IMOD__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_remainder");
const __ILSHIFT__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_lshift");
const __IRSHIFT__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_rshift");
const __IAND__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_and");
const __IXOR__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_xor");
const __IOR__: SlotDef = SlotDef::binary_inplace_operator("Py_nb_inplace_or");

const __IPOW__: SlotDef = SlotDef::new("Py_nb_inplace_power", "ipowfunc")
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .return_self();

const __GETBUFFER__: SlotDef = SlotDef::new("Py_bf_getbuffer", "getbufferproc").require_unsafe();
const __RELEASEBUFFER__: SlotDef =
    SlotDef::new("Py_bf_releasebuffer", "releasebufferproc").require_unsafe();
const __CLEAR__: SlotDef = SlotDef::new("Py_tp_clear", "inquiry");

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
    fn ffi_type(self, ctx: &Ctx) -> TokenStream {
        let Ctx {
            pyo3_path,
            output_span,
        } = ctx;
        let pyo3_path = pyo3_path.to_tokens_spanned(*output_span);
        match self {
            Ty::Object | Ty::MaybeNullObject => quote! { *mut #pyo3_path::ffi::PyObject },
            Ty::NonNullObject => quote! { ::std::ptr::NonNull<#pyo3_path::ffi::PyObject> },
            Ty::IPowModulo => quote! { #pyo3_path::impl_::pymethods::IPowModulo },
            Ty::Int | Ty::CompareOp => quote! { ::std::ffi::c_int },
            Ty::PyHashT => quote! { #pyo3_path::ffi::Py_hash_t },
            Ty::PySsizeT => quote! { #pyo3_path::ffi::Py_ssize_t },
            Ty::Void => quote! { () },
            Ty::PyBuffer => quote! { *mut #pyo3_path::ffi::Py_buffer },
        }
    }

    fn extract(
        self,
        ident: &syn::Ident,
        arg: &FnArg<'_>,
        extract_error_mode: ExtractErrorMode,
        holders: &mut Holders,
        ctx: &Ctx,
    ) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        match self {
            Ty::Object => extract_object(
                extract_error_mode,
                holders,
                arg,
                REF_FROM_PTR,
                CAST_FUNCTION_ARGUMENT,
                quote! { #ident },
                ctx
            ),
            Ty::MaybeNullObject => extract_object(
                extract_error_mode,
                holders,
                arg,
                REF_FROM_PTR,
                CAST_FUNCTION_ARGUMENT,
                quote! {
                    if #ident.is_null() {
                        #pyo3_path::ffi::Py_None()
                    } else {
                        #ident
                    }
                },
                ctx
            ),
            Ty::NonNullObject => extract_object(
                extract_error_mode,
                holders,
                arg,
                REF_FROM_NON_NULL,
                CAST_NON_NULL_FUNCTION_ARGUMENT,
                quote! { #ident },
                ctx
            ),
            Ty::IPowModulo => extract_object(
                extract_error_mode,
                holders,
                arg,
                REF_FROM_PTR,
                CAST_FUNCTION_ARGUMENT,
                quote! { #ident.as_ptr() },
                ctx
            ),
            Ty::CompareOp => extract_error_mode.handle_error(
                quote! {
                    #pyo3_path::class::basic::CompareOp::from_raw(#ident)
                        .ok_or_else(|| #pyo3_path::exceptions::PyValueError::new_err("invalid comparison operator"))
                },
                ctx
            ),
            Ty::PySsizeT => {
                let ty = arg.ty();
                extract_error_mode.handle_error(
                    quote! {
                            ::std::convert::TryInto::<#ty>::try_into(#ident).map_err(|e| #pyo3_path::exceptions::PyValueError::new_err(e.to_string()))
                    },
                    ctx
                )
            }
            // Just pass other types through unmodified
            Ty::PyBuffer | Ty::Int | Ty::PyHashT | Ty::Void => quote! { #ident },
        }
    }
}

const REF_FROM_PTR: StaticIdent = StaticIdent::new("ref_from_ptr");
const REF_FROM_NON_NULL: StaticIdent = StaticIdent::new("ref_from_non_null");

const CAST_FUNCTION_ARGUMENT: StaticIdent = StaticIdent::new("cast_function_argument");
const CAST_NON_NULL_FUNCTION_ARGUMENT: StaticIdent =
    StaticIdent::new("cast_non_null_function_argument");

fn extract_object(
    extract_error_mode: ExtractErrorMode,
    holders: &mut Holders,
    arg: &FnArg<'_>,
    ref_from_method: StaticIdent,
    cast_method: StaticIdent,
    source_ptr: TokenStream,
    ctx: &Ctx,
) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;
    let name = arg.name().unraw().to_string();

    let extract = if let Some(FromPyWithAttribute {
        kw,
        value: extractor,
    }) = arg.from_py_with()
    {
        let extractor = quote_spanned! { kw.span =>
            { let from_py_with: fn(_) -> _ = #extractor; from_py_with }
        };

        quote! {
            #pyo3_path::impl_::extract_argument::from_py_with(
                unsafe { #pyo3_path::impl_::pymethods::BoundRef::#ref_from_method(py, &#source_ptr).0 },
                #name,
                #extractor,
            )
        }
    } else {
        let holder = holders.push_holder(Span::call_site());
        quote! {{
            #[allow(unused_imports, reason = "`Probe` trait used on negative case only")]
            use #pyo3_path::impl_::pyclass::Probe as _;
            #pyo3_path::impl_::extract_argument::extract_argument(
                unsafe { #pyo3_path::impl_::extract_argument::#cast_method(py, #source_ptr) },
                &mut #holder,
                #name
            )
        }}
    };

    let extracted = extract_error_mode.handle_error(extract, ctx);
    quote!(#extracted)
}

enum ReturnMode {
    ReturnSelf,
    Conversion(TokenGenerator),
    SpecializedConversion(TokenGenerator, TokenGenerator),
}

impl ReturnMode {
    fn return_call_output(&self, call: TokenStream, ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        match self {
            ReturnMode::Conversion(conversion) => {
                let conversion = TokenGeneratorCtx(*conversion, ctx);
                quote! {
                    let _result: #pyo3_path::PyResult<#conversion> = #pyo3_path::impl_::callback::convert(py, #call);
                    #pyo3_path::impl_::callback::convert(py, _result)
                }
            }
            ReturnMode::SpecializedConversion(traits, tag) => {
                let traits = TokenGeneratorCtx(*traits, ctx);
                let tag = TokenGeneratorCtx(*tag, ctx);
                quote! {
                    let _result = #call;
                    use #pyo3_path::impl_::pymethods::{#traits};
                    (&_result).#tag().convert(py, _result)
                }
            }
            ReturnMode::ReturnSelf => quote! {
                let _result: #pyo3_path::PyResult<()> = #pyo3_path::impl_::callback::convert(py, #call);
                _result?;
                #pyo3_path::ffi::Py_XINCREF(_slf);
                ::std::result::Result::Ok(_slf)
            },
        }
    }
}

pub struct SlotDef {
    slot: StaticIdent,
    func_ty: StaticIdent,
    calling_convention: SlotCallingConvention,
    ret_ty: Ty,
    extract_error_mode: ExtractErrorMode,
    return_mode: Option<ReturnMode>,
    require_unsafe: bool,
}

enum SlotCallingConvention {
    /// Specific set of arguments for the slot function
    FixedArguments(&'static [Ty]),
    /// Arbitrary arguments for `__new__` from the signature (extracted from args / kwargs)
    TpNew,
    TpInit,
}

impl SlotDef {
    const fn new(slot: &'static str, func_ty: &'static str) -> Self {
        // The FFI function pointer type determines the arguments and return type
        let (calling_convention, ret_ty) = match func_ty.as_bytes() {
            b"newfunc" => (SlotCallingConvention::TpNew, Ty::Object),
            b"initproc" => (SlotCallingConvention::TpInit, Ty::Int),
            b"reprfunc" => (SlotCallingConvention::FixedArguments(&[]), Ty::Object),
            b"hashfunc" => (SlotCallingConvention::FixedArguments(&[]), Ty::PyHashT),
            b"richcmpfunc" => (
                SlotCallingConvention::FixedArguments(&[Ty::Object, Ty::CompareOp]),
                Ty::Object,
            ),
            b"descrgetfunc" => (
                SlotCallingConvention::FixedArguments(&[Ty::MaybeNullObject, Ty::MaybeNullObject]),
                Ty::Object,
            ),
            b"getiterfunc" => (SlotCallingConvention::FixedArguments(&[]), Ty::Object),
            b"iternextfunc" => (SlotCallingConvention::FixedArguments(&[]), Ty::Object),
            b"unaryfunc" => (SlotCallingConvention::FixedArguments(&[]), Ty::Object),
            b"lenfunc" => (SlotCallingConvention::FixedArguments(&[]), Ty::PySsizeT),
            b"objobjproc" => (
                SlotCallingConvention::FixedArguments(&[Ty::Object]),
                Ty::Int,
            ),
            b"binaryfunc" => (
                SlotCallingConvention::FixedArguments(&[Ty::Object]),
                Ty::Object,
            ),
            b"inquiry" => (SlotCallingConvention::FixedArguments(&[]), Ty::Int),
            b"ssizeargfunc" => (
                SlotCallingConvention::FixedArguments(&[Ty::PySsizeT]),
                Ty::Object,
            ),
            b"getbufferproc" => (
                SlotCallingConvention::FixedArguments(&[Ty::PyBuffer, Ty::Int]),
                Ty::Int,
            ),
            b"releasebufferproc" => (
                SlotCallingConvention::FixedArguments(&[Ty::PyBuffer]),
                Ty::Void,
            ),
            b"ipowfunc" => (
                SlotCallingConvention::FixedArguments(&[Ty::Object, Ty::IPowModulo]),
                Ty::Object,
            ),
            _ => panic!("don't know calling convention for func_ty"),
        };

        SlotDef {
            slot: StaticIdent::new(slot),
            func_ty: StaticIdent::new(func_ty),
            calling_convention,
            ret_ty,
            extract_error_mode: ExtractErrorMode::Raise,
            return_mode: None,
            require_unsafe: false,
        }
    }

    /// Specialized constructor for binary inplace operators
    const fn binary_inplace_operator(slot: &'static str) -> Self {
        SlotDef::new(slot, "binaryfunc")
            .extract_error_mode(ExtractErrorMode::NotImplemented)
            .return_self()
    }

    const fn return_conversion(mut self, return_conversion: TokenGenerator) -> Self {
        self.return_mode = Some(ReturnMode::Conversion(return_conversion));
        self
    }

    const fn return_specialized_conversion(
        mut self,
        traits: TokenGenerator,
        tag: TokenGenerator,
    ) -> Self {
        self.return_mode = Some(ReturnMode::SpecializedConversion(traits, tag));
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

    pub fn generate_type_slot(
        &self,
        cls: &syn::Type,
        spec: &FnSpec<'_>,
        method_name: &str,
        ctx: &Ctx,
    ) -> Result<MethodAndSlotDef> {
        let Ctx { pyo3_path, .. } = ctx;
        let SlotDef {
            slot,
            func_ty,
            calling_convention,
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
        let wrapper_ident = format_ident!("__pymethod_{}__", method_name);
        let ret_ty = ret_ty.ffi_type(ctx);
        let mut holders = Holders::new();
        let MethodBody {
            arg_idents,
            arg_types,
            body,
        } = generate_method_body(
            cls,
            spec,
            calling_convention,
            *extract_error_mode,
            &mut holders,
            return_mode.as_ref(),
            ctx,
        )?;
        let name = spec.name;
        let holders = holders.init_holders(ctx);
        let associated_method = quote! {
            #[allow(non_snake_case)]
            unsafe fn #wrapper_ident(
                py: #pyo3_path::Python<'_>,
                #(#arg_idents: #arg_types),*
            ) -> #pyo3_path::PyResult<#ret_ty> {
                let function = #cls::#name; // Shadow the method name to avoid #3017
                #holders
                #body
            }
        };
        let slot_def = quote! {
            #pyo3_path::ffi::PyType_Slot {
                slot: #pyo3_path::ffi::#slot,
                pfunc: #pyo3_path::impl_::trampoline::get_trampoline_function!(#func_ty, #cls::#wrapper_ident) as #pyo3_path::ffi::#func_ty as _
            }
        };
        Ok(MethodAndSlotDef {
            associated_method,
            slot_def,
        })
    }
}

fn generate_method_body(
    cls: &syn::Type,
    spec: &FnSpec<'_>,
    calling_convention: &SlotCallingConvention,
    extract_error_mode: ExtractErrorMode,
    holders: &mut Holders,
    // NB ignored if calling_convention is SlotCallingConvention::TpNew, possibly should merge into that enum
    return_mode: Option<&ReturnMode>,
    ctx: &Ctx,
) -> Result<MethodBody> {
    let Ctx {
        pyo3_path,
        output_span,
    } = ctx;
    let self_arg = spec
        .tp
        .self_arg(Some(cls), extract_error_mode, holders, ctx);
    let rust_name = spec.name;
    let warnings = spec.warnings.build_py_warning(ctx);

    let (arg_idents, arg_types, body) = match calling_convention {
        SlotCallingConvention::TpNew => {
            let arg_idents = vec![
                format_ident!("_slf"),
                format_ident!("_args"),
                format_ident!("_kwargs"),
            ];
            let arg_types = vec![
                quote! { *mut #pyo3_path::ffi::PyTypeObject },
                quote! { *mut #pyo3_path::ffi::PyObject },
                quote! { *mut #pyo3_path::ffi::PyObject },
            ];
            let (arg_convert, args) = impl_arg_params(spec, Some(cls), false, holders, ctx);
            let args = self_arg.into_iter().chain(args);
            let call = quote_spanned! {*output_span=> #cls::#rust_name(#(#args),*) };

            // Use just the text_signature_call_signature() because the class' Python name
            // isn't known to `#[pymethods]` - that has to be attached at runtime from the PyClassImpl
            // trait implementation created by `#[pyclass]`.
            let text_signature_impl = spec.text_signature_call_signature().map(|text_signature| {
                quote! {
                    #[allow(unknown_lints, non_local_definitions)]
                    impl #pyo3_path::impl_::pyclass::doc::PyClassNewTextSignature for #cls {
                        const TEXT_SIGNATURE: &'static str = #text_signature;
                    }
                }
            });

            let output = if let syn::ReturnType::Type(_, ty) = &spec.output {
                ty
            } else {
                &parse_quote!(())
            };
            let body = quote! {
                #text_signature_impl

                use #pyo3_path::impl_::pyclass::Probe as _;
                #warnings
                #arg_convert
                let result = #call;
                #pyo3_path::impl_::pymethods::tp_new_impl::<
                    _,
                    { #pyo3_path::impl_::pyclass::IsPyClass::<#output>::VALUE },
                    { #pyo3_path::impl_::pyclass::IsInitializerTuple::<#output>::VALUE }
                >(py, result, _slf)
            };
            (arg_idents, arg_types, body)
        }
        SlotCallingConvention::TpInit => {
            let arg_idents = vec![
                format_ident!("_slf"),
                format_ident!("_args"),
                format_ident!("_kwargs"),
            ];
            let arg_types = vec![
                quote! { *mut #pyo3_path::ffi::PyObject },
                quote! { *mut #pyo3_path::ffi::PyObject },
                quote! { *mut #pyo3_path::ffi::PyObject },
            ];
            let (arg_convert, args) = impl_arg_params(spec, Some(cls), false, holders, ctx);
            let args = self_arg.into_iter().chain(args);
            let call = quote! {{
                let r = #cls::#rust_name(#(#args),*);
                #pyo3_path::impl_::wrap::converter(&r)
                    .wrap(r)
                    .map_err(::core::convert::Into::<#pyo3_path::PyErr>::into)?
            }};
            let output = quote_spanned! { *output_span => result.convert(py) };

            let body = quote! {
                use #pyo3_path::impl_::callback::IntoPyCallbackOutput;
                #warnings
                #arg_convert
                let result = #call;
                #output
            };
            (arg_idents, arg_types, body)
        }
        SlotCallingConvention::FixedArguments(arguments) => {
            let arg_idents: Vec<_> = std::iter::once(format_ident!("_slf"))
                .chain((0..arguments.len()).map(|i| format_ident!("arg{}", i)))
                .collect();
            let arg_types: Vec<_> = std::iter::once(quote! { *mut #pyo3_path::ffi::PyObject })
                .chain(arguments.iter().map(|arg| arg.ffi_type(ctx)))
                .collect();

            let args = extract_proto_arguments(spec, arguments, extract_error_mode, holders, ctx)?;
            let args = self_arg.into_iter().chain(args);
            let call = quote! { #cls::#rust_name(#(#args),*) };
            let result = if let Some(return_mode) = return_mode {
                return_mode.return_call_output(call, ctx)
            } else {
                quote! {
                    let result = #call;
                    #pyo3_path::impl_::callback::convert(py, result)
                }
            };
            let body = quote! {
                #warnings
                #result
            };
            (arg_idents, arg_types, body)
        }
    };

    Ok(MethodBody {
        arg_idents,
        arg_types,
        body,
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

    /// Specialized constructor for binary operators (which are a common pattern)
    const fn binary_operator(fragment: &'static str) -> Self {
        SlotFragmentDef {
            fragment,
            arguments: &[Ty::Object],
            extract_error_mode: ExtractErrorMode::NotImplemented,
            ret_ty: Ty::Object,
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

    fn generate_pyproto_fragment(
        &self,
        cls: &syn::Type,
        spec: &FnSpec<'_>,
        ctx: &Ctx,
    ) -> Result<TokenStream> {
        let Ctx { pyo3_path, .. } = ctx;
        let SlotFragmentDef {
            fragment,
            arguments,
            extract_error_mode,
            ret_ty,
        } = self;
        let fragment_trait = format_ident!("PyClass{}SlotFragment", fragment);
        let method = syn::Ident::new(fragment, Span::call_site());
        let wrapper_ident = format_ident!("__pymethod_{}__", fragment);

        let mut holders = Holders::new();
        let MethodBody {
            arg_idents,
            arg_types,
            body,
        } = generate_method_body(
            cls,
            spec,
            &SlotCallingConvention::FixedArguments(arguments),
            *extract_error_mode,
            &mut holders,
            None,
            ctx,
        )?;
        let ret_ty = ret_ty.ffi_type(ctx);
        let holders = holders.init_holders(ctx);
        Ok(quote! {
            impl #cls {
                #[allow(non_snake_case)]
                unsafe fn #wrapper_ident(
                    py: #pyo3_path::Python,
                    #(#arg_idents: #arg_types),*
                ) -> #pyo3_path::PyResult<#ret_ty> {
                    #holders
                    #body
                }
            }

            impl #pyo3_path::impl_::pyclass::#fragment_trait<#cls> for #pyo3_path::impl_::pyclass::PyClassImplCollector<#cls> {

                #[inline]
                unsafe fn #method(
                    self,
                    py: #pyo3_path::Python,
                    #(#arg_idents: #arg_types),*
                ) -> #pyo3_path::PyResult<#ret_ty> {
                    #cls::#wrapper_ident(py, #(#arg_idents),*)
                }
            }
        })
    }
}

/// The reusable components of a method body.
pub struct MethodBody {
    pub arg_idents: Vec<Ident>,
    pub arg_types: Vec<TokenStream>,
    pub body: TokenStream,
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

const __ADD__: SlotFragmentDef = SlotFragmentDef::binary_operator("__add__");
const __RADD__: SlotFragmentDef = SlotFragmentDef::binary_operator("__radd__");
const __SUB__: SlotFragmentDef = SlotFragmentDef::binary_operator("__sub__");
const __RSUB__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rsub__");
const __MUL__: SlotFragmentDef = SlotFragmentDef::binary_operator("__mul__");
const __RMUL__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rmul__");
const __MATMUL__: SlotFragmentDef = SlotFragmentDef::binary_operator("__matmul__");
const __RMATMUL__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rmatmul__");
const __FLOORDIV__: SlotFragmentDef = SlotFragmentDef::binary_operator("__floordiv__");
const __RFLOORDIV__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rfloordiv__");
const __TRUEDIV__: SlotFragmentDef = SlotFragmentDef::binary_operator("__truediv__");
const __RTRUEDIV__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rtruediv__");
const __DIVMOD__: SlotFragmentDef = SlotFragmentDef::binary_operator("__divmod__");
const __RDIVMOD__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rdivmod__");
const __MOD__: SlotFragmentDef = SlotFragmentDef::binary_operator("__mod__");
const __RMOD__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rmod__");
const __LSHIFT__: SlotFragmentDef = SlotFragmentDef::binary_operator("__lshift__");
const __RLSHIFT__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rlshift__");
const __RSHIFT__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rshift__");
const __RRSHIFT__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rrshift__");
const __AND__: SlotFragmentDef = SlotFragmentDef::binary_operator("__and__");
const __RAND__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rand__");
const __XOR__: SlotFragmentDef = SlotFragmentDef::binary_operator("__xor__");
const __RXOR__: SlotFragmentDef = SlotFragmentDef::binary_operator("__rxor__");
const __OR__: SlotFragmentDef = SlotFragmentDef::binary_operator("__or__");
const __ROR__: SlotFragmentDef = SlotFragmentDef::binary_operator("__ror__");

const __POW__: SlotFragmentDef = SlotFragmentDef::new("__pow__", &[Ty::Object, Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .ret_ty(Ty::Object);
const __RPOW__: SlotFragmentDef = SlotFragmentDef::new("__rpow__", &[Ty::Object, Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .ret_ty(Ty::Object);

const __LT__: SlotFragmentDef = SlotFragmentDef::new("__lt__", &[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .ret_ty(Ty::Object);
const __LE__: SlotFragmentDef = SlotFragmentDef::new("__le__", &[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .ret_ty(Ty::Object);
const __EQ__: SlotFragmentDef = SlotFragmentDef::new("__eq__", &[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .ret_ty(Ty::Object);
const __NE__: SlotFragmentDef = SlotFragmentDef::new("__ne__", &[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .ret_ty(Ty::Object);
const __GT__: SlotFragmentDef = SlotFragmentDef::new("__gt__", &[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .ret_ty(Ty::Object);
const __GE__: SlotFragmentDef = SlotFragmentDef::new("__ge__", &[Ty::Object])
    .extract_error_mode(ExtractErrorMode::NotImplemented)
    .ret_ty(Ty::Object);

fn extract_proto_arguments(
    spec: &FnSpec<'_>,
    proto_args: &[Ty],
    extract_error_mode: ExtractErrorMode,
    holders: &mut Holders,
    ctx: &Ctx,
) -> Result<Vec<TokenStream>> {
    let mut args = Vec::with_capacity(spec.signature.arguments.len());
    let mut non_python_args = 0;

    for arg in &spec.signature.arguments {
        if let FnArg::Py(..) = arg {
            args.push(quote! { py });
        } else {
            let ident = syn::Ident::new(&format!("arg{non_python_args}"), Span::call_site());
            let conversions = proto_args.get(non_python_args)
                .ok_or_else(|| err_spanned!(arg.ty().span() => format!("Expected at most {} non-python arguments", proto_args.len())))?
                .extract(&ident, arg, extract_error_mode, holders, ctx);
            non_python_args += 1;
            args.push(conversions);
        }
    }

    if non_python_args != proto_args.len() {
        bail_spanned!(spec.name.span() => format!("Expected {} arguments, got {}", proto_args.len(), non_python_args));
    }
    Ok(args)
}

#[derive(Clone, Copy)]
struct TokenGenerator(fn(&Ctx) -> TokenStream);

struct TokenGeneratorCtx<'ctx>(TokenGenerator, &'ctx Ctx);

impl ToTokens for TokenGeneratorCtx<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self(TokenGenerator(gen), ctx) = self;
        (gen)(ctx).to_tokens(tokens)
    }
}

pub fn field_python_name(
    field: &Field,
    name_attr: Option<&NameAttribute>,
    renaming_rule: Option<RenamingRule>,
) -> Result<String> {
    if let Some(name_attr) = name_attr {
        return Ok(name_attr.value.0.to_string());
    }
    let Some(ident) = &field.ident else {
        bail_spanned!(field.span() => "`get` and `set` with tuple struct fields require `name`");
    };
    let mut name = ident.unraw().to_string();
    if let Some(rule) = renaming_rule {
        name = utils::apply_renaming_rule(rule, &name);
    }
    Ok(name)
}
