use std::borrow::Cow;

use crate::attributes::kw::frozen;
use crate::attributes::{
    self, kw, take_pyo3_options, CrateAttribute, ExtendsAttribute, FreelistAttribute,
    ModuleAttribute, NameAttribute, NameLitStr, RenameAllAttribute,
};
use crate::deprecations::Deprecations;
use crate::konst::{ConstAttributes, ConstSpec};
use crate::method::{FnArg, FnSpec};
use crate::pyimpl::{gen_py_const, PyClassMethodsType};
use crate::pymethod::{
    impl_py_getter_def, impl_py_setter_def, MethodAndMethodDef, MethodAndSlotDef, PropertyType,
    SlotDef, __INT__, __REPR__, __RICHCMP__,
};
use crate::utils::{self, apply_renaming_rule, get_pyo3_crate, PythonDoc};
use crate::PyFunctionOptions;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_quote, spanned::Spanned, Result, Token};

/// If the class is derived from a Rust `struct` or `enum`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PyClassKind {
    Struct,
    Enum,
}

/// The parsed arguments of the pyclass macro
#[derive(Clone)]
pub struct PyClassArgs {
    pub class_kind: PyClassKind,
    pub options: PyClassPyO3Options,
}

impl PyClassArgs {
    fn parse(input: ParseStream<'_>, kind: PyClassKind) -> Result<Self> {
        Ok(PyClassArgs {
            class_kind: kind,
            options: PyClassPyO3Options::parse(input)?,
        })
    }

    pub fn parse_stuct_args(input: ParseStream<'_>) -> syn::Result<Self> {
        Self::parse(input, PyClassKind::Struct)
    }

    pub fn parse_enum_args(input: ParseStream<'_>) -> syn::Result<Self> {
        Self::parse(input, PyClassKind::Enum)
    }
}

#[derive(Clone, Default)]
pub struct PyClassPyO3Options {
    pub krate: Option<CrateAttribute>,
    pub dict: Option<kw::dict>,
    pub extends: Option<ExtendsAttribute>,
    pub get_all: Option<kw::get_all>,
    pub freelist: Option<FreelistAttribute>,
    pub frozen: Option<kw::frozen>,
    pub mapping: Option<kw::mapping>,
    pub module: Option<ModuleAttribute>,
    pub name: Option<NameAttribute>,
    pub rename_all: Option<RenameAllAttribute>,
    pub sequence: Option<kw::sequence>,
    pub set_all: Option<kw::set_all>,
    pub subclass: Option<kw::subclass>,
    pub unsendable: Option<kw::unsendable>,
    pub weakref: Option<kw::weakref>,
}

enum PyClassPyO3Option {
    Crate(CrateAttribute),
    Dict(kw::dict),
    Extends(ExtendsAttribute),
    Freelist(FreelistAttribute),
    Frozen(kw::frozen),
    GetAll(kw::get_all),
    Mapping(kw::mapping),
    Module(ModuleAttribute),
    Name(NameAttribute),
    RenameAll(RenameAllAttribute),
    Sequence(kw::sequence),
    SetAll(kw::set_all),
    Subclass(kw::subclass),
    Unsendable(kw::unsendable),
    Weakref(kw::weakref),
}

impl Parse for PyClassPyO3Option {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![crate]) {
            input.parse().map(PyClassPyO3Option::Crate)
        } else if lookahead.peek(kw::dict) {
            input.parse().map(PyClassPyO3Option::Dict)
        } else if lookahead.peek(kw::extends) {
            input.parse().map(PyClassPyO3Option::Extends)
        } else if lookahead.peek(attributes::kw::freelist) {
            input.parse().map(PyClassPyO3Option::Freelist)
        } else if lookahead.peek(attributes::kw::frozen) {
            input.parse().map(PyClassPyO3Option::Frozen)
        } else if lookahead.peek(attributes::kw::get_all) {
            input.parse().map(PyClassPyO3Option::GetAll)
        } else if lookahead.peek(attributes::kw::mapping) {
            input.parse().map(PyClassPyO3Option::Mapping)
        } else if lookahead.peek(attributes::kw::module) {
            input.parse().map(PyClassPyO3Option::Module)
        } else if lookahead.peek(kw::name) {
            input.parse().map(PyClassPyO3Option::Name)
        } else if lookahead.peek(kw::rename_all) {
            input.parse().map(PyClassPyO3Option::RenameAll)
        } else if lookahead.peek(attributes::kw::sequence) {
            input.parse().map(PyClassPyO3Option::Sequence)
        } else if lookahead.peek(attributes::kw::set_all) {
            input.parse().map(PyClassPyO3Option::SetAll)
        } else if lookahead.peek(attributes::kw::subclass) {
            input.parse().map(PyClassPyO3Option::Subclass)
        } else if lookahead.peek(attributes::kw::unsendable) {
            input.parse().map(PyClassPyO3Option::Unsendable)
        } else if lookahead.peek(attributes::kw::weakref) {
            input.parse().map(PyClassPyO3Option::Weakref)
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for PyClassPyO3Options {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut options: PyClassPyO3Options = Default::default();

        for option in Punctuated::<PyClassPyO3Option, syn::Token![,]>::parse_terminated(input)? {
            options.set_option(option)?;
        }

        Ok(options)
    }
}

impl PyClassPyO3Options {
    pub fn take_pyo3_options(&mut self, attrs: &mut Vec<syn::Attribute>) -> syn::Result<()> {
        take_pyo3_options(attrs)?
            .into_iter()
            .try_for_each(|option| self.set_option(option))
    }

    fn set_option(&mut self, option: PyClassPyO3Option) -> syn::Result<()> {
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
            PyClassPyO3Option::Crate(krate) => set_option!(krate),
            PyClassPyO3Option::Dict(dict) => set_option!(dict),
            PyClassPyO3Option::Extends(extends) => set_option!(extends),
            PyClassPyO3Option::Freelist(freelist) => set_option!(freelist),
            PyClassPyO3Option::Frozen(frozen) => set_option!(frozen),
            PyClassPyO3Option::GetAll(get_all) => set_option!(get_all),
            PyClassPyO3Option::Mapping(mapping) => set_option!(mapping),
            PyClassPyO3Option::Module(module) => set_option!(module),
            PyClassPyO3Option::Name(name) => set_option!(name),
            PyClassPyO3Option::RenameAll(rename_all) => set_option!(rename_all),
            PyClassPyO3Option::Sequence(sequence) => set_option!(sequence),
            PyClassPyO3Option::SetAll(set_all) => set_option!(set_all),
            PyClassPyO3Option::Subclass(subclass) => set_option!(subclass),
            PyClassPyO3Option::Unsendable(unsendable) => set_option!(unsendable),
            PyClassPyO3Option::Weakref(weakref) => set_option!(weakref),
        }
        Ok(())
    }
}

