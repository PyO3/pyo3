// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::attributes::{self, take_pyo3_options, NameAttribute, TextSignatureAttribute};
use crate::deprecations::Deprecations;
use crate::konst::{ConstAttributes, ConstSpec};
use crate::pyimpl::{gen_py_const, PyClassMethodsType};
use crate::pymethod::{impl_py_getter_def, impl_py_setter_def, PropertyType};
use crate::utils::{self, unwrap_group, PythonDoc};
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
    pub is_immutable: bool,
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
            base: parse_quote! { ::pyo3::PyAny },
            has_dict: false,
            has_weaklist: false,
            is_gc: false,
            is_basetype: false,
            has_extends: false,
            has_unsendable: false,
            is_immutable: false,
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
            "immutable" => {
                self.is_immutable = true;
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
}

enum PyClassPyO3Option {
    TextSignature(TextSignatureAttribute),
}

impl Parse for PyClassPyO3Option {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(attributes::kw::text_signature) {
            input.parse().map(PyClassPyO3Option::TextSignature)
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
) -> syn::Result<TokenStream> {
    let pytypeinfo_impl = impl_pytypeinfo(cls, attr, Some(&deprecations));

    let py_class_impl = PyClassImplsBuilder::new(cls, attr, methods_type)
        .doc(doc)
        .impl_all();

    let descriptors = impl_descriptors(cls, field_options)?;

    Ok(quote! {
        #pytypeinfo_impl

        #py_class_impl

        #descriptors
    })
}

struct PyClassEnumVariant<'a> {
    ident: &'a syn::Ident,
    /* currently have no more options */
}

pub fn build_py_enum(
    enum_: &syn::ItemEnum,
    args: PyClassArgs,
    method_type: PyClassMethodsType,
) -> syn::Result<TokenStream> {
    let variants: Vec<PyClassEnumVariant> = enum_
        .variants
        .iter()
        .map(|v| extract_variant_data(v))
        .collect::<syn::Result<_>>()?;
    impl_enum(enum_, args, variants, method_type)
}

fn impl_enum(
    enum_: &syn::ItemEnum,
    attrs: PyClassArgs,
    variants: Vec<PyClassEnumVariant>,
    methods_type: PyClassMethodsType,
) -> syn::Result<TokenStream> {
    let enum_name = &enum_.ident;
    let doc = utils::get_doc(&enum_.attrs, None);
    let enum_cls = impl_enum_class(enum_name, &attrs, variants, doc, methods_type)?;

    Ok(quote! {
        #enum_cls
    })
}

fn impl_enum_class(
    cls: &syn::Ident,
    attr: &PyClassArgs,
    variants: Vec<PyClassEnumVariant>,
    doc: PythonDoc,
    methods_type: PyClassMethodsType,
) -> syn::Result<TokenStream> {
    let pytypeinfo = impl_pytypeinfo(cls, attr, None);
    let pyclass_impls = PyClassImplsBuilder::new(cls, attr, methods_type)
        .doc(doc)
        .impl_all();
    let descriptors = unit_variants_as_descriptors(cls, variants.iter().map(|v| v.ident));

    Ok(quote! {

        #pytypeinfo

        #pyclass_impls

        #descriptors

    })
}

fn unit_variants_as_descriptors<'a>(
    cls: &'a syn::Ident,
    variant_names: impl IntoIterator<Item = &'a syn::Ident>,
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
    let py_methods = variant_names
        .into_iter()
        .map(|var| gen_py_const(&cls_type, &variant_to_attribute(var)));

    quote! {
        impl ::pyo3::class::impl_::PyClassDescriptors<#cls>
            for ::pyo3::class::impl_::PyClassImplCollector<#cls>
        {
            fn py_class_descriptors(self) -> &'static [::pyo3::class::methods::PyMethodDefType] {
                static METHODS: &[::pyo3::class::methods::PyMethodDefType] = &[#(#py_methods),*];
                METHODS
            }
        }
    }
}

fn extract_variant_data(variant: &syn::Variant) -> syn::Result<PyClassEnumVariant> {
    use syn::Fields;
    let ident = match variant.fields {
        Fields::Unit => &variant.ident,
        _ => bail_spanned!(variant.span() => "Currently only support unit variants."),
    };
    if let Some(discriminant) = variant.discriminant.as_ref() {
        bail_spanned!(discriminant.0.span() => "Currently does not support discriminats.")
    };
    Ok(PyClassEnumVariant { ident })
}

