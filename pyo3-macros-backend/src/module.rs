//! Code generation for the function that initializes a python module and adds classes and function.

use crate::{
    attributes::{
        self, take_attributes, take_pyo3_options, CrateAttribute, ModuleAttribute, NameAttribute,
    },
    get_doc,
    pyclass::PyClassPyO3Option,
    pyfunction::{impl_wrap_pyfunction, PyFunctionOptions},
    utils::{Ctx, LitCStr},
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
    name: Option<syn::Ident>,
    module: Option<ModuleAttribute>,
}

impl PyModuleOptions {
    pub fn from_attrs(attrs: &mut Vec<syn::Attribute>) -> Result<Self> {
        let mut options: PyModuleOptions = Default::default();

        for option in take_pyo3_options(attrs)? {
            match option {
                PyModulePyO3Option::Name(name) => options.set_name(name.value.0)?,
                PyModulePyO3Option::Crate(path) => options.set_crate(path)?,
                PyModulePyO3Option::Module(module) => options.set_module(module)?,
            }
        }

        Ok(options)
    }

    fn set_name(&mut self, name: syn::Ident) -> Result<()> {
        ensure_spanned!(
            self.name.is_none(),
            name.span() => "`name` may only be specified once"
        );

        self.name = Some(name);
        Ok(())
    }

    fn set_crate(&mut self, path: CrateAttribute) -> Result<()> {
        ensure_spanned!(
            self.krate.is_none(),
            path.span() => "`crate` may only be specified once"
        );

        self.krate = Some(path);
        Ok(())
    }

    fn set_module(&mut self, name: ModuleAttribute) -> Result<()> {
        ensure_spanned!(
            self.module.is_none(),
            name.span() => "`module` may only be specified once"
        );

        self.module = Some(name);
        Ok(())
    }
}

pub fn pymodule_module_impl(mut module: syn::ItemMod) -> Result<TokenStream> {
    let syn::ItemMod {
        attrs,
        vis,
        unsafety: _,
        ident,
        mod_token: _,
        content,
        semi: _,
    } = &mut module;
    let items = if let Some((_, items)) = content {
        items
    } else {
        bail_spanned!(module.span() => "`#[pymodule]` can only be used on inline modules")
    };
    let options = PyModuleOptions::from_attrs(attrs)?;
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = ctx;
    let doc = get_doc(attrs, None, ctx);
    let name = options.name.unwrap_or_else(|| ident.unraw());
    let full_name = if let Some(module) = &options.module {
        format!("{}.{}", module.value.value(), name)
    } else {
        name.to_string()
    };

    let mut module_items = Vec::new();
    let mut module_items_cfg_attrs = Vec::new();

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

    for item in &mut *items {
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
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
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
                } else if has_attribute(&item_fn.attrs, "pyfunction") {
                    module_items.push(ident.clone());
                    module_items_cfg_attrs.push(get_cfg_attributes(&item_fn.attrs));
                }
            }
            Item::Struct(item_struct) => {
                ensure_spanned!(
                    !has_attribute(&item_struct.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
                if has_attribute(&item_struct.attrs, "pyclass") {
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
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
                if has_attribute(&item_enum.attrs, "pyclass") {
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
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
                if has_attribute(&item_mod.attrs, "pymodule") {
                    module_items.push(item_mod.ident.clone());
                    module_items_cfg_attrs.push(get_cfg_attributes(&item_mod.attrs));
                    if !has_pyo3_module_declared::<PyModulePyO3Option>(
                        &item_mod.attrs,
                        "pymodule",
                        |option| matches!(option, PyModulePyO3Option::Module(_)),
                    )? {
                        set_module_attribute(&mut item_mod.attrs, &full_name);
                    }
                }
            }
            Item::ForeignMod(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
            }
            Item::Trait(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
            }
            Item::Const(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
            }
            Item::Static(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
            }
            Item::Macro(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
            }
            Item::ExternCrate(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
            }
            Item::Impl(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
            }
            Item::TraitAlias(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
            }
            Item::Type(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
            }
            Item::Union(item) => {
                ensure_spanned!(
                    !has_attribute(&item.attrs, "pymodule_export"),
                    item.span() => "`#[pymodule_export]` may only be used on `use` statements"
                );
            }
            _ => (),
        }
    }

    let initialization = module_initialization(&name, ctx);
    Ok(quote!(
        #(#attrs)*
        #vis mod #ident {
            #(#items)*

            #initialization

            #[allow(unknown_lints, non_local_definitions)]
            impl MakeDef {
                const fn make_def() -> #pyo3_path::impl_::pymodule::ModuleDef {
                    use #pyo3_path::impl_::pymodule as impl_;
                    const INITIALIZER: impl_::ModuleInitializer = impl_::ModuleInitializer(__pyo3_pymodule);
                    unsafe {
                       impl_::ModuleDef::new(
                            __PYO3_NAME,
                            #doc,
                            INITIALIZER
                        )
                    }
                }
            }

            fn __pyo3_pymodule(module: &#pyo3_path::Bound<'_, #pyo3_path::types::PyModule>) -> #pyo3_path::PyResult<()> {
                use #pyo3_path::impl_::pymodule::PyAddToModule;
                #(
                    #(#module_items_cfg_attrs)*
                    #module_items::_PYO3_DEF.add_to_module(module)?;
                )*
                #pymodule_init
                Ok(())
            }
        }
    ))
}

