use crate::{
    attributes::{self, get_pyo3_options, CrateAttribute, FromPyWithAttribute},
    utils::get_pyo3_crate,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, DataEnum, DeriveInput, Fields, Ident, LitStr, Result, Token,
};

/// Describes derivation input of an enum.
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
                let attrs = ContainerOptions::from_attrs(&variant.attrs)?;
                let var_ident = &variant.ident;
                Container::new(&variant.fields, parse_quote!(#ident::#var_ident), attrs)
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
        let mut variant_names = Vec::new();
        let mut error_names = Vec::new();
        for var in &self.variants {
            let struct_derive = var.build();
            let ext = quote!({
                let maybe_ret = || -> _pyo3::PyResult<Self> {
                    #struct_derive
                }();

                match maybe_ret {
                    ok @ ::std::result::Result::Ok(_) => return ok,
                    ::std::result::Result::Err(err) => err
                }
            });

            var_extracts.push(ext);
            variant_names.push(var.path.segments.last().unwrap().ident.to_string());
            error_names.push(&var.err_name);
        }
        let ty_name = self.enum_ident.to_string();
        quote!(
            let errors = [
                #(#var_extracts),*
            ];
            ::std::result::Result::Err(
                _pyo3::impl_::frompyobject::failed_to_extract_enum(
                    obj.py(),
                    #ty_name,
                    &[#(#variant_names),*],
                    &[#(#error_names),*],
                    &errors
                )
            )
        )
    }
}

struct NamedStructField<'a> {
    ident: &'a syn::Ident,
    getter: Option<FieldGetter>,
    from_py_with: Option<FromPyWithAttribute>,
}

struct TupleStructField {
    from_py_with: Option<FromPyWithAttribute>,
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
    StructNewtype(&'a syn::Ident, Option<FromPyWithAttribute>),
    /// Tuple struct, e.g. `struct Foo(String)`.
    ///
    /// Variant contains a list of conversion methods for each of the fields that are directly
    ///  extracted from the tuple.
    Tuple(Vec<TupleStructField>),
    /// Tuple newtype, e.g. `#[transparent] struct Foo(String)`
    ///
    /// The wrapped field is directly extracted from the object.
    TupleNewtype(Option<FromPyWithAttribute>),
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
                let mut tuple_fields = unnamed
                    .unnamed
                    .iter()
                    .map(|field| {
                        let attrs = FieldPyO3Attributes::from_attrs(&field.attrs)?;
                        ensure_spanned!(
                            attrs.getter.is_none(),
                            field.span() => "`getter` is not permitted on tuple struct elements."
                        );
                        Ok(TupleStructField {
                            from_py_with: attrs.from_py_with,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                if tuple_fields.len() == 1 {
                    // Always treat a 1-length tuple struct as "transparent", even without the
                    // explicit annotation.
                    let field = tuple_fields.pop().unwrap();
                    ContainerType::TupleNewtype(field.from_py_with)
                } else if options.transparent {
                    bail_spanned!(
                        fields.span() => "transparent structs and variants can only have 1 field"
                    );
                } else {
                    ContainerType::Tuple(tuple_fields)
                }
            }
            Fields::Named(named) if !named.named.is_empty() => {
                let mut struct_fields = named
                    .named
                    .iter()
                    .map(|field| {
                        let ident = field
                            .ident
                            .as_ref()
                            .expect("Named fields should have identifiers");
                        let attrs = FieldPyO3Attributes::from_attrs(&field.attrs)?;

                        Ok(NamedStructField {
                            ident,
                            getter: attrs.getter,
                            from_py_with: attrs.from_py_with,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                if options.transparent {
                    ensure_spanned!(
                        struct_fields.len() == 1,
                        fields.span() => "transparent structs and variants can only have 1 field"
                    );
                    let field = struct_fields.pop().unwrap();
                    ensure_spanned!(
                        field.getter.is_none(),
                        field.ident.span() => "`transparent` structs may not have a `getter` for the inner field"
                    );
                    ContainerType::StructNewtype(field.ident, field.from_py_with)
                } else {
                    ContainerType::Struct(struct_fields)
                }
            }
            _ => bail_spanned!(
                fields.span() => "cannot derive FromPyObject for empty structs and variants"
            ),
        };
        let err_name = options.annotation.map_or_else(
            || path.segments.last().unwrap().ident.to_string(),
            |lit_str| lit_str.value(),
        );

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
    fn build(&self) -> TokenStream {
        match &self.ty {
            ContainerType::StructNewtype(ident, from_py_with) => {
                self.build_newtype_struct(Some(ident), from_py_with)
            }
            ContainerType::TupleNewtype(from_py_with) => {
                self.build_newtype_struct(None, from_py_with)
            }
            ContainerType::Tuple(tups) => self.build_tuple_struct(tups),
            ContainerType::Struct(tups) => self.build_struct(tups),
        }
    }

    fn build_newtype_struct(
        &self,
        field_ident: Option<&Ident>,
        from_py_with: &Option<FromPyWithAttribute>,
    ) -> TokenStream {
        let self_ty = &self.path;
        let struct_name = self.name();
        if let Some(ident) = field_ident {
            let field_name = ident.to_string();
            match from_py_with {
                None => quote! {
                    Ok(#self_ty {
                        #ident: _pyo3::impl_::frompyobject::extract_struct_field(obj, #struct_name, #field_name)?
                    })
                },
                Some(FromPyWithAttribute {
                    value: expr_path, ..
                }) => quote! {
                    Ok(#self_ty {
                        #ident: _pyo3::impl_::frompyobject::extract_struct_field_with(#expr_path, obj, #struct_name, #field_name)?
                    })
                },
            }
        } else {
            match from_py_with {
                None => quote!(
                    _pyo3::impl_::frompyobject::extract_tuple_struct_field(obj, #struct_name, 0).map(#self_ty)
                ),
                Some(FromPyWithAttribute {
                    value: expr_path, ..
                }) => quote! (
                    _pyo3::impl_::frompyobject::extract_tuple_struct_field_with(#expr_path, obj, #struct_name, 0).map(#self_ty)
                ),
            }
        }
    }

    fn build_tuple_struct(&self, struct_fields: &[TupleStructField]) -> TokenStream {
        let self_ty = &self.path;
        let struct_name = &self.name();
        let field_idents: Vec<_> = (0..struct_fields.len())
            .into_iter()
            .map(|i| format_ident!("arg{}", i))
            .collect();
        let fields = struct_fields.iter().zip(&field_idents).enumerate().map(|(index, (field, ident))| {
            match &field.from_py_with {
                None => quote!(
                    _pyo3::impl_::frompyobject::extract_tuple_struct_field(#ident, #struct_name, #index)?
                ),
                Some(FromPyWithAttribute {
                    value: expr_path, ..
                }) => quote! (
                    _pyo3::impl_::frompyobject::extract_tuple_struct_field_with(#expr_path, #ident, #struct_name, #index)?
                ),
            }
        });
        quote!(
            match obj.extract() {
                ::std::result::Result::Ok((#(#field_idents),*)) => ::std::result::Result::Ok(#self_ty(#(#fields),*)),
                ::std::result::Result::Err(err) => ::std::result::Result::Err(err),
            }
        )
    }

    fn build_struct(&self, struct_fields: &[NamedStructField<'_>]) -> TokenStream {
        let self_ty = &self.path;
        let struct_name = &self.name();
        let mut fields: Punctuated<TokenStream, syn::Token![,]> = Punctuated::new();
        for field in struct_fields {
            let ident = &field.ident;
            let field_name = ident.to_string();
            let getter = match field.getter.as_ref().unwrap_or(&FieldGetter::GetAttr(None)) {
                FieldGetter::GetAttr(Some(name)) => {
                    quote!(getattr(_pyo3::intern!(obj.py(), #name)))
                }
                FieldGetter::GetAttr(None) => {
                    quote!(getattr(_pyo3::intern!(obj.py(), #field_name)))
                }
                FieldGetter::GetItem(Some(syn::Lit::Str(key))) => {
                    quote!(get_item(_pyo3::intern!(obj.py(), #key)))
                }
                FieldGetter::GetItem(Some(key)) => quote!(get_item(#key)),
                FieldGetter::GetItem(None) => {
                    quote!(get_item(_pyo3::intern!(obj.py(), #field_name)))
                }
            };
            let extractor = match &field.from_py_with {
                None => {
                    quote!(_pyo3::impl_::frompyobject::extract_struct_field(obj.#getter?, #struct_name, #field_name)?)
                }
                Some(FromPyWithAttribute {
                    value: expr_path, ..
                }) => {
                    quote! (_pyo3::impl_::frompyobject::extract_struct_field_with(#expr_path, obj.#getter?, #struct_name, #field_name)?)
                }
            };

            fields.push(quote!(#ident: #extractor));
        }
        quote!(::std::result::Result::Ok(#self_ty{#fields}))
    }
}

#[derive(Default)]
struct ContainerOptions {
    /// Treat the Container as a Wrapper, directly extract its fields from the input object.
    transparent: bool,
    /// Change the name of an enum variant in the generated error message.
    annotation: Option<syn::LitStr>,
    /// Change the path for the pyo3 crate
    krate: Option<CrateAttribute>,
}

/// Attributes for deriving FromPyObject scoped on containers.
enum ContainerPyO3Attribute {
    /// Treat the Container as a Wrapper, directly extract its fields from the input object.
    Transparent(attributes::kw::transparent),
    /// Change the name of an enum variant in the generated error message.
    ErrorAnnotation(LitStr),
    /// Change the path for the pyo3 crate
    Crate(CrateAttribute),
}

impl Parse for ContainerPyO3Attribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::transparent) {
            let kw: attributes::kw::transparent = input.parse()?;
            Ok(ContainerPyO3Attribute::Transparent(kw))
        } else if lookahead.peek(attributes::kw::annotation) {
            let _: attributes::kw::annotation = input.parse()?;
            let _: Token![=] = input.parse()?;
            input.parse().map(ContainerPyO3Attribute::ErrorAnnotation)
        } else if lookahead.peek(Token![crate]) {
            input.parse().map(ContainerPyO3Attribute::Crate)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ContainerOptions {
    fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut options = ContainerOptions::default();

        for attr in attrs {
            if let Some(pyo3_attrs) = get_pyo3_options(attr)? {
                for pyo3_attr in pyo3_attrs {
                    match pyo3_attr {
                        ContainerPyO3Attribute::Transparent(kw) => {
                            ensure_spanned!(
                                !options.transparent,
                                kw.span() => "`transparent` may only be provided once"
                            );
                            options.transparent = true;
                        }
                        ContainerPyO3Attribute::ErrorAnnotation(lit_str) => {
                            ensure_spanned!(
                                options.annotation.is_none(),
                                lit_str.span() => "`annotation` may only be provided once"
                            );
                            options.annotation = Some(lit_str);
                        }
                        ContainerPyO3Attribute::Crate(path) => {
                            ensure_spanned!(
                                options.krate.is_none(),
                                path.span() => "`crate` may only be provided once"
                            );
                            options.krate = Some(path);
                        }
                    }
                }
            }
        }
        Ok(options)
    }
}

/// Attributes for deriving FromPyObject scoped on fields.
#[derive(Clone, Debug)]
struct FieldPyO3Attributes {
    getter: Option<FieldGetter>,
    from_py_with: Option<FromPyWithAttribute>,
}

#[derive(Clone, Debug)]
enum FieldGetter {
    GetItem(Option<syn::Lit>),
    GetAttr(Option<LitStr>),
}

enum FieldPyO3Attribute {
    Getter(FieldGetter),
    FromPyWith(FromPyWithAttribute),
}

impl Parse for FieldPyO3Attribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::attribute) {
            let _: attributes::kw::attribute = input.parse()?;
            if input.peek(syn::token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                let attr_name: LitStr = content.parse()?;
                if !content.is_empty() {
                    return Err(content.error(
                        "expected at most one argument: `attribute` or `attribute(\"name\")`",
                    ));
                }
                ensure_spanned!(
                    !attr_name.value().is_empty(),
                    attr_name.span() => "attribute name cannot be empty"
                );
                Ok(FieldPyO3Attribute::Getter(FieldGetter::GetAttr(Some(
                    attr_name,
                ))))
            } else {
                Ok(FieldPyO3Attribute::Getter(FieldGetter::GetAttr(None)))
            }
        } else if lookahead.peek(attributes::kw::item) {
            let _: attributes::kw::item = input.parse()?;
            if input.peek(syn::token::Paren) {
                let content;
                let _ = parenthesized!(content in input);
                let key = content.parse()?;
                if !content.is_empty() {
                    return Err(
                        content.error("expected at most one argument: `item` or `item(key)`")
                    );
                }
                Ok(FieldPyO3Attribute::Getter(FieldGetter::GetItem(Some(key))))
            } else {
                Ok(FieldPyO3Attribute::Getter(FieldGetter::GetItem(None)))
            }
        } else if lookahead.peek(attributes::kw::from_py_with) {
            input.parse().map(FieldPyO3Attribute::FromPyWith)
        } else {
            Err(lookahead.error())
        }
    }
}

impl FieldPyO3Attributes {
    /// Extract the field attributes.
    fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut getter = None;
        let mut from_py_with = None;

        for attr in attrs {
            if let Some(pyo3_attrs) = get_pyo3_options(attr)? {
                for pyo3_attr in pyo3_attrs {
                    match pyo3_attr {
                        FieldPyO3Attribute::Getter(field_getter) => {
                            ensure_spanned!(
                                getter.is_none(),
                                attr.span() => "only one of `attribute` or `item` can be provided"
                            );
                            getter = Some(field_getter)
                        }
                        FieldPyO3Attribute::FromPyWith(from_py_with_attr) => {
                            ensure_spanned!(
                                from_py_with.is_none(),
                                attr.span() => "`from_py_with` may only be provided once"
                            );
                            from_py_with = Some(from_py_with_attr);
                        }
                    }
                }
            }
        }

        Ok(FieldPyO3Attributes {
            getter,
            from_py_with,
        })
    }
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
///   * Fields of input structs and enums must implement `FromPyObject` or be annotated with `from_py_with`
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
    let options = ContainerOptions::from_attrs(&tokens.attrs)?;
    let krate = get_pyo3_crate(&options.krate);
    let derives = match &tokens.data {
        syn::Data::Enum(en) => {
            if options.transparent || options.annotation.is_some() {
                bail_spanned!(tokens.span() => "`transparent` or `annotation` is not supported \
                                                at top level for enums");
            }
            let en = Enum::new(en, &tokens.ident)?;
            en.build()
        }
        syn::Data::Struct(st) => {
            if let Some(lit_str) = &options.annotation {
                bail_spanned!(lit_str.span() => "`annotation` is unsupported for structs");
            }
            let ident = &tokens.ident;
            let st = Container::new(&st.fields, parse_quote!(#ident), options)?;
            st.build()
        }
        syn::Data::Union(_) => bail_spanned!(
            tokens.span() => "#[derive(FromPyObject)] is not supported for unions"
        ),
    };

    let ident = &tokens.ident;
    Ok(quote!(
        const _: () = {
            use #krate as _pyo3;

            #[automatically_derived]
            impl#trait_generics _pyo3::FromPyObject<#lt_param> for #ident#generics #where_clause {
                fn extract(obj: &#lt_param _pyo3::PyAny) -> _pyo3::PyResult<Self>  {
                    #derives
                }
            }
        };
    ))
}
