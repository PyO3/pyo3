use crate::{
    attributes::{
        self, get_pyo3_options, take_attributes, take_pyo3_options, CrateAttribute,
        FromPyWithAttribute, NameAttribute, TextSignatureAttribute,
    },
    deprecations::Deprecations,
    method::{self, CallingConvention, FnArg},
    pymethod::check_generic,
    utils::get_pyo3_crate,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ext::IdentExt, spanned::Spanned, Result};
use syn::{
    parse::{Parse, ParseStream},
    token::Comma,
};

mod signature;

pub use self::signature::{FunctionSignature, SignatureAttribute};

#[derive(Clone, Debug)]
pub struct PyFunctionArgPyO3Attributes {
    pub from_py_with: Option<FromPyWithAttribute>,
}

enum PyFunctionArgPyO3Attribute {
    FromPyWith(FromPyWithAttribute),
}

impl Parse for PyFunctionArgPyO3Attribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::from_py_with) {
            input.parse().map(PyFunctionArgPyO3Attribute::FromPyWith)
        } else {
            Err(lookahead.error())
        }
    }
}

impl PyFunctionArgPyO3Attributes {
    /// Parses #[pyo3(from_python_with = "func")]
    pub fn from_attrs(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut attributes = PyFunctionArgPyO3Attributes { from_py_with: None };
        take_attributes(attrs, |attr| {
            if let Some(pyo3_attrs) = get_pyo3_options(attr)? {
                for attr in pyo3_attrs {
                    match attr {
                        PyFunctionArgPyO3Attribute::FromPyWith(from_py_with) => {
                            ensure_spanned!(
                                attributes.from_py_with.is_none(),
                                from_py_with.span() => "`from_py_with` may only be specified once per argument"
                            );
                            attributes.from_py_with = Some(from_py_with);
                        }
                    }
                }
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        Ok(attributes)
    }
}

#[derive(Default)]
pub struct PyFunctionOptions {
    pub pass_module: Option<attributes::kw::pass_module>,
    pub name: Option<NameAttribute>,
    pub signature: Option<SignatureAttribute>,
    pub text_signature: Option<TextSignatureAttribute>,
    pub krate: Option<CrateAttribute>,
}

impl Parse for PyFunctionOptions {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut options = PyFunctionOptions::default();

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(attributes::kw::name)
                || lookahead.peek(attributes::kw::pass_module)
                || lookahead.peek(attributes::kw::signature)
                || lookahead.peek(attributes::kw::text_signature)
            {
                options.add_attributes(std::iter::once(input.parse()?))?;
                if !input.is_empty() {
                    let _: Comma = input.parse()?;
                }
            } else if lookahead.peek(syn::Token![crate]) {
                // TODO needs duplicate check?
                options.krate = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(options)
    }
}

pub enum PyFunctionOption {
    Name(NameAttribute),
    PassModule(attributes::kw::pass_module),
    Signature(SignatureAttribute),
    TextSignature(TextSignatureAttribute),
    Crate(CrateAttribute),
}

impl Parse for PyFunctionOption {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::name) {
            input.parse().map(PyFunctionOption::Name)
        } else if lookahead.peek(attributes::kw::pass_module) {
            input.parse().map(PyFunctionOption::PassModule)
        } else if lookahead.peek(attributes::kw::signature) {
            input.parse().map(PyFunctionOption::Signature)
        } else if lookahead.peek(attributes::kw::text_signature) {
            input.parse().map(PyFunctionOption::TextSignature)
        } else if lookahead.peek(syn::Token![crate]) {
            input.parse().map(PyFunctionOption::Crate)
        } else {
            Err(lookahead.error())
        }
    }
}

impl PyFunctionOptions {
    pub fn from_attrs(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut options = PyFunctionOptions::default();
        options.add_attributes(take_pyo3_options(attrs)?)?;
        Ok(options)
    }

