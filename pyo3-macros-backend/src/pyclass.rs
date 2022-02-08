// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::attributes::{
    self, take_pyo3_options, CrateAttribute, NameAttribute, TextSignatureAttribute,
};
use crate::deprecations::Deprecations;
use crate::konst::{ConstAttributes, ConstSpec};
use crate::pyimpl::{gen_default_items, gen_py_const, PyClassMethodsType};
use crate::pymethod::{impl_py_getter_def, impl_py_setter_def, PropertyType};
use crate::utils::{self, get_pyo3_crate, unwrap_group, PythonDoc};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_quote, spanned::Spanned, Expr, Result, Token}; //unraw

/// If the class is derived from a Rust `struct` or `enum`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PyClassKind {
    Struct,
    Enum,
}

/// The parsed arguments of the pyclass macro
pub struct PyClassArgs {
    pub freelist: Option<syn::Expr>,
    pub name: Option<syn::Ident>,
    pub base: syn::TypePath,
    pub has_dict: bool,
    pub has_weaklist: bool,
    pub is_gc: bool,
    pub is_basetype: bool,
    pub has_extends: bool,
    pub has_unsendable: bool,
    pub module: Option<syn::LitStr>,
    pub class_kind: PyClassKind,
}

impl PyClassArgs {
    fn parse(input: ParseStream, kind: PyClassKind) -> Result<Self> {
        let mut slf = PyClassArgs::new(kind);
        let vars = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        for expr in vars {
            slf.add_expr(&expr)?;
        }
        Ok(slf)
    }

    pub fn parse_stuct_args(input: ParseStream) -> syn::Result<Self> {
        Self::parse(input, PyClassKind::Struct)
    }

    pub fn parse_enum_args(input: ParseStream) -> syn::Result<Self> {
        Self::parse(input, PyClassKind::Enum)
    }

    fn new(class_kind: PyClassKind) -> Self {
        PyClassArgs {
            freelist: None,
            name: None,
            module: None,
            base: parse_quote! { _pyo3::PyAny },
            has_dict: false,
            has_weaklist: false,
            is_gc: false,
            is_basetype: false,
            has_extends: false,
            has_unsendable: false,
            class_kind,
        }
    }

    /// Adda single expression from the comma separated list in the attribute, which is
    /// either a single word or an assignment expression
    fn add_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            syn::Expr::Path(exp) if exp.path.segments.len() == 1 => self.add_path(exp),
            syn::Expr::Assign(assign) => self.add_assign(assign),
            _ => bail_spanned!(expr.span() => "failed to parse arguments"),
        }
    }

    /// Match a key/value flag
    fn add_assign(&mut self, assign: &syn::ExprAssign) -> syn::Result<()> {
        let syn::ExprAssign { left, right, .. } = assign;
        let key = match &**left {
            syn::Expr::Path(exp) if exp.path.segments.len() == 1 => {
                exp.path.segments.first().unwrap().ident.to_string()
            }
            _ => bail_spanned!(assign.span() => "failed to parse arguments"),
        };

        macro_rules! expected {
            ($expected: literal) => {
                expected!($expected, right.span())
            };
            ($expected: literal, $span: expr) => {
                bail_spanned!($span => concat!("expected ", $expected))
            };
        }

        match key.as_str() {
            "freelist" => {
                // We allow arbitrary expressions here so you can e.g. use `8*64`
                self.freelist = Some(syn::Expr::clone(right));
            }
            "name" => match unwrap_group(&**right) {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) => {
                    self.name = Some(lit.parse().map_err(|_| {
                        err_spanned!(
                                lit.span() => "expected a single identifier in double-quotes")
                    })?);
                }
                syn::Expr::Path(exp) if exp.path.segments.len() == 1 => {
                    bail_spanned!(
                        exp.span() => format!(
                            "since PyO3 0.13 a pyclass name should be in double-quotes, \
                            e.g. \"{}\"",
                            exp.path.get_ident().expect("path has 1 segment")
                        )
                    );
                }
                _ => expected!("type name (e.g. \"Name\")"),
            },
            "extends" => match unwrap_group(&**right) {
                syn::Expr::Path(exp) => {
                    if self.class_kind == PyClassKind::Enum {
                        bail_spanned!( assign.span() =>  "enums cannot extend from other classes" );
                    }
                    self.base = syn::TypePath {
                        path: exp.path.clone(),
                        qself: None,
                    };
                    self.has_extends = true;
                }
                _ => expected!("type path (e.g., my_mod::BaseClass)"),
            },
            "module" => match unwrap_group(&**right) {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) => {
                    self.module = Some(lit.clone());
                }
                _ => expected!(r#"string literal (e.g., "my_mod")"#),
            },
            _ => expected!("one of freelist/name/extends/module", left.span()),
        };

        Ok(())
    }

    /// Match a single flag
    fn add_path(&mut self, exp: &syn::ExprPath) -> syn::Result<()> {
        let flag = exp.path.segments.first().unwrap().ident.to_string();
        match flag.as_str() {
            "gc" => {
                self.is_gc = true;
            }
            "weakref" => {
                self.has_weaklist = true;
            }
            "subclass" => {
                if self.class_kind == PyClassKind::Enum {
                    bail_spanned!(exp.span() => "enums can't be inherited by other classes");
                }
                self.is_basetype = true;
            }
            "dict" => {
                self.has_dict = true;
            }
            "unsendable" => {
                self.has_unsendable = true;
            }
            _ => bail_spanned!(
                exp.path.span() => "expected one of gc/weakref/subclass/dict/unsendable"
            ),
        };
        Ok(())
    }
}

