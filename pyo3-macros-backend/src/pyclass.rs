use std::borrow::Cow;
use std::fmt::Debug;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_quote, parse_quote_spanned, spanned::Spanned, ImplItemFn, Result, Token};

use crate::attributes::kw::frozen;
use crate::attributes::{
    self, kw, take_pyo3_options, CrateAttribute, ExtendsAttribute, FreelistAttribute,
    ModuleAttribute, NameAttribute, NameLitStr, RenameAllAttribute, StrFormatterAttribute,
};
use crate::combine_errors::CombineErrors;
#[cfg(feature = "experimental-inspect")]
use crate::introspection::{
    class_introspection_code, function_introspection_code, introspection_id_const,
    unique_element_id,
};
use crate::konst::{ConstAttributes, ConstSpec};
use crate::method::{FnArg, FnSpec, PyArg, RegularArg};
use crate::pyfunction::ConstructorAttribute;
#[cfg(feature = "experimental-inspect")]
use crate::pyfunction::FunctionSignature;
use crate::pyimpl::{gen_py_const, get_cfg_attributes, PyClassMethodsType};
use crate::pymethod::{
    impl_py_class_attribute, impl_py_getter_def, impl_py_setter_def, MethodAndMethodDef,
    MethodAndSlotDef, PropertyType, SlotDef, __GETITEM__, __HASH__, __INT__, __LEN__, __REPR__,
    __RICHCMP__, __STR__,
};
use crate::pyversions::{is_abi3_before, is_py_before};
use crate::utils::{self, apply_renaming_rule, Ctx, LitCStr, PythonDoc};
use crate::PyFunctionOptions;

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

    pub fn parse_struct_args(input: ParseStream<'_>) -> syn::Result<Self> {
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
    pub eq: Option<kw::eq>,
    pub eq_int: Option<kw::eq_int>,
    pub extends: Option<ExtendsAttribute>,
    pub get_all: Option<kw::get_all>,
    pub freelist: Option<FreelistAttribute>,
    pub frozen: Option<kw::frozen>,
    pub hash: Option<kw::hash>,
    pub immutable_type: Option<kw::immutable_type>,
    pub mapping: Option<kw::mapping>,
    pub module: Option<ModuleAttribute>,
    pub name: Option<NameAttribute>,
    pub ord: Option<kw::ord>,
    pub rename_all: Option<RenameAllAttribute>,
    pub sequence: Option<kw::sequence>,
    pub set_all: Option<kw::set_all>,
    pub str: Option<StrFormatterAttribute>,
    pub subclass: Option<kw::subclass>,
    pub unsendable: Option<kw::unsendable>,
    pub weakref: Option<kw::weakref>,
    pub generic: Option<kw::generic>,
}

pub enum PyClassPyO3Option {
    Crate(CrateAttribute),
    Dict(kw::dict),
    Eq(kw::eq),
    EqInt(kw::eq_int),
    Extends(ExtendsAttribute),
    Freelist(FreelistAttribute),
    Frozen(kw::frozen),
    GetAll(kw::get_all),
    Hash(kw::hash),
    ImmutableType(kw::immutable_type),
    Mapping(kw::mapping),
    Module(ModuleAttribute),
    Name(NameAttribute),
    Ord(kw::ord),
    RenameAll(RenameAllAttribute),
    Sequence(kw::sequence),
    SetAll(kw::set_all),
    Str(StrFormatterAttribute),
    Subclass(kw::subclass),
    Unsendable(kw::unsendable),
    Weakref(kw::weakref),
    Generic(kw::generic),
}

impl Parse for PyClassPyO3Option {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![crate]) {
            input.parse().map(PyClassPyO3Option::Crate)
        } else if lookahead.peek(kw::dict) {
            input.parse().map(PyClassPyO3Option::Dict)
        } else if lookahead.peek(kw::eq) {
            input.parse().map(PyClassPyO3Option::Eq)
        } else if lookahead.peek(kw::eq_int) {
            input.parse().map(PyClassPyO3Option::EqInt)
        } else if lookahead.peek(kw::extends) {
            input.parse().map(PyClassPyO3Option::Extends)
        } else if lookahead.peek(attributes::kw::freelist) {
            input.parse().map(PyClassPyO3Option::Freelist)
        } else if lookahead.peek(attributes::kw::frozen) {
            input.parse().map(PyClassPyO3Option::Frozen)
        } else if lookahead.peek(attributes::kw::get_all) {
            input.parse().map(PyClassPyO3Option::GetAll)
        } else if lookahead.peek(attributes::kw::hash) {
            input.parse().map(PyClassPyO3Option::Hash)
        } else if lookahead.peek(attributes::kw::immutable_type) {
            input.parse().map(PyClassPyO3Option::ImmutableType)
        } else if lookahead.peek(attributes::kw::mapping) {
            input.parse().map(PyClassPyO3Option::Mapping)
        } else if lookahead.peek(attributes::kw::module) {
            input.parse().map(PyClassPyO3Option::Module)
        } else if lookahead.peek(kw::name) {
            input.parse().map(PyClassPyO3Option::Name)
        } else if lookahead.peek(attributes::kw::ord) {
            input.parse().map(PyClassPyO3Option::Ord)
        } else if lookahead.peek(kw::rename_all) {
            input.parse().map(PyClassPyO3Option::RenameAll)
        } else if lookahead.peek(attributes::kw::sequence) {
            input.parse().map(PyClassPyO3Option::Sequence)
        } else if lookahead.peek(attributes::kw::set_all) {
            input.parse().map(PyClassPyO3Option::SetAll)
        } else if lookahead.peek(attributes::kw::str) {
            input.parse().map(PyClassPyO3Option::Str)
        } else if lookahead.peek(attributes::kw::subclass) {
            input.parse().map(PyClassPyO3Option::Subclass)
        } else if lookahead.peek(attributes::kw::unsendable) {
            input.parse().map(PyClassPyO3Option::Unsendable)
        } else if lookahead.peek(attributes::kw::weakref) {
            input.parse().map(PyClassPyO3Option::Weakref)
        } else if lookahead.peek(attributes::kw::generic) {
            input.parse().map(PyClassPyO3Option::Generic)
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
            PyClassPyO3Option::Dict(dict) => {
                ensure_spanned!(
                    !is_abi3_before(3, 9),
                    dict.span() => "`dict` requires Python >= 3.9 when using the `abi3` feature"
                );
                set_option!(dict);
            }
            PyClassPyO3Option::Eq(eq) => set_option!(eq),
            PyClassPyO3Option::EqInt(eq_int) => set_option!(eq_int),
            PyClassPyO3Option::Extends(extends) => set_option!(extends),
            PyClassPyO3Option::Freelist(freelist) => set_option!(freelist),
            PyClassPyO3Option::Frozen(frozen) => set_option!(frozen),
            PyClassPyO3Option::GetAll(get_all) => set_option!(get_all),
            PyClassPyO3Option::ImmutableType(immutable_type) => {
                ensure_spanned!(
                    !(is_py_before(3, 10) || is_abi3_before(3, 14)),
                    immutable_type.span() => "`immutable_type` requires Python >= 3.10 or >= 3.14 (ABI3)"
                );
                set_option!(immutable_type)
            }
            PyClassPyO3Option::Hash(hash) => set_option!(hash),
            PyClassPyO3Option::Mapping(mapping) => set_option!(mapping),
            PyClassPyO3Option::Module(module) => set_option!(module),
            PyClassPyO3Option::Name(name) => set_option!(name),
            PyClassPyO3Option::Ord(ord) => set_option!(ord),
            PyClassPyO3Option::RenameAll(rename_all) => set_option!(rename_all),
            PyClassPyO3Option::Sequence(sequence) => set_option!(sequence),
            PyClassPyO3Option::SetAll(set_all) => set_option!(set_all),
            PyClassPyO3Option::Str(str) => set_option!(str),
            PyClassPyO3Option::Subclass(subclass) => set_option!(subclass),
            PyClassPyO3Option::Unsendable(unsendable) => set_option!(unsendable),
            PyClassPyO3Option::Weakref(weakref) => {
                ensure_spanned!(
                    !is_abi3_before(3, 9),
                    weakref.span() => "`weakref` requires Python >= 3.9 when using the `abi3` feature"
                );
                set_option!(weakref);
            }
            PyClassPyO3Option::Generic(generic) => set_option!(generic),
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

    let ctx = &Ctx::new(&args.options.krate, None);
    let doc = utils::get_doc(&class.attrs, None, ctx)?;

    if let Some(lt) = class.generics.lifetimes().next() {
        bail_spanned!(
            lt.span() => concat!(
                "#[pyclass] cannot have lifetime parameters. For an explanation, see \
                https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/class.html#no-lifetime-parameters"
            )
        );
    }

    ensure_spanned!(
        class.generics.params.is_empty(),
        class.generics.span() => concat!(
            "#[pyclass] cannot have generic parameters. For an explanation, see \
            https://pyo3.rs/v", env!("CARGO_PKG_VERSION"), "/class.html#no-generic-parameters"
        )
    );

    let mut field_options: Vec<(&syn::Field, FieldPyO3Options)> = match &mut class.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter_mut()
            .map(
                |field| match FieldPyO3Options::take_pyo3_options(&mut field.attrs) {
                    Ok(options) => Ok((&*field, options)),
                    Err(e) => Err(e),
                },
            )
            .collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter_mut()
            .map(
                |field| match FieldPyO3Options::take_pyo3_options(&mut field.attrs) {
                    Ok(options) => Ok((&*field, options)),
                    Err(e) => Err(e),
                },
            )
            .collect::<Vec<_>>(),
        syn::Fields::Unit => {
            let mut results = Vec::new();

            if let Some(attr) = args.options.set_all {
                results.push(Err(syn::Error::new_spanned(attr, UNIT_SET)));
            };
            if let Some(attr) = args.options.get_all {
                results.push(Err(syn::Error::new_spanned(attr, UNIT_GET)));
            };

            results
        }
    }
    .into_iter()
    .try_combine_syn_errors()?;

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

    impl_class(&class.ident, &args, doc, field_options, methods_type, ctx)
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

