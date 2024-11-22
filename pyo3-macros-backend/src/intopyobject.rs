use crate::attributes::{self, get_pyo3_options, CrateAttribute};
use crate::utils::Ctx;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned as _;
use syn::{
    parenthesized, parse_quote, Attribute, DataEnum, DeriveInput, Fields, Ident, Index, Result,
    Token,
};

/// Attributes for deriving `IntoPyObject` scoped on containers.
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

#[derive(Debug, Clone)]
struct ItemOption {
    field: Option<syn::LitStr>,
    span: Span,
}

impl ItemOption {
    fn span(&self) -> Span {
        self.span
    }
}

enum FieldAttribute {
    Item(ItemOption),
}

impl Parse for FieldAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::attribute) {
            let attr: attributes::kw::attribute = input.parse()?;
            bail_spanned!(attr.span => "`attribute` is not supported by `IntoPyObject`");
        } else if lookahead.peek(attributes::kw::item) {
            let attr: attributes::kw::item = input.parse()?;
            if input.peek(syn::token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                let key = content.parse()?;
                if !content.is_empty() {
                    return Err(
                        content.error("expected at most one argument: `item` or `item(key)`")
                    );
                }
                Ok(FieldAttribute::Item(ItemOption {
                    field: Some(key),
                    span: attr.span,
                }))
            } else {
                Ok(FieldAttribute::Item(ItemOption {
                    field: None,
                    span: attr.span,
                }))
            }
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Clone, Debug, Default)]
struct FieldAttributes {
    item: Option<ItemOption>,
}

impl FieldAttributes {
    /// Extract the field attributes.
    fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut options = FieldAttributes::default();

        for attr in attrs {
            if let Some(pyo3_attrs) = get_pyo3_options(attr)? {
                pyo3_attrs
                    .into_iter()
                    .try_for_each(|opt| options.set_option(opt))?;
            }
        }
        Ok(options)
    }

    fn set_option(&mut self, option: FieldAttribute) -> syn::Result<()> {
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
            FieldAttribute::Item(item) => set_option!(item),
        }
        Ok(())
    }
}

enum IntoPyObjectTypes {
    Transparent(syn::Type),
    Opaque {
        target: TokenStream,
        output: TokenStream,
        error: TokenStream,
    },
}

struct IntoPyObjectImpl {
    types: IntoPyObjectTypes,
    body: TokenStream,
}

struct NamedStructField<'a> {
    ident: &'a syn::Ident,
    field: &'a syn::Field,
    item: Option<ItemOption>,
}

struct TupleStructField<'a> {
    field: &'a syn::Field,
}

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
    Tuple(Vec<TupleStructField<'a>>),
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
    receiver: Option<Ident>,
    ty: ContainerType<'a>,
}

