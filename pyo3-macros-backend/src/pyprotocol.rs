//! `pyprotocol!`: declares a `typing.Protocol` that only exists in the generated type stubs.

use crate::attributes::{self, CrateAttribute, NameAttribute, take_pyo3_options};
use crate::combine_errors::CombineErrors;
#[cfg(feature = "experimental-inspect")]
use crate::introspection::{function_introspection_code, protocol_introspection_code};
use crate::method::FnArg;
#[cfg(feature = "experimental-inspect")]
use crate::py_expr::PyExpr;
use crate::pyfunction::{FunctionSignature, PyFunctionOptions};
use crate::pymethod::check_generic;
use crate::utils::Ctx;
#[cfg(feature = "experimental-inspect")]
use crate::utils::get_doc;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Result, TraitItemFn, Visibility, braced};

/// The input of `pyprotocol!`: `pub Name { fn method(&self, ...) -> ...; }`
pub struct PyProtocol {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    methods: Vec<TraitItemFn>,
}

impl Parse for PyProtocol {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(content.parse()?);
        }
        Ok(Self {
            attrs,
            vis,
            ident,
            methods,
        })
    }
}

#[derive(Default)]
struct PyProtocolOptions {
    name: Option<NameAttribute>,
    krate: Option<CrateAttribute>,
}

enum PyProtocolOption {
    Name(NameAttribute),
    Crate(CrateAttribute),
}

impl Parse for PyProtocolOption {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::name) {
            input.parse().map(PyProtocolOption::Name)
        } else if lookahead.peek(syn::Token![crate]) {
            input.parse().map(PyProtocolOption::Crate)
        } else {
            Err(lookahead.error())
        }
    }
}

impl PyProtocolOptions {
    fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut options = Self::default();
        for option in take_pyo3_options(attrs)? {
            match option {
                PyProtocolOption::Name(name) => {
                    ensure_spanned!(options.name.is_none(), name.span() => "`name` may only be specified once");
                    options.name = Some(name);
                }
                PyProtocolOption::Crate(krate) => {
                    ensure_spanned!(options.krate.is_none(), krate.span() => "`crate` may only be specified once");
                    options.krate = Some(krate);
                }
            }
        }
        Ok(options)
    }
}

pub fn build_py_protocol(protocol: PyProtocol) -> Result<TokenStream> {
    let PyProtocol {
        mut attrs,
        vis,
        ident,
        mut methods,
    } = protocol;
    let options = PyProtocolOptions::from_attrs(&mut attrs)?;
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = ctx;
    let python_name = options
        .name
        .as_ref()
        .map_or(&ident, |name| &name.value.0)
        .unraw()
        .to_string();

    let mut introspection = Vec::new();
    #[cfg(feature = "experimental-inspect")]
    introspection.push(protocol_introspection_code(
        pyo3_path,
        &ident,
        &python_name,
        get_doc(&attrs, None).as_ref(),
    ));
    let parent: syn::Type = syn::parse_quote!(#ident);
    introspection.extend(
        methods
            .iter_mut()
            .map(|method| method_introspection_code(method, &parent, ctx))
            .try_combine_syn_errors()?,
    );

    #[cfg(feature = "experimental-inspect")]
    let type_hint = quote! {
        #[doc(hidden)]
        pub const _PYO3_INTROSPECTION_ID: &'static str =
            ::core::concat!(::core::module_path!(), "::", ::core::stringify!(#ident));

        /// The protocol as a type hint, e.g. for `FromPyObject::INPUT_TYPE`
        pub const TYPE_HINT: #pyo3_path::inspect::PyStaticExpr =
            #pyo3_path::inspect::PyStaticExpr::PyClass(
                #pyo3_path::inspect::PyClassNameStaticExpr::new(
                    &#pyo3_path::inspect::PyStaticExpr::Name { id: #python_name },
                    Self::_PYO3_INTROSPECTION_ID,
                ),
            );
    };
    #[cfg(not(feature = "experimental-inspect"))]
    let type_hint = {
        let _ = (pyo3_path, python_name);
        quote! {}
    };

    Ok(quote! {
        #(#attrs)*
        #vis struct #ident;

        impl #ident {
            #type_hint
        }

        #(#introspection)*
    })
}

