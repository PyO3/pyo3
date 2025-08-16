use std::collections::HashSet;

use crate::combine_errors::CombineErrors;
#[cfg(feature = "experimental-inspect")]
use crate::introspection::{attribute_introspection_code, function_introspection_code};
#[cfg(feature = "experimental-inspect")]
use crate::method::{FnSpec, FnType};
#[cfg(feature = "experimental-inspect")]
use crate::utils::expr_to_python;
use crate::utils::{has_attribute, has_attribute_with_namespace, Ctx, PyO3CratePath};
use crate::{
    attributes::{take_pyo3_options, CrateAttribute},
    konst::{ConstAttributes, ConstSpec},
    pyfunction::PyFunctionOptions,
    pymethod::{
        self, is_proto_method, GeneratedPyMethod, MethodAndMethodDef, MethodAndSlotDef, PyMethod,
    },
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
#[cfg(feature = "experimental-inspect")]
use syn::Ident;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    ImplItemFn, Result,
};

/// The mechanism used to collect `#[pymethods]` into the type object
#[derive(Copy, Clone)]
pub enum PyClassMethodsType {
    Specialization,
    Inventory,
}

enum PyImplPyO3Option {
    Crate(CrateAttribute),
}

impl Parse for PyImplPyO3Option {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Token![crate]) {
            input.parse().map(PyImplPyO3Option::Crate)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Default)]
pub struct PyImplOptions {
    krate: Option<CrateAttribute>,
}

impl PyImplOptions {
    pub fn from_attrs(attrs: &mut Vec<syn::Attribute>) -> Result<Self> {
        let mut options: PyImplOptions = Default::default();

        for option in take_pyo3_options(attrs)? {
            match option {
                PyImplPyO3Option::Crate(path) => options.set_crate(path)?,
            }
        }

        Ok(options)
    }

    fn set_crate(&mut self, path: CrateAttribute) -> Result<()> {
        ensure_spanned!(
            self.krate.is_none(),
            path.span() => "`crate` may only be specified once"
        );

        self.krate = Some(path);
        Ok(())
    }
}

pub fn build_py_methods(
    ast: &mut syn::ItemImpl,
    methods_type: PyClassMethodsType,
) -> syn::Result<TokenStream> {
    if let Some((_, path, _)) = &ast.trait_ {
        bail_spanned!(path.span() => "#[pymethods] cannot be used on trait impl blocks");
    } else if ast.generics != Default::default() {
        bail_spanned!(
            ast.generics.span() =>
            "#[pymethods] cannot be used with lifetime parameters or generics"
        );
    } else {
        let options = PyImplOptions::from_attrs(&mut ast.attrs)?;
        impl_methods(&ast.self_ty, &mut ast.items, methods_type, options)
    }
}

fn check_pyfunction(pyo3_path: &PyO3CratePath, meth: &mut ImplItemFn) -> syn::Result<()> {
    let mut error = None;

    meth.attrs.retain(|attr| {
        let attrs = [attr.clone()];

        if has_attribute(&attrs, "pyfunction")
            || has_attribute_with_namespace(&attrs, Some(pyo3_path),  &["pyfunction"])
            || has_attribute_with_namespace(&attrs, Some(pyo3_path),  &["prelude", "pyfunction"]) {
                error = Some(err_spanned!(meth.sig.span() => "functions inside #[pymethods] do not need to be annotated with #[pyfunction]"));
                false
        } else {
            true
        }
    });

    error.map_or(Ok(()), Err)
}