#[derive(Default)]
pub struct PyClassPyO3Options {
    pub text_signature: Option<TextSignatureAttribute>,
    pub deprecations: Deprecations,
    pub krate: Option<CrateAttribute>,
}

enum PyClassPyO3Option {
    TextSignature(TextSignatureAttribute),
    Crate(CrateAttribute),
}

impl Parse for PyClassPyO3Option {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::text_signature) {
            input.parse().map(PyClassPyO3Option::TextSignature)
        } else if lookahead.peek(Token![crate]) {
            input.parse().map(PyClassPyO3Option::Crate)
        } else {
            Err(lookahead.error())
        }
    }
}

impl PyClassPyO3Options {
    pub fn take_pyo3_options(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut options: PyClassPyO3Options = Default::default();
        for option in take_pyo3_options(attrs)? {
            match option {
                PyClassPyO3Option::TextSignature(text_signature) => {
                    options.set_text_signature(text_signature)?;
                }
                PyClassPyO3Option::Crate(path) => {
                    options.set_crate(path)?;
                }
            }
        }
        Ok(options)
    }

    pub fn set_text_signature(
        &mut self,
        text_signature: TextSignatureAttribute,
    ) -> syn::Result<()> {
        ensure_spanned!(
            self.text_signature.is_none(),
            text_signature.kw.span() => "`text_signature` may only be specified once"
        );
        self.text_signature = Some(text_signature);
        Ok(())
    }

    pub fn set_crate(&mut self, path: CrateAttribute) -> syn::Result<()> {
        ensure_spanned!(
            self.krate.is_none(),
            path.0.span() => "`text_signature` may only be specified once"
        );
        self.krate = Some(path);
        Ok(())
    }
}

pub fn build_py_class(
    class: &mut syn::ItemStruct,
    args: &PyClassArgs,
    methods_type: PyClassMethodsType,
) -> syn::Result<TokenStream> {
    let options = PyClassPyO3Options::take_pyo3_options(&mut class.attrs)?;
    let doc = utils::get_doc(
        &class.attrs,
        options
            .text_signature
            .as_ref()
            .map(|attr| (get_class_python_name(&class.ident, args), attr)),
    );
    let krate = get_pyo3_crate(&options.krate);

    ensure_spanned!(
        class.generics.params.is_empty(),
        class.generics.span() => "#[pyclass] cannot have generic parameters"
    );

    let field_options = match &mut class.fields {
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
            // No fields for unit struct
            Vec::new()
        }
    };

    impl_class(
        &class.ident,
        args,
        doc,
        field_options,
        methods_type,
        options.deprecations,
        krate,
    )
}

