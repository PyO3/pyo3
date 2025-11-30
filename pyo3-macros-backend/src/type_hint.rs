//! Define a data structure for Python type hints, mixing static data from macros and call to Pyo3 constants.

use crate::utils::PyO3CratePath;
use proc_macro2::TokenStream;
use quote::quote;
use std::borrow::Cow;
use syn::visit_mut::{visit_type_mut, VisitMut};
use syn::{Lifetime, Type};

#[derive(Clone)]
pub struct PythonTypeHint(PythonTypeHintVariant);

#[derive(Clone)]
enum PythonTypeHintVariant {
    /// The Python type hint of a FromPyObject implementation
    FromPyObject(Type),
    /// The Python type hint of a IntoPyObject implementation
    IntoPyObject(Type),
    /// The Python type matching the given Rust type given as a function argument
    ArgumentType(Type),
    /// The Python type matching the given Rust type given as a function returned value
    ReturnType(Type),
    /// The Python type matching the given Rust type
    Type(Type),
    /// A local type
    Local(Cow<'static, str>),
    /// A type in a module
    ModuleAttribute {
        module: Cow<'static, str>,
        attr: Cow<'static, str>,
    },
    /// A union
    Union(Vec<PythonTypeHint>),
    /// A subscript
    Subscript {
        value: Box<PythonTypeHint>,
        slice: Vec<PythonTypeHint>,
    },
}

impl PythonTypeHint {
    /// Build from a local name
    pub fn local(name: impl Into<Cow<'static, str>>) -> Self {
        Self(PythonTypeHintVariant::Local(name.into()))
    }

    /// Build from a builtins name like `None`
    pub fn builtin(name: impl Into<Cow<'static, str>>) -> Self {
        Self::module_attr("builtins", name)
    }

    /// Build from a module and a name like `collections.abc` and `Sequence`
    pub fn module_attr(
        module: impl Into<Cow<'static, str>>,
        name: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self(PythonTypeHintVariant::ModuleAttribute {
            module: module.into(),
            attr: name.into(),
        })
    }

    /// The type hint of a `FromPyObject` implementation as a function argument
    ///
    /// If self_type is set, self_type will replace Self in the given type
    pub fn from_from_py_object(t: Type, self_type: Option<&Type>) -> Self {
        Self(PythonTypeHintVariant::FromPyObject(clean_type(
            t, self_type,
        )))
    }

    /// The type hint of a `IntoPyObject` implementation as a function argument
    ///
    /// If self_type is set, self_type will replace Self in the given type
    pub fn from_into_py_object(t: Type, self_type: Option<&Type>) -> Self {
        Self(PythonTypeHintVariant::IntoPyObject(clean_type(
            t, self_type,
        )))
    }

    /// The type hint of the Rust type used as a function argument
    ///
    /// If self_type is set, self_type will replace Self in the given type
    pub fn from_argument_type(t: Type, self_type: Option<&Type>) -> Self {
        Self(PythonTypeHintVariant::ArgumentType(clean_type(
            t, self_type,
        )))
    }

    /// The type hint of the Rust type used as a function output type
    ///
    /// If self_type is set, self_type will replace Self in the given type
    pub fn from_return_type(t: Type, self_type: Option<&Type>) -> Self {
        Self(PythonTypeHintVariant::ReturnType(clean_type(t, self_type)))
    }

    /// The type hint of the Rust type `PyTypeCheck` trait.
    ///
    /// If self_type is set, self_type will replace Self in the given type
    pub fn from_type(t: Type, self_type: Option<&Type>) -> Self {
        Self(PythonTypeHintVariant::Type(clean_type(t, self_type)))
    }

    /// Build the union of the different element
    pub fn union(elements: impl IntoIterator<Item = Self>) -> Self {
        let elements = elements.into_iter().collect::<Vec<_>>();
        if elements.len() == 1 {
            return elements.into_iter().next().unwrap();
        }
        Self(PythonTypeHintVariant::Union(elements))
    }

    /// Build the subscripted type value[slice[0], ..., slice[n]]
    pub fn subscript(value: Self, slice: impl IntoIterator<Item = Self>) -> Self {
        Self(crate::type_hint::PythonTypeHintVariant::Subscript {
            value: Box::new(value),
            slice: slice.into_iter().collect(),
        })
    }