pub fn build_py_class(
    class: &mut syn::ItemStruct,
    mut args: PyClassArgs,
    methods_type: PyClassMethodsType,
) -> syn::Result<TokenStream> {
    args.options.take_pyo3_options(&mut class.attrs)?;
    let doc = utils::get_doc(&class.attrs, None);
    let krate = get_pyo3_crate(&args.options.krate);

    if let Some(lt) = class.generics.lifetimes().next() {
        bail_spanned!(
            lt.span() =>
            "#[pyclass] cannot have lifetime parameters. \
            For an explanation, see https://pyo3.rs/latest/class.html#no-lifetime-parameters"
        );
    }

    ensure_spanned!(
        class.generics.params.is_empty(),
        class.generics.span() =>
            "#[pyclass] cannot have generic parameters. \
            For an explanation, see https://pyo3.rs/latest/class.html#no-generic-parameters"
    );

    let mut field_options: Vec<(&syn::Field, FieldPyO3Options)> = match &mut class.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter_mut()
            .map(|field| {
                FieldPyO3Options::take_pyo3_options(&mut field.attrs)
                    .map(move |options| (&*field, options))
            })
            .collect::<Result<_>>()?,
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter_mut()
            .map(|field| {
                FieldPyO3Options::take_pyo3_options(&mut field.attrs)
                    .map(move |options| (&*field, options))
            })
            .collect::<Result<_>>()?,
        syn::Fields::Unit => {
            if let Some(attr) = args.options.set_all {
                return Err(syn::Error::new_spanned(attr, UNIT_SET));
            };
            if let Some(attr) = args.options.get_all {
                return Err(syn::Error::new_spanned(attr, UNIT_GET));
            };
            // No fields for unit struct
            Vec::new()
        }
    };

    if let Some(attr) = args.options.get_all {
        for (_, FieldPyO3Options { get, .. }) in &mut field_options {
            if let Some(old_get) = get.replace(Annotated::Struct(attr)) {
                return Err(syn::Error::new(old_get.span(), DUPE_GET));
            }
        }
    }

    if let Some(attr) = args.options.set_all {
        for (_, FieldPyO3Options { set, .. }) in &mut field_options {
            if let Some(old_set) = set.replace(Annotated::Struct(attr)) {
                return Err(syn::Error::new(old_set.span(), DUPE_SET));
            }
        }
    }

    impl_class(&class.ident, &args, doc, field_options, methods_type, krate)
}

enum Annotated<X, Y> {
    Field(X),
    Struct(Y),
}

impl<X: Spanned, Y: Spanned> Annotated<X, Y> {
    fn span(&self) -> Span {
        match self {
            Self::Field(x) => x.span(),
            Self::Struct(y) => y.span(),
        }
    }
}

/// `#[pyo3()]` options for pyclass fields
struct FieldPyO3Options {
    get: Option<Annotated<kw::get, kw::get_all>>,
    set: Option<Annotated<kw::set, kw::set_all>>,
    name: Option<NameAttribute>,
}

enum FieldPyO3Option {
    Get(attributes::kw::get),
    Set(attributes::kw::set),
    Name(NameAttribute),
}

impl Parse for FieldPyO3Option {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::get) {
            input.parse().map(FieldPyO3Option::Get)
        } else if lookahead.peek(attributes::kw::set) {
            input.parse().map(FieldPyO3Option::Set)
        } else if lookahead.peek(attributes::kw::name) {
            input.parse().map(FieldPyO3Option::Name)
        } else {
            Err(lookahead.error())
        }
    }
}

impl FieldPyO3Options {
    fn take_pyo3_options(attrs: &mut Vec<syn::Attribute>) -> Result<Self> {
        let mut options = FieldPyO3Options {
            get: None,
            set: None,
            name: None,
        };

        for option in take_pyo3_options(attrs)? {
            match option {
                FieldPyO3Option::Get(kw) => {
                    if options.get.replace(Annotated::Field(kw)).is_some() {
                        return Err(syn::Error::new(kw.span(), UNIQUE_GET));
                    }
                }
                FieldPyO3Option::Set(kw) => {
                    if options.set.replace(Annotated::Field(kw)).is_some() {
                        return Err(syn::Error::new(kw.span(), UNIQUE_SET));
                    }
                }
                FieldPyO3Option::Name(name) => {
                    if options.name.replace(name).is_some() {
                        return Err(syn::Error::new(options.name.span(), UNIQUE_NAME));
                    }
                }
            }
        }

        Ok(options)
    }
}

fn get_class_python_name<'a>(cls: &'a syn::Ident, args: &'a PyClassArgs) -> Cow<'a, syn::Ident> {
    args.options
        .name
        .as_ref()
        .map(|name_attr| Cow::Borrowed(&name_attr.value.0))
        .unwrap_or_else(|| Cow::Owned(cls.unraw()))
}

fn impl_class(
    cls: &syn::Ident,
    args: &PyClassArgs,
    doc: PythonDoc,
    field_options: Vec<(&syn::Field, FieldPyO3Options)>,
    methods_type: PyClassMethodsType,
    krate: syn::Path,
) -> syn::Result<TokenStream> {
    let pytypeinfo_impl = impl_pytypeinfo(cls, args, None);

    let py_class_impl = PyClassImplsBuilder::new(
        cls,
        args,
        methods_type,
        descriptors_to_items(
            cls,
            args.options.rename_all.as_ref(),
            args.options.frozen,
            field_options,
        )?,
        vec![],
    )
    .doc(doc)
    .impl_all()?;

    Ok(quote! {
        // FIXME https://github.com/PyO3/pyo3/issues/3903
        #[allow(unknown_lints, non_local_definitions)]
        const _: () = {
            use #krate as _pyo3;

            impl _pyo3::types::DerefToPyAny for #cls {}

            #pytypeinfo_impl

            #py_class_impl
        };
    })
}

enum PyClassEnum<'a> {
    Simple(PyClassSimpleEnum<'a>),
    Complex(PyClassComplexEnum<'a>),
}

impl<'a> PyClassEnum<'a> {
    fn new(enum_: &'a mut syn::ItemEnum) -> syn::Result<Self> {
        let has_only_unit_variants = enum_
            .variants
            .iter()
            .all(|variant| matches!(variant.fields, syn::Fields::Unit));

        Ok(if has_only_unit_variants {
            let simple_enum = PyClassSimpleEnum::new(enum_)?;
            Self::Simple(simple_enum)
        } else {
            let complex_enum = PyClassComplexEnum::new(enum_)?;
            Self::Complex(complex_enum)
        })
    }
}

pub fn build_py_enum(
    enum_: &mut syn::ItemEnum,
    mut args: PyClassArgs,
    method_type: PyClassMethodsType,
) -> syn::Result<TokenStream> {
    args.options.take_pyo3_options(&mut enum_.attrs)?;

    if let Some(extends) = &args.options.extends {
        bail_spanned!(extends.span() => "enums can't extend from other classes");
    } else if let Some(subclass) = &args.options.subclass {
        bail_spanned!(subclass.span() => "enums can't be inherited by other classes");
    } else if enum_.variants.is_empty() {
        bail_spanned!(enum_.brace_token.span.join() => "#[pyclass] can't be used on enums without any variants");
    }

    let doc = utils::get_doc(&enum_.attrs, None);
    let enum_ = PyClassEnum::new(enum_)?;
    impl_enum(enum_, &args, doc, method_type)
}

struct PyClassSimpleEnum<'a> {
    ident: &'a syn::Ident,
    // The underlying #[repr] of the enum, used to implement __int__ and __richcmp__.
    // This matters when the underlying representation may not fit in `isize`.
    repr_type: syn::Ident,
    variants: Vec<PyClassEnumUnitVariant<'a>>,
}

