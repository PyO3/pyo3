use crate::attributes::{self, get_pyo3_options, CrateAttribute};
use crate::utils::Ctx;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned as _;
use syn::{parse_quote, Attribute, DeriveInput, Fields, Result, Token};

/// Attributes for deriving FromPyObject scoped on containers.
enum ContainerPyO3Attribute {
    /// Treat the Container as a Wrapper, directly convert its field into the output object.
    Transparent(attributes::kw::transparent),
    /// Change the path for the pyo3 crate
    Crate(CrateAttribute),
}

impl Parse for ContainerPyO3Attribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::transparent) {
            let kw: attributes::kw::transparent = input.parse()?;
            Ok(ContainerPyO3Attribute::Transparent(kw))
        } else if lookahead.peek(Token![crate]) {
            input.parse().map(ContainerPyO3Attribute::Crate)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Default)]
struct ContainerOptions {
    /// Treat the Container as a Wrapper, directly convert its field into the output object.
    transparent: Option<attributes::kw::transparent>,
    /// Change the path for the pyo3 crate
    krate: Option<CrateAttribute>,
}

impl ContainerOptions {
    fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut options = ContainerOptions::default();

        for attr in attrs {
            if let Some(pyo3_attrs) = get_pyo3_options(attr)? {
                pyo3_attrs
                    .into_iter()
                    .try_for_each(|opt| options.set_option(opt))?;
            }
        }
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
            ContainerPyO3Attribute::Transparent(transparent) => set_option!(transparent),
            ContainerPyO3Attribute::Crate(krate) => set_option!(krate),
        }
        Ok(())
    }
}

struct IntoPyObjectImpl {
    target: TokenStream,
    output: TokenStream,
    error: TokenStream,
    body: TokenStream,
}

struct NamedStructField<'a> {
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
}

struct TupleStructField {}