/// `#[pyo3()]` options for pyclass fields
struct FieldPyO3Options {
    get: bool,
    set: bool,
    name: Option<NameAttribute>,
}

enum FieldPyO3Option {
    Get(attributes::kw::get),
    Set(attributes::kw::set),
    Name(NameAttribute),
}

impl Parse for FieldPyO3Option {
    fn parse(input: ParseStream) -> Result<Self> {
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
            get: false,
            set: false,
            name: None,
        };

        for option in take_pyo3_options(attrs)? {
            match option {
                FieldPyO3Option::Get(kw) => {
                    ensure_spanned!(
                        !options.get,
                        kw.span() => "`get` may only be specified once"
                    );
                    options.get = true;
                }
                FieldPyO3Option::Set(kw) => {
                    ensure_spanned!(
                        !options.set,
                        kw.span() => "`set` may only be specified once"
                    );
                    options.set = true;
                }
                FieldPyO3Option::Name(name) => {
                    ensure_spanned!(
                        options.name.is_none(),
                        name.0.span() => "`name` may only be specified once"
                    );
                    options.name = Some(name);
                }
            }
        }

        Ok(options)
    }
}

fn get_class_python_name<'a>(cls: &'a syn::Ident, attr: &'a PyClassArgs) -> &'a syn::Ident {
    attr.name.as_ref().unwrap_or(cls)
}

fn impl_class(
    cls: &syn::Ident,
    attr: &PyClassArgs,
    doc: PythonDoc,
    field_options: Vec<(&syn::Field, FieldPyO3Options)>,
    methods_type: PyClassMethodsType,
    deprecations: Deprecations,
    krate: syn::Path,
) -> syn::Result<TokenStream> {
    let pytypeinfo_impl = impl_pytypeinfo(cls, attr, Some(&deprecations));

    let py_class_impl = PyClassImplsBuilder::new(
        cls,
        attr,
        methods_type,
        descriptors_to_items(cls, field_options)?,
    )
    .doc(doc)
    .impl_all();

    Ok(quote! {
        const _: () = {
            use #krate as _pyo3;

            #pytypeinfo_impl

            #py_class_impl
        };
    })
}

struct PyClassEnumVariant<'a> {
    ident: &'a syn::Ident,
    /* currently have no more options */
}

struct PyClassEnum<'a> {
    ident: &'a syn::Ident,
    // The underlying #[repr] of the enum, used to implement __int__ and __richcmp__.
    // This matters when the underlying representation may not fit in `isize`.
    repr_type: syn::Ident,
    variants: Vec<PyClassEnumVariant<'a>>,
}

