//! Code generation for the function that initializes a python module and adds classes and function.

#[cfg(feature = "experimental-module-state")]
use crate::attributes::StateAttribute;
#[cfg(feature = "experimental-inspect")]
use crate::introspection::{
    attribute_introspection_code, introspection_id_const, module_introspection_code,
};
#[cfg(feature = "experimental-inspect")]
use crate::py_expr::PyExpr;
use crate::{
    attributes::{
        self, kw, take_attributes, take_pyo3_options, CrateAttribute, GILUsedAttribute,
        ModuleAttribute, NameAttribute, SubmoduleAttribute,
    },
    combine_errors::CombineErrors,
    get_doc,
    pyclass::PyClassPyO3Option,
    pyfunction::{impl_wrap_pyfunction, PyFunctionOptions},
    utils::{has_attribute, has_attribute_with_namespace, Ctx, IdentOrStr, PythonDoc},
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::ffi::CString;
use syn::LitCStr;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_quote, parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Item, Meta, Path, Result,
};

#[derive(Default)]
pub struct PyModuleOptions {
    krate: Option<CrateAttribute>,
    name: Option<NameAttribute>,
    module: Option<ModuleAttribute>,
    submodule: Option<kw::submodule>,
    gil_used: Option<GILUsedAttribute>,
    #[cfg(feature = "experimental-module-state")]
    state: Option<StateAttribute>,
}

impl Parse for PyModuleOptions {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut options: PyModuleOptions = Default::default();

        options.add_attributes(
            Punctuated::<PyModulePyO3Option, syn::Token![,]>::parse_terminated(input)?,
        )?;

        Ok(options)
    }
}

impl PyModuleOptions {
    fn take_pyo3_options(&mut self, attrs: &mut Vec<syn::Attribute>) -> Result<()> {
        self.add_attributes(take_pyo3_options(attrs)?)
    }

    fn add_attributes(
        &mut self,
        attrs: impl IntoIterator<Item = PyModulePyO3Option>,
    ) -> Result<()> {
        macro_rules! set_option {
            ($key:ident $(, $extra:literal)?) => {
                {
                    ensure_spanned!(
                        self.$key.is_none(),
                        $key.span() => concat!("`", stringify!($key), "` may only be specified once" $(, $extra)?)
                    );
                    self.$key = Some($key);
                }
            };
        }
        attrs
            .into_iter()
            .map(|attr| {
                match attr {
                    PyModulePyO3Option::Crate(krate) => set_option!(krate),
                    PyModulePyO3Option::Name(name) => set_option!(name),
                    PyModulePyO3Option::Module(module) => set_option!(module),
                    PyModulePyO3Option::Submodule(submodule) => set_option!(
                        submodule,
                        " (it is implicitly always specified for nested modules)"
                    ),
                    PyModulePyO3Option::GILUsed(gil_used) => {
                        set_option!(gil_used)
                    }
                    #[cfg(feature = "experimental-module-state")]
                    PyModulePyO3Option::State(state) => {
                        set_option!(state)
                    }
                }

                Ok(())
            })
            .try_combine_syn_errors()?;
        Ok(())
    }
}