fn get_class_python_module_and_name<'a>(cls: &'a Ident, args: &'a PyClassArgs) -> String {
    let name = get_class_python_name(cls, args);
    if let Some(module) = &args.options.module {
        let value = module.value.value();
        format!("{value}.{name}")
    } else {
        name.to_string()
    }
}

fn impl_class(
    cls: &syn::Ident,
    args: &PyClassArgs,
    doc: PythonDoc,
    field_options: Vec<(&syn::Field, FieldPyO3Options)>,
    methods_type: PyClassMethodsType,
    ctx: &Ctx,
) -> syn::Result<TokenStream> {
    let Ctx { pyo3_path, .. } = ctx;
    let pytypeinfo_impl = impl_pytypeinfo(cls, args, ctx);

    if let Some(str) = &args.options.str {
        if str.value.is_some() {
            // check if any renaming is present
            let no_naming_conflict = field_options.iter().all(|x| x.1.name.is_none())
                & args.options.name.is_none()
                & args.options.rename_all.is_none();
            ensure_spanned!(no_naming_conflict, str.value.span() => "The format string syntax is incompatible with any renaming via `name` or `rename_all`");
        }
    }

    let mut default_methods = descriptors_to_items(
        cls,
        args.options.rename_all.as_ref(),
        args.options.frozen,
        field_options,
        ctx,
    )?;

    let (default_class_geitem, default_class_geitem_method) =
        pyclass_class_geitem(&args.options, &syn::parse_quote!(#cls), ctx)?;

    if let Some(default_class_geitem_method) = default_class_geitem_method {
        default_methods.push(default_class_geitem_method);
    }

    let (default_str, default_str_slot) =
        implement_pyclass_str(&args.options, &syn::parse_quote!(#cls), ctx);

    let (default_richcmp, default_richcmp_slot) =
        pyclass_richcmp(&args.options, &syn::parse_quote!(#cls), ctx)?;

    let (default_hash, default_hash_slot) =
        pyclass_hash(&args.options, &syn::parse_quote!(#cls), ctx)?;

    let mut slots = Vec::new();
    slots.extend(default_richcmp_slot);
    slots.extend(default_hash_slot);
    slots.extend(default_str_slot);

    let py_class_impl = PyClassImplsBuilder::new(cls, args, methods_type, default_methods, slots)
        .doc(doc)
        .impl_all(ctx)?;

    Ok(quote! {
        impl #pyo3_path::types::DerefToPyAny for #cls {}

        #pytypeinfo_impl

        #py_class_impl

        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl #cls {
            #default_richcmp
            #default_hash
            #default_str
            #default_class_geitem
        }
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

    let ctx = &Ctx::new(&args.options.krate, None);
    if let Some(extends) = &args.options.extends {
        bail_spanned!(extends.span() => "enums can't extend from other classes");
    } else if let Some(subclass) = &args.options.subclass {
        bail_spanned!(subclass.span() => "enums can't be inherited by other classes");
    } else if enum_.variants.is_empty() {
        bail_spanned!(enum_.brace_token.span.join() => "#[pyclass] can't be used on enums without any variants");
    }

    if let Some(generic) = &args.options.generic {
        bail_spanned!(generic.span() => "enums do not support #[pyclass(generic)]");
    }

    let doc = utils::get_doc(&enum_.attrs, None, ctx)?;
    let enum_ = PyClassEnum::new(enum_)?;
    impl_enum(enum_, &args, doc, method_type, ctx)
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
            let cfg_attrs = get_cfg_attributes(&variant.attrs);
            Ok(PyClassEnumUnitVariant {
                ident,
                options,
                cfg_attrs,
            })
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
                            = help: change to an empty tuple variant instead: `{ident}()`\n\
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
                    Fields::Unnamed(types) => {
                        let fields = types
                            .unnamed
                            .iter()
                            .map(|field| PyClassEnumVariantUnnamedField {
                                ty: &field.ty,
                                span: field.span(),
                            })
                            .collect();

                        PyClassEnumVariant::Tuple(PyClassEnumTupleVariant {
                            ident,
                            fields,
                            options,
                        })
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
    Tuple(PyClassEnumTupleVariant<'a>),
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

impl EnumVariant for PyClassEnumVariant<'_> {
    fn get_ident(&self) -> &syn::Ident {
        match self {
            PyClassEnumVariant::Struct(struct_variant) => struct_variant.ident,
            PyClassEnumVariant::Tuple(tuple_variant) => tuple_variant.ident,
        }
    }

    fn get_options(&self) -> &EnumVariantPyO3Options {
        match self {
            PyClassEnumVariant::Struct(struct_variant) => &struct_variant.options,
            PyClassEnumVariant::Tuple(tuple_variant) => &tuple_variant.options,
        }
    }
}

/// A unit variant has no fields
struct PyClassEnumUnitVariant<'a> {
    ident: &'a syn::Ident,
    options: EnumVariantPyO3Options,
    cfg_attrs: Vec<&'a syn::Attribute>,
}

impl EnumVariant for PyClassEnumUnitVariant<'_> {
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

struct PyClassEnumTupleVariant<'a> {
    ident: &'a syn::Ident,
    fields: Vec<PyClassEnumVariantUnnamedField<'a>>,
    options: EnumVariantPyO3Options,
}

struct PyClassEnumVariantNamedField<'a> {
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
    span: Span,
}

struct PyClassEnumVariantUnnamedField<'a> {
    ty: &'a syn::Type,
    span: Span,
}

/// `#[pyo3()]` options for pyclass enum variants
#[derive(Clone, Default)]
struct EnumVariantPyO3Options {
    name: Option<NameAttribute>,
    constructor: Option<ConstructorAttribute>,
}

enum EnumVariantPyO3Option {
    Name(NameAttribute),
    Constructor(ConstructorAttribute),
}

impl Parse for EnumVariantPyO3Option {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::name) {
            input.parse().map(EnumVariantPyO3Option::Name)
        } else if lookahead.peek(attributes::kw::constructor) {
            input.parse().map(EnumVariantPyO3Option::Constructor)
        } else {
            Err(lookahead.error())
        }
    }
}

impl EnumVariantPyO3Options {
    fn take_pyo3_options(attrs: &mut Vec<syn::Attribute>) -> Result<Self> {
        let mut options = EnumVariantPyO3Options::default();

        take_pyo3_options(attrs)?
            .into_iter()
            .try_for_each(|option| options.set_option(option))?;

        Ok(options)
    }

    fn set_option(&mut self, option: EnumVariantPyO3Option) -> syn::Result<()> {
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
            EnumVariantPyO3Option::Constructor(constructor) => set_option!(constructor),
            EnumVariantPyO3Option::Name(name) => set_option!(name),
        }
        Ok(())
    }
}

// todo(remove this dead code allowance once __repr__ is implemented
#[allow(dead_code)]
pub enum PyFmtName {
    Str,
    Repr,
}

fn implement_py_formatting(
    ty: &syn::Type,
    ctx: &Ctx,
    option: &StrFormatterAttribute,
) -> (ImplItemFn, MethodAndSlotDef) {
    let mut fmt_impl = match &option.value {
        Some(opt) => {
            let fmt = &opt.fmt;
            let args = &opt
                .args
                .iter()
                .map(|member| quote! {self.#member})
                .collect::<Vec<TokenStream>>();
            let fmt_impl: ImplItemFn = syn::parse_quote! {
                fn __pyo3__generated____str__(&self) -> ::std::string::String {
                    ::std::format!(#fmt, #(#args, )*)
                }
            };
            fmt_impl
        }
        None => {
            let fmt_impl: syn::ImplItemFn = syn::parse_quote! {
                fn __pyo3__generated____str__(&self) -> ::std::string::String {
                    ::std::format!("{}", &self)
                }
            };
            fmt_impl
        }
    };
    let fmt_slot = generate_protocol_slot(
        ty,
        &mut fmt_impl,
        &__STR__,
        "__str__",
        #[cfg(feature = "experimental-inspect")]
        FunctionIntrospectionData {
            names: &["__str__"],
            arguments: Vec::new(),
            returns: parse_quote! { ::std::string::String },
        },
        ctx,
    )
    .unwrap();
    (fmt_impl, fmt_slot)
}

fn implement_pyclass_str(
    options: &PyClassPyO3Options,
    ty: &syn::Type,
    ctx: &Ctx,
) -> (Option<ImplItemFn>, Option<MethodAndSlotDef>) {
    match &options.str {
        Some(option) => {
            let (default_str, default_str_slot) = implement_py_formatting(ty, ctx, option);
            (Some(default_str), Some(default_str_slot))
        }
        _ => (None, None),
    }
}

fn impl_enum(
    enum_: PyClassEnum<'_>,
    args: &PyClassArgs,
    doc: PythonDoc,
    methods_type: PyClassMethodsType,
    ctx: &Ctx,
) -> Result<TokenStream> {
    if let Some(str_fmt) = &args.options.str {
        ensure_spanned!(str_fmt.value.is_none(), str_fmt.value.span() => "The format string syntax cannot be used with enums")
    }

    match enum_ {
        PyClassEnum::Simple(simple_enum) => {
            impl_simple_enum(simple_enum, args, doc, methods_type, ctx)
        }
        PyClassEnum::Complex(complex_enum) => {
            impl_complex_enum(complex_enum, args, doc, methods_type, ctx)
        }
    }
}

fn impl_simple_enum(
    simple_enum: PyClassSimpleEnum<'_>,
    args: &PyClassArgs,
    doc: PythonDoc,
    methods_type: PyClassMethodsType,
    ctx: &Ctx,
) -> Result<TokenStream> {
    let cls = simple_enum.ident;
    let ty: syn::Type = syn::parse_quote!(#cls);
    let variants = simple_enum.variants;
    let pytypeinfo = impl_pytypeinfo(cls, args, ctx);

    for variant in &variants {
        ensure_spanned!(variant.options.constructor.is_none(), variant.options.constructor.span() => "`constructor` can't be used on a simple enum variant");
    }

    let variant_cfg_check = generate_cfg_check(&variants, cls);

    let (default_repr, default_repr_slot) = {
        let variants_repr = variants.iter().map(|variant| {
            let variant_name = variant.ident;
            let cfg_attrs = &variant.cfg_attrs;
            // Assuming all variants are unit variants because they are the only type we support.
            let repr = format!(
                "{}.{}",
                get_class_python_name(cls, args),
                variant.get_python_name(args),
            );
            quote! { #(#cfg_attrs)* #cls::#variant_name => #repr, }
        });
        let mut repr_impl: syn::ImplItemFn = syn::parse_quote! {
            fn __pyo3__repr__(&self) -> &'static str {
                match *self {
                    #(#variants_repr)*
                }
            }
        };
        let repr_slot = generate_default_protocol_slot(&ty, &mut repr_impl, &__REPR__, ctx)?;
        (repr_impl, repr_slot)
    };

    let (default_str, default_str_slot) = implement_pyclass_str(&args.options, &ty, ctx);

    let repr_type = &simple_enum.repr_type;

    let (default_int, default_int_slot) = {
        // This implementation allows us to convert &T to #repr_type without implementing `Copy`
        let variants_to_int = variants.iter().map(|variant| {
            let variant_name = variant.ident;
            let cfg_attrs = &variant.cfg_attrs;
            quote! { #(#cfg_attrs)* #cls::#variant_name => #cls::#variant_name as #repr_type, }
        });
        let mut int_impl: syn::ImplItemFn = syn::parse_quote! {
            fn __pyo3__int__(&self) -> #repr_type {
                match *self {
                    #(#variants_to_int)*
                }
            }
        };
        let int_slot = generate_default_protocol_slot(&ty, &mut int_impl, &__INT__, ctx)?;
        (int_impl, int_slot)
    };

    let (default_richcmp, default_richcmp_slot) = pyclass_richcmp_simple_enum(
        &args.options,
        &ty,
        repr_type,
        #[cfg(feature = "experimental-inspect")]
        &get_class_python_name(cls, args).to_string(),
        ctx,
    )?;
    let (default_hash, default_hash_slot) = pyclass_hash(&args.options, &ty, ctx)?;

    let mut default_slots = vec![default_repr_slot, default_int_slot];
    default_slots.extend(default_richcmp_slot);
    default_slots.extend(default_hash_slot);
    default_slots.extend(default_str_slot);

    let pyclass_impls = PyClassImplsBuilder::new(
        cls,
        args,
        methods_type,
        simple_enum_default_methods(
            cls,
            variants
                .iter()
                .map(|v| (v.ident, v.get_python_name(args), &v.cfg_attrs)),
            ctx,
        ),
        default_slots,
    )
    .doc(doc)
    .impl_all(ctx)?;

    Ok(quote! {
        #variant_cfg_check

        #pytypeinfo

        #pyclass_impls

        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl #cls {
            #default_repr
            #default_int
            #default_richcmp
            #default_hash
            #default_str
        }
    })
}

fn impl_complex_enum(
    complex_enum: PyClassComplexEnum<'_>,
    args: &PyClassArgs,
    doc: PythonDoc,
    methods_type: PyClassMethodsType,
    ctx: &Ctx,
) -> Result<TokenStream> {
    let Ctx { pyo3_path, .. } = ctx;
    let cls = complex_enum.ident;
    let ty: syn::Type = syn::parse_quote!(#cls);

    // Need to rig the enum PyClass options
    let args = {
        let mut rigged_args = args.clone();
        // Needs to be frozen to disallow `&mut self` methods, which could break a runtime invariant
        rigged_args.options.frozen = parse_quote!(frozen);
        // Needs to be subclassable by the variant PyClasses
        rigged_args.options.subclass = parse_quote!(subclass);
        rigged_args
    };

    let ctx = &Ctx::new(&args.options.krate, None);
    let cls = complex_enum.ident;
    let variants = complex_enum.variants;
    let pytypeinfo = impl_pytypeinfo(cls, &args, ctx);

    let (default_richcmp, default_richcmp_slot) = pyclass_richcmp(&args.options, &ty, ctx)?;
    let (default_hash, default_hash_slot) = pyclass_hash(&args.options, &ty, ctx)?;

    let (default_str, default_str_slot) = implement_pyclass_str(&args.options, &ty, ctx);

    let mut default_slots = vec![];
    default_slots.extend(default_richcmp_slot);
    default_slots.extend(default_hash_slot);
    default_slots.extend(default_str_slot);

    let impl_builder = PyClassImplsBuilder::new(
        cls,
        &args,
        methods_type,
        complex_enum_default_methods(
            cls,
            variants
                .iter()
                .map(|v| (v.get_ident(), v.get_python_name(&args))),
            ctx,
        ),
        default_slots,
    )
    .doc(doc);

    let enum_into_pyobject_impl = {
        let match_arms = variants
            .iter()
            .map(|variant| {
                let variant_ident = variant.get_ident();
                let variant_cls = gen_complex_enum_variant_class_ident(cls, variant.get_ident());
                quote! {
                    #cls::#variant_ident { .. } => {
                        let pyclass_init = <#pyo3_path::PyClassInitializer<Self> as ::std::convert::From<Self>>::from(self).add_subclass(#variant_cls);
                        unsafe { #pyo3_path::Bound::new(py, pyclass_init).map(|b| b.cast_into_unchecked()) }
                    }
                }
            });
        let output_type = if cfg!(feature = "experimental-inspect") {
            let full_name = get_class_python_module_and_name(cls, &args);
            quote! { const OUTPUT_TYPE: &'static str = #full_name; }
        } else {
            quote! {}
        };
        quote! {
            impl<'py> #pyo3_path::conversion::IntoPyObject<'py> for #cls {
                type Target = Self;
                type Output = #pyo3_path::Bound<'py, <Self as #pyo3_path::conversion::IntoPyObject<'py>>::Target>;
                type Error = #pyo3_path::PyErr;
                #output_type

                fn into_pyobject(self, py: #pyo3_path::Python<'py>) -> ::std::result::Result<
                    <Self as #pyo3_path::conversion::IntoPyObject>::Output,
                    <Self as #pyo3_path::conversion::IntoPyObject>::Error,
                > {
                    match self {
                        #(#match_arms)*
                    }
                }
            }
        }
    };

    let pyclass_impls: TokenStream = [
        impl_builder.impl_pyclass(ctx),
        impl_builder.impl_extractext(ctx),
        enum_into_pyobject_impl,
        impl_builder.impl_pyclassimpl(ctx)?,
        impl_builder.impl_add_to_module(ctx),
        impl_builder.impl_freelist(ctx),
        impl_builder.impl_introspection(ctx),
    ]
    .into_iter()
    .collect();

    let mut variant_cls_zsts = vec![];
    let mut variant_cls_pytypeinfos = vec![];
    let mut variant_cls_pyclass_impls = vec![];
    let mut variant_cls_impls = vec![];
    for variant in variants {
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
            options: {
                let mut rigged_options: PyClassPyO3Options = parse_quote!(extends = #cls, frozen);
                // If a specific module was given to the base class, use it for all variants.
                rigged_options.module.clone_from(&args.options.module);
                rigged_options
            },
        };

        let variant_cls_pytypeinfo = impl_pytypeinfo(&variant_cls, &variant_args, ctx);
        variant_cls_pytypeinfos.push(variant_cls_pytypeinfo);

        let (variant_cls_impl, field_getters, mut slots) =
            impl_complex_enum_variant_cls(cls, &variant, ctx)?;
        variant_cls_impls.push(variant_cls_impl);

        let variant_new = complex_enum_variant_new(cls, variant, ctx)?;
        slots.push(variant_new);

        let pyclass_impl = PyClassImplsBuilder::new(
            &variant_cls,
            &variant_args,
            methods_type,
            field_getters,
            slots,
        )
        .impl_all(ctx)?;

        variant_cls_pyclass_impls.push(pyclass_impl);
    }

    Ok(quote! {
        #pytypeinfo

        #pyclass_impls

        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl #cls {
            #default_richcmp
            #default_hash
            #default_str
        }

        #(#variant_cls_zsts)*

        #(#variant_cls_pytypeinfos)*

        #(#variant_cls_pyclass_impls)*

        #(#variant_cls_impls)*
    })
}

fn impl_complex_enum_variant_cls(
    enum_name: &syn::Ident,
    variant: &PyClassEnumVariant<'_>,
    ctx: &Ctx,
) -> Result<(TokenStream, Vec<MethodAndMethodDef>, Vec<MethodAndSlotDef>)> {
    match variant {
        PyClassEnumVariant::Struct(struct_variant) => {
            impl_complex_enum_struct_variant_cls(enum_name, struct_variant, ctx)
        }
        PyClassEnumVariant::Tuple(tuple_variant) => {
            impl_complex_enum_tuple_variant_cls(enum_name, tuple_variant, ctx)
        }
    }
}

fn impl_complex_enum_variant_match_args(
    ctx @ Ctx { pyo3_path, .. }: &Ctx,
    variant_cls_type: &syn::Type,
    field_names: &[Ident],
) -> syn::Result<(MethodAndMethodDef, syn::ImplItemFn)> {
    let ident = format_ident!("__match_args__");
    let field_names_unraw = field_names.iter().map(|name| name.unraw());
    let mut match_args_impl: syn::ImplItemFn = {
        parse_quote! {
            #[classattr]
            fn #ident(py: #pyo3_path::Python<'_>) -> #pyo3_path::PyResult<#pyo3_path::Bound<'_, #pyo3_path::types::PyTuple>> {
                #pyo3_path::types::PyTuple::new::<&str, _>(py, [
                    #(stringify!(#field_names_unraw),)*
                ])
            }
        }
    };

    let spec = FnSpec::parse(
        &mut match_args_impl.sig,
        &mut match_args_impl.attrs,
        Default::default(),
    )?;
    let variant_match_args = impl_py_class_attribute(variant_cls_type, &spec, ctx)?;

    Ok((variant_match_args, match_args_impl))
}

fn impl_complex_enum_struct_variant_cls(
    enum_name: &syn::Ident,
    variant: &PyClassEnumStructVariant<'_>,
    ctx: &Ctx,
) -> Result<(TokenStream, Vec<MethodAndMethodDef>, Vec<MethodAndSlotDef>)> {
    let Ctx { pyo3_path, .. } = ctx;
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

        let field_getter =
            complex_enum_variant_field_getter(&variant_cls_type, field_name, field.span, ctx)?;

        let field_getter_impl = quote! {
            fn #field_name(slf: #pyo3_path::PyClassGuard<'_, Self>, py: #pyo3_path::Python<'_>) -> #pyo3_path::PyResult<#pyo3_path::Py<#pyo3_path::PyAny>> {
                #[allow(unused_imports)]
                use #pyo3_path::impl_::pyclass::Probe as _;
                match &*slf.into_super() {
                    #enum_name::#variant_ident { #field_name, .. } =>
                        #pyo3_path::impl_::pyclass::ConvertField::<
                            { #pyo3_path::impl_::pyclass::IsIntoPyObjectRef::<#field_type>::VALUE },
                            { #pyo3_path::impl_::pyclass::IsIntoPyObject::<#field_type>::VALUE },
                        >::convert_field::<#field_type>(#field_name, py),
                    _ => ::core::unreachable!("Wrong complex enum variant found in variant wrapper PyClass"),
                }
            }
        };

        field_names.push(field_name.clone());
        fields_with_types.push(field_with_type);
        field_getters.push(field_getter);
        field_getter_impls.push(field_getter_impl);
    }

    let (variant_match_args, match_args_const_impl) =
        impl_complex_enum_variant_match_args(ctx, &variant_cls_type, &field_names)?;

    field_getters.push(variant_match_args);

    let cls_impl = quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl #variant_cls {
            #[allow(clippy::too_many_arguments)]
            fn __pymethod_constructor__(py: #pyo3_path::Python<'_>, #(#fields_with_types,)*) -> #pyo3_path::PyClassInitializer<#variant_cls> {
                let base_value = #enum_name::#variant_ident { #(#field_names,)* };
                <#pyo3_path::PyClassInitializer<#enum_name> as ::std::convert::From<#enum_name>>::from(base_value).add_subclass(#variant_cls)
            }

            #match_args_const_impl

            #(#field_getter_impls)*
        }
    };

    Ok((cls_impl, field_getters, Vec::new()))
}

fn impl_complex_enum_tuple_variant_field_getters(
    ctx: &Ctx,
    variant: &PyClassEnumTupleVariant<'_>,
    enum_name: &syn::Ident,
    variant_cls_type: &syn::Type,
    variant_ident: &&Ident,
    field_names: &mut Vec<Ident>,
    fields_types: &mut Vec<syn::Type>,
) -> Result<(Vec<MethodAndMethodDef>, Vec<syn::ImplItemFn>)> {
    let Ctx { pyo3_path, .. } = ctx;

    let mut field_getters = vec![];
    let mut field_getter_impls = vec![];

    for (index, field) in variant.fields.iter().enumerate() {
        let field_name = format_ident!("_{}", index);
        let field_type = field.ty;

        let field_getter =
            complex_enum_variant_field_getter(variant_cls_type, &field_name, field.span, ctx)?;

        // Generate the match arms needed to destructure the tuple and access the specific field
        let field_access_tokens: Vec<_> = (0..variant.fields.len())
            .map(|i| {
                if i == index {
                    quote! { val }
                } else {
                    quote! { _ }
                }
            })
            .collect();
        let field_getter_impl: syn::ImplItemFn = parse_quote! {
            fn #field_name(slf: #pyo3_path::PyClassGuard<'_, Self>, py: #pyo3_path::Python<'_>) -> #pyo3_path::PyResult<#pyo3_path::Py<#pyo3_path::PyAny>> {
                #[allow(unused_imports)]
                use #pyo3_path::impl_::pyclass::Probe as _;
                match &*slf.into_super() {
                    #enum_name::#variant_ident ( #(#field_access_tokens), *) =>
                        #pyo3_path::impl_::pyclass::ConvertField::<
                            { #pyo3_path::impl_::pyclass::IsIntoPyObjectRef::<#field_type>::VALUE },
                            { #pyo3_path::impl_::pyclass::IsIntoPyObject::<#field_type>::VALUE },
                        >::convert_field::<#field_type>(val, py),
                    _ => ::core::unreachable!("Wrong complex enum variant found in variant wrapper PyClass"),
                }
            }
        };

        field_names.push(field_name);
        fields_types.push(field_type.clone());
        field_getters.push(field_getter);
        field_getter_impls.push(field_getter_impl);
    }

    Ok((field_getters, field_getter_impls))
}

fn impl_complex_enum_tuple_variant_len(
    ctx: &Ctx,

    variant_cls_type: &syn::Type,
    num_fields: usize,
) -> Result<(MethodAndSlotDef, syn::ImplItemFn)> {
    let Ctx { pyo3_path, .. } = ctx;

    let mut len_method_impl: syn::ImplItemFn = parse_quote! {
        fn __len__(slf: #pyo3_path::PyClassGuard<'_, Self>) -> #pyo3_path::PyResult<usize> {
            ::std::result::Result::Ok(#num_fields)
        }
    };

    let variant_len =
        generate_default_protocol_slot(variant_cls_type, &mut len_method_impl, &__LEN__, ctx)?;

    Ok((variant_len, len_method_impl))
}

fn impl_complex_enum_tuple_variant_getitem(
    ctx: &Ctx,
    variant_cls: &syn::Ident,
    variant_cls_type: &syn::Type,
    num_fields: usize,
) -> Result<(MethodAndSlotDef, syn::ImplItemFn)> {
    let Ctx { pyo3_path, .. } = ctx;

    let match_arms: Vec<_> = (0..num_fields)
        .map(|i| {
            let field_access = format_ident!("_{}", i);
            quote! { #i =>
                #pyo3_path::IntoPyObjectExt::into_py_any(#variant_cls::#field_access(slf, py)?, py)
            }
        })
        .collect();

    let mut get_item_method_impl: syn::ImplItemFn = parse_quote! {
        fn __getitem__(slf: #pyo3_path::PyClassGuard<'_, Self>, py: #pyo3_path::Python<'_>, idx: usize) -> #pyo3_path::PyResult< #pyo3_path::Py<#pyo3_path::PyAny>> {
            match idx {
                #( #match_arms, )*
                _ => ::std::result::Result::Err(#pyo3_path::exceptions::PyIndexError::new_err("tuple index out of range")),
            }
        }
    };

    let variant_getitem = generate_default_protocol_slot(
        variant_cls_type,
        &mut get_item_method_impl,
        &__GETITEM__,
        ctx,
    )?;

    Ok((variant_getitem, get_item_method_impl))
}

fn impl_complex_enum_tuple_variant_cls(
    enum_name: &syn::Ident,
    variant: &PyClassEnumTupleVariant<'_>,
    ctx: &Ctx,
) -> Result<(TokenStream, Vec<MethodAndMethodDef>, Vec<MethodAndSlotDef>)> {
    let Ctx { pyo3_path, .. } = ctx;
    let variant_ident = &variant.ident;
    let variant_cls = gen_complex_enum_variant_class_ident(enum_name, variant.ident);
    let variant_cls_type = parse_quote!(#variant_cls);

    let mut slots = vec![];

    // represents the index of the field
    let mut field_names: Vec<Ident> = vec![];
    let mut field_types: Vec<syn::Type> = vec![];

    let (mut field_getters, field_getter_impls) = impl_complex_enum_tuple_variant_field_getters(
        ctx,
        variant,
        enum_name,
        &variant_cls_type,
        variant_ident,
        &mut field_names,
        &mut field_types,
    )?;

    let num_fields = variant.fields.len();

    let (variant_len, len_method_impl) =
        impl_complex_enum_tuple_variant_len(ctx, &variant_cls_type, num_fields)?;

    slots.push(variant_len);

    let (variant_getitem, getitem_method_impl) =
        impl_complex_enum_tuple_variant_getitem(ctx, &variant_cls, &variant_cls_type, num_fields)?;

    slots.push(variant_getitem);

    let (variant_match_args, match_args_method_impl) =
        impl_complex_enum_variant_match_args(ctx, &variant_cls_type, &field_names)?;

    field_getters.push(variant_match_args);

    let cls_impl = quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl #variant_cls {
            #[allow(clippy::too_many_arguments)]
            fn __pymethod_constructor__(py: #pyo3_path::Python<'_>, #(#field_names : #field_types,)*) -> #pyo3_path::PyClassInitializer<#variant_cls> {
                let base_value = #enum_name::#variant_ident ( #(#field_names,)* );
                <#pyo3_path::PyClassInitializer<#enum_name> as ::std::convert::From<#enum_name>>::from(base_value).add_subclass(#variant_cls)
            }

            #len_method_impl

            #getitem_method_impl

            #match_args_method_impl

            #(#field_getter_impls)*
        }
    };

    Ok((cls_impl, field_getters, slots))
}

fn gen_complex_enum_variant_class_ident(enum_: &syn::Ident, variant: &syn::Ident) -> syn::Ident {
    format_ident!("{}_{}", enum_, variant)
}

#[cfg(feature = "experimental-inspect")]
struct FunctionIntrospectionData<'a> {
    names: &'a [&'a str],
    arguments: Vec<FnArg<'a>>,
    returns: syn::Type,
}

fn generate_protocol_slot(
    cls: &syn::Type,
    method: &mut syn::ImplItemFn,
    slot: &SlotDef,
    name: &str,
    #[cfg(feature = "experimental-inspect")] introspection_data: FunctionIntrospectionData<'_>,
    ctx: &Ctx,
) -> syn::Result<MethodAndSlotDef> {
    let spec = FnSpec::parse(
        &mut method.sig,
        &mut Vec::new(),
        PyFunctionOptions::default(),
    )?;
    #[cfg_attr(not(feature = "experimental-inspect"), allow(unused_mut))]
    let mut def = slot.generate_type_slot(&syn::parse_quote!(#cls), &spec, name, ctx)?;
    #[cfg(feature = "experimental-inspect")]
    {
        // We generate introspection data
        let associated_method = def.associated_method;
        let signature = FunctionSignature::from_arguments(introspection_data.arguments);
        let returns = introspection_data.returns;
        let introspection = introspection_data
            .names
            .iter()
            .map(|name| {
                function_introspection_code(
                    &ctx.pyo3_path,
                    None,
                    name,
                    &signature,
                    Some("self"),
                    parse_quote!(-> #returns),
                    [],
                    Some(cls),
                )
            })
            .collect::<Vec<_>>();
        let const_name = format_ident!("_{}", unique_element_id()); // We need an explicit name here
        def.associated_method = quote! {
            #associated_method
            const #const_name: () = {
                #(#introspection)*
            };
        };
    }
    Ok(def)
}

fn generate_default_protocol_slot(
    cls: &syn::Type,
    method: &mut syn::ImplItemFn,
    slot: &SlotDef,
    ctx: &Ctx,
) -> syn::Result<MethodAndSlotDef> {
    let spec = FnSpec::parse(
        &mut method.sig,
        &mut Vec::new(),
        PyFunctionOptions::default(),
    )?;
    let name = spec.name.to_string();
    slot.generate_type_slot(
        &syn::parse_quote!(#cls),
        &spec,
        &format!("__default_{name}__"),
        ctx,
    )
}

fn simple_enum_default_methods<'a>(
    cls: &'a syn::Ident,
    unit_variant_names: impl IntoIterator<
        Item = (
            &'a syn::Ident,
            Cow<'a, syn::Ident>,
            &'a Vec<&'a syn::Attribute>,
        ),
    >,
    ctx: &Ctx,
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
        },
    };
    unit_variant_names
        .into_iter()
        .map(|(var, py_name, attrs)| {
            let method = gen_py_const(&cls_type, &variant_to_attribute(var, &py_name), ctx);
            let associated_method_tokens = method.associated_method;
            let method_def_tokens = method.method_def;

            let associated_method = quote! {
                #(#attrs)*
                #associated_method_tokens
            };
            let method_def = quote! {
                #(#attrs)*
                #method_def_tokens
            };

            MethodAndMethodDef {
                associated_method,
                method_def,
            }
        })
        .collect()
}

fn complex_enum_default_methods<'a>(
    cls: &'a syn::Ident,
    variant_names: impl IntoIterator<Item = (&'a syn::Ident, Cow<'a, syn::Ident>)>,
    ctx: &Ctx,
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
        },
    };
    variant_names
        .into_iter()
        .map(|(var, py_name)| {
            gen_complex_enum_variant_attr(cls, &cls_type, &variant_to_attribute(var, &py_name), ctx)
        })
        .collect()
}

