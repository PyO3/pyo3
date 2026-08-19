use crate::attributes::{self, get_pyo3_options, GcAttribute};
use crate::derive_attributes::ContainerAttributes;
use crate::utils::Ctx;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    DeriveInput, Fields, Result, Token,
};

struct GcField<'a> {
    member: syn::Member,
    ty: &'a syn::Type,
    include: bool,
}

enum GcFieldAttribute {
    Gc(GcAttribute),
}

impl Parse for GcFieldAttribute {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::gc) {
            let attr: GcAttribute = input.parse()?;
            Ok(Self::Gc(attr))
        } else {
            Err(lookahead.error())
        }
    }
}

fn parse_gc_field<'a>(field: &'a syn::Field, member: syn::Member) -> Result<GcField<'a>> {
    let mut gc = None;
    for attr in &field.attrs {
        if let Some(options) = get_pyo3_options::<GcFieldAttribute>(attr)? {
            for opt in options {
                let GcFieldAttribute::Gc(opt) = opt;
                ensure_spanned!(
                    gc.is_none(),
                    opt.span() => "`gc` may only be specified once"
                );
                gc = Some(opt);
            }
        } else if attr.path().is_ident("pyo3") {
            let _ =
                attr.parse_args_with(Punctuated::<GcFieldAttribute, Token![,]>::parse_terminated)?;
        }
    }

    let include = gc.as_ref().map(|attr| attr.value.value).unwrap_or(true);
    Ok(GcField {
        member,
        ty: &field.ty,
        include,
    })
}

fn fields_for_struct(fields: &Fields) -> Result<Vec<GcField<'_>>> {
    match fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().expect("named field must have ident");
                parse_gc_field(field, syn::Member::Named(ident.clone()))
            })
            .collect(),
        Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, field)| parse_gc_field(field, syn::Member::Unnamed(i.into())))
            .collect(),
        Fields::Unit => Ok(Vec::new()),
    }
}

fn traverse_stmts_for_struct(
    fields: &[GcField<'_>],
    pyo3_path: &crate::utils::PyO3CratePath,
) -> TokenStream {
    let included = fields.iter().filter(|field| field.include).map(|field| {
        let member = &field.member;
        let ty = field.ty;
        quote! {
            if <#ty as #pyo3_path::pyclass::PyGcTraversable>::MAY_CONTAIN_CYCLES {
                #pyo3_path::pyclass::PyGcTraversable::traverse(&self.#member, visit.clone())?;
            }
        }
    });

    quote! {
        #(#included)*
        Ok(())
    }
}

fn clear_stmts_for_struct(
    fields: &[GcField<'_>],
    pyo3_path: &crate::utils::PyO3CratePath,
) -> TokenStream {
    let included = fields.iter().filter(|field| field.include).map(|field| {
        let member = &field.member;
        let ty = field.ty;
        quote! {
            if <#ty as #pyo3_path::pyclass::PyGcTraversable>::MAY_CONTAIN_CYCLES {
                #pyo3_path::pyclass::PyGcTraversable::clear(&mut self.#member);
            }
        }
    });

    quote! {
        #(#included)*
    }
}