pub fn pymodule_module_impl(
    module: &mut syn::ItemMod,
    mut options: PyModuleOptions,
) -> Result<TokenStream> {
    let syn::ItemMod {
        attrs,
        vis,
        unsafety: _,
        ident,
        mod_token,
        content,
        semi: _,
    } = module;
    let items = if let Some((_, items)) = content {
        items
    } else {
        bail_spanned!(mod_token.span() => "`#[pymodule]` can only be used on inline modules")
    };
    options.take_pyo3_options(attrs)?;
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = ctx;
    let doc = get_doc(attrs, None);
    let name = options
        .name
        .map_or_else(|| ident.unraw(), |name| name.value.0);
    let full_name = if let Some(module) = &options.module {
        format!("{}.{}", module.value.value(), name)
    } else {
        name.to_string()
    };

    let mut module_items = Vec::new();
    let mut module_items_cfg_attrs = Vec::new();
    #[cfg(feature = "experimental-inspect")]
    let mut introspection_chunks = Vec::new();
    #[cfg(not(feature = "experimental-inspect"))]
    let introspection_chunks = Vec::<TokenStream>::new();

    fn extract_use_items(
        source: &syn::UseTree,
        cfg_attrs: &[syn::Attribute],
        target_items: &mut Vec<syn::Ident>,
        target_cfg_attrs: &mut Vec<Vec<syn::Attribute>>,
    ) -> Result<()> {
        match source {
            syn::UseTree::Name(name) => {
                target_items.push(name.ident.clone());
                target_cfg_attrs.push(cfg_attrs.to_vec());
            }
            syn::UseTree::Path(path) => {
                extract_use_items(&path.tree, cfg_attrs, target_items, target_cfg_attrs)?
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    extract_use_items(tree, cfg_attrs, target_items, target_cfg_attrs)?
                }
            }
            syn::UseTree::Glob(glob) => {
                bail_spanned!(glob.span() => "#[pymodule] cannot import glob statements")
            }
            syn::UseTree::Rename(rename) => {
                target_items.push(rename.rename.clone());
                target_cfg_attrs.push(cfg_attrs.to_vec());
            }
        }
        Ok(())
    }

    #[cfg(feature = "experimental-module-state")]
    let mut state_type: Option<syn::Path> = None;
    #[cfg(feature = "experimental-module-state")]
    if let Some(explicit_state) = &options.state {
        state_type = Some(explicit_state.value.clone());
    }

    let mut pymodule_init: Option<TokenStream> = None;
    #[cfg(feature = "experimental-module-state")]
    let mut pymodule_traverse: Option<TokenStream> = None;
    #[cfg(not(feature = "experimental-module-state"))]
    let pymodule_traverse: Option<TokenStream> = None;
    #[cfg(feature = "experimental-module-state")]
    let mut pymodule_clear: Option<TokenStream> = None;
    #[cfg(not(feature = "experimental-module-state"))]
    let pymodule_clear: Option<TokenStream> = None;
    let mut module_consts = Vec::new();
    let mut module_consts_cfg_attrs = Vec::new();

    let _: Vec<()> = (*items).iter_mut().map(|item|{
        match item {
            Item::Use(item_use) => {
                let is_pymodule_export =
                    find_and_remove_attribute(&mut item_use.attrs, "pymodule_export");
                if is_pymodule_export {
                    let cfg_attrs = get_cfg_attributes(&item_use.attrs);
                    extract_use_items(
                        &item_use.tree,
                        &cfg_attrs,
                        &mut module_items,
                        &mut module_items_cfg_attrs,
                    )?;
                }
            }
            Item::Fn(item_fn) => {
                ensure_spanned!(
                    !has_attribute(&item_fn.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
                let is_pymodule_init =
                    find_and_remove_attribute(&mut item_fn.attrs, "pymodule_init");
                #[cfg(feature = "experimental-module-state")]
                let is_pymodule_traverse =
                find_and_remove_attribute(&mut item_fn.attrs, "pymodule_traverse");
                #[cfg(feature = "experimental-module-state")]
                let is_pymodule_clear =
                    find_and_remove_attribute(&mut item_fn.attrs, "pymodule_clear");
                let ident = &item_fn.sig.ident;
                #[cfg(feature = "experimental-module-state")]
                if is_pymodule_traverse {
                    ensure_spanned!(
                        !has_attribute(&item_fn.attrs, "pyfunction"),
                        item_fn.span() => "`#[pyfunction]` cannot be used alongside `#[pymodule_traverse]`"
                    );
                    ensure_spanned!(pymodule_traverse.is_none(), item_fn.span() => "only one `#[pymodule_traverse]` may be specified");
                    pymodule_traverse = Some(quote! { #ident });
                } else if is_pymodule_clear {
                    ensure_spanned!(
                        !has_attribute(&item_fn.attrs, "pyfunction"),
                        item_fn.span() => "`#[pyfunction]` cannot be used alongside `#[pymodule_clear]`"
                    );
                    ensure_spanned!(pymodule_clear.is_none(), item_fn.span() => "only one `#[pymodule_clear]` may be specified");
                    pymodule_clear = Some(quote! { #ident })
                }

                if is_pymodule_init {
                    ensure_spanned!(
                        !has_attribute(&item_fn.attrs, "pyfunction"),
                        item_fn.span() => "`#[pyfunction]` cannot be used alongside `#[pymodule_init]`"
                    );
                    ensure_spanned!(pymodule_init.is_none(), item_fn.span() => "only one `#[pymodule_init]` may be specified");
                    pymodule_init = Some(quote! { #ident(module) });
                } else if has_attribute(&item_fn.attrs, "pyfunction")
                    || has_attribute_with_namespace(
                        &item_fn.attrs,
                        Some(pyo3_path),
                        &["pyfunction"],
                    )
                    || has_attribute_with_namespace(
                        &item_fn.attrs,
                        Some(pyo3_path),
                        &["prelude", "pyfunction"],
                    )
                {
                    module_items.push(ident.clone());
                    module_items_cfg_attrs.push(get_cfg_attributes(&item_fn.attrs));
                }
            }
            Item::Struct(item_struct) => {
                ensure_spanned!(
                    !has_attribute(&item_struct.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
                if has_attribute(&item_struct.attrs, "pyclass")
                    || has_attribute_with_namespace(
                        &item_struct.attrs,
                        Some(pyo3_path),
                        &["pyclass"],
                    )
                    || has_attribute_with_namespace(
                        &item_struct.attrs,
                        Some(pyo3_path),
                        &["prelude", "pyclass"],
                    )
                {
                    module_items.push(item_struct.ident.clone());
                    module_items_cfg_attrs.push(get_cfg_attributes(&item_struct.attrs));
                    if !has_pyo3_module_declared::<PyClassPyO3Option>(
                        &item_struct.attrs,
                        "pyclass",
                        |option| matches!(option, PyClassPyO3Option::Module(_)),
                    )? {
                        set_module_attribute(&mut item_struct.attrs, &full_name);
                    }
                }
                #[cfg(feature = "experimental-module-state")]
                if find_and_remove_attribute(&mut item_struct.attrs, "pymodule_state") {
                    if state_type.is_some() {
                        bail_spanned!(item_struct.span() =>
                            "Multiple `#[pymodule_state]` structs found. Specify state type explicitly with `#[pymodule(state = ...)]`");
                    }
                    state_type = Some(
                        syn::Path {
                            leading_colon: None,
                            segments: std::iter::once(syn::PathSegment {
                                ident: item_struct.ident.clone(),
                                arguments: syn::PathArguments::None,
                            }).collect(),}
                    );
                }
            }
            Item::Enum(item_enum) => {
                ensure_spanned!(
                    !has_attribute(&item_enum.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
                if has_attribute(&item_enum.attrs, "pyclass")
                    || has_attribute_with_namespace(&item_enum.attrs, Some(pyo3_path), &["pyclass"])
                    || has_attribute_with_namespace(
                        &item_enum.attrs,
                        Some(pyo3_path),
                        &["prelude", "pyclass"],
                    )
                {
                    module_items.push(item_enum.ident.clone());
                    module_items_cfg_attrs.push(get_cfg_attributes(&item_enum.attrs));
                    if !has_pyo3_module_declared::<PyClassPyO3Option>(
                        &item_enum.attrs,
                        "pyclass",
                        |option| matches!(option, PyClassPyO3Option::Module(_)),
                    )? {
                        set_module_attribute(&mut item_enum.attrs, &full_name);
                    }
                }
            }
            Item::Mod(item_mod) => {
                ensure_spanned!(
                    !has_attribute(&item_mod.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
                if has_attribute(&item_mod.attrs, "pymodule")
                    || has_attribute_with_namespace(&item_mod.attrs, Some(pyo3_path), &["pymodule"])
                    || has_attribute_with_namespace(
                        &item_mod.attrs,
                        Some(pyo3_path),
                        &["prelude", "pymodule"],
                    )
                {
                    module_items.push(item_mod.ident.clone());
                    module_items_cfg_attrs.push(get_cfg_attributes(&item_mod.attrs));
                    if !has_pyo3_module_declared::<PyModulePyO3Option>(
                        &item_mod.attrs,
                        "pymodule",
                        |option| matches!(option, PyModulePyO3Option::Module(_)),
                    )? {
                        set_module_attribute(&mut item_mod.attrs, &full_name);
                    }
                    item_mod
                        .attrs
                        .push(parse_quote_spanned!(item_mod.mod_token.span()=> #[pyo3(submodule)]));
                }
            }
            Item::ForeignMod(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
            }
            Item::Trait(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
            }
            Item::Const(item) => {
                if !find_and_remove_attribute(&mut item.attrs, "pymodule_export") {
                    return Ok(());
                }
                module_consts.push(item.ident.clone());
                module_consts_cfg_attrs.push(get_cfg_attributes(&item.attrs));
                #[cfg(feature = "experimental-inspect")]
                {
                    let cfg_attrs = get_cfg_attributes(&item.attrs);
                    let chunk = attribute_introspection_code(
                        pyo3_path,
                        None,
                        item.ident.unraw().to_string(),
                        PyExpr::constant_from_expression(&item.expr),
                        (*item.ty).clone(),
                        get_doc(&item.attrs, None).as_ref(),
                        true,
                    );
                    introspection_chunks.push(quote! {
                        #(#cfg_attrs)*
                        #chunk
                    });
                }
            }
            Item::Static(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
            }
            Item::Macro(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
            }
            Item::ExternCrate(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
            }
            Item::Impl(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
            }
            Item::TraitAlias(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
            }
            Item::Type(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
            }
            Item::Union(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` or `const` statements"
                );
            }
            _ => (),
        }
        Ok(())
    }).try_combine_syn_errors()?;

    #[cfg(feature = "experimental-inspect")]
    let introspection = module_introspection_code(
        pyo3_path,
        &name.to_string(),
        &module_items,
        &module_items_cfg_attrs,
        doc.as_ref(),
        pymodule_init.is_some(),
    );
    #[cfg(not(feature = "experimental-inspect"))]
    let introspection = quote! {};
    #[cfg(feature = "experimental-inspect")]
    let introspection_id = introspection_id_const();
    #[cfg(not(feature = "experimental-inspect"))]
    let introspection_id = quote! {};

    let gil_used = options.gil_used.is_some_and(|op| op.value.value);

    #[cfg(feature = "experimental-module-state")]
    let allocate_state = {
        // For mod declarations with state, require an init function
        if let Some(state) = options.state.as_ref().map(|s| &s.value) {
            if pymodule_init.is_none() {
                return Err(syn::Error::new_spanned(
                &ident,
                "module with `state` attribute must have a `#[pymodule_init]` function that returns the state value"
            ));
            }

            pymodule_init = Some(quote! {
                let state: #state = #pymodule_init?;
                #pyo3_path::impl_::pymodule::pyo3_module_state_init(module, state)
            });
            true
        } else {
            if pymodule_init.is_none() {
                pymodule_init = Some(quote! { #pyo3_path::PyResult::Ok(()) });
            };

            false
        }
    };
    #[cfg(not(feature = "experimental-module-state"))]
    let allocate_state = {
        if pymodule_init.is_none() {
            pymodule_init = Some(quote! { #pyo3_path::PyResult::Ok(()) });
        };
        false
    };

    let initialization = module_initialization(
        &full_name,
        &name,
        ctx,
        quote! { __pyo3_pymodule },
        options.submodule.is_some(),
        gil_used,
        doc.as_ref(),
        allocate_state,
        pymodule_traverse,
        pymodule_clear,
    )?;

    let module_consts_names = module_consts.iter().map(|i| i.unraw().to_string());

    Ok(quote!(
        #(#attrs)*
        #vis #mod_token #ident {
            #(#items)*

            #initialization
            #introspection
            #introspection_id
            #(#introspection_chunks)*

            fn __pyo3_pymodule(module: &#pyo3_path::Bound<'_, #pyo3_path::types::PyModule>) -> #pyo3_path::PyResult<()> {
                use #pyo3_path::impl_::pymodule::PyAddToModule;
                #(
                    #(#module_items_cfg_attrs)*
                    #module_items::_PYO3_DEF.add_to_module(module)?;
                )*

                #(
                    #(#module_consts_cfg_attrs)*
                    #pyo3_path::types::PyModuleMethods::add(module, #module_consts_names, #module_consts)?;
                )*

                #pymodule_init
            }
        }
    ))
}

/// Generates the function that is called by the python interpreter to initialize the native
/// module
pub fn pymodule_function_impl(
    function: &mut syn::ItemFn,
    mut options: PyModuleOptions,
) -> Result<TokenStream> {
    options.take_pyo3_options(&mut function.attrs)?;
    process_functions_in_module(&options, function)?;
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = ctx;
    let ident = &function.sig.ident;
    let name = options
        .name
        .map_or_else(|| ident.unraw(), |name| name.value.0);
    let vis = &function.vis;
    let doc = get_doc(&function.attrs, None);

    let gil_used = options.gil_used.is_some_and(|op| op.value.value);

    #[cfg(feature = "experimental-module-state")]
    let allocate_state = options.state.is_some();
    #[cfg(not(feature = "experimental-module-state"))]
    let allocate_state = false;

    let initialization = module_initialization(
        &name.to_string(),
        &name,
        ctx,
        quote! { ModuleExec::__pyo3_module_exec },
        false,
        gil_used,
        doc.as_ref(),
        allocate_state,
        None,
        None,
    )?;

    #[cfg(feature = "experimental-inspect")]
    let introspection = module_introspection_code(
        pyo3_path,
        &name.unraw().to_string(),
        &[],
        &[],
        doc.as_ref(),
        true,
    );
    #[cfg(not(feature = "experimental-inspect"))]
    let introspection = quote! {};
    #[cfg(feature = "experimental-inspect")]
    let introspection_id = introspection_id_const();
    #[cfg(not(feature = "experimental-inspect"))]
    let introspection_id = quote! {};

    // Module function called with optional Python<'_> marker as first arg, followed by the module.
    let mut module_args = Vec::new();
    if function.sig.inputs.len() == 2 {
        module_args.push(quote!(module.py()));
    }
    module_args.push(quote!(::std::convert::Into::into(module)));

    #[cfg(feature = "experimental-module-state")]
    let init_func = {
        if let Some(state) = &options.state {
            let state = &state.value;
            quote! {
                let state: #state = #ident(#(#module_args),*)?;
                #pyo3_path::impl_::pymodule::pyo3_module_state_init(module, state)
            }
        } else {
            quote! {
                #ident(#(#module_args),*)
            }
        }
    };
    #[cfg(not(feature = "experimental-module-state"))]
    let init_func = quote! {
        #ident(#(#module_args),*)
    };

    Ok(quote! {
        #[doc(hidden)]
        #vis mod #ident {
            #initialization
            #introspection
            #introspection_id
        }

        // Generate the definition inside an anonymous function in the same scope as the original function -
        // this avoids complications around the fact that the generated module has a different scope
        // (and `super` doesn't always refer to the outer scope, e.g. if the `#[pymodule] is
        // inside a function body)
        #[allow(unknown_lints, non_local_definitions)]
        impl #ident::ModuleExec {
            fn __pyo3_module_exec(module: &#pyo3_path::Bound<'_, #pyo3_path::types::PyModule>) -> #pyo3_path::PyResult<()> {
                #init_func
            }
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn module_initialization(
    full_name: &str,
    name: &syn::Ident,
    ctx: &Ctx,
    module_exec: TokenStream,
    is_submodule: bool,
    gil_used: bool,
    doc: Option<&PythonDoc>,
    allocate_state: bool,
    pymodule_traverse: Option<TokenStream>,
    pymodule_clear: Option<TokenStream>,
) -> Result<TokenStream> {
    let Ctx { pyo3_path, .. } = ctx;
    let pyinit_symbol = format!("PyInit_{name}");
    let pymodexport_symbol = format!("PyModExport_{name}");
    let pyo3_name = LitCStr::new(&CString::new(full_name).unwrap(), Span::call_site());
    let doc = if let Some(doc) = doc {
        doc.to_cstr_stream(ctx)?
    } else {
        c"".into_token_stream()
    };

    // Generate traverse and clear C callbacks if present
    #[cfg(feature = "experimental-module-state")]
    let (traverse_callback, clear_callback) =
        if pymodule_traverse.is_some() || pymodule_clear.is_some() {
            let traverse_code = if let Some(traverse_fn) = &pymodule_traverse {
                quote! {
                    unsafe extern "C" fn __pyo3_module_traverse(
                        module: *mut #pyo3_path::ffi::PyObject,
                        visit: #pyo3_path::ffi::visitproc,
                        arg: *mut ::std::ffi::c_void,
                    ) -> ::std::ffi::c_int {
                        #pyo3_path::Python::with_gil(|py| {
                            let module_bound = #pyo3_path::Bound::new_borrowed(py, module);
                            let py_module = module_bound.cast_exact::<#pyo3_path::types::PyModule>()
                                .expect("module object is not PyModule");

                            match py_module.module_state::<::std::any::Any>() {
                                Ok(state_any) => {
                                    let visit_fn = #pyo3_path::PyVisit {
                                        visit,
                                        arg,
                                        _guard: ::std::marker::PhantomData,
                                    };
                                    // Call user's traverse function with state and visit
                                    match #traverse_fn(state_any, visit_fn) {
                                        Ok(()) => 0,
                                        Err(_) => -1,
                                    }
                                }
                                Err(_) => -1,
                            }
                        })
                    }
                }
            } else {
                quote! {}
            };

            let clear_code = if let Some(clear_fn) = &pymodule_clear {
                quote! {
                    unsafe extern "C" fn __pyo3_module_clear(
                        module: *mut #pyo3_path::ffi::PyObject,
                    ) -> ::std::ffi::c_int {
                        #pyo3_path::Python::with_gil(|py| {
                            let module_bound = #pyo3_path::Bound::new_borrowed(py, module);
                            let py_module = module_bound.cast_exact::<#pyo3_path::types::PyModule>()
                                .expect("module object is not PyModule");

                            match py_module.module_state_mut::<::std::any::Any>() {
                                Ok(state_any_mut) => {
                                    // Call user's clear function with mutable state
                                    match #clear_fn(state_any_mut) {
                                        Ok(()) => 0,
                                        Err(_) => -1,
                                    }
                                }
                                Err(_) => -1,
                            }
                        })
                    }
                }
            } else {
                quote! {}
            };

            (traverse_code, clear_code)
        } else {
            (quote! {}, quote! {})
        };

    #[cfg(not(feature = "experimental-module-state"))]
    let (traverse_callback, clear_callback) = {
        let (_, _) = (pymodule_traverse, pymodule_clear);
        (quote! {}, quote! {})
    };

    #[cfg(feature = "experimental-module-state")]
    let (traverse_arg, clear_arg) = if pymodule_traverse.is_some() || pymodule_clear.is_some() {
        let traverse = if pymodule_traverse.is_some() {
            quote! { ::core::option::Option::Some(__pyo3_module_traverse) }
        } else {
            quote! { ::core::option::Option::None }
        };
        let clear = if pymodule_clear.is_some() {
            quote! { ::core::option::Option::Some(__pyo3_module_clear) }
        } else {
            quote! { ::core::option::Option::None }
        };
        (traverse, clear)
    } else {
        (
            quote! { ::core::option::Option::None },
            quote! { ::core::option::Option::None },
        )
    };
    #[cfg(not(feature = "experimental-module-state"))]
    let (traverse_arg, clear_arg) = (
        quote! { ::core::option::Option::None },
        quote! { ::core::option::Option::None },
    );

    let mod_new = quote! {
        #pyo3_path::impl_::pymodule::ModuleDef::new(
            __PYO3_NAME,
            #doc,
            &SLOTS,
            #allocate_state,
            #traverse_arg,
            #clear_arg,
        )
    };

    let mut result = quote! {
        #[doc(hidden)]
        pub const __PYO3_NAME: &'static ::std::ffi::CStr = #pyo3_name;

        // This structure exists for `fn` modules declared within `fn` bodies, where due to the hidden
        // module (used for importing) the `fn` to initialize the module cannot be seen from the #module_def
        // declaration just below.
        #[doc(hidden)]
        pub(super) struct ModuleExec;

        #[doc(hidden)]
        pub static _PYO3_DEF: #pyo3_path::impl_::pymodule::ModuleDef = {
            use #pyo3_path::impl_::pymodule as impl_;

            unsafe extern "C" fn __pyo3_module_exec(module: *mut #pyo3_path::ffi::PyObject) -> ::std::ffi::c_int {
                #pyo3_path::impl_::trampoline::module_exec(module, #module_exec)
            }

            #traverse_callback
            #clear_callback

            // The full slots, used for the PyModExport initialization
            static SLOTS: impl_::PyModuleSlots = impl_::PyModuleSlotsBuilder::new()
                .with_mod_exec(__pyo3_module_exec)
                .with_abi_info()
                .with_gil_used(#gil_used)
                .with_name(__PYO3_NAME)
                .with_doc(#doc)
                .build();

            // Since the macros need to be written agnostic to the Python version
            // we need to explicitly pass the name and docstring for PyModuleDef
            // initialization.
            #mod_new
        };
    };
    if !is_submodule {
        result.extend(quote! {
            // We want to define only one or the other of these initialization functions
            // so we implement them in macros to allow the version gating to live there

            // Defines the `PyInit_<name>` entry point used by Python 3.14 and older.
            #pyo3_path::__pyo3_pyinit!(#pyinit_symbol, _PYO3_DEF);

            // Defines the `PyModExport_<name>` entry point used by Python 3.15 and newer.
            #pyo3_path::__pyo3_pymodexport!(#pymodexport_symbol, _PYO3_DEF);
        });
    }
    Ok(result)
}

/// Finds and takes care of the #[pyfn(...)] in `#[pymodule]`
fn process_functions_in_module(options: &PyModuleOptions, func: &mut syn::ItemFn) -> Result<()> {
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = ctx;
    let mut stmts: Vec<syn::Stmt> = Vec::new();

    for mut stmt in func.block.stmts.drain(..) {
        if let syn::Stmt::Item(Item::Fn(func)) = &mut stmt {
            if let Some((pyfn_span, pyfn_args)) = get_pyfn_attr(&mut func.attrs)? {
                let module_name = pyfn_args.modname;
                let wrapped_function = impl_wrap_pyfunction(func, pyfn_args.options)?;
                let name = &func.sig.ident;
                let statements: Vec<syn::Stmt> = syn::parse_quote_spanned! {
                    pyfn_span =>
                    #wrapped_function
                    {
                        use #pyo3_path::types::PyModuleMethods;
                        #module_name.add_function(#pyo3_path::wrap_pyfunction!(#name, #module_name.as_borrowed())?)?;
                        #[deprecated(note = "`pyfn` will be removed in a future PyO3 version, use declarative `#[pymodule]` with `mod` instead")]
                        #[allow(dead_code)]
                        const PYFN_ATTRIBUTE: () = ();
                        const _: () = PYFN_ATTRIBUTE;
                    }
                };
                stmts.extend(statements);
            }
        };
        stmts.push(stmt);
    }

    func.block.stmts = stmts;
    Ok(())
}

pub struct PyFnArgs {
    modname: Path,
    options: PyFunctionOptions,
}

impl Parse for PyFnArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let modname = input.parse().map_err(
            |e| err_spanned!(e.span() => "expected module as first argument to #[pyfn()]"),
        )?;

        if input.is_empty() {
            return Ok(Self {
                modname,
                options: Default::default(),
            });
        }

        let _: Comma = input.parse()?;

        Ok(Self {
            modname,
            options: input.parse()?,
        })
    }
}

/// Extracts the data from the #[pyfn(...)] attribute of a function
fn get_pyfn_attr(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Option<(Span, PyFnArgs)>> {
    let mut pyfn_args: Option<(Span, PyFnArgs)> = None;

    take_attributes(attrs, |attr| {
        if attr.path().is_ident("pyfn") {
            ensure_spanned!(
                pyfn_args.is_none(),
                attr.span() => "`#[pyfn] may only be specified once"
            );
            pyfn_args = Some((attr.path().span(), attr.parse_args()?));
            Ok(true)
        } else {
            Ok(false)
        }
    })?;

    if let Some((_, pyfn_args)) = &mut pyfn_args {
        pyfn_args
            .options
            .add_attributes(take_pyo3_options(attrs)?)?;
    }

    Ok(pyfn_args)
}

fn get_cfg_attributes(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .cloned()
        .collect()
}

fn find_and_remove_attribute(attrs: &mut Vec<syn::Attribute>, ident: &str) -> bool {
    let mut found = false;
    attrs.retain(|attr| {
        if attr.path().is_ident(ident) {
            found = true;
            false
        } else {
            true
        }
    });
    found
}

impl PartialEq<syn::Ident> for IdentOrStr<'_> {
    fn eq(&self, other: &syn::Ident) -> bool {
        match self {
            IdentOrStr::Str(s) => other == s,
            IdentOrStr::Ident(i) => other == i,
        }
    }
}

fn set_module_attribute(attrs: &mut Vec<syn::Attribute>, module_name: &str) {
    attrs.push(parse_quote!(#[pyo3(module = #module_name)]));
}

fn has_pyo3_module_declared<T: Parse>(
    attrs: &[syn::Attribute],
    root_attribute_name: &str,
    is_module_option: impl Fn(&T) -> bool + Copy,
) -> Result<bool> {
    for attr in attrs {
        if (attr.path().is_ident("pyo3") || attr.path().is_ident(root_attribute_name))
            && matches!(attr.meta, Meta::List(_))
        {
            for option in &attr.parse_args_with(Punctuated::<T, Comma>::parse_terminated)? {
                if is_module_option(option) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

enum PyModulePyO3Option {
    Submodule(SubmoduleAttribute),
    Crate(CrateAttribute),
    Name(NameAttribute),
    Module(ModuleAttribute),
    GILUsed(GILUsedAttribute),
    #[cfg(feature = "experimental-module-state")]
    State(StateAttribute),
}

impl Parse for PyModulePyO3Option {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        #[cfg(feature = "experimental-module-state")]
        if lookahead.peek(attributes::kw::state) {
            return input.parse().map(PyModulePyO3Option::State);
        }

        if lookahead.peek(attributes::kw::name) {
            input.parse().map(PyModulePyO3Option::Name)
        } else if lookahead.peek(syn::Token![crate]) {
            input.parse().map(PyModulePyO3Option::Crate)
        } else if lookahead.peek(attributes::kw::module) {
            input.parse().map(PyModulePyO3Option::Module)
        } else if lookahead.peek(attributes::kw::submodule) {
            input.parse().map(PyModulePyO3Option::Submodule)
        } else if lookahead.peek(attributes::kw::gil_used) {
            input.parse().map(PyModulePyO3Option::GILUsed)
        } else {
            Err(lookahead.error())
        }
    }
}