pub fn gen_complex_enum_variant_attr(
    cls: &syn::Ident,
    cls_type: &syn::Type,
    spec: &ConstSpec,
    ctx: &Ctx,
) -> MethodAndMethodDef {
    let Ctx { pyo3_path, .. } = ctx;
    let member = &spec.rust_ident;
    let wrapper_ident = format_ident!("__pymethod_variant_cls_{}__", member);
    let python_name = spec.null_terminated_python_name(ctx);

    let variant_cls = format_ident!("{}_{}", cls, member);
    let associated_method = quote! {
        fn #wrapper_ident(py: #pyo3_path::Python<'_>) -> #pyo3_path::PyResult<#pyo3_path::Py<#pyo3_path::PyAny>> {
            ::std::result::Result::Ok(py.get_type::<#variant_cls>().into_any().unbind())
        }
    };

    let method_def = quote! {
        #pyo3_path::impl_::pyclass::MaybeRuntimePyMethodDef::Static(
            #pyo3_path::impl_::pymethods::PyMethodDefType::ClassAttribute({
                #pyo3_path::impl_::pymethods::PyClassAttributeDef::new(
                    #python_name,
                    #cls_type::#wrapper_ident
                )
            })
        )
    };

    MethodAndMethodDef {
        associated_method,
        method_def,
    }
}

