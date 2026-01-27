//! Define a data structure for Python type hints, mixing static data from macros and call to Pyo3 constants.

use crate::utils::PyO3CratePath;
use proc_macro2::TokenStream;
use quote::quote;
use std::borrow::Cow;
use syn::visit_mut::{visit_type_mut, VisitMut};
use syn::{Expr, ExprLit, ExprPath, Lifetime, Lit, Type};

/// A Python expression
///
/// Please do not construct directly but use the constructor methods that normalize the expression
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PyExpr {
    /// The Python type hint of a FromPyObject implementation
    FromPyObjectType(Type),
    /// The Python type hint of a IntoPyObject implementation
    IntoPyObjectType(Type),
    /// The Python type matching the given Rust type given as a function argument
    ArgumentType(Type),
    /// The Python type matching the given Rust type given as a function returned value
    ReturnType(Type),
    /// The Python type matching the given Rust type
    Type(Type),
    /// A name
    Name { id: Cow<'static, str> },
    /// An attribute `value.attr`
    Attribute {
        value: Box<Self>,
        attr: Cow<'static, str>,
    },
    /// A binary operator
    BinOp {
        left: Box<Self>,
        op: PyOperator,
        right: Box<Self>,
    },
    /// A tuple
    Tuple { elts: Vec<Self> },
    /// A subscript `value[slice]`
    Subscript { value: Box<Self>, slice: Box<Self> },
    /// A constant
    Constant(PyConstant),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PyOperator {
    /// `|` operator
    BitOr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PyConstant {
    /// None
    None,
    /// The `True` and `False` booleans
    Bool(bool),
    /// `int` value written in base 10 ([+-]?[0-9]+)
    Int(String),
    /// `float` value written in base-10 ([+-]?[0-9]*(.[0-9]*)*([eE])[0-9]*), not including Inf and NaN
    Float(String),
    /// `str` value unescaped and without quotes
    Str(String),
    /// `...`
    Ellipsis,
}

impl PyExpr {
    /// Build from a builtins name like `None`
    pub fn builtin(name: impl Into<Cow<'static, str>>) -> Self {
        Self::Name { id: name.into() }
    }

    /// Build from a module and a name like `collections.abc` and `Sequence`
    pub fn module_attr(
        module: impl Into<Cow<'static, str>>,
        name: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::attribute(Self::Name { id: module.into() }, name)
    }

    /// The type hint of a `FromPyObject` implementation as a function argument
    ///
    /// If self_type is set, self_type will replace Self in the given type
    pub fn from_from_py_object(t: Type, self_type: Option<&Type>) -> Self {
        Self::FromPyObjectType(clean_type(t, self_type))
    }

    /// The type hint of a `IntoPyObject` implementation as a function argument
    ///
    /// If self_type is set, self_type will replace Self in the given type
    pub fn from_into_py_object(t: Type, self_type: Option<&Type>) -> Self {
        Self::IntoPyObjectType(clean_type(t, self_type))
    }

    /// The type hint of the Rust type used as a function argument
    ///
    /// If self_type is set, self_type will replace Self in the given type
    pub fn from_argument_type(t: Type, self_type: Option<&Type>) -> Self {
        Self::ArgumentType(clean_type(t, self_type))
    }

    /// The type hint of the Rust type used as a function output type
    ///
    /// If self_type is set, self_type will replace Self in the given type
    pub fn from_return_type(t: Type, self_type: Option<&Type>) -> Self {
        Self::ReturnType(clean_type(t, self_type))
    }

    /// The type hint of the Rust type `PyTypeCheck` trait.
    ///
    /// If self_type is set, self_type will replace Self in the given type
    pub fn from_type(t: Type, self_type: Option<&Type>) -> Self {
        Self::Type(clean_type(t, self_type))
    }

    /// An attribute of a given value: `value.attr`
    pub fn attribute(value: Self, attr: impl Into<Cow<'static, str>>) -> Self {
        Self::Attribute {
            value: Box::new(value),
            attr: attr.into(),
        }
    }

    /// Build the union of the different element
    pub fn union(left: Self, right: Self) -> Self {
        Self::BinOp {
            left: Box::new(left),
            op: PyOperator::BitOr,
            right: Box::new(right),
        }
    }

    /// Build the subscripted type value[slice]
    pub fn subscript(value: Self, slice: Self) -> Self {
        Self::Subscript {
            value: Box::new(value),
            slice: Box::new(slice),
        }
    }

    /// Build a tuple
    pub fn tuple(elts: impl IntoIterator<Item = Self>) -> Self {
        Self::Tuple {
            elts: elts.into_iter().collect(),
        }
    }

    pub fn constant_from_expression(expr: &Expr) -> Self {
        Self::Constant(match expr {
            Expr::Lit(ExprLit { lit, .. }) => match lit {
                Lit::Str(s) => PyConstant::Str(s.value()),
                Lit::Char(c) => PyConstant::Str(c.value().into()),
                Lit::Int(i) => PyConstant::Int(i.base10_digits().into()),
                Lit::Float(f) => PyConstant::Float(f.base10_digits().into()),
                Lit::Bool(b) => PyConstant::Bool(b.value()),
                _ => PyConstant::Ellipsis, // TODO: implement ByteStr and CStr
            },
            Expr::Path(ExprPath { qself, path, .. })
                if qself.is_none() && path.is_ident("None") =>
            {
                PyConstant::None
            }
            _ => PyConstant::Ellipsis,
        })
    }

    pub fn str_constant(value: impl Into<String>) -> Self {
        Self::Constant(PyConstant::Str(value.into()))
    }

    /// `...`
    pub fn ellipsis() -> Self {
        Self::Constant(PyConstant::Ellipsis)
    }

    pub fn to_introspection_token_stream(&self, pyo3_crate_path: &PyO3CratePath) -> TokenStream {
        match self {
            Self::FromPyObjectType(t) => {
                quote! { <#t as #pyo3_crate_path::FromPyObject<'_, '_>>::INPUT_TYPE }
            }
            Self::IntoPyObjectType(t) => {
                quote! { <#t as #pyo3_crate_path::IntoPyObject<'_>>::OUTPUT_TYPE }
            }
            Self::ArgumentType(t) => {
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
            Self::ReturnType(t) => {
                quote! {{
                    #[allow(unused_imports)]
                    use #pyo3_crate_path::impl_::pyclass::Probe as _;
                    const TYPE: #pyo3_crate_path::inspect::PyStaticExpr = if #pyo3_crate_path::impl_::pyclass::IsReturningEmptyTuple::<#t>::VALUE {
                        <#pyo3_crate_path::types::PyNone as #pyo3_crate_path::type_object::PyTypeInfo>::TYPE_HINT
                    } else {
                        <#t as #pyo3_crate_path::impl_::introspection::PyReturnType>::OUTPUT_TYPE
                    };
                    TYPE
                }}
            }
            Self::Type(t) => {
                quote! { <#t as #pyo3_crate_path::type_object::PyTypeCheck>::TYPE_HINT }
            }
            Self::Name { id } => {
                quote! { #pyo3_crate_path::inspect::PyStaticExpr::Name { id: #id } }
            }
            Self::Attribute { value, attr } => {
                let value = value.to_introspection_token_stream(pyo3_crate_path);
                quote! { #pyo3_crate_path::inspect::PyStaticExpr::Attribute { value: &#value, attr: #attr } }
            }
            Self::BinOp { left, op, right } => {
                let left = left.to_introspection_token_stream(pyo3_crate_path);
                let op = match op {
                    PyOperator::BitOr => quote!(#pyo3_crate_path::inspect::PyStaticOperator::BitOr),
                };
                let right = right.to_introspection_token_stream(pyo3_crate_path);
                quote! {
                    #pyo3_crate_path::inspect::PyStaticExpr::BinOp {
                        left: &#left,
                        op: #op,
                        right: &#right,
                    }
                }
            }
            Self::Subscript { value, slice } => {
                let value = value.to_introspection_token_stream(pyo3_crate_path);
                let slice = slice.to_introspection_token_stream(pyo3_crate_path);
                quote! { #pyo3_crate_path::inspect::PyStaticExpr::Subscript { value: &#value, slice: &#slice } }
            }
            Self::Tuple { elts } => {
                let elts = elts
                    .iter()
                    .map(|e| e.to_introspection_token_stream(pyo3_crate_path));
                quote! { #pyo3_crate_path::inspect::PyStaticExpr::Tuple { elts: &[#(#elts),*] } }
            }
            Self::Constant(c) => match c {
                PyConstant::None => {
                    quote! { #pyo3_crate_path::inspect::PyStaticExpr::Constant { value: #pyo3_crate_path::inspect::PyStaticConstant::None } }
                }
                PyConstant::Bool(v) => {
                    quote! { #pyo3_crate_path::inspect::PyStaticExpr::Constant { value: #pyo3_crate_path::inspect::PyStaticConstant::Bool(#v) } }
                }
                PyConstant::Int(v) => {
                    quote! { #pyo3_crate_path::inspect::PyStaticExpr::Constant { value: #pyo3_crate_path::inspect::PyStaticConstant::Int(#v) } }
                }
                PyConstant::Float(v) => {
                    quote! { #pyo3_crate_path::inspect::PyStaticExpr::Constant { value: #pyo3_crate_path::inspect::PyStaticConstant::Float(#v) } }
                }
                PyConstant::Str(v) => {
                    quote! { #pyo3_crate_path::inspect::PyStaticExpr::Constant { value: #pyo3_crate_path::inspect::PyStaticConstant::Str(#v) } }
                }
                PyConstant::Ellipsis => {
                    quote! { #pyo3_crate_path::inspect::PyStaticExpr::Constant { value: #pyo3_crate_path::inspect::PyStaticConstant::Ellipsis } }
                }
            },
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
        fn visit_lifetime_mut(&mut self, l: &mut Lifetime) {
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