impl<'a> PyClassEnum<'a> {
    fn new(enum_: &'a syn::ItemEnum) -> syn::Result<Self> {
        fn is_numeric_type(t: &syn::Ident) -> bool {
            [
                "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "u128", "i128", "usize",
                "isize",
            ]
            .iter()
            .any(|&s| t == s)
        }
        let ident = &enum_.ident;
        // According to the [reference](https://doc.rust-lang.org/reference/items/enumerations.html),
        // "Under the default representation, the specified discriminant is interpreted as an isize
        // value", so `isize` should be enough by default.
        let mut repr_type = syn::Ident::new("isize", proc_macro2::Span::call_site());
        if let Some(attr) = enum_.attrs.iter().find(|attr| attr.path.is_ident("repr")) {
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

        let variants = enum_
            .variants
            .iter()
            .map(extract_variant_data)
            .collect::<syn::Result<_>>()?;
        Ok(Self {
            ident,
            repr_type,
            variants,
        })
    }
}

pub fn build_py_enum(
    enum_: &mut syn::ItemEnum,
    args: &PyClassArgs,
    method_type: PyClassMethodsType,
) -> syn::Result<TokenStream> {
    let options = PyClassPyO3Options::take_pyo3_options(&mut enum_.attrs)?;

    if enum_.variants.is_empty() {
        bail_spanned!(enum_.brace_token.span => "Empty enums can't be #[pyclass].");
    }
    let doc = utils::get_doc(
        &enum_.attrs,
        options
            .text_signature
            .as_ref()
            .map(|attr| (get_class_python_name(&enum_.ident, args), attr)),
    );
    let enum_ = PyClassEnum::new(enum_)?;
    impl_enum(enum_, args, doc, method_type, options)
}

fn impl_enum(
    enum_: PyClassEnum,
    args: &PyClassArgs,
    doc: PythonDoc,
    methods_type: PyClassMethodsType,
    options: PyClassPyO3Options,
) -> syn::Result<TokenStream> {
    let krate = get_pyo3_crate(&options.krate);
    impl_enum_class(enum_, args, doc, methods_type, krate)
}

fn impl_enum_class(
    enum_: PyClassEnum,
    args: &PyClassArgs,
    doc: PythonDoc,
    methods_type: PyClassMethodsType,
    krate: syn::Path,
) -> syn::Result<TokenStream> {
    let cls = enum_.ident;
    let variants = enum_.variants;
    let pytypeinfo = impl_pytypeinfo(cls, args, None);

    let default_repr_impl: syn::ImplItemMethod = {
        let variants_repr = variants.iter().map(|variant| {
            let variant_name = variant.ident;
            // Assuming all variants are unit variants because they are the only type we support.
            let repr = format!("{}.{}", cls, variant_name);
            quote! { #cls::#variant_name => #repr, }
        });
        syn::parse_quote! {
            #[doc(hidden)]
            #[allow(non_snake_case)]
            #[pyo3(name = "__repr__")]
            fn __pyo3__repr__(&self) -> &'static str {
                match self {
                    #(#variants_repr)*
                }
            }
        }
    };

    let repr_type = &enum_.repr_type;

    let default_int = {
        // This implementation allows us to convert &T to #repr_type without implementing `Copy`
        let variants_to_int = variants.iter().map(|variant| {
            let variant_name = variant.ident;
            quote! { #cls::#variant_name => #cls::#variant_name as #repr_type, }
        });
        syn::parse_quote! {
            #[doc(hidden)]
            #[allow(non_snake_case)]
            #[pyo3(name = "__int__")]
            fn __pyo3__int__(&self) -> #repr_type {
                match self {
                    #(#variants_to_int)*
                }
            }
        }
    };

    let default_richcmp = {
        let variants_eq = variants.iter().map(|variant| {
            let variant_name = variant.ident;
            quote! {
                (#cls::#variant_name, #cls::#variant_name) =>
                    Ok(true.to_object(py)),
            }
        });
        syn::parse_quote! {
            #[doc(hidden)]
            #[allow(non_snake_case)]
            #[pyo3(name = "__richcmp__")]
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
                        if let Ok(i) = other.extract::<#repr_type>() {
                            let self_val = self.__pyo3__int__();
                            return Ok((self_val == i).to_object(py));
                        }
                        let other = other.extract::<_pyo3::PyRef<Self>>()?;
                        let other = &*other;
                        match (self, other) {
                            #(#variants_eq)*
                            _ => Ok(false.to_object(py)),
                        }
                    }
                    _ => Ok(py.NotImplemented()),
                }
            }
        }
    };

    let mut default_methods = vec![default_repr_impl, default_richcmp, default_int];

    let pyclass_impls = PyClassImplsBuilder::new(
        cls,
        args,
        methods_type,
        enum_default_items(cls, variants.iter().map(|v| v.ident), &mut default_methods),
    )
    .doc(doc)
    .impl_all();

    Ok(quote! {
        const _: () = {
            use #krate as _pyo3;

            #pytypeinfo

            #pyclass_impls

            impl #cls {
                #(#default_methods)*
            }
        };
    })
}