fn complex_enum_variant_new<'a>(
    cls: &'a syn::Ident,
    variant: PyClassEnumVariant<'a>,
    ctx: &Ctx,
) -> Result<MethodAndSlotDef> {
    match variant {
        PyClassEnumVariant::Struct(struct_variant) => {
            complex_enum_struct_variant_new(cls, struct_variant, ctx)
        }
        PyClassEnumVariant::Tuple(tuple_variant) => {
            complex_enum_tuple_variant_new(cls, tuple_variant, ctx)
        }
    }
}

fn complex_enum_struct_variant_new<'a>(
    cls: &'a syn::Ident,
    variant: PyClassEnumStructVariant<'a>,
    ctx: &Ctx,
) -> Result<MethodAndSlotDef> {
    let Ctx { pyo3_path, .. } = ctx;
    let variant_cls = format_ident!("{}_{}", cls, variant.ident);
    let variant_cls_type: syn::Type = parse_quote!(#variant_cls);

    let arg_py_ident: syn::Ident = parse_quote!(py);
    let arg_py_type: syn::Type = parse_quote!(#pyo3_path::Python<'_>);

    let args = {
        let mut args = vec![
            // py: Python<'_>
            FnArg::Py(PyArg {
                name: &arg_py_ident,
                ty: &arg_py_type,
            }),
        ];

        for field in &variant.fields {
            args.push(FnArg::Regular(RegularArg {
                name: Cow::Borrowed(field.ident),
                ty: field.ty,
                from_py_with: None,
                default_value: None,
                option_wrapped_type: None,
                #[cfg(feature = "experimental-inspect")]
                annotation: None,
            }));
        }
        args
    };

    let signature = if let Some(constructor) = variant.options.constructor {
        crate::pyfunction::FunctionSignature::from_arguments_and_attribute(
            args,
            constructor.into_signature(),
        )?
    } else {
        crate::pyfunction::FunctionSignature::from_arguments(args)
    };

    let spec = FnSpec {
        tp: crate::method::FnType::FnNew,
        name: &format_ident!("__pymethod_constructor__"),
        python_name: format_ident!("__new__"),
        signature,
        convention: crate::method::CallingConvention::TpNew,
        text_signature: None,
        asyncness: None,
        unsafety: None,
        warnings: vec![],
        #[cfg(feature = "experimental-inspect")]
        output: syn::ReturnType::Default,
    };

    crate::pymethod::impl_py_method_def_new(&variant_cls_type, &spec, ctx)
}

fn complex_enum_tuple_variant_new<'a>(
    cls: &'a syn::Ident,
    variant: PyClassEnumTupleVariant<'a>,
    ctx: &Ctx,
) -> Result<MethodAndSlotDef> {
    let Ctx { pyo3_path, .. } = ctx;

    let variant_cls: Ident = format_ident!("{}_{}", cls, variant.ident);
    let variant_cls_type: syn::Type = parse_quote!(#variant_cls);

    let arg_py_ident: syn::Ident = parse_quote!(py);
    let arg_py_type: syn::Type = parse_quote!(#pyo3_path::Python<'_>);

    let args = {
        let mut args = vec![FnArg::Py(PyArg {
            name: &arg_py_ident,
            ty: &arg_py_type,
        })];

        for (i, field) in variant.fields.iter().enumerate() {
            args.push(FnArg::Regular(RegularArg {
                name: std::borrow::Cow::Owned(format_ident!("_{}", i)),
                ty: field.ty,
                from_py_with: None,
                default_value: None,
                option_wrapped_type: None,
                #[cfg(feature = "experimental-inspect")]
                annotation: None,
            }));
        }
        args
    };

    let signature = if let Some(constructor) = variant.options.constructor {
        crate::pyfunction::FunctionSignature::from_arguments_and_attribute(
            args,
            constructor.into_signature(),
        )?
    } else {
        crate::pyfunction::FunctionSignature::from_arguments(args)
    };

    let spec = FnSpec {
        tp: crate::method::FnType::FnNew,
        name: &format_ident!("__pymethod_constructor__"),
        python_name: format_ident!("__new__"),
        signature,
        convention: crate::method::CallingConvention::TpNew,
        text_signature: None,
        asyncness: None,
        unsafety: None,
        warnings: vec![],
        #[cfg(feature = "experimental-inspect")]
        output: syn::ReturnType::Default,
    };

    crate::pymethod::impl_py_method_def_new(&variant_cls_type, &spec, ctx)
}

fn complex_enum_variant_field_getter<'a>(
    variant_cls_type: &'a syn::Type,
    field_name: &'a syn::Ident,
    field_span: Span,
    ctx: &Ctx,
) -> Result<MethodAndMethodDef> {
    let mut arg = parse_quote!(py: Python<'_>);
    let py = FnArg::parse(&mut arg)?;
    let signature = crate::pyfunction::FunctionSignature::from_arguments(vec![py]);

    let self_type = crate::method::SelfType::TryFromBoundRef(field_span);

    let spec = FnSpec {
        tp: crate::method::FnType::Getter(self_type.clone()),
        name: field_name,
        python_name: field_name.unraw(),
        signature,
        convention: crate::method::CallingConvention::Noargs,
        text_signature: None,
        asyncness: None,
        unsafety: None,
        warnings: vec![],
        #[cfg(feature = "experimental-inspect")]
        output: syn::ReturnType::Type(Token![->](field_span), Box::new(variant_cls_type.clone())),
    };

    let property_type = crate::pymethod::PropertyType::Function {
        self_type: &self_type,
        spec: &spec,
        doc: crate::get_doc(&[], None, ctx)?,
    };

    let getter = crate::pymethod::impl_py_getter_def(variant_cls_type, property_type, ctx)?;
    Ok(getter)
}

fn descriptors_to_items(
    cls: &syn::Ident,
    rename_all: Option<&RenameAllAttribute>,
    frozen: Option<frozen>,
    field_options: Vec<(&syn::Field, FieldPyO3Options)>,
    ctx: &Ctx,
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
                ctx,
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
                ctx,
            )?;
            items.push(setter);
        };
    }
    Ok(items)
}

