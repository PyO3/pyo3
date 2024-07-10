use crate::attributes::{self, take_pyo3_options, CrateAttribute, KeywordAttribute, NameAttribute};
use crate::utils::Ctx;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{spanned::Spanned, Attribute, DeriveInput, Result, Token};

pub fn build_derive_pyexception(tokens: &DeriveInput) -> Result<TokenStream> {
    let options = ContainerOptions::from_attrs(&tokens.attrs)?;
    let ctx = &Ctx::new(&options.krate);
    let Ctx { pyo3_path } = &ctx;
    let krate = quote!(#pyo3_path).to_string();

    let derives = match &tokens.data {
        syn::Data::Enum(en) => {
            let vis = &tokens.vis;
            let ident = &tokens.ident;
            let python_name = ident.to_string();
            let base_exception = format_ident!("Py{}", ident);

            let mut variant_match = TokenStream::new();
            let mut variant_exceptions = TokenStream::new();

            for variant in &en.variants {
                let python_name = variant.ident.to_string();
                let exception = format_ident!("Py{}", variant.ident);
                let variant = &variant.ident;

                variant_match.extend(quote! {
                    #ident::#variant { .. } => Self::new::<#exception, _>(::std::string::ToString::to_string(&value)),
                });

                variant_exceptions.extend(quote! {
                    #[#pyo3_path::pyclass(crate = #krate)]
                    #[pyo3(name = #python_name, extends = #base_exception, subclass)]
                    #[automatically_derived]
                    #vis struct #exception;

                    #[#pyo3_path::pymethods(crate = #krate)]
                    #[automatically_derived]
                    impl #exception {
                        #[new]
                        #[pyo3(signature = (*args, **kwargs))]
                        pub fn new(
                            args: #pyo3_path::Bound<'_, #pyo3_path::types::PyTuple>,
                            kwargs: ::std::option::Option<#pyo3_path::Bound<'_, #pyo3_path::types::PyDict>>
                        ) -> #pyo3_path::PyClassInitializer<Self> {
                            #pyo3_path::PyClassInitializer::from(#base_exception).add_subclass(Self)
                        }
                    }
                })
            }

            let (impl_generics, ty_generics, where_clause) = tokens.generics.split_for_impl();
            quote! {
                #[#pyo3_path::pyclass(crate = #krate)]
                #[pyo3(name = #python_name, extends = #pyo3_path::exceptions::PyException, subclass)]
                #[automatically_derived]
                #vis struct #base_exception;

                #[#pyo3_path::pymethods(crate = #krate)]
                #[automatically_derived]
                impl #base_exception {
                    #[new]
                    #[pyo3(signature = (*args, **kwargs))]
                    pub fn new(
                        args: #pyo3_path::Bound<'_, #pyo3_path::types::PyTuple>,
                        kwargs: ::std::option::Option<#pyo3_path::Bound<'_, #pyo3_path::types::PyDict>>
                    ) -> Self {
                        Self
                    }
                }

                #variant_exceptions

                #[automatically_derived]
                impl #impl_generics ::std::convert::From<#ident #ty_generics> for #pyo3_path::PyErr #where_clause {
                    fn from(value: #ident #ty_generics) -> Self {
                        match value {
                            #variant_match
                        }
                    }
                }

            }
        }
        syn::Data::Struct(..) => {
            let vis = &tokens.vis;
            let name_opt = options.name.map(|KeywordAttribute { value, .. }| value.0);
            let ident = &tokens.ident;
            let python_name = name_opt
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| ident.to_string());
            let exception = name_opt.unwrap_or_else(|| format_ident!("Py{}", ident));

            let (impl_generics, ty_generics, where_clause) = tokens.generics.split_for_impl();
            quote! {
                #[#pyo3_path::pyclass(crate = #krate)]
                #[pyo3(name = #python_name, extends = #pyo3_path::exceptions::PyException, subclass)]
                #[automatically_derived]
                #vis struct #exception;

                #[#pyo3_path::pymethods(crate = #krate)]
                #[automatically_derived]
                impl #exception {
                    #[new]
                    #[pyo3(signature = (*args, **kwargs))]
                    pub fn new(
                        args: #pyo3_path::Bound<'_, #pyo3_path::types::PyTuple>,
                        kwargs: ::std::option::Option<#pyo3_path::Bound<'_, #pyo3_path::types::PyDict>>
                    ) -> Self {
                        Self
                    }
                }

                #[automatically_derived]
                impl #impl_generics ::std::convert::From<#ident #ty_generics> for #pyo3_path::PyErr #where_clause {
                    fn from(value: #ident #ty_generics) -> Self {
                        Self::new::<#exception, _>(::std::string::ToString::to_string(&value))
                    }
                }
            }
        }
        syn::Data::Union(_) => bail_spanned!(
            tokens.span() => "#[derive(PyException)] is not supported for unions"
        ),
    };

    Ok(derives)
}

#[derive(Default)]
struct ContainerOptions {
    name: Option<NameAttribute>,
    krate: Option<CrateAttribute>,
}

enum ContainerPyO3Attribute {
    Name(NameAttribute),
    Crate(CrateAttribute),
}

impl Parse for ContainerPyO3Attribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![crate]) {
            input.parse().map(ContainerPyO3Attribute::Crate)
        } else if lookahead.peek(attributes::kw::name) {
            input.parse().map(ContainerPyO3Attribute::Name)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ContainerOptions {
    fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut options = ContainerOptions::default();

        take_pyo3_options(&mut attrs.to_vec())?
            .into_iter()
            .try_for_each(|option| options.set_option(option))?;

        Ok(options)
    }

    fn set_option(&mut self, option: ContainerPyO3Attribute) -> syn::Result<()> {
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

        match option {
            ContainerPyO3Attribute::Crate(krate) => set_option!(krate),
            ContainerPyO3Attribute::Name(name) => set_option!(name),
        }
        Ok(())
    }
}
