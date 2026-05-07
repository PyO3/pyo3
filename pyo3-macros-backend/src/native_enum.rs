use crate::attributes::{kw, take_attributes, CrateAttribute, KeywordAttribute, ModuleAttribute};
use crate::utils::Ctx;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, punctuated::Punctuated, spanned::Spanned, Data, DeriveInput,
    Expr, Fields, LitInt, LitStr, Token,
};

type BaseAttribute = KeywordAttribute<kw::base, LitStr>;
type RenameAttribute = KeywordAttribute<kw::rename, LitStr>;

enum NativeEnumOption {
    Base(BaseAttribute),
    Rename(RenameAttribute),
    Module(ModuleAttribute),
    Crate(CrateAttribute),
}

impl syn::parse::Parse for NativeEnumOption {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::base) {
            input.parse().map(NativeEnumOption::Base)
        } else if lookahead.peek(kw::rename) {
            input.parse().map(NativeEnumOption::Rename)
        } else if lookahead.peek(kw::module) {
            input.parse().map(NativeEnumOption::Module)
        } else if lookahead.peek(Token![crate]) {
            input.parse().map(NativeEnumOption::Crate)
        } else {
            Err(lookahead.error())
        }
    }
}

/// Parsed arguments for the `#[native_enum(...)]` attribute and `#[derive(NativeEnum)]` macro.
#[derive(Default)]
pub struct PyNativeEnumArgs {
    base: Option<BaseAttribute>,
    rename: Option<RenameAttribute>,
    module: Option<ModuleAttribute>,
    krate: Option<CrateAttribute>,
}

impl PyNativeEnumArgs {
    fn set_option(&mut self, option: NativeEnumOption) -> syn::Result<()> {
        macro_rules! set_option {
            ($key:ident) => {{
                ensure_spanned!(
                    self.$key.is_none(),
                    $key.kw.span() => concat!("`", stringify!($key), "` may only be specified once")
                );
                self.$key = Some($key);
            }};
        }
        match option {
            NativeEnumOption::Base(base) => set_option!(base),
            NativeEnumOption::Rename(rename) => set_option!(rename),
            NativeEnumOption::Module(module) => set_option!(module),
            NativeEnumOption::Crate(krate) => set_option!(krate),
        }
        Ok(())
    }

    fn take_from_attrs(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut result = Self::default();
        take_attributes(attrs, |attr| {
            if !attr.path().is_ident("native_enum") {
                return Ok(false);
            }
            for option in
                attr.parse_args_with(Punctuated::<NativeEnumOption, Token![,]>::parse_terminated)?
            {
                result.set_option(option)?;
            }
            Ok(true)
        })?;
        Ok(result)
    }
}

impl Parse for PyNativeEnumArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut result = Self::default();
        for option in Punctuated::<NativeEnumOption, Token![,]>::parse_terminated(input)? {
            result.set_option(option)?;
        }
        Ok(result)
    }
}

/// The right-hand side of `value = ...`: either an integer or a string literal.
enum ValueLit {
    Int(i64),
    Str(String),
}

impl syn::parse::Parse for ValueLit {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitInt) {
            let lit: LitInt = input.parse()?;
            lit.base10_parse::<i64>().map(ValueLit::Int)
        } else if lookahead.peek(LitStr) {
            input.parse::<LitStr>().map(|s| ValueLit::Str(s.value()))
        } else {
            Err(lookahead.error())
        }
    }
}

type VariantRenameAttribute = KeywordAttribute<kw::rename, LitStr>;
type VariantValueAttribute = KeywordAttribute<kw::value, ValueLit>;

enum NativeEnumVariantOption {
    Rename(VariantRenameAttribute),
    Value(VariantValueAttribute),
}

impl syn::parse::Parse for NativeEnumVariantOption {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::rename) {
            input.parse().map(NativeEnumVariantOption::Rename)
        } else if lookahead.peek(kw::value) {
            input.parse().map(NativeEnumVariantOption::Value)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Default)]
struct VariantAttrs {
    rename: Option<VariantRenameAttribute>,
    value: Option<VariantValueAttribute>,
}

impl VariantAttrs {
    fn set_option(&mut self, option: NativeEnumVariantOption) -> syn::Result<()> {
        macro_rules! set_option {
            ($key:ident) => {{
                ensure_spanned!(
                    self.$key.is_none(),
                    $key.kw.span() => concat!("`", stringify!($key), "` may only be specified once")
                );
                self.$key = Some($key);
            }};
        }
        match option {
            NativeEnumVariantOption::Rename(rename) => set_option!(rename),
            NativeEnumVariantOption::Value(value) => set_option!(value),
        }
        Ok(())
    }

    fn take_from_attrs(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut result = Self::default();
        take_attributes(attrs, |attr| {
            if !attr.path().is_ident("native_enum") {
                return Ok(false);
            }
            for option in attr.parse_args_with(
                Punctuated::<NativeEnumVariantOption, Token![,]>::parse_terminated,
            )? {
                result.set_option(option)?;
            }
            Ok(true)
        })?;
        Ok(result)
    }
}

