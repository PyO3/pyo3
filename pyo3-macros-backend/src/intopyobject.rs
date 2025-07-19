use crate::attributes::{IntoPyWithAttribute, RenamingRule};
use crate::derive_attributes::{ContainerAttributes, FieldAttributes};
use crate::utils::{self, Ctx};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::ext::IdentExt;
use syn::spanned::Spanned as _;
use syn::{parse_quote, DataEnum, DeriveInput, Fields, Ident, Index, Result};

struct ItemOption(Option<syn::Lit>);

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
    into_py_with: Option<IntoPyWithAttribute>,
}

struct TupleStructField<'a> {
    field: &'a syn::Field,
    into_py_with: Option<IntoPyWithAttribute>,
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
struct Container<'a, const REF: bool> {
    path: syn::Path,
    receiver: Option<Ident>,
    ty: ContainerType<'a>,
    rename_rule: Option<RenamingRule>,
}

/// Construct a container based on fields, identifier and attributes.
impl<'a, const REF: bool> Container<'a, REF> {
    ///
    /// Fails if the variant has no fields or incompatible attributes.
    fn new(
        receiver: Option<Ident>,
        fields: &'a Fields,
        path: syn::Path,
        options: ContainerAttributes,
    ) -> Result<Self> {
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
                            attrs.getter.unwrap().span() => "`item` and `attribute` are not permitted on tuple struct elements."
                        );
                        Ok(TupleStructField {
                            field,
                            into_py_with: attrs.into_py_with,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                if tuple_fields.len() == 1 {
                    // Always treat a 1-length tuple struct as "transparent", even without the
                    // explicit annotation.
                    let TupleStructField {
                        field,
                        into_py_with,
                    } = tuple_fields.pop().unwrap();
                    ensure_spanned!(
                        into_py_with.is_none(),
                        into_py_with.span() => "`into_py_with` is not permitted on `transparent` structs"
                    );
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
                        attrs.getter.is_none(),
                        attrs.getter.unwrap().span() => "`transparent` structs may not have `item` nor `attribute` for the inner field"
                    );
                    ensure_spanned!(
                        options.rename_all.is_none(),
                        options.rename_all.span() => "`rename_all` is not permitted on `transparent` structs and variants"
                    );
                    ensure_spanned!(
                        attrs.into_py_with.is_none(),
                        attrs.into_py_with.span() => "`into_py_with` is not permitted on `transparent` structs or variants"
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
                                item: attrs.getter.and_then(|getter| match getter {
                                    crate::derive_attributes::FieldGetter::GetItem(_, lit) => {
                                        Some(ItemOption(lit))
                                    }
                                    crate::derive_attributes::FieldGetter::GetAttr(_, _) => None,
                                }),
                                into_py_with: attrs.into_py_with,
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
            rename_rule: options.rename_all.map(|v| v.value.rule),
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
                    .and_then(|item| item.0.as_ref())
                    .map(|item| item.into_token_stream())
                    .unwrap_or_else(|| {
                        let name = f.ident.unraw().to_string();
                        self.rename_rule.map(|rule| utils::apply_renaming_rule(rule, &name)).unwrap_or(name).into_token_stream()
                    });
                let value = Ident::new(&format!("arg{i}"), f.field.ty.span());

                if let Some(expr_path) = f.into_py_with.as_ref().map(|i|&i.value) {
                    let cow = if REF {
                        quote!(::std::borrow::Cow::Borrowed(#value))
                    } else {
                        quote!(::std::borrow::Cow::Owned(#value))
                    };
                    quote! {
                        let into_py_with: fn(::std::borrow::Cow<'_, _>, #pyo3_path::Python<'py>) -> #pyo3_path::PyResult<#pyo3_path::Bound<'py, #pyo3_path::PyAny>> = #expr_path;
                        #pyo3_path::types::PyDictMethods::set_item(&dict, #key, into_py_with(#cow, py)?)?;
                    }
                } else {
                    quote! {
                        #pyo3_path::types::PyDictMethods::set_item(&dict, #key, #value)?;
                    }
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
                let ty = &f.field.ty;
                let value = Ident::new(&format!("arg{i}"), f.field.ty.span());

                if let Some(expr_path) = f.into_py_with.as_ref().map(|i|&i.value) {
                    let cow = if REF {
                        quote!(::std::borrow::Cow::Borrowed(#value))
                    } else {
                        quote!(::std::borrow::Cow::Owned(#value))
                    };
                    quote_spanned! { ty.span() =>
                        {
                            let into_py_with: fn(::std::borrow::Cow<'_, _>, #pyo3_path::Python<'py>) -> #pyo3_path::PyResult<#pyo3_path::Bound<'py, #pyo3_path::PyAny>> = #expr_path;
                            into_py_with(#cow, py)?
                        },
                    }
                } else {
                    quote_spanned! { ty.span() =>
                        #pyo3_path::conversion::IntoPyObject::into_pyobject(#value, py)
                            .map(#pyo3_path::BoundObject::into_any)
                            .map(#pyo3_path::BoundObject::into_bound)?,
                    }
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
struct Enum<'a, const REF: bool> {
    variants: Vec<Container<'a, REF>>,
}

impl<'a, const REF: bool> Enum<'a, REF> {
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
                let attrs = ContainerAttributes::from_attrs(&variant.attrs)?;
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
                output: quote!(#pyo3_path::Bound<'py, <Self as #pyo3_path::conversion::IntoPyObject<'py>>::Target>),
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
    let options = ContainerAttributes::from_attrs(&tokens.attrs)?;
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
            if let Some(rename_all) = options.rename_all {
                bail_spanned!(rename_all.span() => "`rename_all` is not supported at top level for enums");
            }
            let en = Enum::<REF>::new(en, &tokens.ident)?;
            en.build(ctx)
        }
        syn::Data::Struct(st) => {
            let ident = &tokens.ident;
            let st = Container::<REF>::new(
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

            fn into_pyobject(self, py: #pyo3_path::Python<#lt_param>) -> ::std::result::Result<
                <Self as #pyo3_path::conversion::IntoPyObject<#lt_param>>::Output,
                <Self as #pyo3_path::conversion::IntoPyObject<#lt_param>>::Error,
            > {
                #body
            }
        }
    ))
}
