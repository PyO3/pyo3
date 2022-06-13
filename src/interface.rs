//! Data required for Python Interface files (.pyi, also called 'stub files').
//!
//! This module creates three data structures, [`ModuleInfo`], [`ClassInfo`] and [`FieldInfo`],
//! which are responsible for generating parts of the interface files.

/// Python Interface information for a module.
#[derive(Debug)]
pub struct ModuleInfo {}

#[derive(Debug)]
pub struct ClassInfo {
    pub name: &'static str,
    pub base: &'static str,
    pub fields: &'static [FieldInfo],
}

/// Python Interface information for a field (attribute, function, method…).
#[derive(Debug)]
pub struct FieldInfo {
    pub name: &'static str,
    pub kind: FieldKind,
    pub py_type: Option<&'static TypeInfo>,
    pub arguments: &'static [ArgumentInfo],
}

#[derive(Debug)]
pub enum FieldKind {
    /// The special 'new' method
    New,
    /// A top-level or instance attribute
    Attribute,
    /// A top-level or instance getter
    Getter,
    /// A top-level or instance setter
    Setter,
    /// A top-level function or an instance method
    Function,
    /// A class method
    ClassMethod,
    /// A class attribute
    ClassAttribute,
    /// A static method
    StaticMethod,
}

#[derive(Debug)]
pub struct ArgumentInfo {
    pub name: &'static str,
    pub kind: ArgumentKind,
    pub py_type: Option<&'static TypeInfo>,
    pub default_value: bool,
    pub is_modified: bool,
}

#[derive(Debug)]
pub enum ArgumentKind {
    /// A normal argument, that can be passed positionally or by keyword.
    Regular,
    /// A normal argument that can only be passed positionally (not by keyword).
    PositionalOnly,
    /// An argument that represents all positional arguments that were provided on the call-site
    /// but do not match any declared regular argument.
    Vararg,
    /// An argument that represents all keyword arguments that were provided on the call-site
    /// but do not match any declared regular argument.
    KeywordArg,
}

/// The various Python types handled by PyO3.
///
/// The current implementation is not able to generate custom generics.
/// All generic types can only be hardcoded in PyO3.
#[derive(Debug)]
pub enum TypeInfo {
    /// The type `typing.Any`, which represents a dynamically-typed value (unknown type).
    Any,

    /// The type `typing.None`.
    None,

    /// The type `typing.Callable`, which represents a function-like object.
    Callable(Option<&'static [&'static TypeInfo]>, &'static TypeInfo),

    /// The type `typing.NoReturn`, which represents a function that never returns.
    NoReturn,

    /// A tuple of any value and size: `typing.Tuple[...]`.
    AnyTuple,

    /// A tuple of the specified types.
    Tuple(&'static [&'static TypeInfo]),

    /// A tuple of unknown size, in which all elements have the same type.
    UnsizedTuple(&'static TypeInfo),

    /// A union of multiple types.
    Union(&'static [&'static TypeInfo]),

    /// An optional value.
    Optional(&'static TypeInfo),

    Dict(&'static TypeInfo, &'static TypeInfo),

    Mapping(&'static TypeInfo, &'static TypeInfo),

    List(&'static TypeInfo),

    Set(&'static TypeInfo),

    FrozenSet(&'static TypeInfo),

    Sequence(&'static TypeInfo),

    Iterator(&'static TypeInfo),

    Builtin(&'static str),

    /// Any type that doesn't receive special treatment from PyO3.
    Other {
        module: &'static str,
        name: &'static str,
    },
}

pub trait GetClassInfo {
    fn info() -> &'static ClassInfo;
}

pub trait GetClassFields {
    fn fields_info() -> &'static [&'static FieldInfo];
}