fn impl_descriptors(
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
        impl ::pyo3::class::impl_::PyClassDescriptors<#cls>
            for ::pyo3::class::impl_::PyClassImplCollector<#cls>
        {
            fn py_class_descriptors(self) -> &'static [::pyo3::class::methods::PyMethodDefType] {
                static METHODS: &[::pyo3::class::methods::PyMethodDefType] = &[#(#py_methods),*];
                METHODS
            }
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
        unsafe impl ::pyo3::type_object::PyTypeInfo for #cls {
            type AsRefTarget = ::pyo3::PyCell<Self>;

            const NAME: &'static str = #cls_name;
            const MODULE: ::std::option::Option<&'static str> = #module;

            #[inline]
            fn type_object_raw(py: ::pyo3::Python<'_>) -> *mut ::pyo3::ffi::PyTypeObject {
                #deprecations

                use ::pyo3::type_object::LazyStaticType;
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
    doc: Option<PythonDoc>,
}

impl<'a> PyClassImplsBuilder<'a> {
    fn new(cls: &'a syn::Ident, attr: &'a PyClassArgs, methods_type: PyClassMethodsType) -> Self {
        Self {
            cls,
            attr,
            methods_type,
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
            self.impl_methods_inventory(),
            self.impl_pyclassimpl(),
            self.impl_mutability(),
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
            quote! { ::pyo3::pyclass_slots::PyClassDictSlot }
        } else if attr.has_extends {
            quote! { <Self::BaseType as ::pyo3::class::impl_::PyClassBaseType>::Dict }
        } else {
            quote! { ::pyo3::pyclass_slots::PyClassDummySlot }
        };

        // insert space for weak ref
        let weakref = if attr.has_weaklist {
            quote! { ::pyo3::pyclass_slots::PyClassWeakRefSlot }
        } else if attr.has_extends {
            quote! { <Self::BaseType as ::pyo3::class::impl_::PyClassBaseType>::WeakRef }
        } else {
            quote! { ::pyo3::pyclass_slots::PyClassDummySlot }
        };

        let base_nativetype = if attr.has_extends {
            quote! { <Self::BaseType as ::pyo3::class::impl_::PyClassBaseType>::BaseNativeType }
        } else {
            quote! { ::pyo3::PyAny }
        };



        quote! {
            impl ::pyo3::PyClass for #cls {
                type Dict = #dict;
                type WeakRef = #weakref;
                type BaseNativeType = #base_nativetype;
            }
        }
    }
    fn impl_extractext(&self) -> TokenStream {
        let cls = self.cls;
        if self.attr.is_immutable {
            quote! {
                impl<'a> ::pyo3::derive_utils::ExtractExt<'a> for &'a #cls
                {
                    type Target = ::pyo3::PyRef<'a, #cls>;
                }
            }
        } else{
            quote! {
                impl<'a> ::pyo3::derive_utils::ExtractExt<'a> for &'a #cls
                {
                    type Target = ::pyo3::PyRef<'a, #cls>;
                }

                impl<'a> ::pyo3::derive_utils::ExtractExt<'a> for &'a mut #cls
                {
                    type Target = ::pyo3::PyRefMut<'a, #cls>;
                }
            }
        }
    }

    fn impl_into_py(&self) -> TokenStream {
        let cls = self.cls;
        let attr = self.attr;
        // If #cls is not extended type, we allow Self->PyObject conversion
        if !attr.has_extends {
            quote! {
                impl ::pyo3::IntoPy<::pyo3::PyObject> for #cls {
                    fn into_py(self, py: ::pyo3::Python) -> ::pyo3::PyObject {
                        ::pyo3::IntoPy::into_py(::pyo3::Py::new(py, self).unwrap(), py)
                    }
                }
            }
        } else {
            quote! {}
        }
    }

    /// To allow multiple #[pymethods] block, we define inventory types.
    fn impl_methods_inventory(&self) -> TokenStream {
        let cls = self.cls;
        let methods_type = self.methods_type;
        match methods_type {
            PyClassMethodsType::Specialization => quote! {},
            PyClassMethodsType::Inventory => {
                // Try to build a unique type for better error messages
                let name = format!("Pyo3MethodsInventoryFor{}", cls.unraw());
                let inventory_cls = syn::Ident::new(&name, Span::call_site());

                quote! {
                    #[doc(hidden)]
                    pub struct #inventory_cls {
                        methods: ::std::vec::Vec<::pyo3::class::PyMethodDefType>,
                        slots: ::std::vec::Vec<::pyo3::ffi::PyType_Slot>,
                    }
                    impl ::pyo3::class::impl_::PyMethodsInventory for #inventory_cls {
                        fn new(
                            methods: ::std::vec::Vec<::pyo3::class::PyMethodDefType>,
                            slots: ::std::vec::Vec<::pyo3::ffi::PyType_Slot>,
                        ) -> Self {
                            Self { methods, slots }
                        }
                        fn methods(&'static self) -> &'static [::pyo3::class::PyMethodDefType] {
                            &self.methods
                        }
                        fn slots(&'static self) -> &'static [::pyo3::ffi::PyType_Slot] {
                            &self.slots
                        }
                    }

                    impl ::pyo3::class::impl_::HasMethodsInventory for #cls {
                        type Methods = #inventory_cls;
                    }

                    ::pyo3::inventory::collect!(#inventory_cls);
                }
            }
        }
    }
    fn impl_pyclassimpl(&self) -> TokenStream {
        let cls = self.cls;
        let doc = self.doc.as_ref().map_or(quote! {"\0"}, |doc| quote! {#doc});
        let is_gc = self.attr.is_gc;
        let is_basetype = self.attr.is_basetype;
        let base = &self.attr.base;
        let is_subclass = self.attr.has_extends;

        let thread_checker = if self.attr.has_unsendable {
            quote! { ::pyo3::class::impl_::ThreadCheckerImpl<#cls> }
        } else if self.attr.has_extends {
            quote! {
                ::pyo3::class::impl_::ThreadCheckerInherited<#cls, <#cls as ::pyo3::class::impl_::PyClassImpl>::BaseType>
            }
        } else {
            quote! { ::pyo3::class::impl_::ThreadCheckerStub<#cls> }
        };

        let methods_protos = match self.methods_type {
            PyClassMethodsType::Specialization => {
                quote! { visitor(collector.methods_protocol_slots()); }
            }
            PyClassMethodsType::Inventory => {
                quote! {
                    for inventory in ::pyo3::inventory::iter::<<Self as ::pyo3::class::impl_::HasMethodsInventory>::Methods>() {
                        visitor(::pyo3::class::impl_::PyMethodsInventory::slots(inventory));
                    }
                }
            }
        };
        let for_each_py_method = match self.methods_type {
            PyClassMethodsType::Specialization => quote! { visitor(collector.py_methods()); },
            PyClassMethodsType::Inventory => quote! {
                for inventory in ::pyo3::inventory::iter::<<Self as ::pyo3::class::impl_::HasMethodsInventory>::Methods>() {
                    visitor(::pyo3::class::impl_::PyMethodsInventory::methods(inventory));
                }
            },
        };
        quote! {
            impl ::pyo3::class::impl_::PyClassImpl for #cls {
                const DOC: &'static str = #doc;
                const IS_GC: bool = #is_gc;
                const IS_BASETYPE: bool = #is_basetype;
                const IS_SUBCLASS: bool = #is_subclass;

                type Layout = ::pyo3::PyCell<Self>;
                type BaseType = #base;
                type ThreadChecker = #thread_checker;

                fn for_each_method_def(visitor: &mut dyn ::std::ops::FnMut(&[::pyo3::class::PyMethodDefType])) {
                    use ::pyo3::class::impl_::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    #for_each_py_method;
                    visitor(collector.py_class_descriptors());
                    visitor(collector.object_protocol_methods());
                    visitor(collector.async_protocol_methods());
                    visitor(collector.descr_protocol_methods());
                    visitor(collector.mapping_protocol_methods());
                    visitor(collector.number_protocol_methods());
                }
                fn get_new() -> ::std::option::Option<::pyo3::ffi::newfunc> {
                    use ::pyo3::class::impl_::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    collector.new_impl()
                }
                fn get_alloc() -> ::std::option::Option<::pyo3::ffi::allocfunc> {
                    use ::pyo3::class::impl_::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    collector.alloc_impl()
                }
                fn get_free() -> ::std::option::Option<::pyo3::ffi::freefunc> {
                    use ::pyo3::class::impl_::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    collector.free_impl()
                }

                fn for_each_proto_slot(visitor: &mut dyn ::std::ops::FnMut(&[::pyo3::ffi::PyType_Slot])) {
                    // Implementation which uses dtolnay specialization to load all slots.
                    use ::pyo3::class::impl_::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    visitor(collector.object_protocol_slots());
                    visitor(collector.number_protocol_slots());
                    visitor(collector.iter_protocol_slots());
                    visitor(collector.gc_protocol_slots());
                    visitor(collector.descr_protocol_slots());
                    visitor(collector.mapping_protocol_slots());
                    visitor(collector.sequence_protocol_slots());
                    visitor(collector.async_protocol_slots());
                    visitor(collector.buffer_protocol_slots());
                    #methods_protos
                }

                fn get_buffer() -> ::std::option::Option<&'static ::pyo3::class::impl_::PyBufferProcs> {
                    use ::pyo3::class::impl_::*;
                    let collector = PyClassImplCollector::<Self>::new();
                    collector.buffer_procs()
                }
            }
        }
    }

    fn impl_mutability(&self) -> TokenStream {
        let cls = self.cls;
        if self.attr.is_immutable {
            quote! {
                unsafe impl ::pyo3::pyclass::ImmutablePyClass for #cls {}
    
                unsafe impl ::pyo3::class::impl_::BorrowImpl for #cls {
                    fn get_borrow_flag() -> for<'r> fn(&'r ::pyo3::pycell::PyCell<Self>) -> ::pyo3::pycell::BorrowFlag
                    where Self: ::pyo3::PyClass
                    {
                        ::pyo3::pycell::impl_::get_borrow_flag_dummy
                    }
                    fn increment_borrow_flag() -> for<'r> fn(&'r ::pyo3::pycell::PyCell<Self>, ::pyo3::pycell::BorrowFlag)
                    where Self: ::pyo3::PyClass
                    {
                        ::pyo3::pycell::impl_::increment_borrow_flag_dummy
                    }
                    fn decrement_borrow_flag() -> for<'r> fn(&'r ::pyo3::pycell::PyCell<Self>, ::pyo3::pycell::BorrowFlag)
                    where Self: ::pyo3::PyClass
                    {
                        ::pyo3::pycell::impl_::decrement_borrow_flag_dummy
                    }
                }
            }
        } else {
            quote! {
                unsafe impl ::pyo3::pyclass::MutablePyClass for #cls {}
    
                unsafe impl ::pyo3::class::impl_::BorrowImpl for #cls {}
            }
        }
    }

    fn impl_freelist(&self) -> TokenStream {
        let cls = self.cls;

        self.attr.freelist.as_ref().map_or(quote!{}, |freelist| {
            quote! {
                impl ::pyo3::class::impl_::PyClassWithFreeList for #cls {
                    #[inline]
                    fn get_free_list(_py: ::pyo3::Python<'_>) -> &mut ::pyo3::impl_::freelist::FreeList<*mut ::pyo3::ffi::PyObject> {
                        static mut FREELIST: *mut ::pyo3::impl_::freelist::FreeList<*mut ::pyo3::ffi::PyObject> = 0 as *mut _;
                        unsafe {
                            if FREELIST.is_null() {
                                FREELIST = ::std::boxed::Box::into_raw(::std::boxed::Box::new(
                                    ::pyo3::impl_::freelist::FreeList::with_capacity(#freelist)));
                            }
                            &mut *FREELIST
                        }
                    }
                }

                impl ::pyo3::class::impl_::PyClassAllocImpl<#cls> for ::pyo3::class::impl_::PyClassImplCollector<#cls> {
                    #[inline]
                    fn alloc_impl(self) -> ::std::option::Option<::pyo3::ffi::allocfunc> {
                        ::std::option::Option::Some(::pyo3::class::impl_::alloc_with_freelist::<#cls>)
                    }
                }

                impl ::pyo3::class::impl_::PyClassFreeImpl<#cls> for ::pyo3::class::impl_::PyClassImplCollector<#cls> {
                    #[inline]
                    fn free_impl(self) -> ::std::option::Option<::pyo3::ffi::freefunc> {
                        ::std::option::Option::Some(::pyo3::class::impl_::free_with_freelist::<#cls>)
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
                    use ::pyo3::class;

                    fn _assert_implements_protocol<'p, T: ::pyo3::class::PyGCProtocol<'p>>() {}
                    _assert_implements_protocol::<#cls>();
                }
            }
        } else {
            quote! {}
        }
    }
}