fn impl_pytypeinfo(cls: &syn::Ident, attr: &PyClassArgs, ctx: &Ctx) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;
    let cls_name = get_class_python_name(cls, attr).to_string();

    let module = if let Some(ModuleAttribute { value, .. }) = &attr.options.module {
        quote! { ::core::option::Option::Some(#value) }
    } else {
        quote! { ::core::option::Option::None }
    };

    quote! {
        unsafe impl #pyo3_path::type_object::PyTypeInfo for #cls {
            const NAME: &'static str = #cls_name;
            const MODULE: ::std::option::Option<&'static str> = #module;

            #[inline]
            fn type_object_raw(py: #pyo3_path::Python<'_>) -> *mut #pyo3_path::ffi::PyTypeObject {
                use #pyo3_path::prelude::PyTypeMethods;
                <#cls as #pyo3_path::impl_::pyclass::PyClassImpl>::lazy_type_object()
                    .get_or_try_init(py)
                    .unwrap_or_else(|e| #pyo3_path::impl_::pyclass::type_object_init_failed(
                        py,
                        e,
                        <Self as #pyo3_path::type_object::PyTypeInfo>::NAME
                    ))
                    .as_type_ptr()
            }
        }
    }
}

fn pyclass_richcmp_arms(
    options: &PyClassPyO3Options,
    ctx: &Ctx,
) -> std::result::Result<TokenStream, syn::Error> {
    let Ctx { pyo3_path, .. } = ctx;

    let eq_arms = options
        .eq
        .map(|eq| eq.span)
        .or(options.eq_int.map(|eq_int| eq_int.span))
        .map(|span| {
            quote_spanned! { span =>
                #pyo3_path::pyclass::CompareOp::Eq => {
                    #pyo3_path::IntoPyObjectExt::into_py_any(self_val == other, py)
                },
                #pyo3_path::pyclass::CompareOp::Ne => {
                    #pyo3_path::IntoPyObjectExt::into_py_any(self_val != other, py)
                },
            }
        })
        .unwrap_or_default();

    if let Some(ord) = options.ord {
        ensure_spanned!(options.eq.is_some(), ord.span() => "The `ord` option requires the `eq` option.");
    }

    let ord_arms = options
        .ord
        .map(|ord| {
            quote_spanned! { ord.span() =>
                #pyo3_path::pyclass::CompareOp::Gt => {
                    #pyo3_path::IntoPyObjectExt::into_py_any(self_val > other, py)
                },
                #pyo3_path::pyclass::CompareOp::Lt => {
                    #pyo3_path::IntoPyObjectExt::into_py_any(self_val < other, py)
                 },
                #pyo3_path::pyclass::CompareOp::Le => {
                    #pyo3_path::IntoPyObjectExt::into_py_any(self_val <= other, py)
                 },
                #pyo3_path::pyclass::CompareOp::Ge => {
                    #pyo3_path::IntoPyObjectExt::into_py_any(self_val >= other, py)
                 },
            }
        })
        .unwrap_or_else(|| quote! { _ => ::std::result::Result::Ok(py.NotImplemented()) });

    Ok(quote! {
        #eq_arms
        #ord_arms
    })
}

