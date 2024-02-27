use crate::utils::Ctx;
use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};

pub enum Deprecation {
    PyMethodsNewDeprecatedForm,
}

impl Deprecation {
    fn ident(&self, span: Span) -> syn::Ident {
        let string = match self {
            Deprecation::PyMethodsNewDeprecatedForm => "PYMETHODS_NEW_DEPRECATED_FORM",
        };
        syn::Ident::new(string, span)
    }
}

pub struct Deprecations<'ctx>(Vec<(Deprecation, Span)>, &'ctx Ctx);

impl<'ctx> Deprecations<'ctx> {
    pub fn new(ctx: &'ctx Ctx) -> Self {
        Deprecations(Vec::new(), ctx)
    }

    pub fn push(&mut self, deprecation: Deprecation, span: Span) {
        self.0.push((deprecation, span))
    }
}

impl<'ctx> ToTokens for Deprecations<'ctx> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self(deprecations, Ctx { pyo3_path }) = self;

        for (deprecation, span) in deprecations {
            let ident = deprecation.ident(*span);
            quote_spanned!(
                *span =>
                #[allow(clippy::let_unit_value)]
                {
                    let _ = #pyo3_path::impl_::deprecations::#ident;
                }
            )
            .to_tokens(tokens)
        }
    }
}