pub fn impl_methods(
    ty: &syn::Type,
    impls: &mut [syn::ImplItem],
    methods_type: PyClassMethodsType,
    options: PyImplOptions,
) -> syn::Result<TokenStream> {
    let mut extra_fragments = Vec::new();
    let mut proto_impls = Vec::new();
    let mut methods = Vec::new();
    let mut associated_methods = Vec::new();

    let mut implemented_proto_fragments = HashSet::new();

    let _: Vec<()> = impls
        .iter_mut()
        .map(|iimpl| {
            match iimpl {
                syn::ImplItem::Fn(meth) => {
                    let ctx = &Ctx::new(&options.krate, Some(&meth.sig));
                    let mut fun_options = PyFunctionOptions::from_attrs(&mut meth.attrs)?;
                    fun_options.krate = fun_options.krate.or_else(|| options.krate.clone());

                    check_pyfunction(&ctx.pyo3_path, meth)?;
                    let method = PyMethod::parse(&mut meth.sig, &mut meth.attrs, fun_options)?;
                    #[cfg(feature = "experimental-inspect")]
                    extra_fragments.push(method_introspection_code(&method.spec, ty, ctx));
                    match pymethod::gen_py_method(ty, method, &meth.attrs, ctx)? {
                        GeneratedPyMethod::Method(MethodAndMethodDef {
                            associated_method,
                            method_def,
                        }) => {
                            let attrs = get_cfg_attributes(&meth.attrs);
                            associated_methods.push(quote!(#(#attrs)* #associated_method));
                            methods.push(quote!(#(#attrs)* #method_def));
                        }
                        GeneratedPyMethod::SlotTraitImpl(method_name, token_stream) => {
                            implemented_proto_fragments.insert(method_name);
                            let attrs = get_cfg_attributes(&meth.attrs);
                            extra_fragments.push(quote!(#(#attrs)* #token_stream));
                        }
                        GeneratedPyMethod::Proto(MethodAndSlotDef {
                            associated_method,
                            slot_def,
                        }) => {
                            let attrs = get_cfg_attributes(&meth.attrs);
                            proto_impls.push(quote!(#(#attrs)* #slot_def));
                            associated_methods.push(quote!(#(#attrs)* #associated_method));
                        }
                    }
                }
                syn::ImplItem::Const(konst) => {
                    let ctx = &Ctx::new(&options.krate, None);
                    let attributes = ConstAttributes::from_attrs(&mut konst.attrs)?;
                    if attributes.is_class_attr {
                        let spec = ConstSpec {
                            rust_ident: konst.ident.clone(),
                            attributes,
                        };
                        let attrs = get_cfg_attributes(&konst.attrs);
                        let MethodAndMethodDef {
                            associated_method,
                            method_def,
                        } = gen_py_const(ty, &spec, ctx);
                        methods.push(quote!(#(#attrs)* #method_def));
                        associated_methods.push(quote!(#(#attrs)* #associated_method));
                        if is_proto_method(&spec.python_name().to_string()) {
                            // If this is a known protocol method e.g. __contains__, then allow this
                            // symbol even though it's not an uppercase constant.
                            konst
                                .attrs
                                .push(syn::parse_quote!(#[allow(non_upper_case_globals)]));
                        }
                        #[cfg(feature = "experimental-inspect")]
                        extra_fragments.push(attribute_introspection_code(
                            &ctx.pyo3_path,
                            Some(ty),
                            spec.python_name().to_string(),
                            expr_to_python(&konst.expr),
                            konst.ty.clone(),
                            true,
                        ));
                    }
                }
                syn::ImplItem::Macro(m) => bail_spanned!(
                    m.span() =>
                    "macros cannot be used as items in `#[pymethods]` impl blocks\n\
                    = note: this was previously accepted and ignored"
                ),
                _ => {}
            }
            Ok(())
        })
        .try_combine_syn_errors()?;

    let ctx = &Ctx::new(&options.krate, None);

    add_shared_proto_slots(ty, &mut proto_impls, implemented_proto_fragments, ctx);

    let items = match methods_type {
        PyClassMethodsType::Specialization => impl_py_methods(ty, methods, proto_impls, ctx),
        PyClassMethodsType::Inventory => submit_methods_inventory(ty, methods, proto_impls, ctx),
    };

    Ok(quote! {
        #(#extra_fragments)*

        #items

        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl #ty {
            #(#associated_methods)*
        }
    })
}

pub fn gen_py_const(cls: &syn::Type, spec: &ConstSpec, ctx: &Ctx) -> MethodAndMethodDef {
    let member = &spec.rust_ident;
    let wrapper_ident = format_ident!("__pymethod_{}__", member);
    let python_name = spec.null_terminated_python_name(ctx);
    let Ctx { pyo3_path, .. } = ctx;

    let associated_method = quote! {
        fn #wrapper_ident(py: #pyo3_path::Python<'_>) -> #pyo3_path::PyResult<#pyo3_path::Py<#pyo3_path::PyAny>> {
            #pyo3_path::IntoPyObjectExt::into_py_any(#cls::#member, py)
        }
    };

    let method_def = quote! {
        #pyo3_path::impl_::pyclass::MaybeRuntimePyMethodDef::Static(
            #pyo3_path::impl_::pymethods::PyMethodDefType::ClassAttribute({
                #pyo3_path::impl_::pymethods::PyClassAttributeDef::new(
                    #python_name,
                    #cls::#wrapper_ident
                )
            })
        )
    };

    MethodAndMethodDef {
        associated_method,
        method_def,
    }
}

fn impl_py_methods(
    ty: &syn::Type,
    methods: Vec<TokenStream>,
    proto_impls: Vec<TokenStream>,
    ctx: &Ctx,
) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;
    quote! {
        #[allow(unknown_lints, non_local_definitions)]
        impl #pyo3_path::impl_::pyclass::PyMethods<#ty>
            for #pyo3_path::impl_::pyclass::PyClassImplCollector<#ty>
        {
            fn py_methods(self) -> &'static #pyo3_path::impl_::pyclass::PyClassItems {
                static ITEMS: #pyo3_path::impl_::pyclass::PyClassItems = #pyo3_path::impl_::pyclass::PyClassItems {
                    methods: &[#(#methods),*],
                    slots: &[#(#proto_impls),*]
                };
                &ITEMS
            }
        }
    }
}

fn add_shared_proto_slots(
    ty: &syn::Type,
    proto_impls: &mut Vec<TokenStream>,
    mut implemented_proto_fragments: HashSet<String>,
    ctx: &Ctx,
) {
    let Ctx { pyo3_path, .. } = ctx;
    macro_rules! try_add_shared_slot {
        ($slot:ident, $($fragments:literal),*) => {{
            let mut implemented = false;
            $(implemented |= implemented_proto_fragments.remove($fragments));*;
            if implemented {
                proto_impls.push(quote! { #pyo3_path::impl_::pyclass::$slot!(#ty) })
            }
        }};
    }

    try_add_shared_slot!(
        generate_pyclass_getattro_slot,
        "__getattribute__",
        "__getattr__"
    );
    try_add_shared_slot!(generate_pyclass_setattr_slot, "__setattr__", "__delattr__");
    try_add_shared_slot!(generate_pyclass_setdescr_slot, "__set__", "__delete__");
    try_add_shared_slot!(generate_pyclass_setitem_slot, "__setitem__", "__delitem__");
    try_add_shared_slot!(generate_pyclass_add_slot, "__add__", "__radd__");
    try_add_shared_slot!(generate_pyclass_sub_slot, "__sub__", "__rsub__");
    try_add_shared_slot!(generate_pyclass_mul_slot, "__mul__", "__rmul__");
    try_add_shared_slot!(generate_pyclass_mod_slot, "__mod__", "__rmod__");
    try_add_shared_slot!(generate_pyclass_divmod_slot, "__divmod__", "__rdivmod__");
    try_add_shared_slot!(generate_pyclass_lshift_slot, "__lshift__", "__rlshift__");
    try_add_shared_slot!(generate_pyclass_rshift_slot, "__rshift__", "__rrshift__");
    try_add_shared_slot!(generate_pyclass_and_slot, "__and__", "__rand__");
    try_add_shared_slot!(generate_pyclass_or_slot, "__or__", "__ror__");
    try_add_shared_slot!(generate_pyclass_xor_slot, "__xor__", "__rxor__");
    try_add_shared_slot!(generate_pyclass_matmul_slot, "__matmul__", "__rmatmul__");
    try_add_shared_slot!(generate_pyclass_truediv_slot, "__truediv__", "__rtruediv__");
    try_add_shared_slot!(
        generate_pyclass_floordiv_slot,
        "__floordiv__",
        "__rfloordiv__"
    );
    try_add_shared_slot!(generate_pyclass_pow_slot, "__pow__", "__rpow__");
    try_add_shared_slot!(
        generate_pyclass_richcompare_slot,
        "__lt__",
        "__le__",
        "__eq__",
        "__ne__",
        "__gt__",
        "__ge__"
    );

    // if this assertion trips, a slot fragment has been implemented which has not been added in the
    // list above
    assert!(implemented_proto_fragments.is_empty());
}

fn submit_methods_inventory(
    ty: &syn::Type,
    methods: Vec<TokenStream>,
    proto_impls: Vec<TokenStream>,
    ctx: &Ctx,
) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;
    quote! {
        #pyo3_path::inventory::submit! {
            type Inventory = <#ty as #pyo3_path::impl_::pyclass::PyClassImpl>::Inventory;
            Inventory::new(#pyo3_path::impl_::pyclass::PyClassItems { methods: &[#(#methods),*], slots: &[#(#proto_impls),*] })
        }
    }
}

pub(crate) fn get_cfg_attributes(attrs: &[syn::Attribute]) -> Vec<&syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .collect()
}

#[cfg(feature = "experimental-inspect")]
fn method_introspection_code(spec: &FnSpec<'_>, parent: &syn::Type, ctx: &Ctx) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;

    let name = spec.python_name.to_string();

    // __richcmp__ special case
    if name == "__richcmp__" {
        // We expend into each individual method
        return ["__eq__", "__ne__", "__lt__", "__le__", "__gt__", "__ge__"]
            .into_iter()
            .map(|method_name| {
                let mut spec = (*spec).clone();
                spec.python_name = Ident::new(method_name, spec.python_name.span());
                // We remove the CompareOp arg, this is safe because the signature is always the same
                // First the other value to compare with then the CompareOp
                // We cant to keep the first argument type, hence this hack
                spec.signature.arguments.pop();
                spec.signature.python_signature.positional_parameters.pop();
                method_introspection_code(&spec, parent, ctx)
            })
            .collect();
    }
    // We map or ignore some magic methods
    // TODO: this might create a naming conflict
    let name = match name.as_str() {
        "__concat__" => "__add__".into(),
        "__repeat__" => "__mul__".into(),
        "__inplace_concat__" => "__iadd__".into(),
        "__inplace_repeat__" => "__imul__".into(),
        "__getbuffer__" | "__releasebuffer__" | "__traverse__" | "__clear__" => return quote! {},
        _ => name,
    };

    // We introduce self/cls argument and setup decorators
    let mut first_argument = None;
    let mut output = spec.output.clone();
    let mut decorators = Vec::new();
    match &spec.tp {
        FnType::Getter(_) => {
            first_argument = Some("self");
            decorators.push("property".into());
        }
        FnType::Setter(_) => {
            first_argument = Some("self");
            decorators.push(format!("{name}.setter"));
        }
        FnType::Fn(_) => {
            first_argument = Some("self");
        }
        FnType::FnNew | FnType::FnNewClass(_) => {
            first_argument = Some("cls");
            output = syn::ReturnType::Default; // The __new__ Python function return type is None
        }
        FnType::FnClass(_) => {
            first_argument = Some("cls");
            decorators.push("classmethod".into());
        }
        FnType::FnStatic => {
            decorators.push("staticmethod".into());
        }
        FnType::FnModule(_) => (), // TODO: not sure this can happen
        FnType::ClassAttribute => {
            first_argument = Some("cls");
            // TODO: this combination only works with Python 3.9-3.11 https://docs.python.org/3.11/library/functions.html#classmethod
            decorators.push("classmethod".into());
            decorators.push("property".into());
        }
    }
    function_introspection_code(
        pyo3_path,
        None,
        &name,
        &spec.signature,
        first_argument,
        output,
        decorators,
        Some(parent),
    )
}