    pub fn add_attributes(
        &mut self,
        attrs: impl IntoIterator<Item = PyFunctionOption>,
    ) -> Result<()> {
        macro_rules! set_option {
            ($key:ident) => {
                {
                    ensure_spanned!(
                        self.$key.is_none(),
                        $key.span() => concat!("`", stringify!($key), "` may only be specified once")
                    );
                    self.$key = Some($key);
                }
            };
        }
        for attr in attrs {
            match attr {
                PyFunctionOption::Name(name) => set_option!(name),
                PyFunctionOption::PassModule(pass_module) => set_option!(pass_module),
                PyFunctionOption::Signature(signature) => set_option!(signature),
                PyFunctionOption::TextSignature(text_signature) => set_option!(text_signature),
                PyFunctionOption::Crate(krate) => set_option!(krate),
            }
        }
        Ok(())
    }
}

pub fn build_py_function(
    ast: &mut syn::ItemFn,
    mut options: PyFunctionOptions,
) -> syn::Result<TokenStream> {
    options.add_attributes(take_pyo3_options(&mut ast.attrs)?)?;
    impl_wrap_pyfunction(ast, options)
}

/// Generates python wrapper over a function that allows adding it to a python module as a python
/// function
pub fn impl_wrap_pyfunction(
    func: &mut syn::ItemFn,
    options: PyFunctionOptions,
) -> syn::Result<TokenStream> {
    check_generic(&func.sig)?;
    let PyFunctionOptions {
        pass_module,
        name,
        signature,
        text_signature,
        krate,
    } = options;

    let python_name = name.map_or_else(|| func.sig.ident.unraw(), |name| name.value.0);

    let tp = if pass_module.is_some() {
        let span = match func.sig.inputs.first() {
            Some(syn::FnArg::Typed(first_arg)) => first_arg.ty.span(),
            Some(syn::FnArg::Receiver(_)) | None => bail_spanned!(
                func.sig.paren_token.span.join() => "expected `&PyModule` or `Py<PyModule>` as first argument with `pass_module`"
            ),
        };
        method::FnType::FnModule(span)
    } else {
        method::FnType::FnStatic
    };

    let arguments = func
        .sig
        .inputs
        .iter_mut()
        .skip(if tp.skip_first_rust_argument_in_python_signature() {
            1
        } else {
            0
        })
        .map(FnArg::parse)
        .collect::<syn::Result<Vec<_>>>()?;

    let signature = if let Some(signature) = signature {
        FunctionSignature::from_arguments_and_attribute(arguments, signature)?
    } else {
        FunctionSignature::from_arguments(arguments)?
    };

    let ty = method::get_return_info(&func.sig.output);

    let spec = method::FnSpec {
        tp,
        name: &func.sig.ident,
        convention: CallingConvention::from_signature(&signature),
        python_name,
        signature,
        output: ty,
        text_signature,
        asyncness: func.sig.asyncness,
        unsafety: func.sig.unsafety,
        deprecations: Deprecations::new(),
    };

    let krate = get_pyo3_crate(&krate);

    let vis = &func.vis;
    let name = &func.sig.ident;

    let wrapper_ident = format_ident!("__pyfunction_{}", spec.name);
    let wrapper = spec.get_wrapper_function(&wrapper_ident, None)?;
    let methoddef = spec.get_methoddef(wrapper_ident, &spec.get_doc(&func.attrs));

    let wrapped_pyfunction = quote! {

        // Create a module with the same name as the `#[pyfunction]` - this way `use <the function>`
        // will actually bring both the module and the function into scope.
        #[doc(hidden)]
        #vis mod #name {
            pub(crate) struct MakeDef;
            pub const DEF: #krate::impl_::pyfunction::PyMethodDef = MakeDef::DEF;
        }

        // Generate the definition inside an anonymous function in the same scope as the original function -
        // this avoids complications around the fact that the generated module has a different scope
        // (and `super` doesn't always refer to the outer scope, e.g. if the `#[pyfunction] is
        // inside a function body)
        const _: () = {
            use #krate as _pyo3;
            impl #name::MakeDef {
                const DEF: #krate::impl_::pyfunction::PyMethodDef = #methoddef;
            }

            #[allow(non_snake_case)]
            #wrapper
        };
    };
    Ok(wrapped_pyfunction)
}