fn cycle_or_expr(fields: &[GcField<'_>], pyo3_path: &crate::utils::PyO3CratePath) -> TokenStream {
    let mut included = fields.iter().filter(|field| field.include);
    let Some(first) = included.next() else {
        return quote!(false);
    };

    let first_ty = first.ty;
    let rest = included.map(|field| {
        let ty = field.ty;
        quote!(|| <#ty as #pyo3_path::pyclass::PyGcTraversable>::MAY_CONTAIN_CYCLES)
    });

    quote!(
        <#first_ty as #pyo3_path::pyclass::PyGcTraversable>::MAY_CONTAIN_CYCLES #(#rest)*
    )
}

fn collect_where_predicates<'a>(
    fields: impl Iterator<Item = &'a GcField<'a>>,
    pyo3_path: &crate::utils::PyO3CratePath,
) -> Punctuated<syn::WherePredicate, syn::token::Comma> {
    fields
        .filter(|field| field.include)
        .map(|field| -> syn::WherePredicate {
            let ty = field.ty;
            parse_quote!(#ty: #pyo3_path::pyclass::PyGcTraversable)
        })
        .collect::<Punctuated<syn::WherePredicate, syn::token::Comma>>()
}

fn append_where_predicates(
    mut generics: syn::Generics,
    predicates: Punctuated<syn::WherePredicate, syn::token::Comma>,
) -> syn::Generics {
    if predicates.is_empty() {
        return generics;
    }

    let where_clause = generics.make_where_clause();
    where_clause.predicates.extend(predicates);
    generics
}

fn assertion_impl(fields: &[GcField<'_>], pyo3_path: &crate::utils::PyO3CratePath) -> TokenStream {
    let assertions: Vec<_> = fields
        .iter()
        .filter(|field| !field.include)
        .enumerate()
        .map(|(i, field)| {
            let const_ident = format_ident!("__PYO3_GC_FALSE_ASSERT_{i}", span = Span::call_site());
            let ty = field.ty;
            quote! {
                const #const_ident: () = {
                    #[allow(unused_imports, reason = "Probe not used if assertion trips")]
                    use #pyo3_path::impl_::pyclass::{IsPyGcTraversable, Probe as _};
                    assert!(
                        !IsPyGcTraversable::<#ty>::VALUE,
                        "`#[pyo3(gc = false)]` may not be used on fields which implement `PyGcTraversable`"
                    );
                };
            }
        })
        .collect();

    if assertions.is_empty() {
        return quote! {};
    }

    quote! {
        #(#assertions)*
    }
}

pub fn build_derive_py_gc_integration(tokens: &DeriveInput) -> Result<TokenStream> {
    let options = ContainerAttributes::from_attrs(&tokens.attrs)?;
    ensure_spanned!(
        options.transparent.is_none(),
        options.transparent.span() => "`transparent` is not supported for `#[derive(PyGcTraversable)]`"
    );
    ensure_spanned!(
        options.from_item_all.is_none(),
        options.from_item_all.span() => "`from_item_all` is not supported for `#[derive(PyGcTraversable)]`"
    );
    ensure_spanned!(
        options.annotation.is_none(),
        options.annotation.span() => "`annotation` is not supported for `#[derive(PyGcTraversable)]`"
    );
    ensure_spanned!(
        options.rename_all.is_none(),
        options.rename_all.span() => "`rename_all` is not supported for `#[derive(PyGcTraversable)]`"
    );

    let ctx = Ctx::new(&options.krate, None);
    let pyo3_path = &ctx.pyo3_path;
    let ident = &tokens.ident;

    let (fields, traverse_body, clear_body, cycle_expr) = match &tokens.data {
        syn::Data::Struct(data) => {
            let fields = fields_for_struct(&data.fields)?;
            let traverse = traverse_stmts_for_struct(&fields, pyo3_path);
            let clear = clear_stmts_for_struct(&fields, pyo3_path);
            let cycles = cycle_or_expr(&fields, pyo3_path);
            (fields, quote!(#traverse), quote!(#clear), cycles)
        }
        syn::Data::Enum(data) => {
            ensure_spanned!(
                !data.variants.is_empty(),
                tokens.span() => "cannot derive `PyGcTraversable` for empty enum"
            );

            let mut all_fields = Vec::new();
            let mut traverse_arms = Vec::new();
            let mut clear_arms = Vec::new();

            for variant in &data.variants {
                let variant_ident = &variant.ident;
                let variant_fields = fields_for_struct(&variant.fields)?;
                all_fields.extend(variant_fields.iter().map(|field| GcField {
                    member: field.member.clone(),
                    ty: field.ty,
                    include: field.include,
                }));

                match &variant.fields {
                    Fields::Named(named) => {
                        let bindings: Vec<_> = named
                            .named
                            .iter()
                            .enumerate()
                            .map(|(i, field)| {
                                let field_ident = field.ident.as_ref().expect("named field");
                                let binding = format_ident!("field_{i}");
                                quote!(#field_ident: #binding)
                            })
                            .collect();

                        let traverse_stmts = variant_fields
                            .iter()
                            .enumerate()
                            .filter(|(_, field)| field.include)
                            .map(|(i, field)| {
                                let binding = format_ident!("field_{i}");
                                let ty = field.ty;
                                quote! {
                                    if <#ty as #pyo3_path::pyclass::PyGcTraversable>::MAY_CONTAIN_CYCLES {
                                        #pyo3_path::pyclass::PyGcTraversable::traverse(#binding, visit.clone())?;
                                    }
                                }
                            });

                        let clear_stmts = variant_fields
                            .iter()
                            .enumerate()
                            .filter(|(_, field)| field.include)
                            .map(|(i, field)| {
                                let binding = format_ident!("field_{i}");
                                let ty = field.ty;
                                quote! {
                                    if <#ty as #pyo3_path::pyclass::PyGcTraversable>::MAY_CONTAIN_CYCLES {
                                        #pyo3_path::pyclass::PyGcTraversable::clear(#binding);
                                    }
                                }
                            });

                        traverse_arms.push(quote! {
                            Self::#variant_ident { #(#bindings),* } => {
                                #(#traverse_stmts)*
                                Ok(())
                            }
                        });
                        clear_arms.push(quote! {
                            Self::#variant_ident { #(#bindings),* } => {
                                #(#clear_stmts)*
                            }
                        });
                    }
                    Fields::Unnamed(unnamed) => {
                        let bindings: Vec<_> = unnamed
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| format_ident!("field_{i}"))
                            .collect();

                        let traverse_stmts = variant_fields
                            .iter()
                            .enumerate()
                            .filter(|(_, field)| field.include)
                            .map(|(i, field)| {
                                let binding = format_ident!("field_{i}");
                                let ty = field.ty;
                                quote! {
                                    if <#ty as #pyo3_path::pyclass::PyGcTraversable>::MAY_CONTAIN_CYCLES {
                                        #pyo3_path::pyclass::PyGcTraversable::traverse(#binding, visit.clone())?;
                                    }
                                }
                            });

                        let clear_stmts = variant_fields
                            .iter()
                            .enumerate()
                            .filter(|(_, field)| field.include)
                            .map(|(i, field)| {
                                let binding = format_ident!("field_{i}");
                                let ty = field.ty;
                                quote! {
                                    if <#ty as #pyo3_path::pyclass::PyGcTraversable>::MAY_CONTAIN_CYCLES {
                                        #pyo3_path::pyclass::PyGcTraversable::clear(#binding);
                                    }
                                }
                            });

                        traverse_arms.push(quote! {
                            Self::#variant_ident(#(#bindings),*) => {
                                #(#traverse_stmts)*
                                Ok(())
                            }
                        });
                        clear_arms.push(quote! {
                            Self::#variant_ident(#(#bindings),*) => {
                                #(#clear_stmts)*
                            }
                        });
                    }
                    Fields::Unit => {
                        traverse_arms.push(quote!(Self::#variant_ident => Ok(())));
                        clear_arms.push(quote!(Self::#variant_ident => {}));
                    }
                }
            }

            let cycles = cycle_or_expr(&all_fields, pyo3_path);
            (
                all_fields,
                quote! {
                    match self {
                        #(#traverse_arms),*
                    }
                },
                quote! {
                    match self {
                        #(#clear_arms),*
                    }
                },
                cycles,
            )
        }
        syn::Data::Union(_) => {
            bail_spanned!(tokens.span() => "#[derive(PyGcTraversable)] is not supported for unions")
        }
    };

    let predicates = collect_where_predicates(fields.iter(), pyo3_path);
    let impl_generics = append_where_predicates(tokens.generics.clone(), predicates);
    let (impl_generics, ty_generics, where_clause) = impl_generics.split_for_impl();

    let assertions = assertion_impl(&fields, pyo3_path);

    Ok(quote! {
        #[automatically_derived]
        unsafe impl #impl_generics #pyo3_path::pyclass::PyGcTraversable for #ident #ty_generics #where_clause {
            const MAY_CONTAIN_CYCLES: bool = #cycle_expr;

            fn traverse(&self, visit: #pyo3_path::pyclass::PyVisit<'_>) -> ::std::result::Result<(), #pyo3_path::pyclass::PyTraverseError> {
                #traverse_body
            }

            fn clear(&mut self) {
                #clear_body
            }
        }

        #assertions
    })
}
