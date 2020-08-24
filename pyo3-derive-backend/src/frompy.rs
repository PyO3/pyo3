use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{
    parse_quote, Attribute, DataEnum, DeriveInput, Expr, ExprCall, Fields, Ident, PatTuple, Result,
    Variant,
};

/// Describes derivation input of an enum.
#[derive(Debug)]
struct Enum<'a> {
    enum_ident: &'a Ident,
    vars: Vec<Container<'a>>,
}

impl<'a> Enum<'a> {
    /// Construct a new enum representation.
    ///
    /// `data_enum` is the `syn` representation of the input enum, `ident` is the
    /// `Identifier` of the enum.
    fn new(data_enum: &'a DataEnum, ident: &'a Ident) -> Result<Self> {
        if data_enum.variants.is_empty() {
            return Err(syn::Error::new_spanned(
                &data_enum.variants,
                "Cannot derive FromPyObject for empty enum.",
            ));
        }
        let vars = data_enum
            .variants
            .iter()
            .map(Container::from_variant)
            .collect::<Result<Vec<_>>>()?;

        Ok(Enum {
            enum_ident: ident,
            vars,
        })
    }

    /// Build derivation body for enums.
    fn derive_enum(&self) -> TokenStream {
        let mut var_extracts = Vec::new();
        let mut error_names = String::new();
        for (i, var) in self.vars.iter().enumerate() {
            let ext = match &var.style {
                Style::Struct(tups) => self.build_struct_variant(tups, var.ident),
                Style::StructNewtype(ident) => {
                    self.build_transparent_variant(var.ident, Some(ident))
                }
                Style::Tuple(len) => self.build_tuple_variant(var.ident, *len),
                Style::TupleNewtype => self.build_transparent_variant(var.ident, None),
            };
            var_extracts.push(ext);
            error_names.push_str(&var.err_name);
            if i < self.vars.len() - 1 {
                error_names.push_str(", ");
            }
        }
        quote!(
            #(#var_extracts)*
            let type_name = obj.get_type().name();
            let from = obj
                .repr()
                .map(|s| format!("{} ({})", s.to_string_lossy(), type_name))
                .unwrap_or_else(|_| type_name.to_string());
            let err_msg = format!("Can't convert {} to {}", from, #error_names);
            Err(::pyo3::exceptions::PyTypeError::py_err(err_msg))
        )
    }

    /// Build match for tuple struct variant.
    fn build_tuple_variant(&self, var_ident: &Ident, len: usize) -> TokenStream {
        let enum_ident = self.enum_ident;
        let mut ext: Punctuated<Expr, syn::Token![,]> = Punctuated::new();
        let mut fields: Punctuated<Ident, syn::Token![,]> = Punctuated::new();
        let mut field_pats = PatTuple {
            attrs: vec![],
            paren_token: Paren::default(),
            elems: Default::default(),
        };
        for i in 0..len {
            ext.push(parse_quote!(slice[#i].extract()));
            let ident = Ident::new(&format!("_field{}", i), Span::call_site());
            field_pats.elems.push(parse_quote!(Ok(#ident)));
            fields.push(ident);
        }

        quote!(
            match <::pyo3::types::PyTuple as ::pyo3::conversion::PyTryFrom>::try_from(obj) {
                Ok(s) => {
                    if s.len() == #len {
                        let slice = s.as_slice();
                        if let (#field_pats) = (#ext) {
                            return Ok(#enum_ident::#var_ident(#fields))
                        }
                    }
                },
                Err(_) => {}
            }
        )
    }

    /// Build match for transparent enum variants.
    fn build_transparent_variant(
        &self,
        var_ident: &Ident,
        field_ident: Option<&Ident>,
    ) -> TokenStream {
        let enum_ident = self.enum_ident;
        if let Some(ident) = field_ident {
            quote!(
                if let Ok(#ident) = obj.extract() {
                    return Ok(#enum_ident::#var_ident{#ident})
                }
            )
        } else {
            quote!(
                if let Ok(inner) = obj.extract() {
                    return Ok(#enum_ident::#var_ident(inner))
                }
            )
        }
    }

    /// Build match for struct variant with named fields.
    fn build_struct_variant(
        &self,
        tups: &[(&'a Ident, ExprCall)],
        var_ident: &Ident,
    ) -> TokenStream {
        let enum_ident = self.enum_ident;
        let mut field_pats = PatTuple {
            attrs: vec![],
            paren_token: Paren::default(),
            elems: Default::default(),
        };
        let mut fields: Punctuated<Expr, syn::Token![,]> = Punctuated::new();
        let mut ext: Punctuated<Expr, syn::Token![,]> = Punctuated::new();
        for (ident, ext_fn) in tups {
            field_pats.elems.push(parse_quote!(Ok(#ident)));
            fields.push(parse_quote!(#ident));
            ext.push(parse_quote!(obj.#ext_fn.and_then(|o| o.extract())));
        }
        quote!(if let #field_pats = #ext {
            return Ok(#enum_ident::#var_ident{#fields});
        })
    }
}

/// Container Style
///
/// Covers Structs, Tuplestructs and corresponding Newtypes.
#[derive(Clone, Debug)]
enum Style<'a> {
    /// Struct Container, e.g. `struct Foo { a: String }`
    ///
    /// Variant contains the list of field identifiers and the corresponding extraction call.
    Struct(Vec<(&'a Ident, ExprCall)>),
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
    ident: &'a Ident,
    style: Style<'a>,
    err_name: String,
}

impl<'a> Container<'a> {
    /// Construct a container from an enum Variant.
    ///
    /// Fails if the variant has no fields or incompatible attributes.
    fn from_variant(var: &'a Variant) -> Result<Self> {
        Self::new(&var.fields, &var.ident, &var.attrs)
    }

    /// Construct a container based on fields, identifier and attributes.
    ///
    /// Fails if the variant has no fields or incompatible attributes.
    fn new(fields: &'a Fields, ident: &'a Ident, attrs: &'a [Attribute]) -> Result<Self> {
        let transparent = attrs.iter().any(|a| a.path.is_ident("transparent"));
        if transparent {
            Self::check_transparent_len(fields)?;
        }
        let style = match fields {
            Fields::Unnamed(unnamed) => {
                if transparent {
                    Style::TupleNewtype
                } else {
                    Style::Tuple(unnamed.unnamed.len())
                }
            }
            Fields::Named(named) => {
                if transparent {
                    let field = named
                        .named
                        .iter()
                        .next()
                        .expect("Check for len 1 is done above");
                    let ident = field
                        .ident
                        .as_ref()
                        .expect("Named fields should have identifiers");
                    Style::StructNewtype(ident)
                } else {
                    let mut fields = Vec::new();
                    for field in named.named.iter() {
                        let ident = field
                            .ident
                            .as_ref()
                            .expect("Named fields should have identifiers");
                        fields.push((ident, ext_fn(&field.attrs, ident)?))
                    }
                    Style::Struct(fields)
                }
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    &fields,
                    "Cannot derive FromPyObject for Unit structs and variants",
                ))
            }
        };
        let err_name = maybe_renamed_err(&attrs)?
            .map(|s| s.value())
            .unwrap_or_else(|| ident.to_string());

        let v = Container {
            ident: &ident,
            style,
            err_name,
        };
        Ok(v)
    }

    /// Build derivation body for a struct.
    fn derive_struct(&self) -> TokenStream {
        match &self.style {
            Style::StructNewtype(ident) => self.build_newtype_struct(Some(&ident)),
            Style::TupleNewtype => self.build_newtype_struct(None),
            Style::Tuple(len) => self.build_tuple_struct(*len),
            Style::Struct(tups) => self.build_struct(tups),
        }
    }

    fn build_newtype_struct(&self, field_ident: Option<&Ident>) -> TokenStream {
        if let Some(ident) = field_ident {
            quote!(
                Ok(Self{#ident: obj.extract()?})
            )
        } else {
            quote!(Ok(Self(obj.extract()?)))
        }
    }

    fn build_tuple_struct(&self, len: usize) -> TokenStream {
        let mut fields: Punctuated<TokenStream, syn::Token![,]> = Punctuated::new();
        for i in 0..len {
            fields.push(quote!(slice[#i].extract()?));
        }
        quote!(
            let s = <::pyo3::types::PyTuple as ::pyo3::conversion::PyTryFrom>::try_from(obj)?;
            let seq_len = s.len();
            if seq_len != #len {
                let msg = format!(
                    "Expected tuple of length {}, but got length {}.",
                    #len,
                    seq_len
                );
                return Err(::pyo3::exceptions::PyValueError::py_err(msg))
            }
            let slice = s.as_slice();
            Ok(Self(#fields))
        )
    }

    fn build_struct(&self, tups: &[(&Ident, syn::ExprCall)]) -> TokenStream {
        let mut fields: Punctuated<TokenStream, syn::Token![,]> = Punctuated::new();
        for (ident, ext_fn) in tups {
            fields.push(quote!(#ident: obj.#ext_fn?.extract()?));
        }
        quote!(Ok(Self{#fields}))
    }

    fn check_transparent_len(fields: &Fields) -> Result<()> {
        if fields.len() != 1 {
            return Err(syn::Error::new_spanned(
                fields,
                "Transparent structs and variants can only have 1 field",
            ));
        }
        Ok(())
    }
}

/// Get the extraction function that's called on the input object.
///
/// Valid arguments are `get_item`, `get_attr` which are called with the
/// stringified field identifier or a function call on `PyAny`, e.g. `get_attr("attr")`
fn ext_fn(attrs: &[Attribute], field_ident: &Ident) -> Result<syn::ExprCall> {
    let attr = if let Some(attr) = attrs.iter().find(|a| a.path.is_ident("extract")) {
        attr
    } else {
        return Ok(parse_quote!(getattr(stringify!(#field_ident))));
    };
    if let Ok(ident) = attr.parse_args::<Ident>() {
        if ident != "getattr" && ident != "get_item" {
            Err(syn::Error::new_spanned(
                ident,
                "Only get_item and getattr are valid for extraction.",
            ))
        } else {
            let arg = field_ident.to_string();
            Ok(parse_quote!(#ident(#arg)))
        }
    } else if let Ok(call) = attr.parse_args() {
        Ok(call)
    } else {
        Err(syn::Error::new_spanned(
            attr,
            "Only get_item and getattr are valid for extraction,\
            both can be passed with or without an argument, e.g. \
            #[extract(getattr(\"attr\")] and #[extract(getattr)]",
        ))
    }
}

/// Returns the name of the variant for the error message if no variants match.
fn maybe_renamed_err(attrs: &[syn::Attribute]) -> Result<Option<syn::LitStr>> {
    for attr in attrs {
        if !attr.path.is_ident("rename_err") {
            continue;
        }
        let attr = attr.parse_meta()?;
        if let syn::Meta::NameValue(nv) = &attr {
            match &nv.lit {
                syn::Lit::Str(s) => {
                    return Ok(Some(s.clone()));
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "rename_err attribute must be string literal: #[rename_err=\"Name\"]",
                    ))
                }
            }
        }
    }
    Ok(None)
}

fn verify_and_get_lifetime(generics: &syn::Generics) -> Result<Option<&syn::LifetimeDef>> {
    let lifetimes = generics.lifetimes().collect::<Vec<_>>();
    if lifetimes.len() > 1 {
        return Err(syn::Error::new_spanned(
            &generics,
            "Only a single lifetime parameter can be specified.",
        ));
    }
    Ok(lifetimes.into_iter().next())
}

/// Derive FromPyObject for enums and structs.
///
///   * Max 1 lifetime specifier, will be tied to `FromPyObject`'s specifier
///   * At least one field, in case of `#[transparent]`, exactly one field
///   * At least one variant for enums.
///   * Fields of input structs and enums must implement `FromPyObject`
///   * Derivation for structs with generic fields like `struct<T> Foo(T)`
///     adds `T: FromPyObject` on the derived implementation.
pub fn build_derive_from_pyobject(tokens: &mut DeriveInput) -> Result<TokenStream> {
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
            en.derive_enum()
        }
        syn::Data::Struct(st) => {
            let st = Container::new(&st.fields, &tokens.ident, &tokens.attrs)?;
            st.derive_struct()
        }
        _ => {
            return Err(syn::Error::new_spanned(
                tokens,
                "FromPyObject can only be derived for structs and enums.",
            ))
        }
    };

    let ident = &tokens.ident;
    Ok(quote!(
        #[automatically_derived]
        impl#trait_generics ::pyo3::FromPyObject<#lt_param> for #ident#generics #where_clause {
            fn extract(obj: &#lt_param ::pyo3::PyAny) -> ::pyo3::PyResult<Self>  {
                #derives
            }
        }
    ))
}