/// Container Style
///
/// Covers Structs, Tuplestructs and corresponding Newtypes.
enum ContainerType<'a> {
    /// Struct Container, e.g. `struct Foo { a: String }`
    ///
    /// Variant contains the list of field identifiers and the corresponding extraction call.
    Struct(Vec<NamedStructField<'a>>),
    /// Newtype struct container, e.g. `#[transparent] struct Foo { a: String }`
    ///
    /// The field specified by the identifier is extracted directly from the object.
    StructNewtype(&'a syn::Field),
    /// Tuple struct, e.g. `struct Foo(String)`.
    ///
    /// Variant contains a list of conversion methods for each of the fields that are directly
    ///  extracted from the tuple.
    Tuple(Vec<TupleStructField>),
    /// Tuple newtype, e.g. `#[transparent] struct Foo(String)`
    ///
    /// The wrapped field is directly extracted from the object.
    TupleNewtype(&'a syn::Field),
}

/// Data container
///
/// Either describes a struct or an enum variant.
struct Container<'a> {
    path: syn::Path,
    ty: ContainerType<'a>,
    err_name: String,
}

impl<'a> Container<'a> {
    /// Construct a container based on fields, identifier and attributes.
    ///
    /// Fails if the variant has no fields or incompatible attributes.
    fn new(fields: &'a Fields, path: syn::Path, options: ContainerOptions) -> Result<Self> {
        let style = match fields {
            Fields::Unnamed(unnamed) if !unnamed.unnamed.is_empty() => {
                if unnamed.unnamed.iter().count() == 1 {
                    // Always treat a 1-length tuple struct as "transparent", even without the
                    // explicit annotation.
                    let field = unnamed.unnamed.iter().next().unwrap();
                    ContainerType::TupleNewtype(field)
                } else if options.transparent.is_some() {
                    bail_spanned!(
                        fields.span() => "transparent structs and variants can only have 1 field"
                    );
                } else {
                    let tuple_fields = unnamed
                        .unnamed
                        .iter()
                        .map(|_field| Ok(TupleStructField {}))
                        .collect::<Result<Vec<_>>>()?;

                    ContainerType::Tuple(tuple_fields)
                }
            }
            Fields::Named(named) if !named.named.is_empty() => {
                if options.transparent.is_some() {
                    ensure_spanned!(
                        named.named.iter().count() == 1,
                        fields.span() => "transparent structs and variants can only have 1 field"
                    );

                    let field = named.named.iter().next().unwrap();
                    ContainerType::StructNewtype(field)
                } else {
                    let struct_fields = named
                        .named
                        .iter()
                        .map(|field| {
                            let ident = field
                                .ident
                                .as_ref()
                                .expect("Named fields should have identifiers");
                            let ty = &field.ty;

                            Ok(NamedStructField { ident, ty })
                        })
                        .collect::<Result<Vec<_>>>()?;
                    ContainerType::Struct(struct_fields)
                }
            }
            _ => bail_spanned!(
                fields.span() => "cannot derive `IntoPyObject` for empty structs and variants"
            ),
        };
        let err_name = path.segments.last().unwrap().ident.to_string();

        let v = Container {
            path,
            ty: style,
            err_name,
        };
        Ok(v)
    }

    fn name(&self) -> String {
        let mut value = String::new();
        for segment in &self.path.segments {
            if !value.is_empty() {
                value.push_str("::");
            }
            value.push_str(&segment.ident.to_string());
        }
        value
    }

    /// Build derivation body for a struct.
    fn build(&self, ctx: &Ctx) -> IntoPyObjectImpl {
        match &self.ty {
            ContainerType::StructNewtype(field) | ContainerType::TupleNewtype(field) => {
                self.build_newtype_struct(field, ctx)
            }
            ContainerType::Tuple(tups) => todo!(), // self.build_tuple_struct(tups, ctx),
            ContainerType::Struct(tups) => todo!(), // self.build_struct(tups, ctx),
        }
    }

    fn build_newtype_struct(&self, field: &syn::Field, ctx: &Ctx) -> IntoPyObjectImpl {
        let Ctx { pyo3_path, .. } = ctx;
        let ty = &field.ty;
        let ident = if let Some(ident) = &field.ident {
            quote! {self.#ident}
        } else {
            quote! {self.0}
        };

        IntoPyObjectImpl {
            target: quote! {<#ty as #pyo3_path::conversion::IntoPyObject<'py>>::Target},
            output: quote! {<#ty as #pyo3_path::conversion::IntoPyObject<'py>>::Output},
            error: quote! {<#ty as #pyo3_path::conversion::IntoPyObject<'py>>::Error},
            body: quote! { <#ty as #pyo3_path::conversion::IntoPyObject<'py>>::into_pyobject(#ident, py) },
        }
    }
}

pub fn build_derive_into_pyobject(tokens: &DeriveInput) -> Result<TokenStream> {
    let options = ContainerOptions::from_attrs(&tokens.attrs)?;
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = &ctx;

    let mut trait_generics = tokens.generics.clone();
    let generics = &tokens.generics;
    // let lt_param = if let Some(lt) = verify_and_get_lifetime(generics)? {
    //     lt.clone()
    // } else {
    trait_generics.params.push(parse_quote!('py));
    // parse_quote!('py)
    // };
    let mut where_clause: syn::WhereClause = parse_quote!(where);
    for param in generics.type_params() {
        let gen_ident = &param.ident;
        where_clause
            .predicates
            .push(parse_quote!(#gen_ident: #pyo3_path::conversion::IntoPyObject<'py>))
    }

    let IntoPyObjectImpl {
        target,
        output,
        error,
        body,
    } = match &tokens.data {
        syn::Data::Enum(en) => {
            // if options.transparent || options.annotation.is_some() {
            //     bail_spanned!(tokens.span() => "`transparent` or `annotation` is not supported \
            //                                     at top level for enums");
            // }
            // let en = Enum::new(en, &tokens.ident)?;
            // en.build(ctx)
            todo!()
        }
        syn::Data::Struct(st) => {
            let ident = &tokens.ident;
            let st = Container::new(&st.fields, parse_quote!(#ident), options)?;
            st.build(ctx)
        }
        syn::Data::Union(_) => bail_spanned!(
            tokens.span() => "#[derive(FromPyObject)] is not supported for unions"
        ),
    };

    let ident = &tokens.ident;
    Ok(quote!(
        #[automatically_derived]
        impl #trait_generics #pyo3_path::conversion::IntoPyObject<'py> for #ident #generics #where_clause {
            type Target = #target;
            type Output = #output;
            type Error = #error;

            fn into_pyobject(self, py: #pyo3_path::Python<'py>) -> ::std::result::Result<Self::Output, Self::Error> {
                #body
            }
        }
    ))
}