fn impl_native_enum(
    ident: &syn::Ident,
    args: &PyNativeEnumArgs,
    variants: &mut Punctuated<syn::Variant, Token![,]>,
) -> syn::Result<TokenStream> {
    let ctx = Ctx::new(&args.krate, None);
    let pyo3 = &ctx.pyo3_path;

    let py_name = args
        .rename
        .as_ref()
        .map(|r| r.value.value())
        .unwrap_or_else(|| ident.to_string());

    let module_opt = match &args.module {
        Some(m) => {
            let s = &m.value;
            quote! { ::std::option::Option::Some(#s) }
        }
        None => quote! { ::std::option::Option::None },
    };

    let base_str = args
        .base
        .as_ref()
        .map(|b| b.value.value())
        .unwrap_or_else(|| "Enum".to_string());

    let base_expr = match base_str.as_str() {
        "Enum" => quote! { #pyo3::native_enum::NativeEnumBase::Enum },
        "IntEnum" => quote! { #pyo3::native_enum::NativeEnumBase::IntEnum },
        "StrEnum" => quote! { #pyo3::native_enum::NativeEnumBase::StrEnum },
        "Flag" => quote! { #pyo3::native_enum::NativeEnumBase::Flag },
        "IntFlag" => quote! { #pyo3::native_enum::NativeEnumBase::IntFlag },
        other => {
            let span = args
                .base
                .as_ref()
                .map(|b| b.value.span())
                .unwrap_or_else(|| ident.span());
            return Err(syn::Error::new(
                span,
                format!(
                    "unknown native_enum base: `{other}`. \
                     Expected one of: Enum, IntEnum, StrEnum, Flag, IntFlag"
                ),
            ));
        }
    };

    let mut variant_specs: Vec<TokenStream> = Vec::new();
    let mut into_arms: Vec<TokenStream> = Vec::new();
    let mut from_arms: Vec<TokenStream> = Vec::new();

    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "NativeEnum variants must be fieldless (unit variants)",
            ));
        }

        let variant_attrs = VariantAttrs::take_from_attrs(&mut variant.attrs)?;
        let rust_name = variant.ident.to_string();
        let py_member_name = variant_attrs
            .rename
            .as_ref()
            .map(|r| r.value.value())
            .unwrap_or_else(|| rust_name.clone());
        let variant_ident = &variant.ident;

        let value_expr = match &variant_attrs.value {
            Some(v) => match &v.value {
                ValueLit::Int(i) => quote! { #pyo3::native_enum::VariantValue::Int(#i) },
                ValueLit::Str(s) => quote! { #pyo3::native_enum::VariantValue::Str(#s) },
            },
            None => {
                if let Some((_, expr)) = &variant.discriminant {
                    let int_val: i64 = extract_discriminant_i64(expr)?;
                    quote! { #pyo3::native_enum::VariantValue::Int(#int_val) }
                } else if base_str == "StrEnum" {
                    quote! { #pyo3::native_enum::VariantValue::Str(#py_member_name) }
                } else {
                    quote! { #pyo3::native_enum::VariantValue::Auto }
                }
            }
        };

        variant_specs.push(quote! { (#py_member_name, #value_expr) });
        into_arms.push(quote! {
            Self::#variant_ident => #pyo3::intern!(py, #py_member_name),
        });
        from_arms.push(quote! {
            #py_member_name => ::std::result::Result::Ok(Self::#variant_ident),
        });
    }

    let str_enum_cfg_check = if base_str == "StrEnum" {
        quote! {
            #[cfg(not(Py_3_11))]
            const _: () = {
                ::std::compile_error!(
                    "NativeEnum with `base = \"StrEnum\"` requires Python 3.11 or later"
                );
            };
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #str_enum_cfg_check

        #[automatically_derived]
        impl #pyo3::native_enum::NativeEnum for #ident {
            const SPEC: #pyo3::native_enum::NativeEnumSpec = #pyo3::native_enum::NativeEnumSpec {
                name: #py_name,
                base: #base_expr,
                variants: &[#(#variant_specs),*],
                module: #module_opt,
                qualname: ::std::option::Option::None,
            };

            fn py_enum_class(
                py: #pyo3::Python<'_>,
            ) -> #pyo3::PyResult<#pyo3::Bound<'_, #pyo3::types::PyType>> {
                static PY_CLASS: #pyo3::sync::PyOnceLock<#pyo3::Py<#pyo3::types::PyType>> =
                    #pyo3::sync::PyOnceLock::new();
                PY_CLASS
                    .get_or_try_init(py, || {
                        #pyo3::native_enum::build_native_enum(py, &Self::SPEC)
                            .map(|cls| cls.unbind())
                    })
                    .map(|cls| cls.clone_ref(py).into_bound(py))
            }

            fn to_py_member<'py>(
                &self,
                py: #pyo3::Python<'py>,
            ) -> #pyo3::PyResult<#pyo3::Bound<'py, #pyo3::types::PyAny>> {
                let cls = <Self as #pyo3::native_enum::NativeEnum>::py_enum_class(py)?;
                let name = match self {
                    #(#into_arms)*
                };
                #pyo3::types::PyAnyMethods::getattr(cls.as_any(), name).map_err(::std::convert::Into::into)
            }

            fn from_py_member(
                obj: &#pyo3::Bound<'_, #pyo3::types::PyAny>,
            ) -> #pyo3::PyResult<Self> {
                let cls = <Self as #pyo3::native_enum::NativeEnum>::py_enum_class(obj.py())?;
                if !#pyo3::types::PyAnyMethods::is_instance(obj, cls.as_any())? {
                    return ::std::result::Result::Err(
                        #pyo3::exceptions::PyTypeError::new_err(
                            ::std::format!("expected a `{}` member", #py_name)
                        )
                    );
                }
                let name_obj = #pyo3::types::PyAnyMethods::getattr(
                    obj,
                    #pyo3::intern!(obj.py(), "name"),
                )?;
                let name = #pyo3::types::PyAnyMethods::extract::<&str>(
                    &name_obj
                )?;
                match name {
                    #(#from_arms)*
                    other => ::std::result::Result::Err(
                        #pyo3::exceptions::PyValueError::new_err(
                            ::std::format!("unknown `{}` variant: {}", #py_name, other)
                        )
                    ),
                }
            }
        }

        #[automatically_derived]
        impl<'py> #pyo3::IntoPyObject<'py> for #ident {
            type Target = #pyo3::types::PyAny;
            type Output = #pyo3::Bound<'py, Self::Target>;
            type Error = #pyo3::PyErr;

            fn into_pyobject(
                self,
                py: #pyo3::Python<'py>,
            ) -> ::std::result::Result<Self::Output, Self::Error> {
                #pyo3::native_enum::NativeEnum::to_py_member(&self, py)
            }
        }

        #[automatically_derived]
        impl<'py> #pyo3::IntoPyObject<'py> for &#ident {
            type Target = #pyo3::types::PyAny;
            type Output = #pyo3::Bound<'py, Self::Target>;
            type Error = #pyo3::PyErr;

            fn into_pyobject(
                self,
                py: #pyo3::Python<'py>,
            ) -> ::std::result::Result<Self::Output, Self::Error> {
                #pyo3::native_enum::NativeEnum::to_py_member(self, py)
            }
        }

        #[automatically_derived]
        impl<'py> #pyo3::FromPyObject<'_, 'py> for #ident {
            type Error = #pyo3::PyErr;

            fn extract(
                obj: #pyo3::Borrowed<'_, 'py, #pyo3::types::PyAny>,
            ) -> ::std::result::Result<Self, Self::Error> {
                #pyo3::native_enum::NativeEnum::from_py_member(&*obj)
            }
        }
    })
}