fn pyclass_richcmp_simple_enum(
    options: &PyClassPyO3Options,
    cls: &syn::Type,
    repr_type: &syn::Ident,
    #[cfg(feature = "experimental-inspect")] class_name: &str,
    ctx: &Ctx,
) -> Result<(Option<syn::ImplItemFn>, Option<MethodAndSlotDef>)> {
    let Ctx { pyo3_path, .. } = ctx;
    if let Some(eq_int) = options.eq_int {
        ensure_spanned!(options.eq.is_some(), eq_int.span() => "The `eq_int` option requires the `eq` option.");
    }

    if options.eq.is_none() && options.eq_int.is_none() {
        return Ok((None, None));
    }

    let arms = pyclass_richcmp_arms(options, ctx)?;

    let eq = options.eq.map(|eq| {
        quote_spanned! { eq.span() =>
            let self_val = self;
            if let ::std::result::Result::Ok(other) = other.cast::<Self>() {
                let other = &*other.borrow();
                return match op {
                    #arms
                }
            }
        }
    });

    let eq_int = options.eq_int.map(|eq_int| {
        quote_spanned! { eq_int.span() =>
            let self_val = self.__pyo3__int__();
            if let ::std::result::Result::Ok(other) = #pyo3_path::types::PyAnyMethods::extract::<#repr_type>(other).or_else(|_| {
                other.cast::<Self>().map(|o| o.borrow().__pyo3__int__())
            }) {
                return match op {
                    #arms
                }
            }
        }
    });

    let mut richcmp_impl = parse_quote! {
        fn __pyo3__generated____richcmp__(
            &self,
            py: #pyo3_path::Python,
            other: &#pyo3_path::Bound<'_, #pyo3_path::PyAny>,
            op: #pyo3_path::pyclass::CompareOp
        ) -> #pyo3_path::PyResult<#pyo3_path::Py<#pyo3_path::PyAny>> {
            #eq

            #eq_int

            ::std::result::Result::Ok(py.NotImplemented())
        }
    };
    let richcmp_slot = if options.eq.is_some() {
        generate_protocol_slot(
            cls,
            &mut richcmp_impl,
            &__RICHCMP__,
            "__richcmp__",
            #[cfg(feature = "experimental-inspect")]
            FunctionIntrospectionData {
                names: &["__eq__", "__ne__"],
                arguments: vec![FnArg::Regular(RegularArg {
                    name: Cow::Owned(format_ident!("other")),
                    // we need to set a type, let's pick something small, it is overridden by annotation anyway
                    ty: &parse_quote!(!),
                    from_py_with: None,
                    default_value: None,
                    option_wrapped_type: None,
                    annotation: Some(match (options.eq.is_some(), options.eq_int.is_some()) {
                        (true, true) => {
                            format!("{class_name} | int")
                        }
                        (true, false) => class_name.into(),
                        (false, true) => "int".into(),
                        (false, false) => unreachable!(),
                    }),
                })],
                returns: parse_quote! { ::std::primitive::bool },
            },
            ctx,
        )?
    } else {
        generate_default_protocol_slot(cls, &mut richcmp_impl, &__RICHCMP__, ctx)?
    };
    Ok((Some(richcmp_impl), Some(richcmp_slot)))
}

fn pyclass_richcmp(
    options: &PyClassPyO3Options,
    cls: &syn::Type,
    ctx: &Ctx,
) -> Result<(Option<syn::ImplItemFn>, Option<MethodAndSlotDef>)> {
    let Ctx { pyo3_path, .. } = ctx;
    if let Some(eq_int) = options.eq_int {
        bail_spanned!(eq_int.span() => "`eq_int` can only be used on simple enums.")
    }

    let arms = pyclass_richcmp_arms(options, ctx)?;
    if options.eq.is_some() {
        let mut richcmp_impl = parse_quote! {
            fn __pyo3__generated____richcmp__(
                &self,
                py: #pyo3_path::Python,
                other: &#pyo3_path::Bound<'_, #pyo3_path::PyAny>,
                op: #pyo3_path::pyclass::CompareOp
            ) -> #pyo3_path::PyResult<#pyo3_path::Py<#pyo3_path::PyAny>> {
                let self_val = self;
                if let ::std::result::Result::Ok(other) = other.cast::<Self>() {
                    let other = &*other.borrow();
                    match op {
                        #arms
                    }
                } else {
                    ::std::result::Result::Ok(py.NotImplemented())
                }
            }
        };
        let richcmp_slot = generate_protocol_slot(
            cls,
            &mut richcmp_impl,
            &__RICHCMP__,
            "__richcmp__",
            #[cfg(feature = "experimental-inspect")]
            FunctionIntrospectionData {
                names: if options.ord.is_some() {
                    &["__eq__", "__ne__", "__lt__", "__le__", "__gt__", "__ge__"]
                } else {
                    &["__eq__", "__ne__"]
                },
                arguments: vec![FnArg::Regular(RegularArg {
                    name: Cow::Owned(format_ident!("other")),
                    ty: &parse_quote!(&#cls),
                    from_py_with: None,
                    default_value: None,
                    option_wrapped_type: None,
                    annotation: None,
                })],
                returns: parse_quote! { ::std::primitive::bool },
            },
            ctx,
        )?;
        Ok((Some(richcmp_impl), Some(richcmp_slot)))
    } else {
        Ok((None, None))
    }
}

