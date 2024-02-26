//! Code generation for the function that initializes a python module and adds classes and function.

use crate::{
    attributes::{self, take_attributes, take_pyo3_options, CrateAttribute, NameAttribute},
    get_doc,
    pyfunction::{impl_wrap_pyfunction, PyFunctionOptions},
    utils::get_pyo3_crate,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Comma,
    Item, Path, Result,
};

#[derive(Default)]
pub struct PyModuleOptions {
    krate: Option<CrateAttribute>,
    name: Option<syn::Ident>,
}

impl PyModuleOptions {
    pub fn from_attrs(attrs: &mut Vec<syn::Attribute>) -> Result<Self> {
        let mut options: PyModuleOptions = Default::default();

        for option in take_pyo3_options(attrs)? {
            match option {
                PyModulePyO3Option::Name(name) => options.set_name(name.value.0)?,
                PyModulePyO3Option::Crate(path) => options.set_crate(path)?,
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
    let krate = get_pyo3_crate(&options.krate);
    let doc = get_doc(attrs, None);

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
                let mut is_pyo3 = false;
                item_use.attrs.retain(|attr| {
                    let found = attr.path().is_ident("pymodule_export");
                    is_pyo3 |= found;
                    !found
                });
                if is_pyo3 {
                    let cfg_attrs = item_use
                        .attrs
                        .iter()
                        .filter(|attr| attr.path().is_ident("cfg"))
                        .cloned()
                        .collect::<Vec<_>>();
                    extract_use_items(
                        &item_use.tree,
                        &cfg_attrs,
                        &mut module_items,
                        &mut module_items_cfg_attrs,
                    )?;
                }
            }
            Item::Fn(item_fn) => {
                let mut is_module_init = false;
                item_fn.attrs.retain(|attr| {
                    let found = attr.path().is_ident("pymodule_init");
                    is_module_init |= found;
                    !found
                });
                if is_module_init {
                    ensure_spanned!(pymodule_init.is_none(), item_fn.span() => "only one pymodule_init may be specified");
                    let ident = &item_fn.sig.ident;
                    pymodule_init = Some(quote! { #ident(module)?; });
                } else {
                    bail_spanned!(item.span() => "only 'use' statements and and pymodule_init functions are allowed in #[pymodule]")
                }
            }
            item => {
                bail_spanned!(item.span() => "only 'use' statements and and pymodule_init functions are allowed in #[pymodule]")
            }
        }
    }

    let initialization = module_initialization(options, ident);
    Ok(quote!(
        #vis mod #ident {
            #(#items)*

            #initialization

            impl MakeDef {
                const fn make_def() -> #krate::impl_::pymodule::ModuleDef {
                    use #krate::impl_::pymodule as impl_;
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

            fn __pyo3_pymodule(module: &#krate::Bound<'_, #krate::types::PyModule>) -> #krate::PyResult<()> {
                use #krate::impl_::pymodule::PyAddToModule;
                #(
                    #(#module_items_cfg_attrs)*
                    #module_items::add_to_module(module)?;
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
    let krate = get_pyo3_crate(&options.krate);
    let ident = &function.sig.ident;
    let vis = &function.vis;
    let doc = get_doc(&function.attrs, None);

    let initialization = module_initialization(options, ident);

    // Module function called with optional Python<'_> marker as first arg, followed by the module.
    let mut module_args = Vec::new();
    if function.sig.inputs.len() == 2 {
        module_args.push(quote!(module.py()));
    }
    module_args.push(quote!(::std::convert::Into::into(BoundRef(module))));

    Ok(quote! {
        #function
        #vis mod #ident {
            #initialization
        }

        // Generate the definition inside an anonymous function in the same scope as the original function -
        // this avoids complications around the fact that the generated module has a different scope
        // (and `super` doesn't always refer to the outer scope, e.g. if the `#[pymodule] is
        // inside a function body)
        // FIXME https://github.com/PyO3/pyo3/issues/3903
        #[allow(unknown_lints, non_local_definitions)]
        const _: () = {
            use #krate::impl_::pymodule as impl_;
            use #krate::impl_::pymethods::BoundRef;

            fn __pyo3_pymodule(module: &#krate::Bound<'_, #krate::types::PyModule>) -> #krate::PyResult<()> {
                #ident(#(#module_args),*)
            }

            impl #ident::MakeDef {
                const fn make_def() -> impl_::ModuleDef {
                    const INITIALIZER: impl_::ModuleInitializer = impl_::ModuleInitializer(__pyo3_pymodule);
                    unsafe {
                        impl_::ModuleDef::new(
                            #ident::__PYO3_NAME,
                            #doc,
                            INITIALIZER
                        )
                    }
                }
            }
        };
    })
}

fn module_initialization(options: PyModuleOptions, ident: &syn::Ident) -> TokenStream {
    let name = options.name.unwrap_or_else(|| ident.unraw());
    let krate = get_pyo3_crate(&options.krate);
    let pyinit_symbol = format!("PyInit_{}", name);

    quote! {
        pub const __PYO3_NAME: &'static str = concat!(stringify!(#name), "\0");

        pub(super) struct MakeDef;
        pub static DEF: #krate::impl_::pymodule::ModuleDef = MakeDef::make_def();

        pub fn add_to_module(module: &#krate::Bound<'_, #krate::types::PyModule>) -> #krate::PyResult<()> {
            use #krate::prelude::PyModuleMethods;
            module.add_submodule(DEF.make_module(module.py())?.bind(module.py()))
        }

        /// This autogenerated function is called by the python interpreter when importing
        /// the module.
        #[export_name = #pyinit_symbol]
        pub unsafe extern "C" fn __pyo3_init() -> *mut #krate::ffi::PyObject {
            #krate::impl_::trampoline::module_init(|py| DEF.make_module(py))
        }
    }
}

/// Finds and takes care of the #[pyfn(...)] in `#[pymodule]`
fn process_functions_in_module(options: &PyModuleOptions, func: &mut syn::ItemFn) -> Result<()> {
    let krate = get_pyo3_crate(&options.krate);

    let mut stmts: Vec<syn::Stmt> = vec![syn::parse_quote!(
        #[allow(unknown_lints, unused_imports, redundant_imports)]
        use #krate::{PyNativeType, types::PyModuleMethods};
    )];

    for mut stmt in func.block.stmts.drain(..) {
        if let syn::Stmt::Item(Item::Fn(func)) = &mut stmt {
            if let Some(pyfn_args) = get_pyfn_attr(&mut func.attrs)? {
                let module_name = pyfn_args.modname;
                let wrapped_function = impl_wrap_pyfunction(func, pyfn_args.options)?;
                let name = &func.sig.ident;
                let statements: Vec<syn::Stmt> = syn::parse_quote! {
                    #wrapped_function
                    #module_name.as_borrowed().add_function(#krate::wrap_pyfunction!(#name, #module_name.as_borrowed())?)?;
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

enum PyModulePyO3Option {
    Crate(CrateAttribute),
    Name(NameAttribute),
}

impl Parse for PyModulePyO3Option {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::name) {
            input.parse().map(PyModulePyO3Option::Name)
        } else if lookahead.peek(syn::Token![crate]) {
            input.parse().map(PyModulePyO3Option::Crate)
        } else {
            Err(lookahead.error())
        }
    }
}