/// Checks a protocol method and returns its introspection chunk
fn method_introspection_code(
    method: &mut TraitItemFn,
    parent: &syn::Type,
    ctx: &Ctx,
) -> Result<TokenStream> {
    let TraitItemFn {
        attrs,
        sig,
        default,
        ..
    } = method;
    if let Some(default) = default {
        bail_spanned!(default.span() => "protocol methods cannot have a body");
    }
    check_generic(sig)?;
    if let Some(asyncness) = &sig.asyncness {
        bail_spanned!(asyncness.span() => "protocol methods cannot be `async`");
    }
    ensure_spanned!(
        matches!(sig.inputs.first(), Some(syn::FnArg::Receiver(_))),
        sig.paren_token.span.join() => "protocol methods must take `self`"
    );

    let options = PyFunctionOptions::from_attrs(attrs)?;
    if let Some(pass_module) = &options.pass_module {
        bail_spanned!(pass_module.span() => "`pass_module` cannot be used on protocol methods");
    }
    if let Some(text_signature) = &options.text_signature {
        bail_spanned!(text_signature.kw.span() => "`text_signature` cannot be used on protocol methods");
    }
    if let Some(krate) = &options.krate {
        bail_spanned!(krate.span() => "`crate` must be set on the protocol, not on its methods");
    }
    if let Some(warning) = options.warnings.first() {
        bail_spanned!(warning.span => "`warn` cannot be used on protocol methods");
    }

    let arguments = sig
        .inputs
        .iter_mut()
        .skip(1)
        .map(FnArg::parse)
        .try_combine_syn_errors()?;
    for argument in &arguments {
        match argument {
            FnArg::Regular(arg) if arg.from_py_with.is_some() => {
                bail_spanned!(arg.name.span() => "`from_py_with` cannot be used on protocol methods")
            }
            FnArg::Py(arg) => {
                bail_spanned!(arg.ty.span() => "protocol methods cannot take a `Python` token")
            }
            FnArg::CancelHandle(arg) => {
                bail_spanned!(arg.name.span() => "`cancel_handle` cannot be used on protocol methods")
            }
            _ => (),
        }
    }
    #[cfg_attr(
        not(feature = "experimental-inspect"),
        expect(unused_variables, reason = "only used by the introspection")
    )]
    let signature = if let Some(signature) = options.signature {
        FunctionSignature::from_arguments_and_attribute(arguments, signature)?
    } else {
        // Implementations name their parameters freely, e.g. `io.BytesIO.write(self, buffer, /)`,
        // and type checkers only match positional-only parameters regardless of their names.
        let mut signature = FunctionSignature::from_arguments(arguments);
        signature.python_signature.positional_only_parameters =
            signature.python_signature.positional_parameters.len();
        signature
    };

    #[cfg(feature = "experimental-inspect")]
    {
        let mut signature = signature;
        // Rust calls these methods: the arguments are converted to Python and the result back
        for argument in &mut signature.arguments {
            if let FnArg::Regular(arg) = argument {
                arg.annotation = Some(PyExpr::from_into_py_object(arg.ty.clone(), None));
            }
        }
        let python_name = options
            .name
            .as_ref()
            .map_or(&sig.ident, |name| &name.value.0)
            .unraw()
            .to_string();
        Ok(function_introspection_code(
            &ctx.pyo3_path,
            None,
            &python_name,
            &signature,
            Some("self"),
            match &sig.output {
                syn::ReturnType::Type(_, t) => PyExpr::from_from_py_object((**t).clone(), None),
                syn::ReturnType::Default => PyExpr::none(),
            },
            std::iter::empty(),
            false,
            false,
            get_doc(attrs, None).as_ref(),
            Some(parent),
        ))
    }
    #[cfg(not(feature = "experimental-inspect"))]
    {
        let _ = (parent, ctx);
        Ok(TokenStream::new())
    }
}