/// Generates the function that is called by the python interpreter to initialize the native
/// module
pub fn pymodule_function_impl(mut function: syn::ItemFn) -> Result<TokenStream> {
    let options = PyModuleOptions::from_attrs(&mut function.attrs)?;
    process_functions_in_module(&options, &mut function)?;
    let ctx = &Ctx::new(&options.krate, None);
    let stmts = std::mem::take(&mut function.block.stmts);
    let Ctx { pyo3_path, .. } = ctx;
    let ident = &function.sig.ident;
    let name = options.name.unwrap_or_else(|| ident.unraw());
    let vis = &function.vis;
    let doc = get_doc(&function.attrs, None, ctx);

    let initialization = module_initialization(&name, ctx);

    // Module function called with optional Python<'_> marker as first arg, followed by the module.
    let mut module_args = Vec::new();
    if function.sig.inputs.len() == 2 {
        module_args.push(quote!(module.py()));
    }
    module_args
        .push(quote!(::std::convert::Into::into(#pyo3_path::impl_::pymethods::BoundRef(module))));

    let extractors = function
        .sig
        .inputs
        .iter()
        .filter_map(|param| {
            if let syn::FnArg::Typed(pat_type) = param {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    let ident: &syn::Ident = &pat_ident.ident;
                    return Some([
                        parse_quote!{ let check_gil_refs = #pyo3_path::impl_::deprecations::GilRefs::new(); },
                        parse_quote! { let #ident = #pyo3_path::impl_::deprecations::inspect_type(#ident, &check_gil_refs); },
                        parse_quote_spanned! { pat_type.span() => check_gil_refs.function_arg(); },
                    ]);
                }
            }
            None
        })
        .flatten();

    function.block.stmts = extractors.chain(stmts).collect();
    function
        .attrs
        .push(parse_quote!(#[allow(clippy::used_underscore_binding)]));

    Ok(quote! {
        #function
        #[doc(hidden)]
        #vis mod #ident {
            #initialization
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

fn module_initialization(name: &syn::Ident, ctx: &Ctx) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;
    let pyinit_symbol = format!("PyInit_{}", name);
    let name = name.to_string();
    let pyo3_name = LitCStr::new(CString::new(name).unwrap(), Span::call_site(), ctx);

    quote! {
        #[doc(hidden)]
        pub const __PYO3_NAME: &'static ::std::ffi::CStr = #pyo3_name;

        pub(super) struct MakeDef;
        #[doc(hidden)]
        pub static _PYO3_DEF: #pyo3_path::impl_::pymodule::ModuleDef = MakeDef::make_def();

        /// This autogenerated function is called by the python interpreter when importing
        /// the module.
        #[doc(hidden)]
        #[export_name = #pyinit_symbol]
        pub unsafe extern "C" fn __pyo3_init() -> *mut #pyo3_path::ffi::PyObject {
            #pyo3_path::impl_::trampoline::module_init(|py| _PYO3_DEF.make_module(py))
        }
    }
}

/// Finds and takes care of the #[pyfn(...)] in `#[pymodule]`
fn process_functions_in_module(options: &PyModuleOptions, func: &mut syn::ItemFn) -> Result<()> {
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = ctx;
    let mut stmts: Vec<syn::Stmt> = Vec::new();

    #[cfg(feature = "gil-refs")]
    let imports = quote!(use #pyo3_path::{PyNativeType, types::PyModuleMethods};);
    #[cfg(not(feature = "gil-refs"))]
    let imports = quote!(use #pyo3_path::types::PyModuleMethods;);

    for mut stmt in func.block.stmts.drain(..) {
        if let syn::Stmt::Item(Item::Fn(func)) = &mut stmt {
            if let Some(pyfn_args) = get_pyfn_attr(&mut func.attrs)? {
                let module_name = pyfn_args.modname;
                let wrapped_function = impl_wrap_pyfunction(func, pyfn_args.options)?;
                let name = &func.sig.ident;
                let statements: Vec<syn::Stmt> = syn::parse_quote! {
                    #wrapped_function
                    {
                        #[allow(unknown_lints, unused_imports, redundant_imports)]
                        #imports
                        #module_name.as_borrowed().add_function(#pyo3_path::wrap_pyfunction!(#name, #module_name.as_borrowed())?)?;
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

fn has_attribute(attrs: &[syn::Attribute], ident: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(ident))
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
    Crate(CrateAttribute),
    Name(NameAttribute),
    Module(ModuleAttribute),
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
        } else {
            Err(lookahead.error())
        }
    }
}