/// Construct a container based on fields, identifier and attributes.
impl<'a> Container<'a> {
    ///
    /// Fails if the variant has no fields or incompatible attributes.
    fn new(
        receiver: Option<Ident>,
        fields: &'a Fields,
        path: syn::Path,
        options: ContainerOptions,
    ) -> Result<Self> {
        let style = match fields {
            Fields::Unnamed(unnamed) if !unnamed.unnamed.is_empty() => {
                let mut tuple_fields = unnamed
                    .unnamed
                    .iter()
                    .map(|field| {
                        let attrs = FieldAttributes::from_attrs(&field.attrs)?;
                        ensure_spanned!(
                            attrs.item.is_none(),
                            attrs.item.unwrap().span() => "`item` is not permitted on tuple struct elements."
                        );
                        Ok(TupleStructField { field })
                    })
                    .collect::<Result<Vec<_>>>()?;
                if tuple_fields.len() == 1 {
                    // Always treat a 1-length tuple struct as "transparent", even without the
                    // explicit annotation.
                    let TupleStructField { field } = tuple_fields.pop().unwrap();
                    ContainerType::TupleNewtype(field)
                } else if options.transparent.is_some() {
                    bail_spanned!(
                        fields.span() => "transparent structs and variants can only have 1 field"
                    );
                } else {
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
                    let attrs = FieldAttributes::from_attrs(&field.attrs)?;
                    ensure_spanned!(
                        attrs.item.is_none(),
                        attrs.item.unwrap().span() => "`transparent` structs may not have `item` for the inner field"
                    );
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

                            let attrs = FieldAttributes::from_attrs(&field.attrs)?;

                            Ok(NamedStructField {
                                ident,
                                field,
                                item: attrs.item,
                            })
                        })
                        .collect::<Result<Vec<_>>>()?;
                    ContainerType::Struct(struct_fields)
                }
            }
            _ => bail_spanned!(
                fields.span() => "cannot derive `IntoPyObject` for empty structs"
            ),
        };

        let v = Container {
            path,
            receiver,
            ty: style,
        };
        Ok(v)
    }

    fn match_pattern(&self) -> TokenStream {
        let path = &self.path;
        let pattern = match &self.ty {
            ContainerType::Struct(fields) => fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let ident = f.ident;
                    let new_ident = format_ident!("arg{i}");
                    quote! {#ident: #new_ident,}
                })
                .collect::<TokenStream>(),
            ContainerType::StructNewtype(field) => {
                let ident = field.ident.as_ref().unwrap();
                quote!(#ident: arg0)
            }
            ContainerType::Tuple(fields) => {
                let i = (0..fields.len()).map(Index::from);
                let idents = (0..fields.len()).map(|i| format_ident!("arg{i}"));
                quote! { #(#i: #idents,)* }
            }
            ContainerType::TupleNewtype(_) => quote!(0: arg0),
        };

        quote! { #path{ #pattern } }
    }

    /// Build derivation body for a struct.
    fn build(&self, ctx: &Ctx) -> IntoPyObjectImpl {
        match &self.ty {
            ContainerType::StructNewtype(field) | ContainerType::TupleNewtype(field) => {
                self.build_newtype_struct(field, ctx)
            }
            ContainerType::Tuple(fields) => self.build_tuple_struct(fields, ctx),
            ContainerType::Struct(fields) => self.build_struct(fields, ctx),
        }
    }

    fn build_newtype_struct(&self, field: &syn::Field, ctx: &Ctx) -> IntoPyObjectImpl {
        let Ctx { pyo3_path, .. } = ctx;
        let ty = &field.ty;

        let unpack = self
            .receiver
            .as_ref()
            .map(|i| {
                let pattern = self.match_pattern();
                quote! { let #pattern = #i;}
            })
            .unwrap_or_default();

        IntoPyObjectImpl {
            types: IntoPyObjectTypes::Transparent(ty.clone()),
            body: quote_spanned! { ty.span() =>
                #unpack
                #pyo3_path::conversion::IntoPyObject::into_pyobject(arg0, py)
            },
        }
    }

    fn build_struct(&self, fields: &[NamedStructField<'_>], ctx: &Ctx) -> IntoPyObjectImpl {
        let Ctx { pyo3_path, .. } = ctx;

        let unpack = self
            .receiver
            .as_ref()
            .map(|i| {
                let pattern = self.match_pattern();
                quote! { let #pattern = #i;}
            })
            .unwrap_or_default();

        let setter = fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let key = f
                    .item
                    .as_ref()
                    .and_then(|item| item.field.as_ref())
                    .map(|item| item.value())
                    .unwrap_or_else(|| f.ident.unraw().to_string());
                let value = Ident::new(&format!("arg{i}"), f.field.ty.span());
                quote! {
                    #pyo3_path::types::PyDictMethods::set_item(&dict, #key, #value)?;
                }
            })
            .collect::<TokenStream>();

        IntoPyObjectImpl {
            types: IntoPyObjectTypes::Opaque {
                target: quote!(#pyo3_path::types::PyDict),
                output: quote!(#pyo3_path::Bound<'py, Self::Target>),
                error: quote!(#pyo3_path::PyErr),
            },
            body: quote! {
                #unpack
                let dict = #pyo3_path::types::PyDict::new(py);
                #setter
                ::std::result::Result::Ok::<_, Self::Error>(dict)
            },
        }
    }

    fn build_tuple_struct(&self, fields: &[TupleStructField<'_>], ctx: &Ctx) -> IntoPyObjectImpl {
        let Ctx { pyo3_path, .. } = ctx;

        let unpack = self
            .receiver
            .as_ref()
            .map(|i| {
                let pattern = self.match_pattern();
                quote! { let #pattern = #i;}
            })
            .unwrap_or_default();

        let setter = fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let value = Ident::new(&format!("arg{i}"), f.field.ty.span());
                quote_spanned! { f.field.ty.span() =>
                    #pyo3_path::conversion::IntoPyObject::into_pyobject(#value, py)
                        .map(#pyo3_path::BoundObject::into_any)
                        .map(#pyo3_path::BoundObject::into_bound)?,
                }
            })
            .collect::<TokenStream>();

        IntoPyObjectImpl {
            types: IntoPyObjectTypes::Opaque {
                target: quote!(#pyo3_path::types::PyTuple),
                output: quote!(#pyo3_path::Bound<'py, Self::Target>),
                error: quote!(#pyo3_path::PyErr),
            },
            body: quote! {
                #unpack
                #pyo3_path::types::PyTuple::new(py, [#setter])
            },
        }
    }
}

/// Describes derivation input of an enum.
struct Enum<'a> {
    variants: Vec<Container<'a>>,
}

impl<'a> Enum<'a> {
    /// Construct a new enum representation.
    ///
    /// `data_enum` is the `syn` representation of the input enum, `ident` is the
    /// `Identifier` of the enum.
    fn new(data_enum: &'a DataEnum, ident: &'a Ident) -> Result<Self> {
        ensure_spanned!(
            !data_enum.variants.is_empty(),
            ident.span() => "cannot derive `IntoPyObject` for empty enum"
        );
        let variants = data_enum
            .variants
            .iter()
            .map(|variant| {
                let attrs = ContainerOptions::from_attrs(&variant.attrs)?;
                let var_ident = &variant.ident;

                ensure_spanned!(
                    !variant.fields.is_empty(),
                    variant.ident.span() => "cannot derive `IntoPyObject` for empty variants"
                );

                Container::new(
                    None,
                    &variant.fields,
                    parse_quote!(#ident::#var_ident),
                    attrs,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Enum { variants })
    }

    /// Build derivation body for enums.
    fn build(&self, ctx: &Ctx) -> IntoPyObjectImpl {
        let Ctx { pyo3_path, .. } = ctx;

        let variants = self
            .variants
            .iter()
            .map(|v| {
                let IntoPyObjectImpl { body, .. } = v.build(ctx);
                let pattern = v.match_pattern();
                quote! {
                    #pattern => {
                        {#body}
                            .map(#pyo3_path::BoundObject::into_any)
                            .map(#pyo3_path::BoundObject::into_bound)
                            .map_err(::std::convert::Into::<#pyo3_path::PyErr>::into)
                    }
                }
            })
            .collect::<TokenStream>();

        IntoPyObjectImpl {
            types: IntoPyObjectTypes::Opaque {
                target: quote!(#pyo3_path::types::PyAny),
                output: quote!(#pyo3_path::Bound<'py, Self::Target>),
                error: quote!(#pyo3_path::PyErr),
            },
            body: quote! {
                match self {
                    #variants
                }
            },
        }
    }
}

// if there is a `'py` lifetime, we treat it as the `Python<'py>` lifetime
fn verify_and_get_lifetime(generics: &syn::Generics) -> Option<&syn::LifetimeParam> {
    let mut lifetimes = generics.lifetimes();
    lifetimes.find(|l| l.lifetime.ident == "py")
}

pub fn build_derive_into_pyobject<const REF: bool>(tokens: &DeriveInput) -> Result<TokenStream> {
    let options = ContainerOptions::from_attrs(&tokens.attrs)?;
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = &ctx;

    let (_, ty_generics, _) = tokens.generics.split_for_impl();
    let mut trait_generics = tokens.generics.clone();
    if REF {
        trait_generics.params.push(parse_quote!('_a));
    }
    let lt_param = if let Some(lt) = verify_and_get_lifetime(&trait_generics) {
        lt.clone()
    } else {
        trait_generics.params.push(parse_quote!('py));
        parse_quote!('py)
    };
    let (impl_generics, _, where_clause) = trait_generics.split_for_impl();

    let mut where_clause = where_clause.cloned().unwrap_or_else(|| parse_quote!(where));
    for param in trait_generics.type_params() {
        let gen_ident = &param.ident;
        where_clause.predicates.push(if REF {
            parse_quote!(&'_a #gen_ident: #pyo3_path::conversion::IntoPyObject<'py>)
        } else {
            parse_quote!(#gen_ident: #pyo3_path::conversion::IntoPyObject<'py>)
        })
    }

    let IntoPyObjectImpl { types, body } = match &tokens.data {
        syn::Data::Enum(en) => {
            if options.transparent.is_some() {
                bail_spanned!(tokens.span() => "`transparent` is not supported at top level for enums");
            }
            let en = Enum::new(en, &tokens.ident)?;
            en.build(ctx)
        }
        syn::Data::Struct(st) => {
            let ident = &tokens.ident;
            let st = Container::new(
                Some(Ident::new("self", Span::call_site())),
                &st.fields,
                parse_quote!(#ident),
                options,
            )?;
            st.build(ctx)
        }
        syn::Data::Union(_) => bail_spanned!(
            tokens.span() => "#[derive(`IntoPyObject`)] is not supported for unions"
        ),
    };

    let (target, output, error) = match types {
        IntoPyObjectTypes::Transparent(ty) => {
            if REF {
                (
                    quote! { <&'_a #ty as #pyo3_path::IntoPyObject<'py>>::Target },
                    quote! { <&'_a #ty as #pyo3_path::IntoPyObject<'py>>::Output },
                    quote! { <&'_a #ty as #pyo3_path::IntoPyObject<'py>>::Error },
                )
            } else {
                (
                    quote! { <#ty as #pyo3_path::IntoPyObject<'py>>::Target },
                    quote! { <#ty as #pyo3_path::IntoPyObject<'py>>::Output },
                    quote! { <#ty as #pyo3_path::IntoPyObject<'py>>::Error },
                )
            }
        }
        IntoPyObjectTypes::Opaque {
            target,
            output,
            error,
        } => (target, output, error),
    };

    let ident = &tokens.ident;
    let ident = if REF {
        quote! { &'_a #ident}
    } else {
        quote! { #ident }
    };
    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics #pyo3_path::conversion::IntoPyObject<#lt_param> for #ident #ty_generics #where_clause {
            type Target = #target;
            type Output = #output;
            type Error = #error;

            fn into_pyobject(self, py: #pyo3_path::Python<#lt_param>) -> ::std::result::Result<Self::Output, Self::Error> {
                #body
            }
        }
    ))
}
