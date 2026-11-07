//! `pyprotocol!`: declares a `typing.Protocol` that only exists in the generated type stubs.

use crate::attributes::{self, CrateAttribute, NameAttribute, take_pyo3_options};
use crate::combine_errors::CombineErrors;
use crate::introspection::{ProtocolMethod, protocol_introspection_code};
use crate::method::FnArg;
use crate::py_expr::PyExpr;
use crate::pyfunction::{FunctionSignature, SignatureAttribute};
use crate::pymethod::check_generic;
use crate::utils::{Ctx, get_doc};
use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, ReceiverKind, Result, Token, TraitItemFn, Visibility, braced};

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

/// The `#[pyo3]` options of a protocol
#[derive(Default)]
struct ProtocolOptions {
    name: Option<NameAttribute>,
    krate: Option<CrateAttribute>,
}

enum ProtocolOption {
    Name(NameAttribute),
    Crate(CrateAttribute),
}

impl Parse for ProtocolOption {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::name) {
            input.parse().map(Self::Name)
        } else if lookahead.peek(Token![crate]) {
            input.parse().map(Self::Crate)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ProtocolOptions {
    fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut options = Self::default();
        for option in take_pyo3_options(attrs)? {
            match option {
                ProtocolOption::Name(name) => set_option(&mut options.name, name, "name")?,
                ProtocolOption::Crate(krate) => set_option(&mut options.krate, krate, "crate")?,
            }
        }
        Ok(options)
    }
}

/// The `#[pyo3]` options of a protocol method
#[derive(Default)]
struct ProtocolMethodOptions {
    name: Option<NameAttribute>,
    signature: Option<SignatureAttribute>,
}

enum ProtocolMethodOption {
    Name(NameAttribute),
    Signature(SignatureAttribute),
}

impl Parse for ProtocolMethodOption {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::name) {
            input.parse().map(Self::Name)
        } else if lookahead.peek(attributes::kw::signature) {
            input.parse().map(Self::Signature)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ProtocolMethodOptions {
    fn from_attrs(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut options = Self::default();
        for option in take_pyo3_options(attrs)? {
            match option {
                ProtocolMethodOption::Name(name) => set_option(&mut options.name, name, "name")?,
                ProtocolMethodOption::Signature(signature) => {
                    set_option(&mut options.signature, signature, "signature")?
                }
            }
        }
        Ok(options)
    }
}

fn set_option<T: Spanned>(option: &mut Option<T>, value: T, key: &str) -> Result<()> {
    ensure_spanned!(
        option.is_none(),
        value.span() => format!("`{key}` may only be specified once")
    );
    *option = Some(value);
    Ok(())
}

pub fn build_py_protocol(protocol: PyProtocol) -> Result<TokenStream> {
    let PyProtocol {
        mut attrs,
        vis,
        ident,
        mut methods,
    } = protocol;
    let options = ProtocolOptions::from_attrs(&mut attrs)?;
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = ctx;
    let python_name = options
        .name
        .as_ref()
        .map_or(&ident, |name| &name.value.0)
        .unraw()
        .to_string();

    let methods = methods
        .iter_mut()
        .map(protocol_method)
        .try_combine_syn_errors()?;
    let introspection = protocol_introspection_code(
        pyo3_path,
        &ident,
        &python_name,
        get_doc(&attrs, None).as_ref(),
        &methods,
    );

    Ok(quote! {
        #(#attrs)*
        #vis struct #ident;

        impl #ident {
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
        }

        #introspection
    })
}

/// Checks a protocol method and returns what its introspection needs
fn protocol_method(method: &mut TraitItemFn) -> Result<ProtocolMethod<'_>> {
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
        matches!(
            sig.inputs.first(),
            Some(syn::FnArg::Receiver(syn::Receiver {
                kind: ReceiverKind::Reference(_, _, None),
                ..
            }))
        ),
        sig.paren_token.span.join() => "protocol methods must take `&self`"
    );

    let options = ProtocolMethodOptions::from_attrs(attrs)?;
    let arguments = sig
        .inputs
        .iter_mut()
        .skip(1)
        .map(FnArg::parse)
        .try_combine_syn_errors()?;
    let mut signature = if let Some(signature) = options.signature {
        FunctionSignature::from_arguments_and_attribute(arguments, signature)?
    } else {
        // Implementations name their parameters freely, e.g. `io.BytesIO.write(self, buffer, /)`,
        // and type checkers only match positional-only parameters regardless of their names.
        let mut signature = FunctionSignature::from_arguments(arguments);
        signature.python_signature.positional_only_parameters =
            signature.python_signature.positional_parameters.len();
        signature
    };

    // Rust calls these methods, so an argument is annotated with what Rust passes
    // (`IntoPyObject::OUTPUT_TYPE`, e.g. `int` for `isize`), not with what a function would accept
    // (`FromPyObject::INPUT_TYPE`, e.g. `SupportsIndex`), unless the signature gives an annotation
    for argument in &mut signature.arguments {
        if let FnArg::Regular(arg) = argument {
            arg.annotation
                .get_or_insert_with(|| PyExpr::from_into_py_object(arg.ty.clone(), None));
        }
    }
    Ok(ProtocolMethod {
        name: options
            .name
            .as_ref()
            .map_or(&sig.ident, |name| &name.value.0)
            .unraw()
            .to_string(),
        signature,
        returns: match &sig.output {
            syn::ReturnType::Type(_, t) => PyExpr::from_from_py_object((**t).clone(), None),
            syn::ReturnType::Default => PyExpr::none(),
        },
        doc: get_doc(attrs, None),
    })
}