fn pyclass_hash(
    options: &PyClassPyO3Options,
    cls: &syn::Type,
    ctx: &Ctx,
) -> Result<(Option<syn::ImplItemFn>, Option<MethodAndSlotDef>)> {
    if options.hash.is_some() {
        ensure_spanned!(
            options.frozen.is_some(), options.hash.span() => "The `hash` option requires the `frozen` option.";
            options.eq.is_some(), options.hash.span() => "The `hash` option requires the `eq` option.";
        );
    }
    match options.hash {
        Some(opt) => {
            let mut hash_impl = parse_quote_spanned! { opt.span() =>
                fn __pyo3__generated____hash__(&self) -> u64 {
                    let mut s = ::std::collections::hash_map::DefaultHasher::new();
                    ::std::hash::Hash::hash(self, &mut s);
                    ::std::hash::Hasher::finish(&s)
                }
            };
            let hash_slot = generate_protocol_slot(
                cls,
                &mut hash_impl,
                &__HASH__,
                "__hash__",
                #[cfg(feature = "experimental-inspect")]
                FunctionIntrospectionData {
                    names: &["__hash__"],
                    arguments: Vec::new(),
                    returns: parse_quote! { ::std::primitive::u64 },
                },
                ctx,
            )?;
            Ok((Some(hash_impl), Some(hash_slot)))
        }
        None => Ok((None, None)),
    }
}