impl<'a> PyClassSimpleEnum<'a> {
    fn new(enum_: &'a mut syn::ItemEnum) -> syn::Result<Self> {
        fn is_numeric_type(t: &syn::Ident) -> bool {
            [
                "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "u128", "i128", "usize",
                "isize",
            ]
            .iter()
            .any(|&s| t == s)
        }

        fn extract_unit_variant_data(
            variant: &mut syn::Variant,
        ) -> syn::Result<PyClassEnumUnitVariant<'_>> {
            use syn::Fields;
            let ident = match &variant.fields {
                Fields::Unit => &variant.ident,
                _ => bail_spanned!(variant.span() => "Must be a unit variant."),
            };
            let options = EnumVariantPyO3Options::take_pyo3_options(&mut variant.attrs)?;
            Ok(PyClassEnumUnitVariant { ident, options })
        }

        let ident = &enum_.ident;

        // According to the [reference](https://doc.rust-lang.org/reference/items/enumerations.html),
        // "Under the default representation, the specified discriminant is interpreted as an isize
        // value", so `isize` should be enough by default.
        let mut repr_type = syn::Ident::new("isize", proc_macro2::Span::call_site());
        if let Some(attr) = enum_.attrs.iter().find(|attr| attr.path().is_ident("repr")) {
            let args =
                attr.parse_args_with(Punctuated::<TokenStream, Token![!]>::parse_terminated)?;
            if let Some(ident) = args
                .into_iter()
                .filter_map(|ts| syn::parse2::<syn::Ident>(ts).ok())
                .find(is_numeric_type)
            {
                repr_type = ident;
            }
        }

        let variants: Vec<_> = enum_
            .variants
            .iter_mut()
            .map(extract_unit_variant_data)
            .collect::<syn::Result<_>>()?;
        Ok(Self {
            ident,
            repr_type,
            variants,
        })
    }
}

struct PyClassComplexEnum<'a> {
    ident: &'a syn::Ident,
    variants: Vec<PyClassEnumVariant<'a>>,
}

impl<'a> PyClassComplexEnum<'a> {
    fn new(enum_: &'a mut syn::ItemEnum) -> syn::Result<Self> {
        let witness = enum_
            .variants
            .iter()
            .find(|variant| !matches!(variant.fields, syn::Fields::Unit))
            .expect("complex enum has a non-unit variant")
            .ident
            .to_owned();

        let extract_variant_data =
            |variant: &'a mut syn::Variant| -> syn::Result<PyClassEnumVariant<'a>> {
                use syn::Fields;
                let ident = &variant.ident;
                let options = EnumVariantPyO3Options::take_pyo3_options(&mut variant.attrs)?;

                let variant = match &variant.fields {
                    Fields::Unit => {
                        bail_spanned!(variant.span() => format!(
                        "Unit variant `{ident}` is not yet supported in a complex enum\n\
                        = help: change to a struct variant with no fields: `{ident} {{ }}`\n\
                        = note: the enum is complex because of non-unit variant `{witness}`",
                        ident=ident, witness=witness))
                    }
                    Fields::Named(fields) => {
                        let fields = fields
                            .named
                            .iter()
                            .map(|field| PyClassEnumVariantNamedField {
                                ident: field.ident.as_ref().expect("named field has an identifier"),
                                ty: &field.ty,
                                span: field.span(),
                            })
                            .collect();

                        PyClassEnumVariant::Struct(PyClassEnumStructVariant {
                            ident,
                            fields,
                            options,
                        })
                    }
                    Fields::Unnamed(_) => {
                        bail_spanned!(variant.span() => format!(
                        "Tuple variant `{ident}` is not yet supported in a complex enum\n\
                        = help: change to a struct variant with named fields: `{ident} {{ /* fields */ }}`\n\
                        = note: the enum is complex because of non-unit variant `{witness}`",
                        ident=ident, witness=witness))
                    }
                };

                Ok(variant)
            };

        let ident = &enum_.ident;

        let variants: Vec<_> = enum_
            .variants
            .iter_mut()
            .map(extract_variant_data)
            .collect::<syn::Result<_>>()?;

        Ok(Self { ident, variants })
    }
}

enum PyClassEnumVariant<'a> {
    // TODO(mkovaxx): Unit(PyClassEnumUnitVariant<'a>),
    Struct(PyClassEnumStructVariant<'a>),
    // TODO(mkovaxx): Tuple(PyClassEnumTupleVariant<'a>),
}

trait EnumVariant {
    fn get_ident(&self) -> &syn::Ident;
    fn get_options(&self) -> &EnumVariantPyO3Options;

    fn get_python_name(&self, args: &PyClassArgs) -> Cow<'_, syn::Ident> {
        self.get_options()
            .name
            .as_ref()
            .map(|name_attr| Cow::Borrowed(&name_attr.value.0))
            .unwrap_or_else(|| {
                let name = self.get_ident().unraw();
                if let Some(attr) = &args.options.rename_all {
                    let new_name = apply_renaming_rule(attr.value.rule, &name.to_string());
                    Cow::Owned(Ident::new(&new_name, Span::call_site()))
                } else {
                    Cow::Owned(name)
                }
            })
    }
}

impl<'a> EnumVariant for PyClassEnumVariant<'a> {
    fn get_ident(&self) -> &syn::Ident {
        match self {
            PyClassEnumVariant::Struct(struct_variant) => struct_variant.ident,
        }
    }

    fn get_options(&self) -> &EnumVariantPyO3Options {
        match self {
            PyClassEnumVariant::Struct(struct_variant) => &struct_variant.options,
        }
    }
}

/// A unit variant has no fields
struct PyClassEnumUnitVariant<'a> {
    ident: &'a syn::Ident,
    options: EnumVariantPyO3Options,
}

impl<'a> EnumVariant for PyClassEnumUnitVariant<'a> {
    fn get_ident(&self) -> &syn::Ident {
        self.ident
    }

    fn get_options(&self) -> &EnumVariantPyO3Options {
        &self.options
    }
}

/// A struct variant has named fields
struct PyClassEnumStructVariant<'a> {
    ident: &'a syn::Ident,
    fields: Vec<PyClassEnumVariantNamedField<'a>>,
    options: EnumVariantPyO3Options,
}

struct PyClassEnumVariantNamedField<'a> {
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
    span: Span,
}

/// `#[pyo3()]` options for pyclass enum variants
struct EnumVariantPyO3Options {
    name: Option<NameAttribute>,
}