    pub fn to_introspection_token_stream(&self, pyo3_crate_path: &PyO3CratePath) -> TokenStream {
        match &self.0 {
            PythonTypeHintVariant::Local(name) => {
                quote! { #pyo3_crate_path::inspect::TypeHint::local(#name) }
            }
            PythonTypeHintVariant::ModuleAttribute { module, attr } => {
                if module == "builtins" {
                    quote! { #pyo3_crate_path::inspect::TypeHint::builtin(#attr) }
                } else {
                    quote! { #pyo3_crate_path::inspect::TypeHint::module_attr(#module, #attr) }
                }
            }
            PythonTypeHintVariant::FromPyObject(t) => {
                quote! { <#t as #pyo3_crate_path::FromPyObject<'_, '_>>::INPUT_TYPE }
            }
            PythonTypeHintVariant::IntoPyObject(t) => {
                quote! { <#t as #pyo3_crate_path::IntoPyObject<'_>>::OUTPUT_TYPE }
            }
            PythonTypeHintVariant::ArgumentType(t) => {
                quote! {
                    <#t as #pyo3_crate_path::impl_::extract_argument::PyFunctionArgument<
                        {
                            #[allow(unused_imports, reason = "`Probe` trait used on negative case only")]
                            use #pyo3_crate_path::impl_::pyclass::Probe as _;
                            #pyo3_crate_path::impl_::pyclass::IsFromPyObject::<#t>::VALUE
                        }
                    >>::INPUT_TYPE
                }
            }
            PythonTypeHintVariant::ReturnType(t) => {
                quote! { <#t as #pyo3_crate_path::impl_::introspection::PyReturnType>::OUTPUT_TYPE }
            }
            PythonTypeHintVariant::Type(t) => {
                quote! { <#t as #pyo3_crate_path::type_object::PyTypeCheck>::TYPE_HINT }
            }
            PythonTypeHintVariant::Union(elements) => {
                let elements = elements
                    .iter()
                    .map(|elt| elt.to_introspection_token_stream(pyo3_crate_path));
                quote! { #pyo3_crate_path::inspect::TypeHint::union(&[#(#elements),*]) }
            }
            PythonTypeHintVariant::Subscript { value, slice } => {
                let value = value.to_introspection_token_stream(pyo3_crate_path);
                let slice = slice
                    .iter()
                    .map(|elt| elt.to_introspection_token_stream(pyo3_crate_path));
                quote! { #pyo3_crate_path::inspect::TypeHint::subscript(&#value, &[#(#slice),*]) }
            }
        }
    }
}

fn clean_type(mut t: Type, self_type: Option<&Type>) -> Type {
    if let Some(self_type) = self_type {
        replace_self(&mut t, self_type);
    }
    elide_lifetimes(&mut t);
    t
}

/// Replaces all explicit lifetimes in `self` with elided (`'_`) lifetimes
///
/// This is useful if `Self` is used in `const` context, where explicit
/// lifetimes are not allowed (yet).
fn elide_lifetimes(ty: &mut Type) {
    struct ElideLifetimesVisitor;

    impl VisitMut for ElideLifetimesVisitor {
        fn visit_lifetime_mut(&mut self, l: &mut syn::Lifetime) {
            *l = Lifetime::new("'_", l.span());
        }
    }

    ElideLifetimesVisitor.visit_type_mut(ty);
}

// Replace Self in types with the given type
fn replace_self(ty: &mut Type, self_target: &Type) {
    struct SelfReplacementVisitor<'a> {
        self_target: &'a Type,
    }

    impl VisitMut for SelfReplacementVisitor<'_> {
        fn visit_type_mut(&mut self, ty: &mut Type) {
            if let Type::Path(type_path) = ty {
                if type_path.qself.is_none()
                    && type_path.path.segments.len() == 1
                    && type_path.path.segments[0].ident == "Self"
                    && type_path.path.segments[0].arguments.is_empty()
                {
                    // It is Self
                    *ty = self.self_target.clone();
                    return;
                }
            }
            visit_type_mut(self, ty);
        }
    }

    SelfReplacementVisitor { self_target }.visit_type_mut(ty);
}
