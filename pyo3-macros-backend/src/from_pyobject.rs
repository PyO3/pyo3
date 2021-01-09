use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_quote, Attribute, DataEnum, DeriveInput, Fields, Ident, Meta, MetaList, Result};

/// Describes derivation input of an enum.
#[derive(Debug)]
struct Enum<'a> {
    enum_ident: &'a Ident,
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
            ident.span() => "cannot derive FromPyObject for empty enum"
        );
        let variants = data_enum
            .variants
            .iter()
            .map(|variant| {
                let attrs = ContainerAttribute::parse_attrs(&variant.attrs)?;
                let var_ident = &variant.ident;
                Container::new(
                    &variant.fields,
                    parse_quote!(#ident::#var_ident),
                    attrs,
                    true,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Enum {
            enum_ident: ident,
            variants,
        })
    }

    /// Build derivation body for enums.
    fn build(&self) -> TokenStream {
        let mut var_extracts = Vec::new();
        let mut error_names = String::new();
        for (i, var) in self.variants.iter().enumerate() {
            let struct_derive = var.build();
            let ext = quote!(
                let maybe_ret = || -> pyo3::PyResult<Self> {
                    #struct_derive
                }();
                if maybe_ret.is_ok() {
                    return maybe_ret
                }
            );

            var_extracts.push(ext);
            error_names.push_str(&var.err_name);
            if i < self.variants.len() - 1 {
                error_names.push_str(", ");
            }
        }
        let error_names = if self.variants.len() > 1 {
            format!("Union[{}]", error_names)
        } else {
            error_names
        };
        quote!(
            #(#var_extracts)*
            let type_name = obj.get_type().name()?;
            let err_msg = format!("'{}' object cannot be converted to '{}'", type_name, #error_names);
            Err(pyo3::exceptions::PyTypeError::new_err(err_msg))
        )
    }
}

/// Container Style
///
/// Covers Structs, Tuplestructs and corresponding Newtypes.
#[derive(Debug)]
enum ContainerType<'a> {
    /// Struct Container, e.g. `struct Foo { a: String }`
    ///
    /// Variant contains the list of field identifiers and the corresponding extraction call.
    Struct(Vec<(&'a Ident, FieldAttribute)>),
    /// Newtype struct container, e.g. `#[transparent] struct Foo { a: String }`
    ///
    /// The field specified by the identifier is extracted directly from the object.
    StructNewtype(&'a Ident),
    /// Tuple struct, e.g. `struct Foo(String)`.
    ///
    /// Fields are extracted from a tuple.
    Tuple(usize),
    /// Tuple newtype, e.g. `#[transparent] struct Foo(String)`
    ///
    /// The wrapped field is directly extracted from the object.
    TupleNewtype,
}

/// Data container
///
/// Either describes a struct or an enum variant.
#[derive(Debug)]
struct Container<'a> {
    path: syn::Path,
    ty: ContainerType<'a>,
    err_name: String,
    is_enum_variant: bool,
}

impl<'a> Container<'a> {
    /// Construct a container based on fields, identifier and attributes.
    ///
    /// Fails if the variant has no fields or incompatible attributes.
    fn new(
        fields: &'a Fields,
        path: syn::Path,
        attrs: Vec<ContainerAttribute>,
        is_enum_variant: bool,
    ) -> Result<Self> {
        ensure_spanned!(
            !fields.is_empty(),
            fields.span() => "cannot derive FromPyObject for empty structs and variants"
        );
        let transparent = attrs
            .iter()
            .any(|attr| *attr == ContainerAttribute::Transparent);
        if transparent {
            Self::check_transparent_len(fields)?;
        }
        let style = match (fields, transparent) {
            (Fields::Unnamed(_), true) => ContainerType::TupleNewtype,
            (Fields::Unnamed(unnamed), false) => match unnamed.unnamed.len() {
                1 => ContainerType::TupleNewtype,
                len => ContainerType::Tuple(len),
            },
            (Fields::Named(named), true) => {
                let field = named
                    .named
                    .iter()
                    .next()
                    .expect("Check for len 1 is done above");
                let ident = field
                    .ident
                    .as_ref()
                    .expect("Named fields should have identifiers");
                ContainerType::StructNewtype(ident)
            }
            (Fields::Named(named), false) => {
                let mut fields = Vec::new();
                for field in named.named.iter() {
                    let ident = field
                        .ident
                        .as_ref()
                        .expect("Named fields should have identifiers");
                    let attr = FieldAttribute::parse_attrs(&field.attrs)?
                        .unwrap_or(FieldAttribute::GetAttr(None));
                    fields.push((ident, attr))
                }
                ContainerType::Struct(fields)
            }
            (Fields::Unit, _) => unreachable!(), // covered by length check above
        };
        let err_name = attrs
            .iter()
            .find_map(|a| a.annotation().map(syn::LitStr::value))
            .unwrap_or_else(|| path.segments.last().unwrap().ident.to_string());

        let v = Container {
            path,
            ty: style,
            err_name,
            is_enum_variant,
        };
        Ok(v)
    }

    fn verify_struct_container_attrs(attrs: &'a [ContainerAttribute]) -> Result<()> {
        for attr in attrs {
            match attr {
                ContainerAttribute::Transparent => {}
                ContainerAttribute::ErrorAnnotation(annotation) => bail_spanned!(
                    annotation.span() => "annotation is not supported for structs"
                ),
            }
        }
        Ok(())
    }

    /// Build derivation body for a struct.
    fn build(&self) -> TokenStream {
        match &self.ty {
            ContainerType::StructNewtype(ident) => self.build_newtype_struct(Some(&ident)),
            ContainerType::TupleNewtype => self.build_newtype_struct(None),
            ContainerType::Tuple(len) => self.build_tuple_struct(*len),
            ContainerType::Struct(tups) => self.build_struct(tups),
        }
    }

    fn build_newtype_struct(&self, field_ident: Option<&Ident>) -> TokenStream {
        let self_ty = &self.path;
        if let Some(ident) = field_ident {
            quote!(
                Ok(#self_ty{#ident: obj.extract()?})
            )
        } else {
            quote!(Ok(#self_ty(obj.extract()?)))
        }
    }

    fn build_tuple_struct(&self, len: usize) -> TokenStream {
        let self_ty = &self.path;
        let mut fields: Punctuated<TokenStream, syn::Token![,]> = Punctuated::new();
        for i in 0..len {
            fields.push(quote!(s.get_item(#i).extract()?));
        }
        let msg = if self.is_enum_variant {
            quote!(format!(
                "expected tuple of length {}, but got length {}",
                #len,
                s.len()
            ))
        } else {
            quote!("")
        };
        quote!(
            let s = <pyo3::types::PyTuple as pyo3::conversion::PyTryFrom>::try_from(obj)?;
            if s.len() != #len {
                return Err(pyo3::exceptions::PyValueError::new_err(#msg))
            }
            Ok(#self_ty(#fields))
        )
    }

    fn build_struct(&self, tups: &[(&Ident, FieldAttribute)]) -> TokenStream {
        let self_ty = &self.path;
        let mut fields: Punctuated<TokenStream, syn::Token![,]> = Punctuated::new();
        for (ident, attr) in tups {
            let ext_fn = match attr {
                FieldAttribute::GetAttr(Some(name)) => quote!(getattr(#name)),
                FieldAttribute::GetAttr(None) => quote!(getattr(stringify!(#ident))),
                FieldAttribute::GetItem(Some(key)) => quote!(get_item(#key)),
                FieldAttribute::GetItem(None) => quote!(get_item(stringify!(#ident))),
            };
            fields.push(quote!(#ident: obj.#ext_fn?.extract()?));
        }
        quote!(Ok(#self_ty{#fields}))
    }

    fn check_transparent_len(fields: &Fields) -> Result<()> {
        ensure_spanned!(
            fields.len() == 1,
            fields.span() => "transparent structs and variants can only have 1 field"
        );
        Ok(())
    }
}

/// Attributes for deriving FromPyObject scoped on containers.
#[derive(Clone, Debug, PartialEq)]
enum ContainerAttribute {
    /// Treat the Container as a Wrapper, directly extract its fields from the input object.
    Transparent,
    /// Change the name of an enum variant in the generated error message.
    ErrorAnnotation(syn::LitStr),
}

impl ContainerAttribute {
    /// Convenience method to access `ErrorAnnotation`.
    fn annotation(&self) -> Option<&syn::LitStr> {
        match self {
            ContainerAttribute::ErrorAnnotation(s) => Some(s),
            _ => None,
        }
    }

    /// Parse valid container arguments
    ///
    /// Fails if any are invalid.
    fn parse_attrs(value: &[Attribute]) -> Result<Vec<Self>> {
        get_pyo3_meta_list(value)?
            .nested
            .into_iter()
            .map(|meta| {
                if let syn::NestedMeta::Meta(metaitem) = &meta {
                    match metaitem {
                        Meta::Path(p) if p.is_ident("transparent") => {
                            return Ok(ContainerAttribute::Transparent);
                        }
                        Meta::NameValue(nv) if nv.path.is_ident("annotation") => {
                            if let syn::Lit::Str(s) = &nv.lit {
                                return Ok(ContainerAttribute::ErrorAnnotation(s.clone()));
                            } else {
                                bail_spanned!(nv.lit.span() => "expected string literal for annotation");
                            }
                        }
                        _ => {} // return Err below
                    }
                }

                bail_spanned!(meta.span() => "unknown `pyo3` container attribute");
            })
            .collect()
    }
}

/// Attributes for deriving FromPyObject scoped on fields.
#[derive(Clone, Debug)]
enum FieldAttribute {
    GetItem(Option<syn::Lit>),
    GetAttr(Option<syn::LitStr>),
}

impl FieldAttribute {
    /// Extract the field attribute.
    ///
    /// Currently fails if more than 1 attribute is passed in `pyo3`
    fn parse_attrs(attrs: &[Attribute]) -> Result<Option<Self>> {
        let list = get_pyo3_meta_list(attrs)?;
        let metaitem = match list.nested.len() {
            0 => return Ok(None),
            1 => list.nested.into_iter().next().unwrap(),
            _ => bail_spanned!(
                list.nested.span() =>
                "only one of `attribute` or `item` can be provided"
            ),
        };
        let meta = match metaitem {
            syn::NestedMeta::Meta(meta) => meta,
            syn::NestedMeta::Lit(lit) => bail_spanned!(
                lit.span() =>
                "expected `attribute` or `item`, got a literal"
            ),
        };
        let path = meta.path();
        if path.is_ident("attribute") {
            Ok(Some(FieldAttribute::GetAttr(Self::attribute_arg(meta)?)))
        } else if path.is_ident("item") {
            Ok(Some(FieldAttribute::GetItem(Self::item_arg(meta)?)))
        } else {
            bail_spanned!(meta.span() => "expected `attribute` or `item`");
        }
    }

    fn attribute_arg(meta: Meta) -> syn::Result<Option<syn::LitStr>> {
        let mut arg_list = match meta {
            Meta::List(list) => list,
            Meta::Path(_) => return Ok(None),
            Meta::NameValue(nv) => bail_spanned!(
                nv.span() =>
                "expected a string literal or no argument: `pyo3(attribute(\"name\")` or \
                `pyo3(attribute)`"
            ),
        };

        if arg_list.nested.len() == 1 {
            let arg = arg_list.nested.pop().unwrap().into_value();

            if let syn::NestedMeta::Lit(syn::Lit::Str(litstr)) = arg {
                ensure_spanned!(
                    !litstr.value().is_empty(),
                    litstr.span() => "attribute name cannot be empty"
                );
                return Ok(Some(litstr));
            }
        }

        bail_spanned!(arg_list.span() => "expected a single string literal argument");
    }

    fn item_arg(meta: Meta) -> syn::Result<Option<syn::Lit>> {
        let mut arg_list = match meta {
            Meta::List(list) => list,
            Meta::Path(_) => return Ok(None),
            Meta::NameValue(nv) => bail_spanned!(
                nv.span() => "expected a literal or no argument: `pyo3(item(key)` or `pyo3(item)`"
            ),
        };

        if arg_list.nested.len() == 1 {
            let arg = arg_list.nested.pop().unwrap().into_value();
            if let syn::NestedMeta::Lit(lit) = arg {
                return Ok(Some(lit));
            }
        }

        bail_spanned!(arg_list.span() => "expected a single literal argument");
    }
}

/// Extract pyo3 metalist, flattens multiple lists into a single one.
fn get_pyo3_meta_list(attrs: &[Attribute]) -> Result<MetaList> {
    let mut list: Punctuated<syn::NestedMeta, syn::Token![,]> = Punctuated::new();
    for value in attrs {
        match value.parse_meta()? {
            Meta::List(ml) if value.path.is_ident("pyo3") => {
                for meta in ml.nested {
                    list.push(meta);
                }
            }
            _ => continue,
        }
    }
    Ok(MetaList {
        path: parse_quote!(pyo3),
        paren_token: syn::token::Paren::default(),
        nested: list,
    })
}

fn verify_and_get_lifetime(generics: &syn::Generics) -> Result<Option<&syn::LifetimeDef>> {
    let mut lifetimes = generics.lifetimes();
    let lifetime = lifetimes.next();
    ensure_spanned!(
        lifetimes.next().is_none(),
        generics.span() => "FromPyObject can be derived with at most one lifetime parameter"
    );
    Ok(lifetime)
}

/// Derive FromPyObject for enums and structs.
///
///   * Max 1 lifetime specifier, will be tied to `FromPyObject`'s specifier
///   * At least one field, in case of `#[transparent]`, exactly one field
///   * At least one variant for enums.
///   * Fields of input structs and enums must implement `FromPyObject`
///   * Derivation for structs with generic fields like `struct<T> Foo(T)`
///     adds `T: FromPyObject` on the derived implementation.
pub fn build_derive_from_pyobject(tokens: &DeriveInput) -> Result<TokenStream> {
    let mut trait_generics = tokens.generics.clone();
    let generics = &tokens.generics;
    let lt_param = if let Some(lt) = verify_and_get_lifetime(generics)? {
        lt.clone()
    } else {
        trait_generics.params.push(parse_quote!('source));
        parse_quote!('source)
    };
    let mut where_clause: syn::WhereClause = parse_quote!(where);
    for param in generics.type_params() {
        let gen_ident = &param.ident;
        where_clause
            .predicates
            .push(parse_quote!(#gen_ident: FromPyObject<#lt_param>))
    }
    let derives = match &tokens.data {
        syn::Data::Enum(en) => {
            let en = Enum::new(en, &tokens.ident)?;
            en.build()
        }
        syn::Data::Struct(st) => {
            let attrs = ContainerAttribute::parse_attrs(&tokens.attrs)?;
            Container::verify_struct_container_attrs(&attrs)?;
            let ident = &tokens.ident;
            let st = Container::new(&st.fields, parse_quote!(#ident), attrs, false)?;
            st.build()
        }
        syn::Data::Union(_) => bail_spanned!(
            tokens.span() => "#[derive(FromPyObject)] is not supported for unions"
        ),
    };

    let ident = &tokens.ident;
    Ok(quote!(
        #[automatically_derived]
        impl#trait_generics pyo3::FromPyObject<#lt_param> for #ident#generics #where_clause {
            fn extract(obj: &#lt_param pyo3::PyAny) -> pyo3::PyResult<Self>  {
                #derives
            }
        }
    ))
}