fn pyclass_class_geitem(
    options: &PyClassPyO3Options,
    cls: &syn::Type,
    ctx: &Ctx,
) -> Result<(Option<syn::ImplItemFn>, Option<MethodAndMethodDef>)> {
    let Ctx { pyo3_path, .. } = ctx;
    match options.generic {
        Some(_) => {
            let ident = format_ident!("__class_getitem__");
            let mut class_geitem_impl: syn::ImplItemFn = {
                parse_quote! {
                    #[classmethod]
                    fn #ident<'py>(
                        cls: &#pyo3_path::Bound<'py, #pyo3_path::types::PyType>,
                        key: &#pyo3_path::Bound<'py, #pyo3_path::types::PyAny>
                    ) -> #pyo3_path::PyResult<#pyo3_path::Bound<'py, #pyo3_path::types::PyGenericAlias>> {
                        #pyo3_path::types::PyGenericAlias::new(cls.py(), cls.as_any(), key)
                    }
                }
            };

            let spec = FnSpec::parse(
                &mut class_geitem_impl.sig,
                &mut class_geitem_impl.attrs,
                Default::default(),
            )?;

            let class_geitem_method = crate::pymethod::impl_py_method_def(
                cls,
                &spec,
                &spec.get_doc(&class_geitem_impl.attrs, ctx)?,
                Some(quote!(#pyo3_path::ffi::METH_CLASS)),
                ctx,
            )?;
            Ok((Some(class_geitem_impl), Some(class_geitem_method)))
        }
        None => Ok((None, None)),
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

    fn impl_all(&self, ctx: &Ctx) -> Result<TokenStream> {
        Ok([
            self.impl_pyclass(ctx),
            self.impl_extractext(ctx),
            self.impl_into_py(ctx),
            self.impl_pyclassimpl(ctx)?,
            self.impl_add_to_module(ctx),
            self.impl_freelist(ctx),
            self.impl_introspection(ctx),
        ]
        .into_iter()
        .collect())
    }

    fn impl_pyclass(&self, ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        let cls = self.cls;

        let frozen = if self.attr.options.frozen.is_some() {
            quote! { #pyo3_path::pyclass::boolean_struct::True }
        } else {
            quote! { #pyo3_path::pyclass::boolean_struct::False }
        };

        quote! {
            impl #pyo3_path::PyClass for #cls {
                type Frozen = #frozen;
            }
        }
    }
    fn impl_extractext(&self, ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        let cls = self.cls;

        let input_type = if cfg!(feature = "experimental-inspect") {
            let full_name = get_class_python_module_and_name(cls, self.attr);
            quote! { const INPUT_TYPE: &'static str = #full_name; }
        } else {
            quote! {}
        };
        if self.attr.options.frozen.is_some() {
            quote! {
                impl<'a, 'holder, 'py> #pyo3_path::impl_::extract_argument::PyFunctionArgument<'a, 'holder, 'py, false> for &'holder #cls
                {
                    type Holder = ::std::option::Option<#pyo3_path::PyClassGuard<'a, #cls>>;

                    #input_type

                    #[inline]
                    fn extract(obj: &'a #pyo3_path::Bound<'py, #pyo3_path::PyAny>, holder: &'holder mut Self::Holder) -> #pyo3_path::PyResult<Self> {
                        #pyo3_path::impl_::extract_argument::extract_pyclass_ref(obj, holder)
                    }
                }
            }
        } else {
            quote! {
                impl<'a, 'holder, 'py> #pyo3_path::impl_::extract_argument::PyFunctionArgument<'a, 'holder, 'py, false> for &'holder #cls
                {
                    type Holder = ::std::option::Option<#pyo3_path::PyClassGuard<'a, #cls>>;

                    #input_type

                    #[inline]
                    fn extract(obj: &'a #pyo3_path::Bound<'py, #pyo3_path::PyAny>, holder: &'holder mut Self::Holder) -> #pyo3_path::PyResult<Self> {
                        #pyo3_path::impl_::extract_argument::extract_pyclass_ref(obj, holder)
                    }
                }

                impl<'a, 'holder, 'py> #pyo3_path::impl_::extract_argument::PyFunctionArgument<'a, 'holder, 'py, false> for &'holder mut #cls
                {
                    type Holder = ::std::option::Option<#pyo3_path::PyClassGuardMut<'a, #cls>>;

                    #input_type

                    #[inline]
                    fn extract(obj: &'a #pyo3_path::Bound<'py, #pyo3_path::PyAny>, holder: &'holder mut Self::Holder) -> #pyo3_path::PyResult<Self> {
                        #pyo3_path::impl_::extract_argument::extract_pyclass_ref_mut(obj, holder)
                    }
                }
            }
        }
    }

    fn impl_into_py(&self, ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        let cls = self.cls;
        let attr = self.attr;
        // If #cls is not extended type, we allow Self->PyObject conversion
        if attr.options.extends.is_none() {
            let output_type = if cfg!(feature = "experimental-inspect") {
                let full_name = get_class_python_module_and_name(cls, self.attr);
                quote! { const OUTPUT_TYPE: &'static str = #full_name; }
            } else {
                quote! {}
            };
            quote! {
                impl<'py> #pyo3_path::conversion::IntoPyObject<'py> for #cls {
                    type Target = Self;
                    type Output = #pyo3_path::Bound<'py, <Self as #pyo3_path::conversion::IntoPyObject<'py>>::Target>;
                    type Error = #pyo3_path::PyErr;
                    #output_type

                    fn into_pyobject(self, py: #pyo3_path::Python<'py>) -> ::std::result::Result<
                        <Self as #pyo3_path::conversion::IntoPyObject>::Output,
                        <Self as #pyo3_path::conversion::IntoPyObject>::Error,
                    > {
                        #pyo3_path::Bound::new(py, self)
                    }
                }
            }
        } else {
            quote! {}
        }
    }
    fn impl_pyclassimpl(&self, ctx: &Ctx) -> Result<TokenStream> {
        let Ctx { pyo3_path, .. } = ctx;
        let cls = self.cls;
        let doc = self.doc.as_ref().map_or(
            LitCStr::empty(ctx).to_token_stream(),
            PythonDoc::to_token_stream,
        );
        let is_basetype = self.attr.options.subclass.is_some();
        let base = match &self.attr.options.extends {
            Some(extends_attr) => extends_attr.value.clone(),
            None => parse_quote! { #pyo3_path::PyAny },
        };
        let is_subclass = self.attr.options.extends.is_some();
        let is_mapping: bool = self.attr.options.mapping.is_some();
        let is_sequence: bool = self.attr.options.sequence.is_some();
        let is_immutable_type = self.attr.options.immutable_type.is_some();

        ensure_spanned!(
            !(is_mapping && is_sequence),
            self.cls.span() => "a `#[pyclass]` cannot be both a `mapping` and a `sequence`"
        );

        let dict_offset = if self.attr.options.dict.is_some() {
            quote! {
                fn dict_offset() -> ::std::option::Option<#pyo3_path::ffi::Py_ssize_t> {
                    ::std::option::Option::Some(#pyo3_path::impl_::pyclass::dict_offset::<Self>())
                }
            }
        } else {
            TokenStream::new()
        };

        // insert space for weak ref
        let weaklist_offset = if self.attr.options.weakref.is_some() {
            quote! {
                fn weaklist_offset() -> ::std::option::Option<#pyo3_path::ffi::Py_ssize_t> {
                    ::std::option::Option::Some(#pyo3_path::impl_::pyclass::weaklist_offset::<Self>())
                }
            }
        } else {
            TokenStream::new()
        };

        let thread_checker = if self.attr.options.unsendable.is_some() {
            quote! { #pyo3_path::impl_::pyclass::ThreadCheckerImpl }
        } else {
            quote! { #pyo3_path::impl_::pyclass::SendablePyClass<#cls> }
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
                                #pyo3_path::inventory::iter::<<Self as #pyo3_path::impl_::pyclass::PyClassImpl>::Inventory>(),
                                #pyo3_path::impl_::pyclass::PyClassInventory::items
                            )
                        )
                    },
                    Some(quote! { type Inventory = #inventory_class_name; }),
                    Some(define_inventory_class(&inventory_class_name, ctx)),
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
        let freelist_slots = self.freelist_slots(ctx);

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
            quote! { #pyo3_path::impl_::pyclass::PyClassDictSlot }
        } else {
            quote! { #pyo3_path::impl_::pyclass::PyClassDummySlot }
        };

        // insert space for weak ref
        let weakref = if attr.options.weakref.is_some() {
            quote! { #pyo3_path::impl_::pyclass::PyClassWeakRefSlot }
        } else {
            quote! { #pyo3_path::impl_::pyclass::PyClassDummySlot }
        };

        let base_nativetype = if attr.options.extends.is_some() {
            quote! { <Self::BaseType as #pyo3_path::impl_::pyclass::PyClassBaseType>::BaseNativeType }
        } else {
            quote! { #pyo3_path::PyAny }
        };

        let pyclass_base_type_impl = attr.options.subclass.map(|subclass| {
            quote_spanned! { subclass.span() =>
                impl #pyo3_path::impl_::pyclass::PyClassBaseType for #cls {
                    type LayoutAsBase = #pyo3_path::impl_::pycell::PyClassObject<Self>;
                    type BaseNativeType = <Self as #pyo3_path::impl_::pyclass::PyClassImpl>::BaseNativeType;
                    type Initializer = #pyo3_path::pyclass_init::PyClassInitializer<Self>;
                    type PyClassMutability = <Self as #pyo3_path::impl_::pyclass::PyClassImpl>::PyClassMutability;
                }
            }
        });

        let assertions = if attr.options.unsendable.is_some() {
            TokenStream::new()
        } else {
            let assert = quote_spanned! { cls.span() => #pyo3_path::impl_::pyclass::assert_pyclass_sync::<#cls>(); };
            quote! {
                const _: () = {
                    #assert
                };
            }
        };

        let type_name = if cfg!(feature = "experimental-inspect") {
            let full_name = get_class_python_module_and_name(cls, self.attr);
            quote! { const TYPE_NAME: &'static str = #full_name; }
        } else {
            quote! {}
        };

        Ok(quote! {
            #assertions

            #pyclass_base_type_impl

            impl #pyo3_path::impl_::pyclass::PyClassImpl for #cls {
                const IS_BASETYPE: bool = #is_basetype;
                const IS_SUBCLASS: bool = #is_subclass;
                const IS_MAPPING: bool = #is_mapping;
                const IS_SEQUENCE: bool = #is_sequence;
                const IS_IMMUTABLE_TYPE: bool = #is_immutable_type;

                type BaseType = #base;
                type ThreadChecker = #thread_checker;
                #inventory
                type PyClassMutability = <<#base as #pyo3_path::impl_::pyclass::PyClassBaseType>::PyClassMutability as #pyo3_path::impl_::pycell::PyClassMutability>::#class_mutability;
                type Dict = #dict;
                type WeakRef = #weakref;
                type BaseNativeType = #base_nativetype;

                #type_name

                fn items_iter() -> #pyo3_path::impl_::pyclass::PyClassItemsIter {
                    use #pyo3_path::impl_::pyclass::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
                        methods: &[#(#default_method_defs),*],
                        slots: &[#(#default_slot_defs),* #(#freelist_slots),*],
                    };
                    PyClassItemsIter::new(&INTRINSIC_ITEMS, #pymethods_items)
                }

                const RAW_DOC: &'static ::std::ffi::CStr = #doc;

                const DOC: &'static ::std::ffi::CStr = {
                    use #pyo3_path::impl_ as impl_;
                    use impl_::pyclass::Probe as _;
                    const DOC_PIECES: &'static [&'static [u8]] = impl_::pyclass::doc::PyClassDocGenerator::<
                        #cls,
                        { impl_::pyclass::HasNewTextSignature::<#cls>::VALUE }
                    >::DOC_PIECES;
                    const LEN: usize = impl_::concat::combined_len(DOC_PIECES);
                    const DOC: &'static [u8] = &impl_::concat::combine_to_array::<LEN>(DOC_PIECES);
                    impl_::pyclass::doc::doc_bytes_as_cstr(DOC)
                };

                #dict_offset

                #weaklist_offset

                fn lazy_type_object() -> &'static #pyo3_path::impl_::pyclass::LazyTypeObject<Self> {
                    use #pyo3_path::impl_::pyclass::LazyTypeObject;
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

    fn impl_add_to_module(&self, ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        let cls = self.cls;
        quote! {
            impl #cls {
                #[doc(hidden)]
                pub const _PYO3_DEF: #pyo3_path::impl_::pymodule::AddClassToModule<Self> = #pyo3_path::impl_::pymodule::AddClassToModule::new();
            }
        }
    }

    fn impl_freelist(&self, ctx: &Ctx) -> TokenStream {
        let cls = self.cls;
        let Ctx { pyo3_path, .. } = ctx;

        self.attr.options.freelist.as_ref().map_or(quote! {}, |freelist| {
            let freelist = &freelist.value;
            quote! {
                impl #pyo3_path::impl_::pyclass::PyClassWithFreeList for #cls {
                    #[inline]
                    fn get_free_list(py: #pyo3_path::Python<'_>) -> &'static ::std::sync::Mutex<#pyo3_path::impl_::freelist::PyObjectFreeList> {
                        static FREELIST: #pyo3_path::sync::PyOnceLock<::std::sync::Mutex<#pyo3_path::impl_::freelist::PyObjectFreeList>> = #pyo3_path::sync::PyOnceLock::new();
                        &FREELIST.get_or_init(py, || ::std::sync::Mutex::new(#pyo3_path::impl_::freelist::PyObjectFreeList::with_capacity(#freelist)))
                    }
                }
            }
        })
    }

    fn freelist_slots(&self, ctx: &Ctx) -> Vec<TokenStream> {
        let Ctx { pyo3_path, .. } = ctx;
        let cls = self.cls;

        if self.attr.options.freelist.is_some() {
            vec![
                quote! {
                    #pyo3_path::ffi::PyType_Slot {
                        slot: #pyo3_path::ffi::Py_tp_alloc,
                        pfunc: #pyo3_path::impl_::pyclass::alloc_with_freelist::<#cls> as *mut _,
                    }
                },
                quote! {
                    #pyo3_path::ffi::PyType_Slot {
                        slot: #pyo3_path::ffi::Py_tp_free,
                        pfunc: #pyo3_path::impl_::pyclass::free_with_freelist::<#cls> as *mut _,
                    }
                },
            ]
        } else {
            Vec::new()
        }
    }

    #[cfg(feature = "experimental-inspect")]
    fn impl_introspection(&self, ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path, .. } = ctx;
        let name = get_class_python_name(self.cls, self.attr).to_string();
        let ident = self.cls;
        let static_introspection = class_introspection_code(pyo3_path, ident, &name);
        let introspection_id = introspection_id_const();
        quote! {
            #static_introspection
            impl #ident {
                #introspection_id
            }
        }
    }

    #[cfg(not(feature = "experimental-inspect"))]
    fn impl_introspection(&self, _ctx: &Ctx) -> TokenStream {
        quote! {}
    }
}

fn define_inventory_class(inventory_class_name: &syn::Ident, ctx: &Ctx) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;
    quote! {
        #[doc(hidden)]
        pub struct #inventory_class_name {
            items: #pyo3_path::impl_::pyclass::PyClassItems,
        }
        impl #inventory_class_name {
            pub const fn new(items: #pyo3_path::impl_::pyclass::PyClassItems) -> Self {
                Self { items }
            }
        }

        impl #pyo3_path::impl_::pyclass::PyClassInventory for #inventory_class_name {
            fn items(&self) -> &#pyo3_path::impl_::pyclass::PyClassItems {
                &self.items
            }
        }

        #pyo3_path::inventory::collect!(#inventory_class_name);
    }
}

fn generate_cfg_check(variants: &[PyClassEnumUnitVariant<'_>], cls: &syn::Ident) -> TokenStream {
    if variants.is_empty() {
        return quote! {};
    }

    let mut conditions = Vec::new();

    for variant in variants {
        let cfg_attrs = &variant.cfg_attrs;

        if cfg_attrs.is_empty() {
            // There's at least one variant of the enum without cfg attributes,
            // so the check is not necessary
            return quote! {};
        }

        for attr in cfg_attrs {
            if let syn::Meta::List(meta) = &attr.meta {
                let cfg_tokens = &meta.tokens;
                conditions.push(quote! { not(#cfg_tokens) });
            }
        }
    }

    quote_spanned! {
        cls.span() =>
        #[cfg(all(#(#conditions),*))]
        ::core::compile_error!(concat!("#[pyclass] can't be used on enums without any variants - all variants of enum `", stringify!(#cls), "` have been configured out by cfg attributes"));
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
