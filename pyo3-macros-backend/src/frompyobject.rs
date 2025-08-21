use crate::attributes::{DefaultAttribute, FromPyWithAttribute, RenamingRule};
use crate::derive_attributes::{ContainerAttributes, FieldAttributes, FieldGetter};
#[cfg(feature = "experimental-inspect")]
use crate::introspection::ConcatenationBuilder;
#[cfg(feature = "experimental-inspect")]
use crate::utils::TypeExt;
use crate::utils::{self, Ctx};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    ext::IdentExt, parse_quote, punctuated::Punctuated, spanned::Spanned, DataEnum, DeriveInput,
    Fields, Ident, Result, Token,
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
    fn new(
        data_enum: &'a DataEnum,
        ident: &'a Ident,
        options: ContainerAttributes,
    ) -> Result<Self> {
        ensure_spanned!(
            !data_enum.variants.is_empty(),
            ident.span() => "cannot derive FromPyObject for empty enum"
        );
        let variants = data_enum
            .variants
            .iter()
            .map(|variant| {
                let mut variant_options = ContainerAttributes::from_attrs(&variant.attrs)?;
                if let Some(rename_all) = &options.rename_all {
                    ensure_spanned!(
                        variant_options.rename_all.is_none(),
                        variant_options.rename_all.span() => "Useless variant `rename_all` - enum is already annotated with `rename_all"
                    );
                    variant_options.rename_all = Some(rename_all.clone());

                }
                let var_ident = &variant.ident;
                Container::new(
                    &variant.fields,
                    parse_quote!(#ident::#var_ident),
                    variant_options,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Enum {
            enum_ident: ident,
            variants,
        })
    }

    /// Build derivation body for enums.
    fn build(&self, ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        let mut var_extracts = Vec::new();
        let mut variant_names = Vec::new();
        let mut error_names = Vec::new();

        for var in &self.variants {
            let struct_derive = var.build(ctx);
            let ext = quote!({
                let maybe_ret = || -> #pyo3_path::PyResult<Self> {
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
                #pyo3_path::impl_::frompyobject::failed_to_extract_enum(
                    obj.py(),
                    #ty_name,
                    &[#(#variant_names),*],
                    &[#(#error_names),*],
                    &errors
                )
            )
        )
    }

    #[cfg(feature = "experimental-inspect")]
    fn write_input_type(&self, builder: &mut ConcatenationBuilder, ctx: &Ctx) {
        for (i, var) in self.variants.iter().enumerate() {
            if i > 0 {
                builder.push_str(" | ");
            }
            var.write_input_type(builder, ctx);
        }
    }
}

struct NamedStructField<'a> {
    ident: &'a syn::Ident,
    getter: Option<FieldGetter>,
    from_py_with: Option<FromPyWithAttribute>,
    default: Option<DefaultAttribute>,
    ty: &'a syn::Type,
}

struct TupleStructField {
    from_py_with: Option<FromPyWithAttribute>,
    ty: syn::Type,
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
    #[cfg_attr(not(feature = "experimental-inspect"), allow(unused))]
    StructNewtype(&'a syn::Ident, Option<FromPyWithAttribute>, &'a syn::Type),
    /// Tuple struct, e.g. `struct Foo(String)`.
    ///
    /// Variant contains a list of conversion methods for each of the fields that are directly
    ///  extracted from the tuple.
    Tuple(Vec<TupleStructField>),
    /// Tuple newtype, e.g. `#[transparent] struct Foo(String)`
    ///
    /// The wrapped field is directly extracted from the object.
    #[cfg_attr(not(feature = "experimental-inspect"), allow(unused))]
    TupleNewtype(Option<FromPyWithAttribute>, Box<syn::Type>),
}

/// Data container
///
/// Either describes a struct or an enum variant.
struct Container<'a> {
    path: syn::Path,
    ty: ContainerType<'a>,
    err_name: String,
    rename_rule: Option<RenamingRule>,
}

impl<'a> Container<'a> {
    /// Construct a container based on fields, identifier and attributes.
    ///
    /// Fails if the variant has no fields or incompatible attributes.
    fn new(fields: &'a Fields, path: syn::Path, options: ContainerAttributes) -> Result<Self> {
        let style = match fields {
            Fields::Unnamed(unnamed) if !unnamed.unnamed.is_empty() => {
                ensure_spanned!(
                    options.rename_all.is_none(),
                    options.rename_all.span() => "`rename_all` is useless on tuple structs and variants."
                );
                let mut tuple_fields = unnamed
                    .unnamed
                    .iter()
                    .map(|field| {
                        let attrs = FieldAttributes::from_attrs(&field.attrs)?;
                        ensure_spanned!(
                            attrs.getter.is_none(),
                            field.span() => "`getter` is not permitted on tuple struct elements."
                        );
                        ensure_spanned!(
                            attrs.default.is_none(),
                            field.span() => "`default` is not permitted on tuple struct elements."
                        );
                        Ok(TupleStructField {
                            from_py_with: attrs.from_py_with,
                            ty: field.ty.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                if tuple_fields.len() == 1 {
                    // Always treat a 1-length tuple struct as "transparent", even without the
                    // explicit annotation.
                    let field = tuple_fields.pop().unwrap();
                    ContainerType::TupleNewtype(field.from_py_with, Box::new(field.ty))
                } else if options.transparent.is_some() {
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
                        let mut attrs = FieldAttributes::from_attrs(&field.attrs)?;

                        if let Some(ref from_item_all) = options.from_item_all {
                            if let Some(replaced) = attrs.getter.replace(FieldGetter::GetItem(parse_quote!(item), None))
                            {
                                match replaced {
                                    FieldGetter::GetItem(item, Some(item_name)) => {
                                        attrs.getter = Some(FieldGetter::GetItem(item, Some(item_name)));
                                    }
                                    FieldGetter::GetItem(_, None) => bail_spanned!(from_item_all.span() => "Useless `item` - the struct is already annotated with `from_item_all`"),
                                    FieldGetter::GetAttr(_, _) => bail_spanned!(
                                        from_item_all.span() => "The struct is already annotated with `from_item_all`, `attribute` is not allowed"
                                    ),
                                }
                            }
                        }

                        Ok(NamedStructField {
                            ident,
                            getter: attrs.getter,
                            from_py_with: attrs.from_py_with,
                            default: attrs.default,
                            ty: &field.ty,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                if struct_fields.iter().all(|field| field.default.is_some()) {
                    bail_spanned!(
                        fields.span() => "cannot derive FromPyObject for structs and variants with only default values"
                    )
                } else if options.transparent.is_some() {
                    ensure_spanned!(
                        struct_fields.len() == 1,
                        fields.span() => "transparent structs and variants can only have 1 field"
                    );
                    ensure_spanned!(
                        options.rename_all.is_none(),
                        options.rename_all.span() => "`rename_all` is not permitted on `transparent` structs and variants"
                    );
                    let field = struct_fields.pop().unwrap();
                    ensure_spanned!(
                        field.getter.is_none(),
                        field.ident.span() => "`transparent` structs may not have a `getter` for the inner field"
                    );
                    ContainerType::StructNewtype(field.ident, field.from_py_with, field.ty)
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
            rename_rule: options.rename_all.map(|v| v.value.rule),
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
    fn build(&self, ctx: &Ctx) -> TokenStream {
        match &self.ty {
            ContainerType::StructNewtype(ident, from_py_with, _) => {
                self.build_newtype_struct(Some(ident), from_py_with, ctx)
            }
            ContainerType::TupleNewtype(from_py_with, _) => {
                self.build_newtype_struct(None, from_py_with, ctx)
            }
            ContainerType::Tuple(tups) => self.build_tuple_struct(tups, ctx),
            ContainerType::Struct(tups) => self.build_struct(tups, ctx),
        }
    }

    fn build_newtype_struct(
        &self,
        field_ident: Option<&Ident>,
        from_py_with: &Option<FromPyWithAttribute>,
        ctx: &Ctx,
    ) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        let self_ty = &self.path;
        let struct_name = self.name();
        if let Some(ident) = field_ident {
            let field_name = ident.to_string();
            if let Some(FromPyWithAttribute {
                kw,
                value: expr_path,
            }) = from_py_with
            {
                let extractor = quote_spanned! { kw.span =>
                    { let from_py_with: fn(_) -> _ = #expr_path; from_py_with }
                };
                quote! {
                    Ok(#self_ty {
                        #ident: #pyo3_path::impl_::frompyobject::extract_struct_field_with(#extractor, obj, #struct_name, #field_name)?
                    })
                }
            } else {
                quote! {
                    Ok(#self_ty {
                        #ident: #pyo3_path::impl_::frompyobject::extract_struct_field(obj, #struct_name, #field_name)?
                    })
                }
            }
        } else if let Some(FromPyWithAttribute {
            kw,
            value: expr_path,
        }) = from_py_with
        {
            let extractor = quote_spanned! { kw.span =>
                { let from_py_with: fn(_) -> _ = #expr_path; from_py_with }
            };
            quote! {
                #pyo3_path::impl_::frompyobject::extract_tuple_struct_field_with(#extractor, obj, #struct_name, 0).map(#self_ty)
            }
        } else {
            quote! {
                #pyo3_path::impl_::frompyobject::extract_tuple_struct_field(obj, #struct_name, 0).map(#self_ty)
            }
        }
    }

    fn build_tuple_struct(&self, struct_fields: &[TupleStructField], ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        let self_ty = &self.path;
        let struct_name = &self.name();
        let field_idents: Vec<_> = (0..struct_fields.len())
            .map(|i| format_ident!("arg{}", i))
            .collect();
        let fields = struct_fields.iter().zip(&field_idents).enumerate().map(|(index, (field, ident))| {
            if let Some(FromPyWithAttribute {
                kw,
                value: expr_path, ..
            }) = &field.from_py_with {
                let extractor = quote_spanned! { kw.span =>
                    { let from_py_with: fn(_) -> _ = #expr_path; from_py_with }
                };
               quote! {
                    #pyo3_path::impl_::frompyobject::extract_tuple_struct_field_with(#extractor, &#ident, #struct_name, #index)?
               }
            } else {
                quote!{
                    #pyo3_path::impl_::frompyobject::extract_tuple_struct_field(&#ident, #struct_name, #index)?
            }}
        });

        quote!(
            match #pyo3_path::types::PyAnyMethods::extract(obj) {
                ::std::result::Result::Ok((#(#field_idents),*)) => ::std::result::Result::Ok(#self_ty(#(#fields),*)),
                ::std::result::Result::Err(err) => ::std::result::Result::Err(err),
            }
        )
    }

    fn build_struct(&self, struct_fields: &[NamedStructField<'_>], ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        let self_ty = &self.path;
        let struct_name = self.name();
        let mut fields: Punctuated<TokenStream, Token![,]> = Punctuated::new();
        for field in struct_fields {
            let ident = field.ident;
            let field_name = ident.unraw().to_string();
            let getter = match field
                .getter
                .as_ref()
                .unwrap_or(&FieldGetter::GetAttr(parse_quote!(attribute), None))
            {
                FieldGetter::GetAttr(_, Some(name)) => {
                    quote!(#pyo3_path::types::PyAnyMethods::getattr(obj, #pyo3_path::intern!(obj.py(), #name)))
                }
                FieldGetter::GetAttr(_, None) => {
                    let name = self
                        .rename_rule
                        .map(|rule| utils::apply_renaming_rule(rule, &field_name));
                    let name = name.as_deref().unwrap_or(&field_name);
                    quote!(#pyo3_path::types::PyAnyMethods::getattr(obj, #pyo3_path::intern!(obj.py(), #name)))
                }
                FieldGetter::GetItem(_, Some(syn::Lit::Str(key))) => {
                    quote!(#pyo3_path::types::PyAnyMethods::get_item(obj, #pyo3_path::intern!(obj.py(), #key)))
                }
                FieldGetter::GetItem(_, Some(key)) => {
                    quote!(#pyo3_path::types::PyAnyMethods::get_item(obj, #key))
                }
                FieldGetter::GetItem(_, None) => {
                    let name = self
                        .rename_rule
                        .map(|rule| utils::apply_renaming_rule(rule, &field_name));
                    let name = name.as_deref().unwrap_or(&field_name);
                    quote!(#pyo3_path::types::PyAnyMethods::get_item(obj, #pyo3_path::intern!(obj.py(), #name)))
                }
            };
            let extractor = if let Some(FromPyWithAttribute {
                kw,
                value: expr_path,
            }) = &field.from_py_with
            {
                let extractor = quote_spanned! { kw.span =>
                    { let from_py_with: fn(_) -> _ = #expr_path; from_py_with }
                };
                quote! (#pyo3_path::impl_::frompyobject::extract_struct_field_with(#extractor, &#getter?, #struct_name, #field_name)?)
            } else {
                quote!(#pyo3_path::impl_::frompyobject::extract_struct_field(&value, #struct_name, #field_name)?)
            };
            let extracted = if let Some(default) = &field.default {
                let default_expr = if let Some(default_expr) = &default.value {
                    default_expr.to_token_stream()
                } else {
                    quote!(::std::default::Default::default())
                };
                quote!(if let ::std::result::Result::Ok(value) = #getter {
                    #extractor
                } else {
                    #default_expr
                })
            } else {
                quote!({
                    let value = #getter?;
                    #extractor
                })
            };

            fields.push(quote!(#ident: #extracted));
        }

        quote!(::std::result::Result::Ok(#self_ty{#fields}))
    }

    #[cfg(feature = "experimental-inspect")]
    fn write_input_type(&self, builder: &mut ConcatenationBuilder, ctx: &Ctx) {
        match &self.ty {
            ContainerType::StructNewtype(_, from_py_with, ty) => {
                Self::write_field_input_type(from_py_with, ty, builder, ctx);
            }
            ContainerType::TupleNewtype(from_py_with, ty) => {
                Self::write_field_input_type(from_py_with, ty, builder, ctx);
            }
            ContainerType::Tuple(tups) => {
                builder.push_str("tuple[");
                for (i, TupleStructField { from_py_with, ty }) in tups.iter().enumerate() {
                    if i > 0 {
                        builder.push_str(", ");
                    }
                    Self::write_field_input_type(from_py_with, ty, builder, ctx);
                }
                builder.push_str("]");
            }
            ContainerType::Struct(_) => {
                // TODO: implement using a Protocol?
                builder.push_str("_typeshed.Incomplete")
            }
        }
    }

    #[cfg(feature = "experimental-inspect")]
    fn write_field_input_type(
        from_py_with: &Option<FromPyWithAttribute>,
        ty: &syn::Type,
        builder: &mut ConcatenationBuilder,
        ctx: &Ctx,
    ) {
        if from_py_with.is_some() {
            // We don't know what from_py_with is doing
            builder.push_str("_typeshed.Incomplete")
        } else {
            let ty = ty.clone().elide_lifetimes();
            let pyo3_crate_path = &ctx.pyo3_path;
            builder.push_tokens(
                quote! { <#ty as #pyo3_crate_path::FromPyObject<'_>>::INPUT_TYPE.as_bytes() },
            )
        }
    }
}

fn verify_and_get_lifetime(generics: &syn::Generics) -> Result<Option<&syn::LifetimeParam>> {
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
    let options = ContainerAttributes::from_attrs(&tokens.attrs)?;
    let ctx = &Ctx::new(&options.krate, None);
    let Ctx { pyo3_path, .. } = &ctx;

    let (_, ty_generics, _) = tokens.generics.split_for_impl();
    let mut trait_generics = tokens.generics.clone();
    let lt_param = if let Some(lt) = verify_and_get_lifetime(&trait_generics)? {
        lt.clone()
    } else {
        trait_generics.params.push(parse_quote!('py));
        parse_quote!('py)
    };
    let (impl_generics, _, where_clause) = trait_generics.split_for_impl();

    let mut where_clause = where_clause.cloned().unwrap_or_else(|| parse_quote!(where));
    for param in trait_generics.type_params() {
        let gen_ident = &param.ident;
        where_clause
            .predicates
            .push(parse_quote!(#gen_ident: #pyo3_path::FromPyObject<'py>))
    }

    let derives = match &tokens.data {
        syn::Data::Enum(en) => {
            if options.transparent.is_some() || options.annotation.is_some() {
                bail_spanned!(tokens.span() => "`transparent` or `annotation` is not supported \
                                                at top level for enums");
            }
            let en = Enum::new(en, &tokens.ident, options.clone())?;
            en.build(ctx)
        }
        syn::Data::Struct(st) => {
            if let Some(lit_str) = &options.annotation {
                bail_spanned!(lit_str.span() => "`annotation` is unsupported for structs");
            }
            let ident = &tokens.ident;
            let st = Container::new(&st.fields, parse_quote!(#ident), options.clone())?;
            st.build(ctx)
        }
        syn::Data::Union(_) => bail_spanned!(
            tokens.span() => "#[derive(FromPyObject)] is not supported for unions"
        ),
    };

    #[cfg(feature = "experimental-inspect")]
    let input_type = {
        let mut builder = ConcatenationBuilder::default();
        if tokens
            .generics
            .params
            .iter()
            .all(|p| matches!(p, syn::GenericParam::Lifetime(_)))
        {
            match &tokens.data {
                syn::Data::Enum(en) => {
                    Enum::new(en, &tokens.ident, options)?.write_input_type(&mut builder, ctx)
                }
                syn::Data::Struct(st) => {
                    let ident = &tokens.ident;
                    Container::new(&st.fields, parse_quote!(#ident), options.clone())?
                        .write_input_type(&mut builder, ctx)
                }
                syn::Data::Union(_) => {
                    // Not supported at this point
                    builder.push_str("_typeshed.Incomplete")
                }
            }
        } else {
            // We don't know how to deal with generic parameters
            // Blocked by https://github.com/rust-lang/rust/issues/76560
            builder.push_str("_typeshed.Incomplete")
        };
        let input_type = builder.into_token_stream(&ctx.pyo3_path);
        quote! { const INPUT_TYPE: &'static str = unsafe { ::std::str::from_utf8_unchecked(#input_type) }; }
    };
    #[cfg(not(feature = "experimental-inspect"))]
    let input_type = quote! {};

    let ident = &tokens.ident;
    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics #pyo3_path::FromPyObject<#lt_param> for #ident #ty_generics #where_clause {
            fn extract_bound(obj: &#pyo3_path::Bound<#lt_param, #pyo3_path::PyAny>) -> #pyo3_path::PyResult<Self>  {
                #derives
            }
            #input_type
        }
    ))
}
