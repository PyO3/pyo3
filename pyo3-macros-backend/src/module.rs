//! Code generation for the function that initializes a python module and adds classes and function.

#[cfg(feature = "experimental-inspect")]
use crate::introspection::{
    attribute_introspection_code, introspection_id_const, module_introspection_code,
};
#[cfg(feature = "experimental-inspect")]
use crate::utils::expr_to_python;
use crate::{
    attributes::{
        self, kw, take_attributes, take_pyo3_options, CrateAttribute, GILUsedAttribute,
        ModuleAttribute, NameAttribute, SubmoduleAttribute,
    },
    combine_errors::CombineErrors,
    get_doc,
    pyclass::PyClassPyO3Option,
    pyfunction::{impl_wrap_pyfunction, PyFunctionOptions},
    utils::{has_attribute, has_attribute_with_namespace, Ctx, IdentOrStr, LitCStr},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::ffi::CString;
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
    let doc = get_doc(attrs, None, ctx)?;
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

    let mut pymodule_init = None;
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
                let ident = &item_fn.sig.ident;
                if is_pymodule_init {
                    ensure_spanned!(
                        !has_attribute(&item_fn.attrs, "pyfunction"),
                        item_fn.span() => "`#[pyfunction]` cannot be used alongside `#[pymodule_init]`"
                    );
                    ensure_spanned!(pymodule_init.is_none(), item_fn.span() => "only one `#[pymodule_init]` may be specified");
                    pymodule_init = Some(quote! { #ident(module)?; });
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
                        expr_to_python(&item.expr),
                        (*item.ty).clone(),
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
        pymodule_init.is_some(),
    );
    #[cfg(not(feature = "experimental-inspect"))]
    let introspection = quote! {};
    #[cfg(feature = "experimental-inspect")]
    let introspection_id = introspection_id_const();
    #[cfg(not(feature = "experimental-inspect"))]
    let introspection_id = quote! {};

    let module_def = quote! {{
        use #pyo3_path::impl_::pymodule as impl_;
        const INITIALIZER: impl_::ModuleInitializer = impl_::ModuleInitializer(__pyo3_pymodule);
        unsafe {
           impl_::ModuleDef::new(
                __PYO3_NAME,
                #doc,
                INITIALIZER
            )
        }
    }};
    let initialization = module_initialization(
        &name,
        ctx,
        module_def,
        options.submodule.is_some(),
        options.gil_used.map_or(true, |op| op.value.value),
    );

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
                ::std::result::Result::Ok(())
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
    let doc = get_doc(&function.attrs, None, ctx)?;

    let initialization = module_initialization(
        &name,
        ctx,
        quote! { MakeDef::make_def() },
        false,
        options.gil_used.map_or(true, |op| op.value.value),
    );

    #[cfg(feature = "experimental-inspect")]
    let introspection =
        module_introspection_code(pyo3_path, &name.unraw().to_string(), &[], &[], true);
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
    module_args
        .push(quote!(::std::convert::Into::into(#pyo3_path::impl_::pymethods::BoundRef(module))));

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
        impl #ident::MakeDef {
            const fn make_def() -> #pyo3_path::impl_::pymodule::ModuleDef {
                fn __pyo3_pymodule(module: &#pyo3_path::Bound<'_, #pyo3_path::types::PyModule>) -> #pyo3_path::PyResult<()> {
                    #ident(#(#module_args),*)
                }

                const INITIALIZER: #pyo3_path::impl_::pymodule::ModuleInitializer = #pyo3_path::impl_::pymodule::ModuleInitializer(__pyo3_pymodule);
                unsafe {
                    #pyo3_path::impl_::pymodule::ModuleDef::new(
                        #ident::__PYO3_NAME,
                        #doc,
                        INITIALIZER
                    )
                }
            }
        }
    })
}

fn module_initialization(
    name: &syn::Ident,
    ctx: &Ctx,
    module_def: TokenStream,
    is_submodule: bool,
    gil_used: bool,
) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;
    let pyinit_symbol = format!("PyInit_{name}");
    let name = name.to_string();
    let pyo3_name = LitCStr::new(CString::new(name).unwrap(), Span::call_site(), ctx);

    let mut result = quote! {
        #[doc(hidden)]
        pub const __PYO3_NAME: &'static ::std::ffi::CStr = #pyo3_name;

        pub(super) struct MakeDef;
        #[doc(hidden)]
        pub static _PYO3_DEF: #pyo3_path::impl_::pymodule::ModuleDef = #module_def;
        #[doc(hidden)]
        // so wrapped submodules can see what gil_used is
        pub static __PYO3_GIL_USED: bool = #gil_used;
    };
    if !is_submodule {
        result.extend(quote! {
            /// This autogenerated function is called by the python interpreter when importing
            /// the module.
            #[doc(hidden)]
            #[export_name = #pyinit_symbol]
            pub unsafe extern "C" fn __pyo3_init() -> *mut #pyo3_path::ffi::PyObject {
                unsafe { #pyo3_path::impl_::trampoline::module_init(|py| _PYO3_DEF.make_module(py, #gil_used)) }
            }
        });
    }
    result
}

/// Finds and takes care of the #[pyfn(...)] in `#[pymodule]`
fn process_functions_in_module(options: &PyModuleOptions, func: &mut syn::ItemFn) -> Result<()> {
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = ctx;
    let mut stmts: Vec<syn::Stmt> = Vec::new();

    for mut stmt in func.block.stmts.drain(..) {
        if let syn::Stmt::Item(Item::Fn(func)) = &mut stmt {
            if let Some(pyfn_args) = get_pyfn_attr(&mut func.attrs)? {
                let module_name = pyfn_args.modname;
                let wrapped_function = impl_wrap_pyfunction(func, pyfn_args.options)?;
                let name = &func.sig.ident;
                let statements: Vec<syn::Stmt> = syn::parse_quote! {
                    #wrapped_function
                    {
                        use #pyo3_path::types::PyModuleMethods;
                        #module_name.add_function(#pyo3_path::wrap_pyfunction!(#name, #module_name.as_borrowed())?)?;
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
fn get_pyfn_attr(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Option<PyFnArgs>> {
    let mut pyfn_args: Option<PyFnArgs> = None;

    take_attributes(attrs, |attr| {
        if attr.path().is_ident("pyfn") {
            ensure_spanned!(
                pyfn_args.is_none(),
                attr.span() => "`#[pyfn] may only be specified once"
            );
            pyfn_args = Some(attr.parse_args()?);
            Ok(true)
        } else {
            Ok(false)
        }
    })?;

    if let Some(pyfn_args) = &mut pyfn_args {
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
}

impl Parse for PyModulePyO3Option {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
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