enum EnumVariantPyO3Option {
    Name(NameAttribute),
}

impl Parse for EnumVariantPyO3Option {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::name) {
            input.parse().map(EnumVariantPyO3Option::Name)
        } else {
            Err(lookahead.error())
        }
    }
}

impl EnumVariantPyO3Options {
    fn take_pyo3_options(attrs: &mut Vec<syn::Attribute>) -> Result<Self> {
        let mut options = EnumVariantPyO3Options { name: None };

        for option in take_pyo3_options(attrs)? {
            match option {
                EnumVariantPyO3Option::Name(name) => {
                    ensure_spanned!(
                        options.name.is_none(),
                        name.span() => "`name` may only be specified once"
                    );
                    options.name = Some(name);
                }
            }
        }

        Ok(options)
    }
}

fn impl_enum(
    enum_: PyClassEnum<'_>,
    args: &PyClassArgs,
    doc: PythonDoc,
    methods_type: PyClassMethodsType,
) -> Result<TokenStream> {
    match enum_ {
        PyClassEnum::Simple(simple_enum) => impl_simple_enum(simple_enum, args, doc, methods_type),
        PyClassEnum::Complex(complex_enum) => {
            impl_complex_enum(complex_enum, args, doc, methods_type)
        }
    }
}

fn impl_simple_enum(
    simple_enum: PyClassSimpleEnum<'_>,
    args: &PyClassArgs,
    doc: PythonDoc,
    methods_type: PyClassMethodsType,
) -> Result<TokenStream> {
    let krate = get_pyo3_crate(&args.options.krate);
    let cls = simple_enum.ident;
    let ty: syn::Type = syn::parse_quote!(#cls);
    let variants = simple_enum.variants;
    let pytypeinfo = impl_pytypeinfo(cls, args, None);

    let (default_repr, default_repr_slot) = {
        let variants_repr = variants.iter().map(|variant| {
            let variant_name = variant.ident;
            // Assuming all variants are unit variants because they are the only type we support.
            let repr = format!(
                "{}.{}",
                get_class_python_name(cls, args),
                variant.get_python_name(args),
            );
            quote! { #cls::#variant_name => #repr, }
        });
        let mut repr_impl: syn::ImplItemFn = syn::parse_quote! {
            fn __pyo3__repr__(&self) -> &'static str {
                match self {
                    #(#variants_repr)*
                }
            }
        };
        let repr_slot = generate_default_protocol_slot(&ty, &mut repr_impl, &__REPR__).unwrap();
        (repr_impl, repr_slot)
    };

    let repr_type = &simple_enum.repr_type;

    let (default_int, default_int_slot) = {
        // This implementation allows us to convert &T to #repr_type without implementing `Copy`
        let variants_to_int = variants.iter().map(|variant| {
            let variant_name = variant.ident;
            quote! { #cls::#variant_name => #cls::#variant_name as #repr_type, }
        });
        let mut int_impl: syn::ImplItemFn = syn::parse_quote! {
            fn __pyo3__int__(&self) -> #repr_type {
                match self {
                    #(#variants_to_int)*
                }
            }
        };
        let int_slot = generate_default_protocol_slot(&ty, &mut int_impl, &__INT__).unwrap();
        (int_impl, int_slot)
    };

    let (default_richcmp, default_richcmp_slot) = {
        let mut richcmp_impl: syn::ImplItemFn = syn::parse_quote! {
            fn __pyo3__richcmp__(
                &self,
                py: _pyo3::Python,
                other: &_pyo3::PyAny,
                op: _pyo3::basic::CompareOp
            ) -> _pyo3::PyResult<_pyo3::PyObject> {
                use _pyo3::conversion::ToPyObject;
                use ::core::result::Result::*;
                match op {
                    _pyo3::basic::CompareOp::Eq => {
                        let self_val = self.__pyo3__int__();
                        if let Ok(i) = other.extract::<#repr_type>() {
                            return Ok((self_val == i).to_object(py));
                        }
                        if let Ok(other) = other.extract::<_pyo3::PyRef<Self>>() {
                            return Ok((self_val == other.__pyo3__int__()).to_object(py));
                        }

                        return Ok(py.NotImplemented());
                    }
                    _pyo3::basic::CompareOp::Ne => {
                        let self_val = self.__pyo3__int__();
                        if let Ok(i) = other.extract::<#repr_type>() {
                            return Ok((self_val != i).to_object(py));
                        }
                        if let Ok(other) = other.extract::<_pyo3::PyRef<Self>>() {
                            return Ok((self_val != other.__pyo3__int__()).to_object(py));
                        }

                        return Ok(py.NotImplemented());
                    }
                    _ => Ok(py.NotImplemented()),
                }
            }
        };
        let richcmp_slot =
            generate_default_protocol_slot(&ty, &mut richcmp_impl, &__RICHCMP__).unwrap();
        (richcmp_impl, richcmp_slot)
    };

    let default_slots = vec![default_repr_slot, default_int_slot, default_richcmp_slot];

    let pyclass_impls = PyClassImplsBuilder::new(
        cls,
        args,
        methods_type,
        simple_enum_default_methods(
            cls,
            variants.iter().map(|v| (v.ident, v.get_python_name(args))),
        ),
        default_slots,
    )
    .doc(doc)
    .impl_all()?;

    Ok(quote! {
        // FIXME https://github.com/PyO3/pyo3/issues/3903
        #[allow(unknown_lints, non_local_definitions)]
        const _: () = {
            use #krate as _pyo3;

            #pytypeinfo

            #pyclass_impls

            #[doc(hidden)]
            #[allow(non_snake_case)]
            impl #cls {
                #default_repr
                #default_int
                #default_richcmp
            }
        };
    })
}