/// Entry point for `#[derive(NativeEnum)]`.
pub fn build_derive_native_enum(input: &mut DeriveInput) -> syn::Result<TokenStream> {
    let data_enum = match &mut input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                &*input,
                "NativeEnum can only be derived for enums",
            ))
        }
    };
    if let Some(lt) = input.generics.lifetimes().next() {
        bail_spanned!(lt.span() => "#[derive(NativeEnum)] cannot have lifetime parameters");
    }
    ensure_spanned!(
        input.generics.params.is_empty(),
        input.generics.span() => "#[derive(NativeEnum)] cannot have generic parameters"
    );
    let args = PyNativeEnumArgs::take_from_attrs(&mut input.attrs)?;
    impl_native_enum(&input.ident, &args, &mut data_enum.variants)
}

/// Entry point for `#[native_enum(...)]` attribute macro.
pub fn native_enum_impl(
    item: &mut syn::ItemEnum,
    args: PyNativeEnumArgs,
) -> syn::Result<TokenStream> {
    if let Some(lt) = item.generics.lifetimes().next() {
        bail_spanned!(lt.span() => "#[native_enum] cannot have lifetime parameters");
    }
    ensure_spanned!(
        item.generics.params.is_empty(),
        item.generics.span() => "#[native_enum] cannot have generic parameters"
    );
    impl_native_enum(&item.ident, &args, &mut item.variants)
}

/// Extracts an integer literal from a simple discriminant expression.
///
/// Only supports integer literals and negated integer literals. Complex expressions
/// must use `#[native_enum(value = N)]`.
fn extract_discriminant_i64(expr: &Expr) -> syn::Result<i64> {
    match expr {
        Expr::Lit(expr_lit) => {
            if let syn::Lit::Int(lit) = &expr_lit.lit {
                return lit.base10_parse::<i64>();
            }
        }
        Expr::Unary(expr_unary) => {
            if matches!(expr_unary.op, syn::UnOp::Neg(_)) {
                if let Expr::Lit(inner) = &*expr_unary.expr {
                    if let syn::Lit::Int(lit) = &inner.lit {
                        return lit.base10_parse::<i64>().map(|v| -v);
                    }
                }
            }
        }
        _ => {}
    }
    Err(syn::Error::new_spanned(
        expr,
        "NativeEnum only supports integer literal discriminants; \
         use `#[native_enum(value = N)]` for complex expressions",
    ))
}