fn enum_default_items<'a>(
    cls: &'a syn::Ident,
    unit_variant_names: impl IntoIterator<Item = &'a syn::Ident>,
    default_items: &mut [syn::ImplItemMethod],
) -> TokenStream {
    let cls_type = syn::parse_quote!(#cls);
    let variant_to_attribute = |ident: &syn::Ident| ConstSpec {
        rust_ident: ident.clone(),
        attributes: ConstAttributes {
            is_class_attr: true,
            name: Some(NameAttribute(ident.clone())),
            deprecations: Default::default(),
        },
    };
    let py_methods = unit_variant_names
        .into_iter()
        .map(|var| gen_py_const(&cls_type, &variant_to_attribute(var)));

    let slots = gen_default_items(cls, default_items);

    quote! {
        _pyo3::impl_::pyclass::PyClassItems {
            methods: &[#(#py_methods),*],
            slots: &[#(#slots),*]
        }
    }
}

fn extract_variant_data(variant: &syn::Variant) -> syn::Result<PyClassEnumVariant> {
    use syn::Fields;
    let ident = match variant.fields {
        Fields::Unit => &variant.ident,
        _ => bail_spanned!(variant.span() => "Currently only support unit variants."),
    };
    Ok(PyClassEnumVariant { ident })
}

fn descriptors_to_items(
    cls: &syn::Ident,
    field_options: Vec<(&syn::Field, FieldPyO3Options)>,
) -> syn::Result<TokenStream> {
    let ty = syn::parse_quote!(#cls);
    let py_methods: Vec<TokenStream> = field_options
        .into_iter()
        .enumerate()
        .flat_map(|(field_index, (field, options))| {
            let name_err = if options.name.is_some() && !options.get && !options.set {
                Some(Err(err_spanned!(options.name.as_ref().unwrap().0.span() => "`name` is useless without `get` or `set`")))
            } else {
                None
            };

            let getter = if options.get {
                Some(impl_py_getter_def(&ty, PropertyType::Descriptor {
                    field_index,
                    field,
                    python_name: options.name.as_ref()
                }))
            } else {
                None
            };

            let setter = if options.set {
                Some(impl_py_setter_def(&ty, PropertyType::Descriptor {
                    field_index,
                    field,
                    python_name: options.name.as_ref()
                }))
            } else {
                None
            };

            name_err.into_iter().chain(getter).chain(setter)
        })
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        _pyo3::impl_::pyclass::PyClassItems {
            methods: &[#(#py_methods),*],
            slots: &[]
        }
    })
}

fn impl_pytypeinfo(
    cls: &syn::Ident,
    attr: &PyClassArgs,
    deprecations: Option<&Deprecations>,
) -> TokenStream {
    let cls_name = get_class_python_name(cls, attr).to_string();

    let module = if let Some(m) = &attr.module {
        quote! { ::core::option::Option::Some(#m) }
    } else {
        quote! { ::core::option::Option::None }
    };

    quote! {
        unsafe impl _pyo3::type_object::PyTypeInfo for #cls {
            type AsRefTarget = _pyo3::PyCell<Self>;

            const NAME: &'static str = #cls_name;
            const MODULE: ::std::option::Option<&'static str> = #module;

            #[inline]
            fn type_object_raw(py: _pyo3::Python<'_>) -> *mut _pyo3::ffi::PyTypeObject {
                #deprecations

                use _pyo3::type_object::LazyStaticType;
                static TYPE_OBJECT: LazyStaticType = LazyStaticType::new();
                TYPE_OBJECT.get_or_init::<Self>(py)
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
    default_items: TokenStream,
    doc: Option<PythonDoc>,
}

impl<'a> PyClassImplsBuilder<'a> {
    fn new(
        cls: &'a syn::Ident,
        attr: &'a PyClassArgs,
        methods_type: PyClassMethodsType,
        default_items: TokenStream,
    ) -> Self {
        Self {
            cls,
            attr,
            methods_type,
            default_items,
            doc: None,
        }
    }

    fn doc(self, doc: PythonDoc) -> Self {
        Self {
            doc: Some(doc),
            ..self
        }
    }

    fn impl_all(&self) -> TokenStream {
        vec![
            self.impl_pyclass(),
            self.impl_extractext(),
            self.impl_into_py(),
            self.impl_pyclassimpl(),
            self.impl_freelist(),
            self.impl_gc(),
        ]
        .into_iter()
        .collect()
    }

    fn impl_pyclass(&self) -> TokenStream {
        let cls = self.cls;
        let attr = self.attr;
        let dict = if attr.has_dict {
            quote! { _pyo3::impl_::pyclass::PyClassDictSlot }
        } else {
            quote! { _pyo3::impl_::pyclass::PyClassDummySlot }
        };

        // insert space for weak ref
        let weakref = if attr.has_weaklist {
            quote! { _pyo3::impl_::pyclass::PyClassWeakRefSlot }
        } else {
            quote! { _pyo3::impl_::pyclass::PyClassDummySlot }
        };

        let base_nativetype = if attr.has_extends {
            quote! { <Self::BaseType as _pyo3::impl_::pyclass::PyClassBaseType>::BaseNativeType }
        } else {
            quote! { _pyo3::PyAny }
        };
        quote! {
            impl _pyo3::PyClass for #cls {
                type Dict = #dict;
                type WeakRef = #weakref;
                type BaseNativeType = #base_nativetype;
            }
        }
    }
    fn impl_extractext(&self) -> TokenStream {
        let cls = self.cls;
        quote! {
            impl<'a> _pyo3::derive_utils::ExtractExt<'a> for &'a #cls
            {
                type Target = _pyo3::PyRef<'a, #cls>;
            }

            impl<'a> _pyo3::derive_utils::ExtractExt<'a> for &'a mut #cls
            {
                type Target = _pyo3::PyRefMut<'a, #cls>;
            }
        }
    }

    fn impl_into_py(&self) -> TokenStream {
        let cls = self.cls;
        let attr = self.attr;
        // If #cls is not extended type, we allow Self->PyObject conversion
        if !attr.has_extends {
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
    fn impl_pyclassimpl(&self) -> TokenStream {
        let cls = self.cls;
        let doc = self.doc.as_ref().map_or(quote! {"\0"}, |doc| quote! {#doc});
        let is_gc = self.attr.is_gc;
        let is_basetype = self.attr.is_basetype;
        let base = &self.attr.base;
        let is_subclass = self.attr.has_extends;

        let dict_offset = if self.attr.has_dict {
            quote! {
                fn dict_offset() -> ::std::option::Option<_pyo3::ffi::Py_ssize_t> {
                    ::std::option::Option::Some(_pyo3::impl_::pyclass::dict_offset::<Self>())
                }
            }
        } else {
            TokenStream::new()
        };

        // insert space for weak ref
        let weaklist_offset = if self.attr.has_weaklist {
            quote! {
                fn weaklist_offset() -> ::std::option::Option<_pyo3::ffi::Py_ssize_t> {
                    ::std::option::Option::Some(_pyo3::impl_::pyclass::weaklist_offset::<Self>())
                }
            }
        } else {
            TokenStream::new()
        };

        let thread_checker = if self.attr.has_unsendable {
            quote! { _pyo3::impl_::pyclass::ThreadCheckerImpl<#cls> }
        } else if self.attr.has_extends {
            quote! {
                _pyo3::impl_::pyclass::ThreadCheckerInherited<#cls, <#cls as _pyo3::impl_::pyclass::PyClassImpl>::BaseType>
            }
        } else {
            quote! { _pyo3::impl_::pyclass::ThreadCheckerStub<#cls> }
        };

        let (pymethods_items, inventory, inventory_class) = match self.methods_type {
            PyClassMethodsType::Specialization => {
                (quote! { visitor(collector.py_methods()); }, None, None)
            }
            PyClassMethodsType::Inventory => {
                // To allow multiple #[pymethods] block, we define inventory types.
                let inventory_class_name = syn::Ident::new(
                    &format!("Pyo3MethodsInventoryFor{}", cls.unraw()),
                    Span::call_site(),
                );
                (
                    quote! {
                        for inventory in _pyo3::inventory::iter::<<Self as _pyo3::impl_::pyclass::PyClassImpl>::Inventory>() {
                            visitor(_pyo3::impl_::pyclass::PyClassInventory::items(inventory));
                        }
                    },
                    Some(quote! { type Inventory = #inventory_class_name; }),
                    Some(define_inventory_class(&inventory_class_name)),
                )
            }
        };

        let pyproto_items = if cfg!(feature = "pyproto") {
            Some(quote! {
                visitor(collector.object_protocol_items());
                visitor(collector.number_protocol_items());
                visitor(collector.iter_protocol_items());
                visitor(collector.gc_protocol_items());
                visitor(collector.descr_protocol_items());
                visitor(collector.mapping_protocol_items());
                visitor(collector.sequence_protocol_items());
                visitor(collector.async_protocol_items());
                visitor(collector.buffer_protocol_items());
            })
        } else {
            None
        };

        let default_items = &self.default_items;

        quote! {
            impl _pyo3::impl_::pyclass::PyClassImpl for #cls {
                const DOC: &'static str = #doc;
                const IS_GC: bool = #is_gc;
                const IS_BASETYPE: bool = #is_basetype;
                const IS_SUBCLASS: bool = #is_subclass;

                type Layout = _pyo3::PyCell<Self>;
                type BaseType = #base;
                type ThreadChecker = #thread_checker;
                #inventory

                fn for_all_items(visitor: &mut dyn ::std::ops::FnMut(& _pyo3::impl_::pyclass::PyClassItems)) {
                    use _pyo3::impl_::pyclass::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    static INTRINSIC_ITEMS: PyClassItems = #default_items;
                    visitor(&INTRINSIC_ITEMS);
                    #pymethods_items
                    #pyproto_items
                }
                fn get_new() -> ::std::option::Option<_pyo3::ffi::newfunc> {
                    use _pyo3::impl_::pyclass::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    collector.new_impl()
                }
                fn get_alloc() -> ::std::option::Option<_pyo3::ffi::allocfunc> {
                    use _pyo3::impl_::pyclass::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    collector.alloc_impl()
                }
                fn get_free() -> ::std::option::Option<_pyo3::ffi::freefunc> {
                    use _pyo3::impl_::pyclass::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    collector.free_impl()
                }

                #dict_offset

                #weaklist_offset
            }

            #inventory_class
        }
    }

    fn impl_freelist(&self) -> TokenStream {
        let cls = self.cls;

        self.attr.freelist.as_ref().map_or(quote!{}, |freelist| {
            quote! {
                impl _pyo3::impl_::pyclass::PyClassWithFreeList for #cls {
                    #[inline]
                    fn get_free_list(_py: _pyo3::Python<'_>) -> &mut _pyo3::impl_::freelist::FreeList<*mut _pyo3::ffi::PyObject> {
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

                impl _pyo3::impl_::pyclass::PyClassAllocImpl<#cls> for _pyo3::impl_::pyclass::PyClassImplCollector<#cls> {
                    #[inline]
                    fn alloc_impl(self) -> ::std::option::Option<_pyo3::ffi::allocfunc> {
                        ::std::option::Option::Some(_pyo3::impl_::pyclass::alloc_with_freelist::<#cls>)
                    }
                }

                impl _pyo3::impl_::pyclass::PyClassFreeImpl<#cls> for _pyo3::impl_::pyclass::PyClassImplCollector<#cls> {
                    #[inline]
                    fn free_impl(self) -> ::std::option::Option<_pyo3::ffi::freefunc> {
                        ::std::option::Option::Some(_pyo3::impl_::pyclass::free_with_freelist::<#cls>)
                    }
                }
            }
        })
    }
    /// Enforce at compile time that PyGCProtocol is implemented
    fn impl_gc(&self) -> TokenStream {
        let cls = self.cls;
        let attr = self.attr;
        if attr.is_gc {
            let closure_name = format!("__assertion_closure_{}", cls);
            let closure_token = syn::Ident::new(&closure_name, Span::call_site());
            quote! {
                fn #closure_token() {
                    use _pyo3::class;

                    fn _assert_implements_protocol<'p, T: _pyo3::class::PyGCProtocol<'p>>() {}
                    _assert_implements_protocol::<#cls>();
                }
            }
        } else {
            quote! {}
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
            const fn new(items: _pyo3::impl_::pyclass::PyClassItems) -> Self {
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