fn impl_complex_enum(
    complex_enum: PyClassComplexEnum<'_>,
    args: &PyClassArgs,
    doc: PythonDoc,
    methods_type: PyClassMethodsType,
) -> Result<TokenStream> {
    // Need to rig the enum PyClass options
    let args = {
        let mut rigged_args = args.clone();
        // Needs to be frozen to disallow `&mut self` methods, which could break a runtime invariant
        rigged_args.options.frozen = parse_quote!(frozen);
        // Needs to be subclassable by the variant PyClasses
        rigged_args.options.subclass = parse_quote!(subclass);
        rigged_args
    };

    let krate = get_pyo3_crate(&args.options.krate);
    let cls = complex_enum.ident;
    let variants = complex_enum.variants;
    let pytypeinfo = impl_pytypeinfo(cls, &args, None);

    let default_slots = vec![];

    let impl_builder = PyClassImplsBuilder::new(
        cls,
        &args,
        methods_type,
        complex_enum_default_methods(
            cls,
            variants
                .iter()
                .map(|v| (v.get_ident(), v.get_python_name(&args))),
        ),
        default_slots,
    )
    .doc(doc);

    // Need to customize the into_py impl so that it returns the variant PyClass
    let enum_into_py_impl = {
        let match_arms: Vec<TokenStream> = variants
            .iter()
            .map(|variant| {
                let variant_ident = variant.get_ident();
                let variant_cls = gen_complex_enum_variant_class_ident(cls, variant.get_ident());
                quote! {
                    #cls::#variant_ident { .. } => {
                        let pyclass_init = _pyo3::PyClassInitializer::from(self).add_subclass(#variant_cls);
                        let variant_value = _pyo3::Py::new(py, pyclass_init).unwrap();
                        _pyo3::IntoPy::into_py(variant_value, py)
                    }
                }
            })
            .collect();

        quote! {
            impl _pyo3::IntoPy<_pyo3::PyObject> for #cls {
                fn into_py(self, py: _pyo3::Python) -> _pyo3::PyObject {
                    match self {
                        #(#match_arms)*
                    }
                }
            }
        }
    };

    let pyclass_impls: TokenStream = vec![
        impl_builder.impl_pyclass(),
        impl_builder.impl_extractext(),
        enum_into_py_impl,
        impl_builder.impl_pyclassimpl()?,
        impl_builder.impl_freelist(),
    ]
    .into_iter()
    .collect();

    let mut variant_cls_zsts = vec![];
    let mut variant_cls_pytypeinfos = vec![];
    let mut variant_cls_pyclass_impls = vec![];
    let mut variant_cls_impls = vec![];
    for variant in &variants {
        let variant_cls = gen_complex_enum_variant_class_ident(cls, variant.get_ident());

        let variant_cls_zst = quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            struct #variant_cls;
        };
        variant_cls_zsts.push(variant_cls_zst);

        let variant_args = PyClassArgs {
            class_kind: PyClassKind::Struct,
            // TODO(mkovaxx): propagate variant.options
            options: parse_quote!(extends = #cls, frozen),
        };

        let variant_cls_pytypeinfo = impl_pytypeinfo(&variant_cls, &variant_args, None);
        variant_cls_pytypeinfos.push(variant_cls_pytypeinfo);

        let variant_new = complex_enum_variant_new(cls, variant)?;

        let (variant_cls_impl, field_getters) = impl_complex_enum_variant_cls(cls, variant)?;
        variant_cls_impls.push(variant_cls_impl);

        let pyclass_impl = PyClassImplsBuilder::new(
            &variant_cls,
            &variant_args,
            methods_type,
            field_getters,
            vec![variant_new],
        )
        .impl_all()?;

        variant_cls_pyclass_impls.push(pyclass_impl);
    }

    Ok(quote! {
        // FIXME https://github.com/PyO3/pyo3/issues/3903
        #[allow(unknown_lints, non_local_definitions)]
        const _: () = {
            use #krate as _pyo3;

            #pytypeinfo

            #pyclass_impls

            #[doc(hidden)]
            #[allow(non_snake_case)]
            impl #cls {}

            #(#variant_cls_zsts)*

            #(#variant_cls_pytypeinfos)*

            #(#variant_cls_pyclass_impls)*

            #(#variant_cls_impls)*
        };
    })
}

fn impl_complex_enum_variant_cls(
    enum_name: &syn::Ident,
    variant: &PyClassEnumVariant<'_>,
) -> Result<(TokenStream, Vec<MethodAndMethodDef>)> {
    match variant {
        PyClassEnumVariant::Struct(struct_variant) => {
            impl_complex_enum_struct_variant_cls(enum_name, struct_variant)
        }
    }
}

fn impl_complex_enum_struct_variant_cls(
    enum_name: &syn::Ident,
    variant: &PyClassEnumStructVariant<'_>,
) -> Result<(TokenStream, Vec<MethodAndMethodDef>)> {
    let variant_ident = &variant.ident;
    let variant_cls = gen_complex_enum_variant_class_ident(enum_name, variant.ident);
    let variant_cls_type = parse_quote!(#variant_cls);

    let mut field_names: Vec<Ident> = vec![];
    let mut fields_with_types: Vec<TokenStream> = vec![];
    let mut field_getters = vec![];
    let mut field_getter_impls: Vec<TokenStream> = vec![];
    for field in &variant.fields {
        let field_name = field.ident;
        let field_type = field.ty;
        let field_with_type = quote! { #field_name: #field_type };

        let field_getter = complex_enum_variant_field_getter(
            &variant_cls_type,
            field_name,
            field_type,
            field.span,
        )?;

        let field_getter_impl = quote! {
            fn #field_name(slf: _pyo3::PyRef<Self>) -> _pyo3::PyResult<#field_type> {
                match &*slf.into_super() {
                    #enum_name::#variant_ident { #field_name, .. } => Ok(#field_name.clone()),
                    _ => unreachable!("Wrong complex enum variant found in variant wrapper PyClass"),
                }
            }
        };

        field_names.push(field_name.clone());
        fields_with_types.push(field_with_type);
        field_getters.push(field_getter);
        field_getter_impls.push(field_getter_impl);
    }

    let cls_impl = quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl #variant_cls {
            fn __pymethod_constructor__(py: _pyo3::Python<'_>, #(#fields_with_types,)*) -> _pyo3::PyClassInitializer<#variant_cls> {
                let base_value = #enum_name::#variant_ident { #(#field_names,)* };
                _pyo3::PyClassInitializer::from(base_value).add_subclass(#variant_cls)
            }

            #(#field_getter_impls)*
        }
    };

    Ok((cls_impl, field_getters))
}

fn gen_complex_enum_variant_class_ident(enum_: &syn::Ident, variant: &syn::Ident) -> syn::Ident {
    format_ident!("{}_{}", enum_, variant)
}

fn generate_default_protocol_slot(
    cls: &syn::Type,
    method: &mut syn::ImplItemFn,
    slot: &SlotDef,
) -> syn::Result<MethodAndSlotDef> {
    let spec = FnSpec::parse(
        &mut method.sig,
        &mut Vec::new(),
        PyFunctionOptions::default(),
    )
    .unwrap();
    let name = spec.name.to_string();
    slot.generate_type_slot(
        &syn::parse_quote!(#cls),
        &spec,
        &format!("__default_{}__", name),
    )
}

fn simple_enum_default_methods<'a>(
    cls: &'a syn::Ident,
    unit_variant_names: impl IntoIterator<Item = (&'a syn::Ident, Cow<'a, syn::Ident>)>,
) -> Vec<MethodAndMethodDef> {
    let cls_type = syn::parse_quote!(#cls);
    let variant_to_attribute = |var_ident: &syn::Ident, py_ident: &syn::Ident| ConstSpec {
        rust_ident: var_ident.clone(),
        attributes: ConstAttributes {
            is_class_attr: true,
            name: Some(NameAttribute {
                kw: syn::parse_quote! { name },
                value: NameLitStr(py_ident.clone()),
            }),
            deprecations: Default::default(),
        },
    };
    unit_variant_names
        .into_iter()
        .map(|(var, py_name)| gen_py_const(&cls_type, &variant_to_attribute(var, &py_name)))
        .collect()
}

fn complex_enum_default_methods<'a>(
    cls: &'a syn::Ident,
    variant_names: impl IntoIterator<Item = (&'a syn::Ident, Cow<'a, syn::Ident>)>,
) -> Vec<MethodAndMethodDef> {
    let cls_type = syn::parse_quote!(#cls);
    let variant_to_attribute = |var_ident: &syn::Ident, py_ident: &syn::Ident| ConstSpec {
        rust_ident: var_ident.clone(),
        attributes: ConstAttributes {
            is_class_attr: true,
            name: Some(NameAttribute {
                kw: syn::parse_quote! { name },
                value: NameLitStr(py_ident.clone()),
            }),
            deprecations: Default::default(),
        },
    };
    variant_names
        .into_iter()
        .map(|(var, py_name)| {
            gen_complex_enum_variant_attr(cls, &cls_type, &variant_to_attribute(var, &py_name))
        })
        .collect()
}

pub fn gen_complex_enum_variant_attr(
    cls: &syn::Ident,
    cls_type: &syn::Type,
    spec: &ConstSpec,
) -> MethodAndMethodDef {
    let member = &spec.rust_ident;
    let wrapper_ident = format_ident!("__pymethod_variant_cls_{}__", member);
    let deprecations = &spec.attributes.deprecations;
    let python_name = &spec.null_terminated_python_name();

    let variant_cls = format_ident!("{}_{}", cls, member);
    let associated_method = quote! {
        fn #wrapper_ident(py: _pyo3::Python<'_>) -> _pyo3::PyResult<_pyo3::PyObject> {
            #deprecations
            ::std::result::Result::Ok(py.get_type_bound::<#variant_cls>().into_any().unbind())
        }
    };

    let method_def = quote! {
        _pyo3::class::PyMethodDefType::ClassAttribute({
            _pyo3::class::PyClassAttributeDef::new(
                #python_name,
                _pyo3::impl_::pymethods::PyClassAttributeFactory(#cls_type::#wrapper_ident)
            )
        })
    };

    MethodAndMethodDef {
        associated_method,
        method_def,
    }
}

fn complex_enum_variant_new<'a>(
    cls: &'a syn::Ident,
    variant: &'a PyClassEnumVariant<'a>,
) -> Result<MethodAndSlotDef> {
    match variant {
        PyClassEnumVariant::Struct(struct_variant) => {
            complex_enum_struct_variant_new(cls, struct_variant)
        }
    }
}

fn complex_enum_struct_variant_new<'a>(
    cls: &'a syn::Ident,
    variant: &'a PyClassEnumStructVariant<'a>,
) -> Result<MethodAndSlotDef> {
    let variant_cls = format_ident!("{}_{}", cls, variant.ident);
    let variant_cls_type: syn::Type = parse_quote!(#variant_cls);

    let arg_py_ident: syn::Ident = parse_quote!(py);
    let arg_py_type: syn::Type = parse_quote!(_pyo3::Python<'_>);

    let args = {
        let mut no_pyo3_attrs = vec![];
        let attrs = crate::pyfunction::PyFunctionArgPyO3Attributes::from_attrs(&mut no_pyo3_attrs)?;

        let mut args = vec![
            // py: Python<'_>
            FnArg {
                name: &arg_py_ident,
                ty: &arg_py_type,
                optional: None,
                default: None,
                py: true,
                attrs: attrs.clone(),
                is_varargs: false,
                is_kwargs: false,
                is_cancel_handle: false,
            },
        ];

        for field in &variant.fields {
            args.push(FnArg {
                name: field.ident,
                ty: field.ty,
                optional: None,
                default: None,
                py: false,
                attrs: attrs.clone(),
                is_varargs: false,
                is_kwargs: false,
                is_cancel_handle: false,
            });
        }
        args
    };
    let signature = crate::pyfunction::FunctionSignature::from_arguments(args)?;

    let spec = FnSpec {
        tp: crate::method::FnType::FnNew,
        name: &format_ident!("__pymethod_constructor__"),
        python_name: format_ident!("__new__"),
        signature,
        output: variant_cls_type.clone(),
        convention: crate::method::CallingConvention::TpNew,
        text_signature: None,
        asyncness: None,
        unsafety: None,
        deprecations: Deprecations::default(),
    };

    crate::pymethod::impl_py_method_def_new(&variant_cls_type, &spec)
}

fn complex_enum_variant_field_getter<'a>(
    variant_cls_type: &'a syn::Type,
    field_name: &'a syn::Ident,
    field_type: &'a syn::Type,
    field_span: Span,
) -> Result<MethodAndMethodDef> {
    let signature = crate::pyfunction::FunctionSignature::from_arguments(vec![])?;

    let self_type = crate::method::SelfType::TryFromBoundRef(field_span);

    let spec = FnSpec {
        tp: crate::method::FnType::Getter(self_type.clone()),
        name: field_name,
        python_name: field_name.clone(),
        signature,
        output: field_type.clone(),
        convention: crate::method::CallingConvention::Noargs,
        text_signature: None,
        asyncness: None,
        unsafety: None,
        deprecations: Deprecations::default(),
    };

    let property_type = crate::pymethod::PropertyType::Function {
        self_type: &self_type,
        spec: &spec,
        doc: crate::get_doc(&[], None),
    };

    let getter = crate::pymethod::impl_py_getter_def(variant_cls_type, property_type)?;
    Ok(getter)
}

fn descriptors_to_items(
    cls: &syn::Ident,
    rename_all: Option<&RenameAllAttribute>,
    frozen: Option<frozen>,
    field_options: Vec<(&syn::Field, FieldPyO3Options)>,
) -> syn::Result<Vec<MethodAndMethodDef>> {
    let ty = syn::parse_quote!(#cls);
    let mut items = Vec::new();
    for (field_index, (field, options)) in field_options.into_iter().enumerate() {
        if let FieldPyO3Options {
            name: Some(name),
            get: None,
            set: None,
        } = options
        {
            return Err(syn::Error::new_spanned(name, USELESS_NAME));
        }

        if options.get.is_some() {
            let getter = impl_py_getter_def(
                &ty,
                PropertyType::Descriptor {
                    field_index,
                    field,
                    python_name: options.name.as_ref(),
                    renaming_rule: rename_all.map(|rename_all| rename_all.value.rule),
                },
            )?;
            items.push(getter);
        }

        if let Some(set) = options.set {
            ensure_spanned!(frozen.is_none(), set.span() => "cannot use `#[pyo3(set)]` on a `frozen` class");
            let setter = impl_py_setter_def(
                &ty,
                PropertyType::Descriptor {
                    field_index,
                    field,
                    python_name: options.name.as_ref(),
                    renaming_rule: rename_all.map(|rename_all| rename_all.value.rule),
                },
            )?;
            items.push(setter);
        };
    }
    Ok(items)
}

fn impl_pytypeinfo(
    cls: &syn::Ident,
    attr: &PyClassArgs,
    deprecations: Option<&Deprecations>,
) -> TokenStream {
    let cls_name = get_class_python_name(cls, attr).to_string();

    let module = if let Some(ModuleAttribute { value, .. }) = &attr.options.module {
        quote! { ::core::option::Option::Some(#value) }
    } else {
        quote! { ::core::option::Option::None }
    };

    quote! {
        unsafe impl _pyo3::type_object::HasPyGilRef for #cls {
            type AsRefTarget = _pyo3::PyCell<Self>;
        }

        unsafe impl _pyo3::type_object::PyTypeInfo for #cls {
            const NAME: &'static str = #cls_name;
            const MODULE: ::std::option::Option<&'static str> = #module;

            #[inline]
            fn type_object_raw(py: _pyo3::Python<'_>) -> *mut _pyo3::ffi::PyTypeObject {
                use _pyo3::prelude::PyTypeMethods;
                #deprecations

                <#cls as _pyo3::impl_::pyclass::PyClassImpl>::lazy_type_object()
                    .get_or_init(py)
                    .as_type_ptr()
            }
        }
    }
}

/// Implements most traits used by `#[pyclass]`.
///
/// Specifically, it implements traits that only depend on class name,
/// and attributes of `#[pyclass]`, and docstrings.
/// Therefore it doesn't implement traits that depends on struct fields and enum variants.
struct PyClassImplsBuilder<'a> {
    cls: &'a syn::Ident,
    attr: &'a PyClassArgs,
    methods_type: PyClassMethodsType,
    default_methods: Vec<MethodAndMethodDef>,
    default_slots: Vec<MethodAndSlotDef>,
    doc: Option<PythonDoc>,
}

impl<'a> PyClassImplsBuilder<'a> {
    fn new(
        cls: &'a syn::Ident,
        attr: &'a PyClassArgs,
        methods_type: PyClassMethodsType,
        default_methods: Vec<MethodAndMethodDef>,
        default_slots: Vec<MethodAndSlotDef>,
    ) -> Self {
        Self {
            cls,
            attr,
            methods_type,
            default_methods,
            default_slots,
            doc: None,
        }
    }

    fn doc(self, doc: PythonDoc) -> Self {
        Self {
            doc: Some(doc),
            ..self
        }
    }

    fn impl_all(&self) -> Result<TokenStream> {
        let tokens = vec![
            self.impl_pyclass(),
            self.impl_extractext(),
            self.impl_into_py(),
            self.impl_pyclassimpl()?,
            self.impl_freelist(),
        ]
        .into_iter()
        .collect();
        Ok(tokens)
    }

    fn impl_pyclass(&self) -> TokenStream {
        let cls = self.cls;

        let frozen = if self.attr.options.frozen.is_some() {
            quote! { _pyo3::pyclass::boolean_struct::True }
        } else {
            quote! { _pyo3::pyclass::boolean_struct::False }
        };

        quote! {
            impl _pyo3::PyClass for #cls {
                type Frozen = #frozen;
            }
        }
    }
    fn impl_extractext(&self) -> TokenStream {
        let cls = self.cls;
        if self.attr.options.frozen.is_some() {
            quote! {
                impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py> for &'a #cls
                {
                    type Holder = ::std::option::Option<_pyo3::PyRef<'py, #cls>>;

                    #[inline]
                    fn extract(obj: &'py _pyo3::PyAny, holder: &'a mut Self::Holder) -> _pyo3::PyResult<Self> {
                        _pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
                    }
                }
            }
        } else {
            quote! {
                impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py> for &'a #cls
                {
                    type Holder = ::std::option::Option<_pyo3::PyRef<'py, #cls>>;

                    #[inline]
                    fn extract(obj: &'py _pyo3::PyAny, holder: &'a mut Self::Holder) -> _pyo3::PyResult<Self> {
                        _pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
                    }
                }

                impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py> for &'a mut #cls
                {
                    type Holder = ::std::option::Option<_pyo3::PyRefMut<'py, #cls>>;

                    #[inline]
                    fn extract(obj: &'py _pyo3::PyAny, holder: &'a mut Self::Holder) -> _pyo3::PyResult<Self> {
                        _pyo3::impl_::extract_argument::extract_pyclass_ref_mut(obj, holder)
                    }
                }
            }
        }
    }

    fn impl_into_py(&self) -> TokenStream {
        let cls = self.cls;
        let attr = self.attr;
        // If #cls is not extended type, we allow Self->PyObject conversion
        if attr.options.extends.is_none() {
            quote! {
                impl _pyo3::IntoPy<_pyo3::PyObject> for #cls {
                    fn into_py(self, py: _pyo3::Python) -> _pyo3::PyObject {
                        _pyo3::IntoPy::into_py(_pyo3::Py::new(py, self).unwrap(), py)
                    }
                }
            }
        } else {
            quote! {}
        }
    }
    fn impl_pyclassimpl(&self) -> Result<TokenStream> {
        let cls = self.cls;
        let doc = self.doc.as_ref().map_or(quote! {"\0"}, |doc| quote! {#doc});
        let is_basetype = self.attr.options.subclass.is_some();
        let base = match &self.attr.options.extends {
            Some(extends_attr) => extends_attr.value.clone(),
            None => parse_quote! { _pyo3::PyAny },
        };
        let is_subclass = self.attr.options.extends.is_some();
        let is_mapping: bool = self.attr.options.mapping.is_some();
        let is_sequence: bool = self.attr.options.sequence.is_some();

        ensure_spanned!(
            !(is_mapping && is_sequence),
            self.cls.span() => "a `#[pyclass]` cannot be both a `mapping` and a `sequence`"
        );

        let dict_offset = if self.attr.options.dict.is_some() {
            quote! {
                fn dict_offset() -> ::std::option::Option<_pyo3::ffi::Py_ssize_t> {
                    ::std::option::Option::Some(_pyo3::impl_::pyclass::dict_offset::<Self>())
                }
            }
        } else {
            TokenStream::new()
        };

        // insert space for weak ref
        let weaklist_offset = if self.attr.options.weakref.is_some() {
            quote! {
                fn weaklist_offset() -> ::std::option::Option<_pyo3::ffi::Py_ssize_t> {
                    ::std::option::Option::Some(_pyo3::impl_::pyclass::weaklist_offset::<Self>())
                }
            }
        } else {
            TokenStream::new()
        };

        let thread_checker = if self.attr.options.unsendable.is_some() {
            quote! { _pyo3::impl_::pyclass::ThreadCheckerImpl }
        } else {
            quote! { _pyo3::impl_::pyclass::SendablePyClass<#cls> }
        };

        let (pymethods_items, inventory, inventory_class) = match self.methods_type {
            PyClassMethodsType::Specialization => (quote! { collector.py_methods() }, None, None),
            PyClassMethodsType::Inventory => {
                // To allow multiple #[pymethods] block, we define inventory types.
                let inventory_class_name = syn::Ident::new(
                    &format!("Pyo3MethodsInventoryFor{}", cls.unraw()),
                    Span::call_site(),
                );
                (
                    quote! {
                        ::std::boxed::Box::new(
                            ::std::iter::Iterator::map(
                                _pyo3::inventory::iter::<<Self as _pyo3::impl_::pyclass::PyClassImpl>::Inventory>(),
                                _pyo3::impl_::pyclass::PyClassInventory::items
                            )
                        )
                    },
                    Some(quote! { type Inventory = #inventory_class_name; }),
                    Some(define_inventory_class(&inventory_class_name)),
                )
            }
        };

        let default_methods = self
            .default_methods
            .iter()
            .map(|meth| &meth.associated_method)
            .chain(
                self.default_slots
                    .iter()
                    .map(|meth| &meth.associated_method),
            );

        let default_method_defs = self.default_methods.iter().map(|meth| &meth.method_def);
        let default_slot_defs = self.default_slots.iter().map(|slot| &slot.slot_def);
        let freelist_slots = self.freelist_slots();

        let class_mutability = if self.attr.options.frozen.is_some() {
            quote! {
                ImmutableChild
            }
        } else {
            quote! {
                MutableChild
            }
        };

        let cls = self.cls;
        let attr = self.attr;
        let dict = if attr.options.dict.is_some() {
            quote! { _pyo3::impl_::pyclass::PyClassDictSlot }
        } else {
            quote! { _pyo3::impl_::pyclass::PyClassDummySlot }
        };

        // insert space for weak ref
        let weakref = if attr.options.weakref.is_some() {
            quote! { _pyo3::impl_::pyclass::PyClassWeakRefSlot }
        } else {
            quote! { _pyo3::impl_::pyclass::PyClassDummySlot }
        };

        let base_nativetype = if attr.options.extends.is_some() {
            quote! { <Self::BaseType as _pyo3::impl_::pyclass::PyClassBaseType>::BaseNativeType }
        } else {
            quote! { _pyo3::PyAny }
        };

        Ok(quote! {
            impl _pyo3::impl_::pyclass::PyClassImpl for #cls {
                const IS_BASETYPE: bool = #is_basetype;
                const IS_SUBCLASS: bool = #is_subclass;
                const IS_MAPPING: bool = #is_mapping;
                const IS_SEQUENCE: bool = #is_sequence;

                type BaseType = #base;
                type ThreadChecker = #thread_checker;
                #inventory
                type PyClassMutability = <<#base as _pyo3::impl_::pyclass::PyClassBaseType>::PyClassMutability as _pyo3::impl_::pycell::PyClassMutability>::#class_mutability;
                type Dict = #dict;
                type WeakRef = #weakref;
                type BaseNativeType = #base_nativetype;

                fn items_iter() -> _pyo3::impl_::pyclass::PyClassItemsIter {
                    use _pyo3::impl_::pyclass::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
                        methods: &[#(#default_method_defs),*],
                        slots: &[#(#default_slot_defs),* #(#freelist_slots),*],
                    };
                    PyClassItemsIter::new(&INTRINSIC_ITEMS, #pymethods_items)
                }

                fn doc(py: _pyo3::Python<'_>) -> _pyo3::PyResult<&'static ::std::ffi::CStr>  {
                    use _pyo3::impl_::pyclass::*;
                    static DOC: _pyo3::sync::GILOnceCell<::std::borrow::Cow<'static, ::std::ffi::CStr>> = _pyo3::sync::GILOnceCell::new();
                    DOC.get_or_try_init(py, || {
                        let collector = PyClassImplCollector::<Self>::new();
                        build_pyclass_doc(<#cls as _pyo3::PyTypeInfo>::NAME, #doc, collector.new_text_signature())
                    }).map(::std::ops::Deref::deref)
                }

                #dict_offset

                #weaklist_offset

                fn lazy_type_object() -> &'static _pyo3::impl_::pyclass::LazyTypeObject<Self> {
                    use _pyo3::impl_::pyclass::LazyTypeObject;
                    static TYPE_OBJECT: LazyTypeObject<#cls> = LazyTypeObject::new();
                    &TYPE_OBJECT
                }
            }

            #[doc(hidden)]
            #[allow(non_snake_case)]
            impl #cls {
                #(#default_methods)*
            }

            #inventory_class
        })
    }

    fn impl_freelist(&self) -> TokenStream {
        let cls = self.cls;

        self.attr.options.freelist.as_ref().map_or(quote!{}, |freelist| {
            let freelist = &freelist.value;
            quote! {
                impl _pyo3::impl_::pyclass::PyClassWithFreeList for #cls {
                    #[inline]
                    fn get_free_list(py: _pyo3::Python<'_>) -> &mut _pyo3::impl_::freelist::FreeList<*mut _pyo3::ffi::PyObject> {
                        static mut FREELIST: *mut _pyo3::impl_::freelist::FreeList<*mut _pyo3::ffi::PyObject> = 0 as *mut _;
                        unsafe {
                            if FREELIST.is_null() {
                                FREELIST = ::std::boxed::Box::into_raw(::std::boxed::Box::new(
                                    _pyo3::impl_::freelist::FreeList::with_capacity(#freelist)));
                            }
                            &mut *FREELIST
                        }
                    }
                }
            }
        })
    }

    fn freelist_slots(&self) -> Vec<TokenStream> {
        let cls = self.cls;

        if self.attr.options.freelist.is_some() {
            vec![
                quote! {
                    _pyo3::ffi::PyType_Slot {
                        slot: _pyo3::ffi::Py_tp_alloc,
                        pfunc: _pyo3::impl_::pyclass::alloc_with_freelist::<#cls> as *mut _,
                    }
                },
                quote! {
                    _pyo3::ffi::PyType_Slot {
                        slot: _pyo3::ffi::Py_tp_free,
                        pfunc: _pyo3::impl_::pyclass::free_with_freelist::<#cls> as *mut _,
                    }
                },
            ]
        } else {
            Vec::new()
        }
    }
}

fn define_inventory_class(inventory_class_name: &syn::Ident) -> TokenStream {
    quote! {
        #[doc(hidden)]
        pub struct #inventory_class_name {
            items: _pyo3::impl_::pyclass::PyClassItems,
        }
        impl #inventory_class_name {
            pub const fn new(items: _pyo3::impl_::pyclass::PyClassItems) -> Self {
                Self { items }
            }
        }

        impl _pyo3::impl_::pyclass::PyClassInventory for #inventory_class_name {
            fn items(&self) -> &_pyo3::impl_::pyclass::PyClassItems {
                &self.items
            }
        }

        _pyo3::inventory::collect!(#inventory_class_name);
    }
}

const UNIQUE_GET: &str = "`get` may only be specified once";
const UNIQUE_SET: &str = "`set` may only be specified once";
const UNIQUE_NAME: &str = "`name` may only be specified once";

const DUPE_SET: &str = "useless `set` - the struct is already annotated with `set_all`";
const DUPE_GET: &str = "useless `get` - the struct is already annotated with `get_all`";
const UNIT_GET: &str =
    "`get_all` on an unit struct does nothing, because unit structs have no fields";
const UNIT_SET: &str =
    "`set_all` on an unit struct does nothing, because unit structs have no fields";

const USELESS_NAME: &str = "`name` is useless without `get` or `set`";
